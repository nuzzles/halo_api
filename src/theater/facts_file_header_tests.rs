use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn error(e: Option<&FactsFileHeaderError>) -> (&'static str, Vec<u8>) {
    match e {
        None => ("", Vec::new()),
        Some(e) => {
            let k = match e {
                FactsFileHeaderError::Version(_) => "version",
                FactsFileHeaderError::Revisions(_) => "revisions",
                FactsFileHeaderError::Cooking(FactsHeaderError::Map(_)) => "map",
                FactsFileHeaderError::Cooking(FactsHeaderError::Layout(_)) => "layout",
                _ => "other",
            };
            (k, e.message_bytes())
        }
    }
}
#[test]
fn native_facts_file_header_codec() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-file-header-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let f: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(f["coverage"].as_array().unwrap().len(), 6201);
    assert_eq!(f["headers"].as_array().unwrap().len(), 8492);
    assert_eq!(f["usability"].as_array().unwrap().len(), 448);
    for (i, row) in f["coverage"].as_array().unwrap().iter().enumerate() {
        let c = serde_json::from_value(row["source"].clone()).unwrap();
        let mut w = NativeFactsWriter::default();
        encode_facts_decoder_coverage(&mut w, &c);
        assert_eq!(
            w.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "coverage encode {i}"
        );
        let input = unhex(row["input"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let c = decode_facts_decoder_coverage(&mut r);
        assert_eq!(json!(c), row["decoded"], "coverage {i}");
        assert_eq!(r.offset(), row["offset"], "coverage offset {i}");
        assert_eq!(r.error().unwrap_or(""), row["error"], "coverage error {i}");
    }
    let entry = FactsMapEntry {
        module: b"map".to_vec(),
        axis_widths: [13, 14, 15],
        ..Default::default()
    };
    for (i, row) in f["headers"].as_array().unwrap().iter().enumerate() {
        let h = serde_json::from_value(row["source"].clone()).unwrap();
        let mut w = NativeFactsWriter::default();
        encode_facts_file_header(&mut w, &h);
        assert_eq!(
            w.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "header encode {i}"
        );
        let input = unhex(row["input"].as_str().unwrap());
        let got = decode_facts_file_header(&input);
        assert_eq!(json!(got.header), row["decoded"], "header {i}");
        let (kind, message) = error(got.error.as_ref());
        assert_eq!(kind, row["kind"], "kind {i}");
        assert_eq!(json!(message), row["error"], "header error {i}");
        let (kind, message) = error(got.header.usable(&entry).err().as_ref());
        assert_eq!(kind, row["usable_kind"], "usable kind {i}");
        assert_eq!(json!(message), row["usable_error"], "usable error {i}");
    }
    for (i, row) in f["usability"].as_array().unwrap().iter().enumerate() {
        let h: FactsFileHeader = serde_json::from_value(row["header"].clone()).unwrap();
        let (kind, message) = error(h.usable(&entry).err().as_ref());
        assert_eq!(kind, row["kind"], "freshness kind {i}");
        assert_eq!(json!(message), row["error"], "freshness {i}");
    }
}
