//! Native dead-state candidate scan, deduplication and repeated-position counts.
use super::{
    DecodeError, KillPacketIdentity, KillSourceCandidate, bits::Bits, fire_events, native_sort,
    pinned_kill_damage_catalog,
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
/// Read direct dead-state candidates. `relaxed` retains tags outside the damage
/// catalog for diagnostics; it does not relax index, category or presence gates.
pub fn scan_kill_source_payload(
    data: &[u8],
    participants: usize,
    relaxed: bool,
) -> Vec<KillSourceCandidate> {
    let bits = Bits(data);
    let mut out = Vec::new();
    if bits.len() < 58 {
        return out;
    }
    for p in 0..=bits.len() - 58 {
        if bits.read(p, 2) != Some(3) {
            continue;
        }
        let tag = bits.read(p + 2, 32).unwrap() as u32;
        if !relaxed && !pinned_kill_damage_catalog().is_damage_effect(tag) {
            continue;
        }
        let q = p + 42;
        if bits.read(q, 1) != Some(0) || bits.read(q + 6, 1) != Some(0) {
            continue;
        }
        let victim = bits.read(q + 1, 5).unwrap() as i32;
        let killer = bits.read(q + 7, 5).unwrap() as i32;
        let category = bits.read(q + 12, 4).unwrap() as i32;
        if victim as usize >= participants || killer as usize >= participants || category > 9 {
            continue;
        }
        out.push(KillSourceCandidate {
            bit: p as i64,
            tag,
            victim,
            killer,
            category,
            ..Default::default()
        });
    }
    out
}
fn identity(c: &KillSourceCandidate) -> (i64, i64) {
    c.packet.map(|p| (p.chunk, p.packet)).unwrap_or((0, 0))
}
/// Retain the first identical packet/tag/victim/killer/category candidate. Bit
/// position is intentionally absent from this key, matching the native pass.
pub fn deduplicate_kill_source_candidates(
    candidates: Vec<KillSourceCandidate>,
) -> Vec<KillSourceCandidate> {
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|c| seen.insert((identity(c), c.tag, c.victim, c.killer, c.category)))
        .collect()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KillSourceMultiplicityKey {
    pub bit: i64,
    pub tag: u32,
    pub victim: i32,
}
pub fn kill_source_multiplicity(
    candidates: &[KillSourceCandidate],
) -> BTreeMap<KillSourceMultiplicityKey, usize> {
    let mut packets = BTreeMap::<_, BTreeSet<_>>::new();
    for c in candidates {
        packets
            .entry(KillSourceMultiplicityKey {
                bit: c.bit,
                tag: c.tag,
                victim: c.victim,
            })
            .or_default()
            .insert(identity(c));
    }
    packets.into_iter().map(|(k, v)| (k, v.len())).collect()
}
/// Scan event-bearing replication packets in native timestamp order. Candidate
/// times are relative to the earliest replication packet, including eventless ones.
pub fn scan_film_kill_sources(
    chunks: &[FilmChunkData],
    major_version: i32,
    participants: usize,
) -> Result<Vec<KillSourceCandidate>, DecodeError> {
    scan_film_kill_source_mode(chunks, major_version, participants, false)
}
/// Relaxed diagnostics retain every hit, including repeated packet candidates.
pub fn scan_film_relaxed_kill_sources(
    chunks: &[FilmChunkData],
    major_version: i32,
    participants: usize,
) -> Result<Vec<KillSourceCandidate>, DecodeError> {
    scan_film_kill_source_mode(chunks, major_version, participants, true)
}
fn scan_film_kill_source_mode(
    chunks: &[FilmChunkData],
    major_version: i32,
    participants: usize,
    relaxed: bool,
) -> Result<Vec<KillSourceCandidate>, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let mut packets = Vec::new();
    for (chunk_position, c) in chunks.iter().enumerate() {
        for (index, p) in fire_events::native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type == 0 {
                packets.push((
                    p,
                    index,
                    chunk_position,
                    &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                ));
            }
        }
    }
    native_sort::sort_by(&mut packets, |a, b| a.0.timestamp_us.cmp(&b.0.timestamp_us));
    let base = packets
        .first()
        .ok_or(DecodeError::Missing("replication packets"))?
        .0
        .timestamp_us;
    let mut out = Vec::new();
    for (p, index, chunk_position, data) in packets {
        if Bits(data).read(1, 1) != Some(1) {
            continue;
        }
        for mut c in scan_kill_source_payload(data, participants, relaxed) {
            c.time_ms = ((p.timestamp_us - base) / 1000) as i64;
            c.packet = Some(KillPacketIdentity {
                chunk: chunk_position as i64,
                packet: index as i64,
            });
            out.push(c);
        }
    }
    Ok(if relaxed {
        out
    } else {
        deduplicate_kill_source_candidates(out)
    })
}

