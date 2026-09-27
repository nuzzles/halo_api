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
fn native_facts_identity_fallback_json() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-identity-json-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let f: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(f["reads"].as_array().unwrap().len(), 1457);
    assert_eq!(f["writers"].as_array().unwrap().len(), 512);
    for (i, row) in f["writers"].as_array().unwrap().iter().enumerate() {
        let bytes = if row["kind"] == "identity" {
            let v: Option<FactsFileIdentity> =
                serde_json::from_value(row["source"].clone()).unwrap();
            encode_facts_identity_json(v.as_ref())
        } else {
            let v: Option<Vec<FactsFallback>> =
                serde_json::from_value(row["source"].clone()).unwrap();
            encode_facts_fallbacks_json(v.as_deref())
        };
        assert_eq!(bytes, unhex(row["encoded"].as_str().unwrap()), "writer {i}");
    }
    for (i, row) in f["reads"].as_array().unwrap().iter().enumerate() {
        let mut identity = FactsIdentityJsonReader::default();
        let mut fallbacks = FactsFallbacksJsonReader::default();
        for (j, step) in row["steps"].as_array().unwrap().iter().enumerate() {
            let bytes = unhex(step["input"].as_str().unwrap());
            let got = if row["kind"] == "identity" {
                identity.read(&bytes).map(|v| json!(v))
            } else {
                fallbacks.read(&bytes).map(|v| json!(v))
            };
            match got {
                Ok(v) => {
                    assert_eq!(step["error"], "", "success {i}/{j}");
                    assert_eq!(v, step["decoded"], "fields {i}/{j}");
                }
                Err(e) => {
                    assert_eq!(e, step["error"], "error {i}/{j}");
                }
            }
        }
    }
}
