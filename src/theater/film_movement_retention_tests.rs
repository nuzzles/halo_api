use super::*;
use crate::clients::hi::models::{FilmChunk, FilmChunkData};
use serde_json::Value;
use std::io::Read;
#[test]
fn explicit_film_constructor_retains_native_movement() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/movement-positive-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 32);
    let mut count = 0;
    for row in rows {
        let mut bootstrap = vec![0; 8 + 37 * 64 * 260];
        bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
        bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
        let name = b"unit-crouch-component";
        let at = 8 + 35 * 64 * 260;
        bootstrap[at..at + name.len()].copy_from_slice(name);
        bootstrap[8 + 36 * 64 * 260] = 0xff;
        let chunks: Vec<_> = row["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|input| {
                let index = input["index"].as_i64().unwrap() as i32;
                let hex = input["hex"].as_str().unwrap();
                let data = if index == 0 {
                    bootstrap.clone()
                } else {
                    (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect()
                };
                FilmChunkData {
                    metadata: FilmChunk {
                        index,
                        chunk_type: if index == 0 { 1 } else { 2 },
                        start_time_offset_ms: 0,
                        duration_ms: 1,
                        size: data.len() as i64,
                        file_relative_path: String::new(),
                    },
                    data,
                }
            })
            .collect();
        let encoding:FrameEncoding=serde_json::from_value(serde_json::json!({"ids":{"low_bits":13,"base":0},"mpp_widths":[9,5],"extra_fields":false,"corruption_check":false,"position":row["encoding"]})).unwrap();
        let film =
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding)
                .unwrap();
        assert!(
            film.movement_states_error.is_none(),
            "{:?}",
            film.movement_states_error
        );
        assert!(film.movement_states_error_stats.is_none());
        let movement = film.movement_states.as_ref().unwrap();
        assert_eq!(serde_json::json!(movement.records), row["reads"]);
        assert_eq!(serde_json::json!(movement.stats), row["stats"]);
        count += movement.records.len();
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored, film);
    }
    assert_eq!(count, 64);
}