#[cfg(test)]
mod tests {
    use super::super::{
        KillDamageProvenance, KillSourceTruth, kill_damage_category_name, kill_source_truth,
        strong_kill_damage_tag,
    };
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        data: String,
        participants: usize,
        strict: Vec<KillSourceCandidate>,
        relaxed: Vec<KillSourceCandidate>,
    }
    #[derive(Deserialize)]
    struct Label {
        truth: KillSourceTruth,
        category_name: String,
        strong: bool,
        catalogued: bool,
    }
    #[derive(Deserialize)]
    struct Count {
        bit: i64,
        tag: u32,
        victim: i32,
        count: usize,
    }
    #[derive(Deserialize)]
    struct Oracle {
        rows: Vec<Row>,
        labels: Vec<Label>,
        ids: BTreeSet<u32>,
        provenance: KillDamageProvenance,
        candidates: Vec<KillSourceCandidate>,
        dedup: Vec<KillSourceCandidate>,
        multiplicity: Vec<Count>,
    }
    #[test]
    fn source_positions_survive_manifest_gaps() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut payload = vec![0u8; 16];
        let mut put = |at: usize, width: usize, value: u64| {
            for j in 0..width {
                payload[(at + j) / 8] |=
                    (((value >> (width - j - 1)) & 1) as u8) << (7 - (at + j) % 8);
            }
        };
        put(1, 1, 1);
        put(16, 2, 3);
        put(
            18,
            32,
            u64::from(*pinned_kill_damage_catalog().ids.first().unwrap()),
        );
        put(16 + 49, 5, 1);
        let chunks: Vec<_> = [10, 20]
            .into_iter()
            .enumerate()
            .map(|(i, index)| {
                // Include a different packet type before replication to verify that
                // the source packet index does not become a filtered ordinal.
                let mut data = vec![0u8; 16];
                data[..2].copy_from_slice(&12u16.to_le_bytes());
                data[4..8].copy_from_slice(&1u32.to_le_bytes());
                data.push(0);
                let mut header = [0u8; 16];
                header[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
                header[8..].copy_from_slice(&((i as u64 + 1) * 1000).to_le_bytes());
                data.extend_from_slice(&header);
                data.extend_from_slice(&payload);
                FilmChunkData {
                    metadata: FilmChunk {
                        index,
                        chunk_type: 2,
                        start_time_offset_ms: 0,
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data,
                }
            })
            .collect();
        let candidates = scan_film_kill_sources(&chunks, 41, 2).unwrap();
        let actual: Vec<_> = candidates
            .iter()
            .filter(|c| c.bit == 16)
            .map(|c| (c.packet.unwrap(), c.time_ms, c.victim, c.killer))
            .collect();
        assert_eq!(
            actual,
            vec![
                (
                    KillPacketIdentity {
                        chunk: 0,
                        packet: 1
                    },
                    0,
                    0,
                    1
                ),
                (
                    KillPacketIdentity {
                        chunk: 1,
                        packet: 1
                    },
                    1,
                    0,
                    1
                )
            ]
        );
        assert!(super::super::scan_film_kill_events(&chunks, 41).is_ok());
    }
    #[test]
    fn native_kill_source_scan_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-source-scan-v41.json.zlib")[..],
        )
        .read_to_end(&mut json)
        .unwrap();
        let oracle: Oracle = serde_json::from_slice(&json).unwrap();
        assert_eq!(oracle.rows.len(), 2048);
        let catalog = pinned_kill_damage_catalog();
        assert_eq!(catalog.ids, oracle.ids);
        assert_eq!(catalog.provenance, oracle.provenance);
        for l in oracle.labels {
            assert_eq!(kill_source_truth(l.truth.tag, l.truth.category), l.truth);
            assert_eq!(kill_damage_category_name(l.truth.category), l.category_name);
            assert_eq!(strong_kill_damage_tag(l.truth.tag), l.strong);
            assert_eq!(catalog.is_damage_effect(l.truth.tag), l.catalogued);
        }
        let mut strict_count = 0;
        for (i, row) in oracle.rows.into_iter().enumerate() {
            let data: Vec<u8> = row
                .data
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                .collect();
            assert_eq!(
                scan_kill_source_payload(&data, row.participants, false),
                row.strict,
                "strict {i}"
            );
            assert_eq!(
                scan_kill_source_payload(&data, row.participants, true),
                row.relaxed,
                "relaxed {i}"
            );
            strict_count += row.strict.len();
        }
        assert!(strict_count > 100);
        let multiplicity: BTreeMap<_, _> = oracle
            .multiplicity
            .into_iter()
            .map(|c| {
                (
                    KillSourceMultiplicityKey {
                        bit: c.bit,
                        tag: c.tag,
                        victim: c.victim,
                    },
                    c.count,
                )
            })
            .collect();
        assert_eq!(kill_source_multiplicity(&oracle.candidates), multiplicity);
        assert_eq!(
            deduplicate_kill_source_candidates(oracle.candidates),
            oracle.dedup
        );
    }
}
