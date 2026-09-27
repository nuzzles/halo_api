//! Presence census at recovered keyframe anchors, ported from world_object_census.go.
//! Presence bounds disappearance; it does not establish its exact time.
use super::{
    fire_events::{native_chunk_packets, native_chunk_prefix},
    recover_keyframe_anchors,
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldObjectLifeSeen {
    pub slot: u32,
    pub generation: u32,
    pub times_us: Vec<u64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldObjectKeyframes {
    pub band: BTreeSet<u32>,
    /// All keyframe times sorted, including repeated timestamps.
    pub times_us: Vec<u64>,
    /// Life-key order; each life retains the reference's consecutive-time deduplication.
    pub seen_us: Vec<WorldObjectLifeSeen>,
}
impl WorldObjectKeyframes {
    pub fn last_time_us(&self) -> u64 {
        self.times_us.last().copied().unwrap_or(0)
    }
}
#[derive(Default)]
struct Census {
    seen: BTreeSet<u32>,
    others: BTreeSet<u32>,
    lives: BTreeMap<(u32, u32), Vec<u64>>,
}
/// Scan one requested archetype. Missing chunks produce an empty census, as in Go.
pub fn scan_world_object_keyframes(
    chunks: &[FilmChunkData],
    archetype: u32,
) -> WorldObjectKeyframes {
    scan_world_object_keyframes_for_archetypes(chunks, &[archetype])
        .remove(&archetype)
        .unwrap_or_default()
}
/// Read keyframe anchors once for all requested archetypes.
pub fn scan_world_object_keyframes_for_archetypes(
    chunks: &[FilmChunkData],
    archetypes: &[u32],
) -> BTreeMap<u32, WorldObjectKeyframes> {
    let mut states: BTreeMap<u32, Census> =
        archetypes.iter().map(|a| (*a, Census::default())).collect();
    let mut times = Vec::new();
    for c in native_chunk_prefix(chunks).unwrap_or_default() {
        for p in native_chunk_packets(c) {
            if p.packet_type != 2 {
                continue;
            }
            times.push(p.timestamp_us);
            for r in recover_keyframe_anchors(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
            ) {
                accumulate(&mut states, &r, p.timestamp_us);
            }
        }
    }
    finish(states, times)
}

fn accumulate(states: &mut BTreeMap<u32, Census>, r: &super::RecoveredKeyframeAnchor, time: u64) {
    let slot = r.id & 0x3fff_ffff;
    let generation = r.id >> 30;
    for (ti, state) in states {
        if r.archetype != *ti {
            state.others.insert(slot);
            continue;
        }
        state.seen.insert(slot);
        let seen = state.lives.entry((slot, generation)).or_default();
        if seen.last() != Some(&time) {
            seen.push(time);
        }
    }
}

/// A recovered keyframe and its original packet identity. Anchors are heuristic
/// selections, not proof that intervening component bodies were decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceWorldObjectKeyframe {
    pub chunk: i64,
    pub packet_index: usize,
    pub source: super::FilmPacket,
    pub anchors: Vec<super::RecoveredKeyframeAnchor>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceWorldObjectCensus {
    pub keyframes: Vec<SourceWorldObjectKeyframe>,
    pub archetypes: BTreeMap<u32, WorldObjectKeyframes>,
}
/// Census over already-loaded buffers, preserving file numbers separately from
/// buffer identities and retaining empty keyframes and recovered anchor order.
/// Missing sources produce empty results for each requested archetype.
pub fn scan_source_world_object_keyframes(
    source: Option<&super::FilmSource>,
    archetypes: &[u32],
) -> SourceWorldObjectCensus {
    let mut states = archetypes.iter().map(|a| (*a, Census::default())).collect();
    let mut times = Vec::new();
    let mut keyframes = Vec::new();
    if let Some(source) = source {
        for chunk in source.data_chunk_numbers() {
            let Some((data, packets)) = source.chunk_by_number(chunk) else {
                continue;
            };
            for (packet_index, &packet) in packets.iter().enumerate() {
                if packet.packet_type != 2 {
                    continue;
                }
                times.push(packet.timestamp_us);
                let anchors = recover_keyframe_anchors(
                    &data[packet.payload_offset..packet.payload_offset + packet.payload_size],
                );
                for anchor in &anchors {
                    accumulate(&mut states, anchor, packet.timestamp_us);
                }
                keyframes.push(SourceWorldObjectKeyframe {
                    chunk,
                    packet_index,
                    source: packet,
                    anchors,
                });
            }
        }
    }
    SourceWorldObjectCensus {
        keyframes,
        archetypes: finish(states, times),
    }
}

fn finish(
    states: BTreeMap<u32, Census>,
    mut times: Vec<u64>,
) -> BTreeMap<u32, WorldObjectKeyframes> {
    times.sort_unstable();
    states
        .into_iter()
        .map(|(ti, state)| {
            let band = match (state.seen.first(), state.seen.last()) {
                (Some(&lo), Some(&hi)) => (lo..=hi).filter(|s| !state.others.contains(s)).collect(),
                _ => BTreeSet::new(),
            };
            let seen_us = state
                .lives
                .into_iter()
                .map(|((slot, generation), mut times_us)| {
                    times_us.sort_unstable();
                    WorldObjectLifeSeen {
                        slot,
                        generation,
                        times_us,
                    }
                })
                .collect();
            (
                ti,
                WorldObjectKeyframes {
                    band,
                    times_us: times.clone(),
                    seen_us,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_census_order_and_exclusions() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/world-census-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            if row["folder"] != "generated" {
                continue;
            }
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let actual = scan_world_object_keyframes_for_archetypes(&chunks, &[37, 38, 41, 42]);
            let expected: BTreeMap<u32, WorldObjectKeyframes> =
                serde_json::from_value(row["census"].clone()).unwrap();
            assert_eq!(actual, expected, "census {i}");
            let mut buffers = vec![vec![0; 16]];
            buffers.extend(chunks.iter().map(|c| c.data.clone()));
            let mut metadata = vec![super::super::FilmSourceMetadata {
                index: -9,
                chunk_type: 0,
                start_ms: 0,
            }];
            metadata.extend(chunks.iter().map(|c| super::super::FilmSourceMetadata {
                index: i64::from(c.metadata.index),
                chunk_type: 2,
                start_ms: 0,
            }));
            let source = super::super::FilmSource::load(&buffers, &metadata).unwrap();
            let loaded = scan_source_world_object_keyframes(Some(&source), &[37, 38, 41, 42]);
            assert_eq!(loaded.archetypes, expected, "loaded census {i}");
            for frame in &loaded.keyframes {
                assert_eq!(
                    metadata[frame.source.chunk_index as usize].index,
                    frame.chunk
                );
                assert_eq!(frame.packet_index, 0);
                let payload = source.payload(&frame.source).unwrap();
                for anchor in &frame.anchors {
                    assert!(anchor.bit + 64 <= payload.len() * 8);
                    let bits = super::super::bits::Bits(payload);
                    assert_eq!(bits.read(anchor.bit, 32), Some(u64::from(anchor.id)));
                    assert_eq!(
                        bits.read(anchor.bit + 32, 32),
                        Some(u64::from(anchor.archetype))
                    );
                }
            }

            let restored = serde_json::from_slice::<BTreeMap<u32, WorldObjectKeyframes>>(
                &serde_json::to_vec(&actual).unwrap(),
            )
            .unwrap();
            assert_eq!(restored, actual);
        }
    }
}
