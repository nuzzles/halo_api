use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_source_identity_indices_and_clock() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/source-identity-scans-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 256);
    let mut totals = [0; 4];
    for (case, row) in rows.iter().enumerate() {
        let inputs = row["inputs"].as_array().unwrap();
        let buffers: Vec<Vec<u8>> = inputs
            .iter()
            .map(|c| {
                let h = c["hex"].as_str().unwrap();
                (0..h.len())
                    .step_by(2)
                    .map(|p| u8::from_str_radix(&h[p..p + 2], 16).unwrap())
                    .collect()
            })
            .collect();
        let metadata: Vec<_> = inputs
            .iter()
            .map(|c| FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            })
            .collect();
        let meta = if row["no_metadata"] == true {
            &[][..]
        } else {
            &metadata[..]
        };
        let source = (row["nil_source"] != true).then(|| FilmSource::load(&buffers, meta).unwrap());
        let roster: Vec<u64> = serde_json::from_value(row["roster"].clone()).unwrap_or_default();
        let out = scan_source_player_indices(source.as_ref(), &roster);
        assert_eq!(json!(out.table), row["table"], "table {case}");
        assert_eq!(
            out.error
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            row["index_error"].as_str().unwrap(),
            "index error {case}"
        );
        let (filtered, collisions) = injective_player_indices(out.table.clone());
        assert_eq!(json!(filtered), row["filtered"], "filtered {case}");
        assert_eq!(
            collisions as u64,
            row["collisions"].as_u64().unwrap(),
            "collisions {case}"
        );
        let scanned: Vec<_> = out
            .chunks
            .iter()
            .map(|c| {
                let s = source.as_ref().unwrap();
                assert_eq!(s.chunk_position(c.chunk), Some(c.source_index));
                let data = s.chunk(c.source_index).unwrap();
                let reads: Vec<_> = c
                    .reads
                    .iter()
                    .map(|r| {
                        assert_eq!(r.index_start_bit, r.xuid_start_bit as i64 - 5);
                        assert_eq!(
                            r.leading_padding_bits,
                            5_usize.saturating_sub(r.xuid_start_bit)
                        );
                        assert!(r.xuid_start_bit + 64 <= data.len() * 8);
                        totals[2] += 1;
                        totals[3] += usize::from(r.leading_padding_bits > 0);
                        json!({"xuid":r.xuid,"index":r.index,"bit":r.xuid_start_bit})
                    })
                    .collect();
                json!({"chunk":c.chunk,"reads":reads})
            })
            .collect();
        assert_eq!(json!(scanned), row["scanned"], "source reads {case}");
        let clock = scan_source_clock_origin(source.as_ref());
        match clock {
            Ok(p) => {
                totals[0] += 1;
                assert_eq!(
                    p.timestamp_us,
                    row["clock"].as_u64().unwrap(),
                    "clock {case}"
                );
                assert_eq!(row["clock_error"], "");
                assert_eq!(
                    source.as_ref().unwrap().chunk_position(1),
                    Some(p.chunk_index as usize)
                );
                assert_eq!(
                    json!({"type":p.packet_type,"start":p.payload_offset,"size":p.payload_size,"index":0}),
                    row["clock_packet"],
                    "packet {case}"
                );
            }
            Err(e) => {
                totals[1] += 1;
                assert_eq!(
                    e.to_string(),
                    row["clock_error"].as_str().unwrap(),
                    "clock error {case}"
                );
            }
        }
    }
    assert!(totals.iter().all(|n| *n > 0), "{totals:?}");
}
