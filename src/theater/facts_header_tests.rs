use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_facts_header_codec() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-header-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    for (i, row) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let source: FactsHeader = serde_json::from_value(row["source"].clone()).unwrap();
        let mut w = NativeFactsWriter::default();
        encode_facts_header(&mut w, &source);
        let mut encoded = FILM_FACTS_MAGIC.to_vec();
        encoded.extend_from_slice(w.bytes());
        assert_eq!(
            encoded,
            unhex(row["encoded"].as_str().unwrap()),
            "encode {i}"
        );
        let e = &row["entry"];
        let bits: [[u32; 2]; 3] = serde_json::from_value(e["bounds"].clone()).unwrap();
        let entry = FactsMapEntry {
            module: serde_json::from_value(e["module"].clone()).unwrap(),
            axis_widths: serde_json::from_value(e["widths"].clone()).unwrap(),
            region: e["region"].as_u64().unwrap() as u32,
            region_index_bits: e["index"].as_u64().unwrap(),
            bounds: bits.map(|a| a.map(f32::from_bits)),
        };
        let input = unhex(row["input"].as_str().unwrap());
        match decode_facts_header(&input, &entry) {
            Err(err) => {
                let kind = match err {
                    FactsHeaderError::Magic => "magic",
                    FactsHeaderError::Map(_) => "map",
                    FactsHeaderError::Layout(_) => "layout",
                };
                assert_eq!(kind, row["kind"], "kind {i}");
                assert_eq!(err.to_string(), row["error"], "error {i}");
                assert!(row["header"].is_null());
            }
            Ok(got) => {
                assert_eq!(row["kind"], "", "admission {i}");
                assert_eq!(json!(got.header), row["header"], "header {i}");
                assert_eq!(got.reader.offset(), row["offset"], "cursor {i}");
                assert_eq!(
                    got.reader.error().unwrap_or(""),
                    row["transport"],
                    "transport {i}"
                );
                assert_eq!(
                    json!(got.layout.axis_widths),
                    row["layout"]["AxisW"],
                    "axes {i}"
                );
                assert_eq!(got.layout.gate_bits, row["layout"]["GateBits"], "gate {i}");
                assert_eq!(got.layout.region, row["layout"]["Region"], "region {i}");
                assert_eq!(
                    json!(got.bounds.map(|a| a.map(f32::to_bits))),
                    row["bounds"],
                    "bounds {i}"
                );
            }
        }
    }
    let entry = FactsMapEntry::default();
    for row in fixture["quotes"].as_array().unwrap() {
        let bytes: Vec<u8> = serde_json::from_value(row["bytes"].clone()).unwrap();
        assert_eq!(
            verify_facts_cooking_key(&bytes, [0; 3], false, &entry)
                .unwrap_err()
                .to_string(),
            row["error"]
        );
    }
    let mut hash = 0xcbf29ce484222325u64;
    for c in (1..=0x10ffff).filter_map(char::from_u32) {
        let mut bytes = [0; 4];
        let error =
            verify_facts_cooking_key(c.encode_utf8(&mut bytes).as_bytes(), [0; 3], false, &entry)
                .unwrap_err()
                .to_string();
        for b in error.bytes().chain([0]) {
            hash = (hash ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    }
    assert_eq!(hash, fixture["unicode_hash"].as_u64().unwrap());
}
