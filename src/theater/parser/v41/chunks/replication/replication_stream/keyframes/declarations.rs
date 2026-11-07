//! Corroborated schema hints and complete object baselines from type-2 snapshots.
use std::collections::BTreeMap;

use crate::theater::film::{
    DatumEntry, DefaultStateStatus, Film, FilmRegistry, KeyframeRecord, KeyframeStop, PacketRead,
    ReplicationStreamPacketBody,
};
use crate::theater::parser::v41::context::V41DecodeConfig;

use crate::theater::parser::v41::chunks::replication::bindings::{
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
    let mut snapshots = Vec::<(i64, BTreeMap<u32, Candidate>)>::new();
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
                            baselines.insert((chunk.source_position, packet_index), records);
                        }
                        snapshots.push((
                            chunk.source.index.unwrap_or(chunk.source_position as i64),
                            found,
                        ));
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
                .any(|c| c.name() == Some("object-position-component"))
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

fn corroborated(snapshots: &[(i64, BTreeMap<u32, Candidate>)]) -> AnticipatedBindings {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::film::BitRange;

    fn candidate(bit: usize, slot: u32, generation: u32, archetype: u32) -> Candidate {
        Candidate {
            bit,
            id: generation << 30 | slot,
            archetype,
        }
    }

    #[test]
    fn repeated_keyframe_headers_need_matching_id_and_archetype() {
        let first = BTreeMap::from([(4, candidate(1, 4, 1, 35)), (5, candidate(200, 5, 1, 35))]);
        let changed = BTreeMap::from([(4, candidate(1, 4, 2, 35)), (5, candidate(200, 5, 1, 9))]);
        assert!(
            corroborated(&[(1, first.clone()), (2, changed)])
                .entries
                .is_empty()
        );
        let same_chunk = corroborated(&[(1, first.clone()), (1, first.clone())]);
        assert!(same_chunk.entries.is_empty());
        let consistent = corroborated(&[(1, first.clone()), (2, first.clone()), (3, first)]);
        assert_eq!(consistent.entries[&(1 << 30 | 4)].len(), 3);
        assert_eq!(consistent.entries[&(1 << 30 | 5)].len(), 3);
    }

    #[test]
    fn ordered_headers_reject_out_of_order_payload_patterns() {
        let found = [
            candidate(1, 1, 1, 2),
            candidate(100, 2, 1, 5),
            candidate(120, 99, 1, 1),
            candidate(200, 3, 1, 35),
            candidate(300, 4, 1, 35),
        ];
        let ordered = ordered_candidates(&found);
        assert_eq!(ordered.keys().copied().collect::<Vec<_>>(), [1, 2, 3, 4]);
    }

    #[test]
    fn equal_slots_prefer_lower_generation_then_earlier_bit() {
        let found = [
            candidate(1, 1, 1, 3),
            candidate(100, 2, 3, 4),
            candidate(120, 2, 1, 5),
            candidate(140, 2, 1, 6),
            candidate(200, 3, 1, 7),
        ];
        let ordered = ordered_candidates(&found);
        assert_eq!(ordered[&2], found[2]);
    }

    #[test]
    fn complete_header_at_payload_end_is_read_without_body_bits() {
        let payload = [0x4000_0001_u32.to_be_bytes(), 35_u32.to_be_bytes()].concat();
        let mut datums = vec![datum(0, 0); 2];
        datums[1] = datum(5, 1);
        assert_eq!(
            candidates(&payload, 50, &datums)[&1],
            candidate(0, 1, 1, 35)
        );
        assert!(candidates(&payload[..7], 50, &datums).is_empty());
        assert!(candidates(&[0; 8], 50, &datums).is_empty());
        datums[1] = datum(5, 2);
        assert!(candidates(&payload, 50, &datums).is_empty());
        datums[1] = datum(0, 1);
        assert!(candidates(&payload, 50, &datums).is_empty());
        datums[1] = datum(7, 1);
        assert!(candidates(&payload, 50, &datums).is_empty());
        datums[1] = datum(0x25, 1);
        assert!(candidates(&payload, 50, &datums).is_empty());
    }

    fn datum(flags: u8, generation_tag: u8) -> DatumEntry {
        DatumEntry {
            source: BitRange { start: 0, end: 79 },
            flags,
            generation_tag,
            generation: u32::from(generation_tag) + 1,
            view_mask: 0,
        }
    }

    #[test]
    fn live_generation_zero_can_be_corroborated() {
        let payload = [0_u32.to_be_bytes(), 35_u32.to_be_bytes()].concat();
        let datums = [datum(5, 0)];
        assert_eq!(
            candidates(&payload, 50, &datums)[&0],
            candidate(0, 0, 0, 35)
        );
    }

    #[test]
    fn captured_device_baseline_reaches_the_next_record() {
        use crate::theater::parser::test_support::recharge_device_baseline;
        let record = recharge_device_baseline();
        assert_eq!(record.id, 1073743104);
        assert_eq!(record.stop, KeyframeStop::Complete);
        assert_eq!(record.end_bit, 1123);
        assert_eq!(record.components.len(), 41);
        let payload = include_bytes!(
            "../../../../../../fixtures/movement-schema-v41/recharge-device-keyframe.bin"
        );
        let next = crate::theater::parser::v41::reference_bits::reference_bits_at(
            payload,
            record.end_bit,
            32,
        );
        assert_eq!(next, 1073743105);
        assert_eq!(
            record.components[0].end_bit - record.components[0].start_bit,
            99
        );
    }
}
