use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn dump(a: &KeyframeSlotAmmo) -> Value {
    json!({"mag":a.mag,"res":a.res,"gauge_bits":a.gauge.map(f64::to_bits),"overheat":a.overheat,"flags":a.flags})
}
#[test]
fn native_facts_ammo_codec() {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(include_bytes!("fixtures/facts-ammo-v41.json.zlib").as_slice())
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 11224);
    for (i, row) in rows.iter().enumerate() {
        let a = &row["source"];
        let ammo = KeyframeSlotAmmo {
            mag: a["mag"].as_u64().map(|v| v as u32),
            res: a["res"].as_u64().map(|v| v as u32),
            gauge: a["gauge_bits"].as_u64().map(f64::from_bits),
            gauge_quantum: None,
            overheat: a["overheat"].as_u64().unwrap() as u32,
            flags: a["flags"].as_u64().unwrap() as u32,
        };
        let mut writer = NativeFactsWriter::default();
        encode_facts_ammo(&mut writer, &ammo);
        assert_eq!(
            writer.bytes(),
            bytes(row["encoded_hex"].as_str().unwrap()),
            "encoded {i}"
        );
        let input = bytes(row["input_hex"].as_str().unwrap());
        let mut reader = NativeFactsReader::new(&input);
        let decoded = decode_facts_ammo(&mut reader);
        assert_eq!(dump(&decoded), row["decoded"], "decoded {i}");
        assert!(decoded.gauge_quantum.is_none());
        assert_eq!(
            reader.offset(),
            row["first_offset"].as_u64().unwrap() as usize,
            "first offset {i}"
        );
        assert_eq!(
            reader.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
        let again = decode_facts_ammo(&mut reader);
        assert_eq!(dump(&again), row["again"], "again {i}");
        assert_eq!(
            reader.error().unwrap_or(""),
            row["after_error"].as_str().unwrap(),
            "after error {i}"
        );
        assert_eq!(
            reader.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
    }
}
