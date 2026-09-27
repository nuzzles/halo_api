use super::*;
use serde_json::Value;
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_facts_mode_guards() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-guards-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 31975);
    for (i, row) in rows.iter().enumerate() {
        let source: FactsModeGuards = serde_json::from_value(row["source"].clone()).unwrap();
        let mut w = NativeFactsWriter::default();
        encode_facts_mode_guards(&mut w, &source);
        assert_eq!(
            w.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encoded {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let decoded = decode_facts_mode_guards(&mut r);
        if row["panic"].as_bool().unwrap_or(false) {
            assert!(r.error().is_some(), "safe refusal {i}");
            assert_eq!(
                r.offset(),
                row["offset"].as_u64().unwrap() as usize,
                "panic cursor {i}"
            );
            continue;
        }
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            row["decoded"],
            "decoded {i}"
        );
        assert_eq!(
            r.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        assert_eq!(
            r.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
    }
}
