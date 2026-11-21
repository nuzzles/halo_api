//! Corroborated schema hints and complete object baselines from type-2 snapshots.
use std::collections::BTreeMap;

use crate::theater::film::{Film};
use crate::theater::parser::v41::replication::source::{DatumEntry, DefaultStateStatus, KeyframeRecord, KeyframeStop, ReplicationStreamPacketBody};
use crate::theater::parser::v41::registry::source::{FilmRegistry};
use crate::theater::parser::v41::packet::{PacketRead};
use crate::theater::parser::v41::context::V41DecodeConfig;

use crate::theater::parser::v41::replication::bindings::{
    AnticipatedBindings, AnticipatedDeclaration,
};

pub(crate) struct KeyframeEvidence {
    pub anticipated: AnticipatedBindings,
    pub baselines: BTreeMap<(usize, usize), Vec<KeyframeRecord>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Candidate {
    bit: usize,
    id: u32,
    archetype: u32,
}

const ALLOCATED_AND_PUBLISHED: u8 = 0x05;
const RELEASED_OR_EXCLUDED: u8 = 0x22;

fn candidates(
    payload: &[u8],
    archetype_count: usize,
    datums: &[DatumEntry],
) -> BTreeMap<u32, Candidate> {
    // Keyframe headers begin with [id:32][archetype:32] in slot order.
    // The paired datum table proves that the slot and generation were live.
    let bits = payload.len().saturating_mul(8);
    if bits < 64 {
        return BTreeMap::new();
    }
    let mut window = payload[..8]
        .iter()
        .fold(0u64, |word, byte| (word << 8) | u64::from(*byte));
    let mut found = Vec::<Candidate>::new();
    for bit in 0..=bits - 64 {
        let id = (window >> 32) as u32;
        let archetype = window as u32;
        let slot = id & 0x3fff_ffff;
        if slot < 8192
            && (archetype as usize) < archetype_count.min(50)
            && datums.get(slot as usize).is_some_and(|datum| {
                datum.generation_tag == (id >> 30) as u8
                    && datum.flags & ALLOCATED_AND_PUBLISHED == ALLOCATED_AND_PUBLISHED
                    && datum.flags & RELEASED_OR_EXCLUDED == 0
            })
        {
            found.push(Candidate { bit, id, archetype });
        }
        let next = bit + 64;
        if next < bits {
            window = (window << 1) | u64::from((payload[next / 8] >> (7 - next % 8)) & 1);
        }
    }
    ordered_candidates(&found)
}

fn ordered_candidates(found: &[Candidate]) -> BTreeMap<u32, Candidate> {
    let mut tails = Vec::<usize>::new();
    let mut previous = vec![None; found.len()];
    for (index, candidate) in found.iter().enumerate() {
        let slot = candidate.id & 0x3fff_ffff;
        let rank = tails.partition_point(|&tail| (found[tail].id & 0x3fff_ffff) < slot);
        if rank < tails.len() && (found[tails[rank]].id & 0x3fff_ffff) == slot {
            let incumbent = found[tails[rank]];
            if (candidate.id >> 30, candidate.bit) >= (incumbent.id >> 30, incumbent.bit) {
                continue;
            }
        }
        if rank > 0 {
            previous[index] = Some(tails[rank - 1]);
        }
        if rank == tails.len() {
            tails.push(index);
        } else {
            tails[rank] = index;
        }
    }
    let mut out = BTreeMap::new();
    let mut index = tails.last().copied();
    while let Some(current) = index {
        let candidate = found[current];
        out.insert(candidate.id & 0x3fff_ffff, candidate);
        index = previous[current];
    }
    out
}

/// A lone bit-pattern match is not a declaration. Require the same unique
/// full ID and archetype in adjacent keyframes before using it as a schema hint.
/// A transform baseline must also decode completely up to the next slot header.
pub(crate) fn evidence_from_keyframes(film: &Film) -> KeyframeEvidence {
    let mut baselines = BTreeMap::new();
    let mut config = V41DecodeConfig::default();
    config.profile.grammar.baseline_scope = true;
    config.profile.grammar.writer_absolute = true;
    let mut snapshots = Vec::<(usize, BTreeMap<u32, Candidate>)>::new();
    for chunk in &film.replication_stream {
        let mut datums = None;
        for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
            match packet.header.packet_type {
                1 => {
                    datums = match &packet.body {
                        PacketRead::Decoded(ReplicationStreamPacketBody::DatumsPacketBody(
                            table,
                        )) => Some(table.entries.as_slice()),
                        _ => None,
                    };
                }
                2 => {
                    if let (Some(payload), Some(entries)) = (chunk.payload(packet), datums.take()) {
                        let found = candidates(
                            payload,
                            film.registry.body.registry.archetypes.len(),
                            entries,
                        );
                        let records = object_baselines(
                            payload,
                            &film.registry.body.registry,
                            &config,
                            &found,
                        );
                        if !records.is_empty() {
                            baselines.insert(((chunk.index), packet_index), records);
                        }
                        snapshots.push((chunk.index, found));
                    }
                }
                _ => {}
            }
        }
    }
    KeyframeEvidence {
        anticipated: corroborated(&snapshots),
        baselines,
    }
}

fn object_baselines(
    payload: &[u8],
    registry: &FilmRegistry,
    config: &V41DecodeConfig,
    found: &BTreeMap<u32, Candidate>,
) -> Vec<KeyframeRecord> {
    let layout = config.record_layout();
    found
        .iter()
        .filter_map(|(slot, candidate)| {
            let next = found.get(&(slot + 1))?;
            let archetype = registry.archetype(candidate.archetype as usize)?;
            if !archetype
                .components
                .iter()
                .any(|c| c.name == "object-position-component")
            {
                return None;
            }
            let record = super::record::decode_reference_keyframe_record_contextual(
                payload,
                candidate.bit,
                registry,
                &layout,
                Some(&config.profile),
            )?;
            (record.stop == KeyframeStop::Complete
                && record.end_bit == next.bit as i64
                && record
                    .default_state
                    .as_ref()
                    .is_some_and(|d| d.status == DefaultStateStatus::Complete)
                && !record.components.is_empty())
            .then_some(record)
        })
        .collect()
}

fn corroborated(snapshots: &[(usize, BTreeMap<u32, Candidate>)]) -> AnticipatedBindings {
    let mut out = AnticipatedBindings::default();
    for pair in snapshots.windows(2) {
        let [(left_chunk, left), (right_chunk, right)] = pair else {
            unreachable!()
        };
        if left_chunk == right_chunk {
            continue;
        }
        for (slot, candidate) in left {
            if !right.get(slot).is_some_and(|other| {
                other.id == candidate.id && other.archetype == candidate.archetype
            }) {
                continue;
            }
            let entries = out.entries.entry(candidate.id).or_default();
            for chunk_index in [*left_chunk, *right_chunk] {
                if entries
                    .last()
                    .is_none_or(|last| last.chunk_index != chunk_index)
                {
                    entries.push(AnticipatedDeclaration {
                        chunk_index,
                        archetype: candidate.archetype,
                    });
                    out.declarations += 1;
                }
            }
        }
    }
    out
}
