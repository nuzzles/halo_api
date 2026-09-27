//! Stateful ReadQuantizedVec3 uses the native cursor and float evaluation order.
use super::*;
use std::io::Read;

#[test]
fn native_reader_quantized_vectors() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(include_bytes!("fixtures/quantize-v41.json.zlib").as_slice())
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 4096);
    for (i, row) in rows.iter().enumerate() {
        let hex = row["data"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let raw_bounds: [[u32; 2]; 3] = serde_json::from_value(row["bounds"].clone()).unwrap();
        let mut reader = NativeFilmReader::new(&data);
        reader.set_native_bit_position(row["start"].as_i64().unwrap());
        let vector = reader.read_quantized_vec3(
            row["width"].as_u64().unwrap(),
            raw_bounds.map(|a| a.map(f32::from_bits)),
        );
        let expected: [u32; 3] = serde_json::from_value(row["vector"].clone()).unwrap();
        for (axis, (actual, bits)) in vector.into_iter().zip(expected).enumerate() {
            if f32::from_bits(bits).is_nan() {
                assert!(actual.is_nan(), "case {i} axis {axis}");
            } else {
                assert_eq!(actual.to_bits(), bits, "case {i} axis {axis}");
            }
        }
        assert_eq!(
            Some(reader.native_bit_position()),
            row["end"].as_i64(),
            "case {i}"
        );
    }
}

#[test]
fn native_reader_quantized_cursor_overflow() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/reader-quantization-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 72);
    for row in rows {
        if cfg!(target_arch = "wasm32") && row["panic"] == true {
            continue;
        }
        let mut reader = NativeFilmReader::new(&[0xab, 0xcd]);
        reader.set_native_bit_position(row["start"].as_i64().unwrap());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reader.read_quantized_vec3(
                row["width"].as_u64().unwrap(),
                [[-3., 3.], [0., 0.], [1., 100.]],
            )
        }));
        assert_eq!(result.is_err(), row["panic"] == true, "{row}");
        if let Ok(vector) = result {
            let expected: [u32; 3] = serde_json::from_value(row["vector"].clone()).unwrap();
            for (actual, bits) in vector.into_iter().zip(expected) {
                if f32::from_bits(bits).is_nan() {
                    assert!(actual.is_nan(), "{row}");
                } else {
                    assert_eq!(actual.to_bits(), bits, "{row}");
                }
            }
        }
        assert_eq!(
            Some(reader.native_bit_position()),
            row["end"].as_i64(),
            "{row}"
        );
        reader.set_bit_position(0);
        assert_eq!(reader.read_bits(16), row["recovery"].as_u64());
    }
}
