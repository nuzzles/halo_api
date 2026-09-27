//! The stateful grammar reader must retain the source reader's signed cursor.
use super::*;
use std::io::Read;

#[test]
fn native_reader_signed_cursor_sequences() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(include_bytes!("fixtures/source-bits-v41.json.zlib").as_slice())
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    let mut checked = 0;
    for row in rows {
        let data: Vec<u8> = serde_json::from_value(row["data"].clone()).unwrap();
        for sequence in row["sequences"].as_array().unwrap() {
            let mut reader = NativeFilmReader::new(&data);
            reader.set_capture_slot(37);
            let profile = reader.profile();
            reader.set_native_bit_position(sequence["start"].as_i64().unwrap());
            for (i, step) in sequence["steps"].as_array().unwrap().iter().enumerate() {
                // wasm32 uses panic=abort. Check each sequence's actual prefix
                // before its first panic; do not synthesize recovery state.
                if cfg!(target_arch = "wasm32") && step["panic"] == true {
                    break;
                }
                let width = step["width"].as_i64().unwrap();
                if step["op"] == "skip" {
                    reader.skip_signed(width);
                } else {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        reader.read_bits_wide(width.try_into().unwrap())
                    }));
                    assert_eq!(result.is_err(), step["panic"] == true, "step {i}");
                    if let Ok(value) = result {
                        assert_eq!(Some(value), step["value"].as_u64(), "step {i}");
                    }
                }
                assert_eq!(Some(reader.native_bit_position()), step["end"].as_i64());
                assert_eq!(Some(reader.remaining_bits()), step["remaining"].as_i64());
                assert_eq!(reader.capture_slot(), 37);
                assert_eq!(reader.profile(), profile);
                checked += 1;
            }
        }
    }
    assert_eq!(
        checked,
        if cfg!(target_arch = "wasm32") {
            4896
        } else {
            13608
        }
    );
}

#[test]
fn native_reader_cursor_overflow_and_recovery() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/reader-cursor-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let data: Vec<u8> = serde_json::from_value(fixture["data"].clone()).unwrap();
    for row in fixture["rows"].as_array().unwrap() {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        let mut reader = NativeFilmReader::new(&data);
        reader.set_native_bit_position(row["start"].as_i64().unwrap());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if row["variable"] == true {
                serde_json::json!(reader.read_signed_variable().unwrap())
            } else {
                serde_json::json!(reader.read_bits_wide(row["width"].as_u64().unwrap()))
            }
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "{row}");
        if let Ok(value) = result {
            let key = if row["variable"] == true {
                "signed_value"
            } else {
                "value"
            };
            assert_eq!(value, row[key], "{row}");
        }
        assert_eq!(
            Some(reader.native_bit_position()),
            row["end"].as_i64(),
            "{row}"
        );
        assert_eq!(
            Some(reader.remaining_bits()),
            row["remaining"].as_i64(),
            "{row}"
        );
        reader.set_bit_position(0);
        assert_eq!(reader.read_bits(16), row["recovery"].as_u64());
        assert_eq!(
            Some(reader.native_bit_position()),
            row["recovery_end"].as_i64()
        );
    }
}
