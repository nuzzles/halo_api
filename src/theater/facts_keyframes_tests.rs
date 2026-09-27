use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn read_fixture(data: &[u8]) -> Vec<Value> {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(data)
        .read_to_end(&mut raw)
        .unwrap();
    serde_json::from_slice(&raw).unwrap()
}
fn frames(v: &Value) -> FactsWorldKeyframes {
    FactsWorldKeyframes {
        times_us: serde_json::from_value(v["times"].clone()).unwrap(),
        seen_us: v["seen"]
            .as_array()
            .unwrap()
            .iter()
            .map(|life| {
                (
                    (
                        life["slot"].as_u64().unwrap() as u32,
                        life["generation"].as_u64().unwrap() as u32,
                    ),
                    serde_json::from_value(life["times"].clone()).unwrap(),
                )
            })
            .collect(),
    }
}
fn dump(v: &FactsWorldKeyframes) -> Value {
    json!({"times":v.times_us,"seen":v.seen_us.iter().map(|(&(slot,generation),times)|json!({"slot":slot,"generation":generation,"times":times})).collect::<Vec<_>>()})
}
#[test]
fn native_facts_keyframe_codec() {
    let rows = read_fixture(include_bytes!(
        "fixtures/facts-keyframe-codec-v41.json.zlib"
    ));
    assert_eq!(rows.len(), 11792);
    for (i, row) in rows.iter().enumerate() {
        let mut writer = NativeFactsWriter::default();
        encode_facts_keyframes(&mut writer, &frames(&row["source"]));
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encoded {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut reader = NativeFactsReader::new(&input);
        let decoded = decode_facts_keyframes(&mut reader);
        assert_eq!(
            reader.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        if row["panic"].as_bool().unwrap() {
            assert!(reader.error().is_some());
            continue;
        }
        assert_eq!(dump(&decoded), row["decoded"], "decoded {i}");
        assert_eq!(
            reader.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
    }
}
#[test]
fn native_facts_keyframe_trailing_byte_guard() {
    let rows = read_fixture(include_bytes!(
        "fixtures/facts-keyframe-count-v41.json.zlib"
    ));
    assert_eq!(rows.len(), 64);
    for (i, row) in rows.iter().enumerate() {
        let source = FactsWorldKeyframes {
            times_us: vec![],
            seen_us: [((1, 2), (0..row["count"].as_u64().unwrap()).collect())].into(),
        };
        let mut writer = NativeFactsWriter::default();
        encode_facts_keyframes(&mut writer, &source);
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encoded {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut reader = NativeFactsReader::new(&input);
        let decoded = decode_facts_keyframes(&mut reader);
        assert_eq!(json!(decoded.times_us), row["times"]);
        assert_eq!(
            json!(decoded.seen_us.get(&(1, 2)).unwrap()),
            row["seen"],
            "seen {i}"
        );
        assert_eq!(reader.offset(), row["offset"].as_u64().unwrap() as usize);
        assert_eq!(
            reader.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
    }
}
