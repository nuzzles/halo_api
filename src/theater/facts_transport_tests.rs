use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn native_facts_transport_sequences() {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/facts-transport-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1120);
    for (i, row) in rows.iter().enumerate() {
        let mut writer = NativeFactsWriter::default();
        if let Some(writes) = row["writes"].as_array() {
            for w in writes {
                let v = &w["value"];
                match w["op"].as_str().unwrap() {
                    "u" => writer.unsigned(v.as_u64().unwrap()),
                    "i" => writer.signed(v.as_i64().unwrap()),
                    "byte" => writer.byte(v.as_u64().unwrap() as u8),
                    "f32" => writer.float32(f32::from_bits(v.as_u64().unwrap() as u32)),
                    "str" => writer.string_bytes(&unhex(v.as_str().unwrap())),
                    "bool" => writer.boolean(v.as_bool().unwrap()),
                    _ => unreachable!(),
                }
            }
        }
        assert_eq!(
            hex(writer.bytes()),
            row["written_hex"].as_str().unwrap(),
            "writer {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut reader = NativeFactsReader::new(&input);
        for (j, (op, expected)) in row["operations"]
            .as_array()
            .unwrap()
            .iter()
            .zip(row["results"].as_array().unwrap())
            .enumerate()
        {
            let value = match op["op"].as_str().unwrap() {
                "u" => json!(reader.unsigned()),
                "i" => json!(reader.signed()),
                "byte" => json!(reader.byte()),
                "f32" => json!(reader.float32().to_bits()),
                "str" => json!(hex(reader.string_bytes())),
                "bool" => json!(reader.boolean()),
                "count" => json!(reader.count(op["arg"].as_i64().unwrap())),
                "section" => json!(reader.section(op["arg"].as_i64().unwrap()).map(hex)),
                _ => unreachable!(),
            };
            assert_eq!(
                reader.offset(),
                expected["offset"].as_u64().unwrap() as usize,
                "offset {i}/{j}"
            );
            if expected["panic"].as_bool().unwrap() {
                assert!(reader.error().is_some(), "safe refusal {i}/{j}");
                break;
            }
            assert_eq!(value, expected["value"], "value {i}/{j}");
            assert_eq!(
                reader.error().unwrap_or(""),
                expected["error"].as_str().unwrap(),
                "error {i}/{j}"
            );
        }
    }
}
