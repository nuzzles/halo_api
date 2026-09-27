use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn fixture() -> Vec<Value> {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/callouts-catalog-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 141);
    rows
}
fn input(row: &Value) -> Vec<u8> {
    let s = row["input_hex"].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn canonical_numbers(v: &mut Value) {
    match v {
        Value::Number(n) => *v = json!(n.as_f64().unwrap()),
        Value::Array(a) => a.iter_mut().for_each(canonical_numbers),
        Value::Object(m) => m.values_mut().for_each(canonical_numbers),
        _ => {}
    }
}
fn check(parsed: Result<MapCalloutsCatalog, CalloutsCatalogError>, row: &Value, index: usize) {
    let error = match &parsed {
        Ok(_) => Value::Null,
        Err(CalloutsCatalogError::Json(_)) => json!("json"),
        Err(CalloutsCatalogError::Schema(_)) => json!("schema"),
        Err(e) => panic!("unexpected {e}"),
    };
    assert_eq!(error, row["error"], "case {index}");
    let catalog = parsed.as_ref().ok();
    if let Some(catalog) = catalog {
        let expected: MapCalloutsCatalog = serde_json::from_value(row["retained"].clone()).unwrap();
        assert_eq!(*catalog, expected, "case {index}");
        let mut encoded = serde_json::to_value(catalog).unwrap();
        let restored: MapCalloutsCatalog = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            encoded,
            "canonical JSON roundtrip {index}"
        );
        let mut native = row["catalog"].clone();
        canonical_numbers(&mut encoded);
        canonical_numbers(&mut native);
        assert_eq!(encoded, native, "all native JSON fields {index}");
    }
    for q in row["lookups"].as_array().unwrap() {
        let result = lookup_callouts(
            catalog,
            q["query"].as_str().unwrap(),
            q["by_id"].as_bool().unwrap(),
        );
        assert_eq!(
            result.as_ref().is_err_and(|e| e.is_unknown_map()),
            q["unknown"].as_bool().unwrap(),
            "lookup case {index}: {q}"
        );
        let actual = result.cloned().unwrap_or_default();
        let expected: MapCalloutsEntry =
            serde_json::from_value(q["retained_entry"].clone()).unwrap();
        assert_eq!(actual, expected, "lookup case {index}: {q}");
    }
}
#[test]
fn native_callouts_catalog_fields_and_lookups() {
    assert_eq!(MAP_CALLOUTS_SCHEMA_VERSION, 1);
    assert_eq!(
        [
            CALLOUTS_PROVENANCE_RAW,
            CALLOUTS_PROVENANCE_CLIPPED,
            CALLOUTS_PROVENANCE_MVAR
        ],
        ["brut", "decoupe", "mvar"]
    );
    for (i, row) in fixture().iter().enumerate() {
        check(parse_map_callouts(&input(row)), row, i);
    }
}
#[test]
fn native_callouts_catalog_filesystem() {
    let root = std::env::temp_dir().join(format!(
        "halo-callouts-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("catalog.json");
    for (i, row) in fixture().iter().enumerate() {
        std::fs::write(&path, input(row)).unwrap();
        check(load_map_callouts(&path), row, i);
    }
    std::fs::remove_dir_all(root).unwrap();
}
