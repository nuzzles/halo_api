use super::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn values<T: DeserializeOwned>(v: &Value) -> Vec<T> {
    if v.is_null() {
        Vec::new()
    } else {
        serde_json::from_value(v.clone()).unwrap()
    }
}
fn leaf<T: DeserializeOwned + Serialize>(
    row: &Value,
    w: &mut NativeFactsWriter,
    r: &mut NativeFactsReader<'_>,
    encode: fn(&mut NativeFactsWriter, &[T]),
    decode: fn(&mut NativeFactsReader<'_>) -> Vec<T>,
) -> (Value, Value) {
    encode(w, &values::<T>(&row["source"]));
    (json!(decode(r)), Value::Null)
}
#[test]
fn native_facts_channel_codecs() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-channels-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 31841);
    for (i, row) in rows.iter().enumerate() {
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let mut w = NativeFactsWriter::default();
        let (actual, stats) = match row["kind"].as_str().unwrap() {
            "bipeds" => leaf(
                row,
                &mut w,
                &mut r,
                encode_facts_biped_creations,
                decode_facts_biped_creations,
            ),
            "weapons" => leaf(
                row,
                &mut w,
                &mut r,
                encode_facts_weapon_changes,
                decode_facts_weapon_changes,
            ),
            "zoom" => leaf(
                row,
                &mut w,
                &mut r,
                encode_facts_zoom_events,
                decode_facts_zoom_events,
            ),
            "events" => leaf(
                row,
                &mut w,
                &mut r,
                encode_facts_vehicle_events,
                decode_facts_vehicle_events,
            ),
            "aims" => leaf(
                row,
                &mut w,
                &mut r,
                encode_facts_vehicle_aims,
                decode_facts_vehicle_aims,
            ),
            "occupancy" => leaf(
                row,
                &mut w,
                &mut r,
                encode_facts_vehicle_occupancy,
                decode_facts_vehicle_occupancy,
            ),
            "pickups" => {
                encode_facts_pickups(
                    &mut w,
                    &values(&row["source"]),
                    &serde_json::from_value(row["source_stats"].clone()).unwrap(),
                );
                let (v, s) = decode_facts_pickups(&mut r);
                (json!(v), json!(s))
            }
            "equipment" => {
                encode_facts_equipment_changes(
                    &mut w,
                    &values(&row["source"]),
                    &serde_json::from_value(row["source_stats"].clone()).unwrap(),
                );
                let (v, s) = decode_facts_equipment_changes(&mut r);
                (json!(v), json!(s))
            }
            other => panic!("unknown channel {other}"),
        };
        assert_eq!(
            w.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encode {i}"
        );
        assert_eq!(
            r.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        if row["panic"].as_bool().unwrap() {
            assert!(r.error().is_some(), "safe refusal {i}");
            continue;
        }
        assert_eq!(
            r.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
        assert_eq!(actual, row["decoded"], "records {i}");
        assert_eq!(stats, row["decoded_stats"], "stats {i}");
    }
}
