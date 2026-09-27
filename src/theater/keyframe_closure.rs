//! Native keyframe closure diagnostics against recovered anchor boundaries.
//! Agreement measures this heuristic population; it does not prove that anchor
//! recovery found every record or replace the sequential native record stream.
use super::fire_events::{native_chunk_packets, native_chunk_prefix};
use super::{DecodeError, FilmRegistry, FrameEncoding, KeyframeStop};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeClosureStat {
    pub closed: usize,
    /// Excludes the last recovered record of each payload: it has no next anchor.
    pub total: usize,
    /// Most frequent failed component; lexical name order breaks frequency ties.
    pub blocking: String,
}

#[derive(Default)]
struct ClosureAccumulator {
    stats: BTreeMap<u32, KeyframeClosureStat>,
    blockers: BTreeMap<u32, BTreeMap<String, usize>>,
}
impl ClosureAccumulator {
    fn payload(
        &mut self,
        pay: &[u8],
        registry: &FilmRegistry,
        encoding: &FrameEncoding,
    ) -> Result<(), DecodeError> {
        let mut anchors = super::recover_keyframe_anchors(pay);
        super::native_sort::sort_by(&mut anchors, |a, b| a.bit.cmp(&b.bit));
        for pair in anchors.windows(2) {
            let a = &pair[0];
            let record =
                super::components::decode_native_keyframe_record(pay, a.bit, registry, encoding)
                    .ok_or(DecodeError::Missing("valid keyframe closure encoding"))?;
            let stat = self.stats.entry(a.archetype).or_default();
            stat.total += 1;
            let failed = match record.stop {
                KeyframeStop::Complete => None,
                KeyframeStop::UnsupportedComponent { index, .. } => Some(index),
                _ => Some(0),
            };
            if let Some(index) = failed {
                let name = registry
                    .archetype(a.archetype as usize)
                    .and_then(|a| a.components.get(index))
                    .map_or_else(|| format!("i{index}"), |name| format!("i{index} {name}"));
                *self
                    .blockers
                    .entry(a.archetype)
                    .or_default()
                    .entry(name)
                    .or_default() += 1;
            } else if record.end_bit == (pair[1].bit) as i64 {
                stat.closed += 1;
            }
        }
        Ok(())
    }
    fn finish(mut self) -> BTreeMap<u32, KeyframeClosureStat> {
        for (ti, blockers) in self.blockers {
            let mut best_count = 0;
            for (name, count) in blockers {
                if count > best_count {
                    self.stats.get_mut(&ti).unwrap().blocking = name;
                    best_count = count;
                }
            }
        }
        self.stats
    }
}

/// Native `KeyframeClosure`: all type-2 packets in native chunk order, with
/// format-27 MPP widths installed for this measurement only. Other inherited
/// settings remain intact. Empty films return an empty report.
/// The report does not add closure claims to decoded records or discard their fields.
pub fn measure_keyframe_closure(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> Result<BTreeMap<u32, KeyframeClosureStat>, DecodeError> {
    if registry.major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(
            registry.major_version as i32,
        ));
    }
    let mut encoding = encoding.clone();
    if let Some(widths) = super::resolve_film_mpp(registry.format_version).widths {
        encoding.mpp_widths = widths;
    }
    let mut out = ClosureAccumulator::default();
    // No contiguous native chunk prefix means an empty successful measurement.
    for chunk in native_chunk_prefix(chunks).unwrap_or_default() {
        for packet in native_chunk_packets(chunk) {
            if packet.packet_type == 2 {
                out.payload(
                    &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size],
                    registry,
                    &encoding,
                )?;
            }
        }
    }
    Ok(out.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::io::Read;
    #[test]
    fn native_keyframe_closure_reports() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/keyframe-closure-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 256);
        let (mut total, mut closed) = (0, 0);
        for (i, row) in rows.into_iter().enumerate() {
            let mut ranking = ClosureAccumulator::default();
            ranking.stats.insert(99, KeyframeClosureStat::default());
            ranking
                .blockers
                .insert(99, serde_json::from_value(row["ranking"].clone()).unwrap());
            assert_eq!(
                ranking.finish()[&99].blocking,
                row["winner"].as_str().unwrap(),
                "ranking {i}"
            );
            let registry = FilmRegistry {
                major_version: 41,
                format_version: row["format"].as_u64().unwrap() as u32,
                archetypes: serde_json::from_value(row["archetypes"].clone()).unwrap(),
                end_byte: 0,
                truncated: false,
            };
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: Some(row["simulation"].as_bool().unwrap()),
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: super::super::RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
                mpp_widths: serde_json::from_value(row["widths"].clone()).unwrap(),
                position: Some(serde_json::from_value(row["encoding"].clone()).unwrap()),
                extra_fields: false,
                corruption_check: row["check"].as_bool().unwrap(),
            };
            let before = encoding.clone();
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(j, hex)| {
                    let hex = hex.as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: j as i32 + 1,
                            chunk_type: 0,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let got = measure_keyframe_closure(&chunks, &registry, &encoding).unwrap();
            let expected: BTreeMap<u32, KeyframeClosureStat> =
                serde_json::from_value(row["stats"].clone()).unwrap();
            assert_eq!(got, expected, "case {i}");
            total += got.values().map(|s| s.total).sum::<usize>();
            closed += got.values().map(|s| s.closed).sum::<usize>();
            assert_eq!(before, encoding);
            assert_eq!(
                serde_json::from_value::<BTreeMap<u32, KeyframeClosureStat>>(
                    serde_json::to_value(&got).unwrap()
                )
                .unwrap(),
                got
            );
            assert!(
                measure_keyframe_closure(&[], &registry, &encoding)
                    .unwrap()
                    .is_empty()
            );
        }
        assert_eq!((total, closed), (17112, 6830));
    }
}
