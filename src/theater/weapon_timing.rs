//! Native byte-position timestamp estimator for weapon observations.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponTimestampFrame {
    pub start_byte: usize,
    pub end_byte: usize,
    pub timestamp_us: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponTimestampEstimator {
    pub start_ms: i64,
    pub frames: Vec<WeaponTimestampFrame>,
}
impl WeaponTimestampEstimator {
    pub fn new(chunk: &[u8], start_ms: i64) -> Self {
        Self {
            start_ms,
            frames: super::fire_events::native_packet_bytes(chunk, 0)
                .into_iter()
                .filter(|p| p.packet_type == 0)
                .map(|p| WeaponTimestampFrame {
                    start_byte: p.payload_offset,
                    end_byte: p.payload_offset + p.payload_size,
                    timestamp_us: p.timestamp_us,
                })
                .collect(),
        }
    }
    /// Before the first frame, use its timestamp. Gaps and bytes past the final
    /// payload use the preceding frame, as does the native binary search.
    pub fn estimate_ms(&self, byte_position: i64) -> f64 {
        let Some(first) = self.frames.first() else {
            return self.start_ms as f64;
        };
        let index = self
            .frames
            .partition_point(|f| byte_position >= 0 && f.start_byte as u64 <= byte_position as u64)
            .saturating_sub(1);
        self.start_ms as f64
            + (self.frames[index]
                .timestamp_us
                .wrapping_sub(first.timestamp_us) as i64) as f64
                / 1000.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{
        resolve_best_player_indices, resolve_player_index_strings, resolve_player_indices,
    };
    use std::{collections::BTreeMap, io::Read};
    #[test]
    fn native_weapon_roster_and_timing_helpers() {
        #[derive(Deserialize)]
        struct Query {
            byte: i64,
            ms: f64,
        }
        #[derive(Deserialize)]
        struct Canon {
            id: u64,
            high: u32,
            known: bool,
            name: String,
        }
        #[derive(Deserialize)]
        struct Case {
            families: BTreeMap<u32, String>,
            pi_bits: usize,
            priority_xuid: u64,
            priority_chunks: Vec<String>,
            priority_resolved: BTreeMap<u64, i64>,
            canon: Vec<Canon>,
            roster: Vec<u64>,
            strings: Vec<String>,
            chunks: Vec<String>,
            resolved: BTreeMap<u64, i64>,
            string_resolved: BTreeMap<String, i64>,
            best: BTreeMap<u64, i64>,
            packet_hex: String,
            start: i64,
            queries: Vec<Query>,
            frames: Vec<WeaponTimestampFrame>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/weapon-helpers-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 512);
        let bytes = |s: &str| {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(crate::theater::PLAYER_INDEX_BITS, c.pi_bits);
            assert_eq!(crate::theater::v41_weapon_families(), &c.families);
            let priority: Vec<_> = c.priority_chunks.iter().map(|s| bytes(s)).collect();
            let roster = [c.priority_xuid, c.priority_xuid];
            let first = resolve_player_indices(&roster, &priority[0]);
            let second = resolve_player_indices(&roster, &priority[1]);
            assert_ne!(first, second, "positive conflict {i}");
            assert_eq!(first, c.priority_resolved, "native first wins {i}");
            assert_eq!(
                resolve_best_player_indices(
                    &roster,
                    &priority.iter().map(Vec::as_slice).collect::<Vec<_>>()
                ),
                c.priority_resolved,
                "merge conflict {i}"
            );
            for id in c.canon {
                assert_eq!(
                    crate::theater::canonical_weapon_id(id.id),
                    (id.high, id.known),
                    "canon {i}"
                );
                assert_eq!(
                    crate::theater::canonical_weapon_name(id.high),
                    id.name,
                    "name {i}"
                );
            }
            let chunks: Vec<_> = c.chunks.iter().map(|s| bytes(s)).collect();
            assert_eq!(
                resolve_player_indices(&c.roster, &chunks[0]),
                c.resolved,
                "numeric {i}"
            );
            assert_eq!(
                resolve_player_index_strings(&c.strings, &chunks[0]),
                c.string_resolved,
                "string {i}"
            );
            assert_eq!(
                resolve_best_player_indices(
                    &c.roster,
                    &chunks.iter().map(Vec::as_slice).collect::<Vec<_>>()
                ),
                c.best,
                "best {i}"
            );
            let estimator = WeaponTimestampEstimator::new(&bytes(&c.packet_hex), c.start);
            assert_eq!(estimator.frames, c.frames, "frames {i}");
            for q in c.queries {
                assert_eq!(estimator.estimate_ms(q.byte), q.ms, "time {i}/{}", q.byte);
            }
        }
    }
}
