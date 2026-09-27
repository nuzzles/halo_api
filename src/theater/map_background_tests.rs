use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn fixture() -> Vec<Value> {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/map-background-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1031);
    rows
}
fn input(row: &Value) -> Vec<u8> {
    let s = row["input_hex"].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn numbers(v: &mut Value) {
    match v {
        Value::Number(n) => *v = json!(n.as_f64().unwrap()),
        Value::Array(a) => a.iter_mut().for_each(numbers),
        Value::Object(o) => o.values_mut().for_each(numbers),
        _ => {}
    }
}
fn check(result: Result<MapBackground, MapBackgroundError>, row: &Value, i: usize) {
    let error = match &result {
        Ok(_) => Value::Null,
        Err(MapBackgroundError::Schema(_)) => json!("schema"),
        Err(MapBackgroundError::Json(_)) => json!("json"),
        Err(e) => panic!("unexpected {e}"),
    };
    assert_eq!(
        error,
        row["error"],
        "case {i}, input={}",
        String::from_utf8_lossy(&input(row))
    );
    if let Ok(actual) = result {
        let expected: MapBackground = serde_json::from_value(row["retained"].clone()).unwrap();
        assert_eq!(actual, expected, "retained fields {i}");
        let floats = |b: &MapBackground| {
            [
                b.calibration.meters_per_pixel,
                b.calibration.origin_x,
                b.calibration.origin_y,
                b.stats.play_level_z,
                b.stats.covered_share,
            ]
        };
        for (a, b) in floats(&actual).into_iter().zip(floats(&expected)) {
            assert_eq!(a.to_bits(), b.to_bits(), "float field bits {i}");
        }
        assert_eq!(
            actual.stats.anchor_median_gap_m.map(f64::to_bits),
            expected.stats.anchor_median_gap_m.map(f64::to_bits),
            "optional float bits {i}"
        );
        let encoded = serde_json::to_value(&actual);
        assert_eq!(
            encoded.is_err(),
            row["marshal_error"].as_bool().unwrap(),
            "publication refusal {i}"
        );
        if let Ok(mut encoded) = encoded {
            let restored: MapBackground = serde_json::from_value(encoded.clone()).unwrap();
            assert_eq!(
                serde_json::to_value(restored).unwrap(),
                encoded,
                "canonical roundtrip {i}"
            );
            let mut native = row["published"].clone();
            numbers(&mut encoded);
            numbers(&mut native);
            assert_eq!(encoded, native, "publication fields {i}");
        }
    }
}
#[test]
fn native_map_background_fields_and_timestamps() {
    assert_eq!(MAP_BACKGROUND_SCHEMA_VERSION, 1);
    for (i, row) in fixture().iter().enumerate() {
        check(parse_map_background(&input(row)), row, i);
    }
}
#[test]
fn native_map_background_filesystem() {
    let root = std::env::temp_dir().join(format!(
        "halo-background-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("background.json");
    for (i, row) in fixture().iter().enumerate() {
        std::fs::write(&path, input(row)).unwrap();
        check(load_map_background(&path), row, i);
    }
    std::fs::remove_dir_all(root).unwrap();
}
