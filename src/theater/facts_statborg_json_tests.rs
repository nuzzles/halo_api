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
fn native_facts_statborg_json() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-statborg-json-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let f: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(f["writers"].as_array().unwrap().len(), 256);
    for (i, row) in f["writers"].as_array().unwrap().iter().enumerate() {
        let v: FactsFileStatborg = serde_json::from_value(row["source"].clone()).unwrap();
        assert_eq!(
            encode_facts_statborg_json(&v),
            unhex(row["encoded"].as_str().unwrap()),
            "writer {i}"
        );
    }
    for (i, row) in f["reads"].as_array().unwrap().iter().enumerate() {
        let mut reader = FactsStatborgJsonReader::default();
        for (j, step) in row["steps"].as_array().unwrap().iter().enumerate() {
            let bytes = unhex(step["input"].as_str().unwrap());
            match reader.read(&bytes) {
                Ok(v) => {
                    assert_eq!(step["error"], "", "success {i}/{j}");
                    assert_eq!(
                        encode_facts_statborg_json(&v),
                        unhex(step["encoded"].as_str().unwrap()),
                        "read {i}/{j}"
                    );
                }
                Err(e) => {
                    assert_eq!(
                        e,
                        step["error"].as_str().unwrap(),
                        "error {i}/{j}: {}",
                        String::from_utf8_lossy(&bytes)
                    );
                    assert!(reader.read(b"{}").is_err());
                    break;
                }
            }
        }
    }
}
