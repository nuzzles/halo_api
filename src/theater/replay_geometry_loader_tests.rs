use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn fixture() -> Value {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/geometry-loader-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    serde_json::from_slice(&raw).unwrap()
}
fn bytes(v: &Value) -> Vec<u8> {
    let s = v.as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}
fn check_load(result: Result<LoadedReplayGeometry, ReplayGeometryError>, row: &Value) {
    match result {
        Ok(result) => {
            assert!(row["error"].is_null(), "{row}");
            assert_eq!(json!(result.skipped), row["skipped"], "{row}");
            let values: Vec<_> = result.objects.iter().map(|o| json!({"id":o.type_id,"bits":[o.x.to_bits(),o.y.to_bits(),o.z.to_bits(),o.dx.to_bits(),o.dy.to_bits(),o.yaw.to_bits()]})).collect();
            assert_eq!(json!(values), row["objects"], "{row}");
            assert_eq!(
                serde_json::to_value(&result).is_err(),
                row["json_error"].as_bool().unwrap(),
                "{row}"
            );
            // Finite DTOs survive the public portable JSON projection.
            if result.objects.iter().all(|o| {
                [o.x, o.y, o.z, o.dx, o.dy, o.yaw]
                    .iter()
                    .all(|v| v.is_finite())
            }) {
                let restored: LoadedReplayGeometry =
                    serde_json::from_value(serde_json::to_value(&result).unwrap()).unwrap();
                assert_eq!(restored, result);
            }
        }
        Err(e) => {
            assert_eq!(serde_json::to_value(e.file).unwrap(), row["file"], "{row}");
            let mut expected = row["error"].clone();
            let failure = match e.cause {
                ReplayGeometryFailure::Csv(e) => serde_json::to_value(e).unwrap(),
                ReplayGeometryFailure::Io(e) => {
                    // Native adds a French header wrapper around directory reads.
                    // Our I/O cause retains ErrorKind and file, not that wrapper.
                    expected.as_object_mut().unwrap().remove("stage");
                    json!({"kind":if e.kind()==std::io::ErrorKind::NotFound {"missing"} else {"io"}})
                }
            };
            assert_eq!(failure, expected, "{row}");
        }
    }
}
#[test]
fn native_geometry_float_syntax_and_bits() {
    for row in fixture()["floats"].as_array().unwrap() {
        assert_eq!(
            json!(
                super::replay_geometry_float::geometry_float(&bytes(&row["input_hex"])).to_bits()
            ),
            row["bits"],
            "{row}"
        );
    }
}
#[test]
fn native_geometry_csv_records_and_errors() {
    for row in fixture()["csv"].as_array().unwrap() {
        match super::replay_geometry_csv::read_geometry_csv(&bytes(&row["input_hex"])) {
            Ok(csv) => {
                assert!(row["error"].is_null(), "{row}");
                let records: Vec<Vec<_>> = csv
                    .rows
                    .iter()
                    .map(|r| r.iter().map(|f| hex(f)).collect())
                    .collect();
                let columns: Vec<_> = csv
                    .columns
                    .iter()
                    .map(|(k, v)| json!([hex(k), v]))
                    .collect();
                assert_eq!(json!(records), row["rows"], "{row}");
                assert_eq!(json!(columns), row["columns"], "{row}");
            }
            Err(e) => assert_eq!(serde_json::to_value(e).unwrap(), row["error"], "{row}"),
        }
    }
}
#[test]
fn native_geometry_loader_in_memory() {
    assert_eq!(REPLAY_MAP_OBJECTS_FILE, "map_objects.csv");
    assert_eq!(REPLAY_OBJECT_TYPES_FILE, "forge_object_types.csv");
    for row in fixture()["loads"].as_array().unwrap() {
        if row["missing_types"] == true
            || row["directory_types"] == true
            || row["directory_map"] == true
        {
            continue;
        }
        let types = bytes(&row["types_hex"]);
        let map = bytes(&row["map_hex"]);
        check_load(
            parse_replay_geometry_csv(
                if row["missing_map"] == true {
                    None
                } else {
                    Some(&map)
                },
                &types,
            ),
            row,
        );
    }
}
#[test]
fn native_geometry_loader_filesystem() {
    let root = std::env::temp_dir().join(format!(
        "halo-geometry-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    for (i, row) in fixture()["loads"].as_array().unwrap().iter().enumerate() {
        let map = root.join(format!("map-{i}"));
        let types = root.join(format!("types-{i}"));
        std::fs::create_dir(&map).unwrap();
        std::fs::create_dir(&types).unwrap();
        if row["directory_types"] == true {
            std::fs::create_dir(types.join(REPLAY_OBJECT_TYPES_FILE)).unwrap();
        } else if row["missing_types"] != true {
            std::fs::write(
                types.join(REPLAY_OBJECT_TYPES_FILE),
                bytes(&row["types_hex"]),
            )
            .unwrap();
        }
        if row["directory_map"] == true {
            std::fs::create_dir(map.join(REPLAY_MAP_OBJECTS_FILE)).unwrap();
        } else if row["missing_map"] != true {
            std::fs::write(map.join(REPLAY_MAP_OBJECTS_FILE), bytes(&row["map_hex"])).unwrap();
        }
        check_load(load_replay_geometry(&map, &types), row);
    }
    std::fs::remove_dir_all(root).unwrap();
}
