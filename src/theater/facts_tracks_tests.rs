use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn tracks(v: &Value) -> Vec<FactsProjectileTrack> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|tr| FactsProjectileTrack {
            slot: tr["slot"].as_u64().unwrap() as u32,
            generation: tr["generation"].as_u64().unwrap() as u32,
            points: tr["points"].as_array().map(|pts| {
                pts.iter()
                    .map(|p| FactsProjectileSample {
                        timestamp_us: p["timestamp_us"].as_u64().unwrap(),
                        chunk: p["chunk"].as_i64().unwrap(),
                        position: std::array::from_fn(|i| {
                            f32::from_bits(p["bits"][i].as_u64().unwrap() as u32)
                        }),
                        at_rest: p["at_rest"].as_bool().unwrap(),
                    })
                    .collect()
            }),
        })
        .collect()
}
fn dump(tracks: &[FactsProjectileTrack]) -> Value {
    json!(tracks.iter().map(|tr|json!({"slot":tr.slot,"generation":tr.generation,"points":tr.points.as_ref().map(|pts|pts.iter().map(|p|json!({"timestamp_us":p.timestamp_us,"chunk":p.chunk,"bits":p.position.map(f32::to_bits),"at_rest":p.at_rest})).collect::<Vec<_>>())})).collect::<Vec<_>>())
}
#[test]
fn native_facts_track_codec() {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/facts-tracks-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 23464);
    for (i, row) in rows.iter().enumerate() {
        let source = tracks(&row["source"]);
        let mut writer = NativeFactsWriter::default();
        encode_facts_tracks(&mut writer, &source);
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "bytes {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut reader = NativeFactsReader::new(&input);
        let decoded = decode_facts_tracks(&mut reader);
        assert_eq!(dump(&decoded), row["decoded"], "decoded {i}");
        assert_eq!(
            reader.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        assert_eq!(
            reader.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
        let again = decode_facts_tracks(&mut reader);
        assert_eq!(dump(&again), row["again"], "again {i}");
        assert_eq!(
            reader.offset(),
            row["after_offset"].as_u64().unwrap() as usize,
            "after offset {i}"
        );
        assert_eq!(
            reader.error().unwrap_or(""),
            row["after_error"].as_str().unwrap(),
            "after error {i}"
        );
    }
}
