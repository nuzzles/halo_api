//! Native public keyframe-position pattern probe and spatial team inference.
//! These heuristic observations are not record boundaries or player identities.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframePositionProbe {
    pub source: FilmPacket,
    /// Native probe uses chunk start time, not the embedded packet timestamp.
    pub time_ms: i64,
    pub comb_bit: usize,
    /// Native out-of-range bit reads are zero, including a prefix before bit zero.
    pub vector_start_bit: i64,
    pub position: [f32; 3],
    /// Spatial cluster label only; never a decoded team ID or player identity.
    pub inferred_team: Option<u8>,
}
fn first_keyframe(chunk: &crate::clients::hi::models::FilmChunkData) -> Option<FilmPacket> {
    let data = &chunk.data;
    let mut off = 0usize;
    while off.checked_add(16)? <= data.len() {
        let header = &data[off..off + 16];
        let typ = u16::from_le_bytes(header[0..2].try_into().ok()?);
        let size = u32::from_le_bytes(header[4..8].try_into().ok()?) as usize;
        let end = off.checked_add(16)?.checked_add(size)?;
        if end > data.len() {
            return None;
        }
        if typ == 2 {
            return Some(FilmPacket {
                chunk_index: chunk.metadata.index,
                packet_type: typ,
                byte_2: header[2],
                byte_3: header[3],
                payload_offset: off + 16,
                payload_size: size,
                timestamp_us: u64::from_le_bytes(header[8..16].try_into().ok()?),
            });
        }
        off = end;
        if typ == 7 {
            break;
        }
    }
    None
}
fn float_at(data: &[u8], bit: i64) -> f32 {
    let mut raw = 0u32;
    for i in 0..32 {
        raw = (raw << 1)
            | usize::try_from(bit + i)
                .ok()
                .and_then(|b| data.get(b / 8).map(|v| (v >> (7 - b % 8)) & 1))
                .unwrap_or(0) as u32;
    }
    f32::from_bits(raw.swap_bytes())
}
fn structural(v: [f32; 3]) -> bool {
    let near = |a: f32, b: f32| f64::from(a - b).abs() <= 0.1;
    (near(v[0], 0.) && near(v[1], 2.) && near(v[2], 0.))
        || (near(v[0], -2.1) && near(v[1], 0.) && near(v[2], 0.))
}
fn infer_teams(points: &mut [KeyframePositionProbe]) {
    if points.len() < 6 {
        return;
    }
    let mut order: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.position[0]))
        .collect();
    order.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let span = f64::from(order.last().unwrap().1 - order[0].1);
    if span <= 0. {
        return;
    }
    let mut gap = 0.;
    let mut split = 0;
    for rank in 1..order.len() {
        let g = f64::from(order[rank].1 - order[rank - 1].1);
        if g > gap {
            gap = g;
            split = rank;
        }
    }
    if split < 3 || order.len() - split < 3 || gap < 0.35 * span {
        return;
    }
    for (rank, (i, _)) in order.into_iter().enumerate() {
        points[i].inferred_team = Some(u8::from(rank >= split));
    }
}
/// Port of native positions.DecodeKeyframePositions. Reads only the first type-2
/// packet of each type-2 chunk, and stops packet search at type 7 or invalid bounds.
/// The comb pattern, -273-bit offset, magnitude/artifact filters and team split
/// are native heuristics retained for parity, with their provenance kept explicit.
pub fn scan_keyframe_position_probes(
    chunks: &[crate::clients::hi::models::FilmChunkData],
) -> Vec<KeyframePositionProbe> {
    let mut out = vec![];
    for chunk in chunks {
        if chunk.metadata.chunk_type != 2 {
            continue;
        }
        let Some(source) = first_keyframe(chunk) else {
            continue;
        };
        let payload =
            &chunk.data[source.payload_offset..source.payload_offset + source.payload_size];
        let start = out.len();
        let mut bit = 0usize;
        while bit + 96 <= payload.len() * 8 {
            // Native combAt requires another byte beyond the pattern's end.
            let matches = (bit + 96) / 8 < payload.len()
                && (0..4)
                    .all(|r| super::bits::Bits(payload).read(bit + r * 24, 24) == Some(0xff0000));
            if !matches {
                bit += 1;
                continue;
            }
            let at = bit as i64 - 273;
            let position = std::array::from_fn(|i| float_at(payload, at + 32 * i as i64));
            if position
                .iter()
                .all(|v| v.is_finite() && *v > -200. && *v < 200.)
                && position.iter().map(|v| f64::from(*v).abs()).sum::<f64>() >= 1.
                && !structural(position)
            {
                out.push(KeyframePositionProbe {
                    source,
                    time_ms: chunk.metadata.start_time_offset_ms,
                    comb_bit: bit,
                    vector_start_bit: at,
                    position,
                    inferred_team: None,
                });
            }
            bit += 96;
        }
        infer_teams(&mut out[start..]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::{FilmChunk, FilmChunkData};
    use std::io::Read;
    fn unhex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect()
    }
    fn compare(out: &[KeyframePositionProbe], expected: &serde_json::Value) {
        let rows = expected.as_array().unwrap();
        assert_eq!(out.len(), rows.len());
        for (p, r) in out.iter().zip(rows) {
            assert_eq!(p.time_ms, r["TimeMS"].as_i64().unwrap());
            assert_eq!(
                p.inferred_team.map(i64::from).unwrap_or(-1),
                r["Team"].as_i64().unwrap()
            );
            for (i, k) in ["X", "Y", "Z"].iter().enumerate() {
                assert_eq!(
                    p.position[i].to_bits(),
                    (r[k].as_f64().unwrap() as f32).to_bits()
                );
            }
        }
    }
    #[test]
    fn native_keyframe_position_probe() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/keyframe-position-probe-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_slice(&data).unwrap();
        let mut accepted = 0;
        for (i, c) in cases.iter().enumerate() {
            let chunk = FilmChunkData {
                metadata: FilmChunk {
                    index: i as i32,
                    chunk_type: c["type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start"].as_i64().unwrap(),
                    duration_ms: 20000,
                    size: 0,
                    file_relative_path: String::new(),
                },
                data: unhex(c["hex"].as_str().unwrap()),
            };
            let out = scan_keyframe_position_probes(&[chunk]);
            compare(&out, &c["out"]);
            accepted += out.len();
            let restored: Vec<KeyframePositionProbe> =
                serde_json::from_slice(&serde_json::to_vec(&out).unwrap()).unwrap();
            assert_eq!(restored, out);
            let payload = unhex(c["payload"].as_str().unwrap());
            for (j, v) in c["reads"].as_array().unwrap().iter().enumerate() {
                assert_eq!(
                    float_at(&payload, j as i64 - 40).to_bits(),
                    v.as_u64().unwrap() as u32,
                    "read {i}/{j}"
                )
            }
            let mut points: Vec<_> = c["xs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| KeyframePositionProbe {
                    source: FilmPacket {
                        chunk_index: 0,
                        packet_type: 2,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 16,
                        payload_size: 0,
                        timestamp_us: 0,
                    },
                    time_ms: 0,
                    comb_bit: 0,
                    vector_start_bit: 0,
                    position: [x.as_f64().unwrap() as f32, 0., 0.],
                    inferred_team: None,
                })
                .collect();
            infer_teams(&mut points);
            for (p, t) in points.iter().zip(c["teams"].as_array().unwrap()) {
                assert_eq!(
                    p.inferred_team.map(i64::from).unwrap_or(-1),
                    t["Team"].as_i64().unwrap(),
                    "teams {i}"
                )
            }
        }
        assert!(accepted > 100);
        eprintln!("matched {accepted} native position probes");
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_keyframe_position_corpus() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/keyframe-position-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_slice(&data).unwrap();
        for c in cases {
            let folder = c["folder"].as_str().unwrap();
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
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
            let out = scan_keyframe_position_probes(&chunks);
            compare(&out, &c["out"]);
            if folder == "maps/01-bazaar-idle" {
                let film = LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap();
                assert_eq!(film.keyframe_position_probes, out);
                assert!(!out.is_empty());
                let restored: LegacyFilm =
                    serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                assert_eq!(restored.keyframe_position_probes, out);
            }
            eprintln!("matched {folder}: {} probes", out.len());
        }
    }
}
