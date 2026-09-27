//! Native legacy weapon-pattern probes. Pattern matches and estimated timestamps
//! are observations, not decoded record boundaries, hits, or player identities.
use super::bits::Bits;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

pub fn v41_filmshell_catalog() -> &'static BTreeMap<u64, String> {
    static CATALOG: OnceLock<BTreeMap<u64, String>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("reference/filmshell-catalog-v41.json"))
            .expect("validated pinned filmshell catalog")
    })
}
const COMMON_SUFFIX: [u8; 4] = [0x42, 0xc9, 0x67, 0x9f];
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponFormulaProbe {
    pub offset: usize,
    pub film_index: u8,
    pub weapon_bytes: [u8; 8],
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponFireProbe {
    #[serde(rename = "TimestampMS")]
    pub timestamp_ms: f64,
    /// Historical four-bit truncation; retain the five-bit value for joins.
    pub film_index: u8,
    pub film_index5: u8,
    pub slot: u8,
    pub b5: u8,
    pub weapon_name: String,
    pub weapon_bytes: [u8; 8],
    pub fire_seq: u8,
    pub fire_counter: u8,
    pub byte_pos: usize,
    pub burst_end: bool,
    pub hit_likely: Option<bool>,
    pub chunk_idx: i32,
}
/// Byte occurrences of the native A0 7B 42 frame marker.
pub fn weapon_frame_marker_positions(data: &[u8]) -> Vec<usize> {
    data.windows(3)
        .enumerate()
        .filter_map(|(i, w)| (w == [0xa0, 0x7b, 0x42]).then_some(i))
        .collect()
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponPatternTimestampEstimator {
    pub frame_positions: Vec<usize>,
    pub start_ms: i64,
    pub duration_ms: i64,
}
impl WeaponPatternTimestampEstimator {
    pub fn new(data: &[u8], start_ms: i64, duration_ms: i64) -> Self {
        Self {
            frame_positions: weapon_frame_marker_positions(data),
            start_ms,
            duration_ms,
        }
    }
    pub fn estimate_ms(&self, byte_pos: i64) -> f64 {
        let n = self.frame_positions.len();
        let step = if n > 0 {
            self.duration_ms as f64 / n as f64
        } else {
            16.67
        };
        let rank = self
            .frame_positions
            .partition_point(|&b| byte_pos >= 0 && b as u64 <= byte_pos as u64)
            .saturating_sub(1);
        // The pinned Go reference fuses this multiply/add on its native target.
        (rank as f64).mul_add(step, self.start_ms as f64)
    }
}
/// Native Formula A: first occurrence of each suffix within the next 64 bytes.
/// An early invalid occurrence suppresses later occurrences of that same suffix.
pub fn scan_weapon_formula_a(data: &[u8]) -> Vec<WeaponFormulaProbe> {
    let suffixes: BTreeSet<[u8; 4]> = v41_filmshell_catalog()
        .keys()
        .map(|id| id.to_be_bytes()[4..].try_into().unwrap())
        .collect();
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(idx) = data[pos..].windows(3).position(|w| w == [0x20, 0, 2]) {
        let at = pos + idx;
        if at + 4 > data.len() {
            break;
        }
        let end = (at + 68).min(data.len());
        let mut best = None;
        for suffix in &suffixes {
            let Some(relative) = data[at + 4..end].windows(4).position(|w| w == suffix) else {
                continue;
            };
            let sx = at + 4 + relative;
            let ws = sx - 4;
            if ws <= at + 3 {
                continue;
            }
            let bytes: [u8; 8] = data[ws..ws + 8].try_into().unwrap();
            if *suffix != COMMON_SUFFIX
                && !v41_filmshell_catalog().contains_key(&u64::from_be_bytes(bytes))
            {
                continue;
            }
            if best.is_none_or(|old| sx < old) {
                best = Some(sx)
            }
        }
        if let Some(sx) = best {
            out.push(WeaponFormulaProbe {
                offset: at,
                film_index: data[at + 3] >> 5,
                weapon_bytes: data[sx - 4..sx + 4].try_into().unwrap(),
            })
        }
        pos = at + 4;
    }
    out
}
/// Native nibble-shifted known-ID scan; offsets refer to the shifted byte layer.
pub fn scan_weapon_formula_a_nibble(data: &[u8]) -> Vec<WeaponFormulaProbe> {
    let shifted: Vec<u8> = data.windows(2).map(|w| (w[0] << 4) | (w[1] >> 4)).collect();
    let mut out = Vec::new();
    let catalog = v41_filmshell_catalog();
    let mut final_bytes = [false; 256];
    for &id in catalog.keys() {
        final_bytes[id as u8 as usize] = true;
    }
    for (at, w) in shifted.windows(8).enumerate() {
        if at < 5 || shifted[at - 5] == 0x26 || !final_bytes[w[7] as usize] {
            continue;
        }
        let bytes: [u8; 8] = w.try_into().unwrap();
        if catalog.contains_key(&u64::from_be_bytes(bytes)) {
            out.push(WeaponFormulaProbe {
                offset: at,
                film_index: shifted[at - 1] >> 5,
                weapon_bytes: bytes,
            });
        }
    }
    out.sort_by_key(|r| r.offset);
    out
}
/// Universal 11-bit marker scan, native two-byte deduplication and timestamp sort.
/// Caller supplies the timestamp policy independently of packet decoding.
pub fn scan_weapon_fire_b5(
    data: &[u8],
    mut estimate: impl FnMut(i64) -> f64,
) -> Vec<WeaponFireProbe> {
    let bits = Bits(data);
    let mut out = Vec::new();
    if bits.len() < 115 {
        return out;
    }
    for bit in 0..=bits.len() - 115 {
        let byte = bit / 8;
        // The native 115-bit scan guard guarantees all three bytes are in bounds.
        let word = (u32::from(data[byte]) << 16)
            | (u32::from(data[byte + 1]) << 8)
            | u32::from(data[byte + 2]);
        if ((word >> (13 - bit % 8)) & 0x7ff) != 0b10100100110 {
            continue;
        }
        let start = bit + 3;
        let weapon = bits.read(start + 40, 64).unwrap();
        let bytes = weapon.to_be_bytes();
        let name = v41_filmshell_catalog().get(&weapon);
        if name.is_none() && bytes[4..] != COMMON_SUFFIX {
            continue;
        }
        let b5 = bits.read(start + 32, 8).unwrap() as u8;
        let post = start + 104;
        let (burst_end, hit_likely) = if post + 32 <= bits.len() {
            (
                bits.read(post + 8, 8).unwrap() & 1 != 0,
                Some(bits.read(post + 16, 8).unwrap() & 1 == 0),
            )
        } else {
            (false, None)
        };
        out.push(WeaponFireProbe {
            timestamp_ms: estimate((bit / 8) as i64),
            film_index: b5 >> 4,
            film_index5: bits.read(start + 31, 5).unwrap() as u8,
            slot: b5 & 3,
            b5,
            weapon_name: name.cloned().unwrap_or_else(|| "INCONNU".into()),
            weapon_bytes: bytes,
            fire_seq: bits.read(start + 8, 8).unwrap() as u8,
            fire_counter: bits.read(start + 24, 8).unwrap() as u8,
            byte_pos: bit / 8,
            burst_end,
            hit_likely,
            chunk_idx: 0,
        });
    }
    super::native_sort::sort_by(&mut out, |a, b| a.byte_pos.cmp(&b.byte_pos));
    let mut last = None;
    out.retain(|ev| {
        if last.is_none_or(|p| ev.byte_pos > p + 2) {
            last = Some(ev.byte_pos);
            true
        } else {
            false
        }
    });
    super::native_sort::sort_by(&mut out, |a, b| {
        a.timestamp_ms
            .partial_cmp(&b.timestamp_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}
/// Per-chunk native legacy probes, retaining the byte-offset coordinate system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponPatternChunk {
    pub chunk_index: i32,
    pub timing: WeaponPatternTimestampEstimator,
    pub formula_a: Vec<WeaponFormulaProbe>,
    pub formula_a_nibble: Vec<WeaponFormulaProbe>,
    pub fire: Vec<WeaponFireProbe>,
}
pub fn scan_weapon_patterns(
    chunks: &[crate::clients::hi::models::FilmChunkData],
) -> Vec<WeaponPatternChunk> {
    chunks
        .iter()
        .map(|c| {
            let timing = WeaponPatternTimestampEstimator::new(
                &c.data,
                c.metadata.start_time_offset_ms,
                c.metadata.duration_ms,
            );
            let mut fire = scan_weapon_fire_b5(&c.data, |b| timing.estimate_ms(b));
            for f in &mut fire {
                f.chunk_idx = c.metadata.index
            }
            WeaponPatternChunk {
                chunk_index: c.metadata.index,
                formula_a: scan_weapon_formula_a(&c.data),
                formula_a_nibble: scan_weapon_formula_a_nibble(&c.data),
                fire,
                timing,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    fn unhex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn native_weapon_patterns() {
        #[derive(Deserialize)]
        struct Query {
            pos: i64,
            ms: f64,
        }
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            start: i64,
            duration: i64,
            queries: Vec<Query>,
            frames: Vec<usize>,
            formula: Vec<WeaponFormulaProbe>,
            nibble: Vec<WeaponFormulaProbe>,
            fire: Vec<WeaponFireProbe>,
            custom_calls: Vec<i64>,
            custom_fire: Vec<WeaponFireProbe>,
        }
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/weapon-patterns-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        let mut counts = [0; 3];
        for (i, c) in cases.into_iter().enumerate() {
            let bytes = unhex(&c.hex);
            let timing = WeaponPatternTimestampEstimator::new(&bytes, c.start, c.duration);
            assert_eq!(timing.frame_positions, c.frames, "frames {i}");
            for q in c.queries {
                assert_eq!(timing.estimate_ms(q.pos), q.ms, "timestamp {i}")
            }
            assert_eq!(scan_weapon_formula_a(&bytes), c.formula, "formula {i}");
            assert_eq!(scan_weapon_formula_a_nibble(&bytes), c.nibble, "nibble {i}");
            let fire = scan_weapon_fire_b5(&bytes, |b| timing.estimate_ms(b));
            assert_eq!(fire, c.fire, "fire {i}");
            let mut calls = Vec::new();
            let custom = scan_weapon_fire_b5(&bytes, |pos| {
                calls.push(pos);
                ((calls.len() as i64 * 17 + pos) % 7 - 3) as f64
            });
            assert_eq!(calls, c.custom_calls, "callback calls {i}");
            assert_eq!(custom, c.custom_fire, "callback publication {i}");
            let restored: Vec<WeaponFireProbe> =
                serde_json::from_slice(&serde_json::to_vec(&fire).unwrap()).unwrap();
            assert_eq!(restored, fire);
            counts[0] += c.formula.len();
            counts[1] += c.nibble.len();
            counts[2] += c.fire.len();
        }
        eprintln!("matched formula/nibble/fire: {counts:?}");
        assert!(counts.iter().all(|&n| n > 100));
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_weapon_patterns_corpus() {
        use crate::{
            clients::hi::models::{FilmChunk, FilmChunkData},
            theater::{DecodeOptions, LegacyFilm},
        };
        #[derive(Deserialize)]
        struct Chunk {
            index: i32,
            formula: Vec<WeaponFormulaProbe>,
            nibble: Vec<WeaponFormulaProbe>,
            fire: Vec<WeaponFireProbe>,
            frames: Vec<usize>,
        }
        #[derive(Deserialize)]
        struct Case {
            folder: String,
            chunks: Vec<Chunk>,
        }
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/weapon-patterns-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        for c in cases {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(&c.folder);
            let meta: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("film.json")).unwrap()).unwrap();
            let chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: c["duration_ms"].as_i64().unwrap(),
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: std::fs::read(root.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let scans = scan_weapon_patterns(&chunks);
            assert_eq!(scans.len(), c.chunks.len());
            for (r, expected) in scans.iter().zip(&c.chunks) {
                assert_eq!(r.chunk_index, expected.index);
                assert_eq!(
                    r.formula_a, expected.formula,
                    "formula {} chunk {}",
                    c.folder, r.chunk_index
                );
                assert_eq!(
                    r.formula_a_nibble, expected.nibble,
                    "nibble {} chunk {}",
                    c.folder, r.chunk_index
                );
                assert_eq!(
                    r.fire, expected.fire,
                    "fire {} chunk {}",
                    c.folder, r.chunk_index
                );
                assert_eq!(r.timing.frame_positions, expected.frames)
            }
            if c.folder == "maps/01-bazaar-idle" {
                let film = LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap();
                assert_eq!(film.weapon_patterns, scans);
                assert!(scans.iter().any(|s| !s.fire.is_empty()));
                let restored: LegacyFilm =
                    serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                assert_eq!(restored.weapon_patterns, scans);
            }
            eprintln!("matched weapon patterns: {}", c.folder);
        }
    }
}
