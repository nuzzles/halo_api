//! Native generic quantization primitives. Values remain raw where the reference
//! deliberately omits category bases; conversion does not infer gameplay meaning.
use super::FilmBitReader;

/// Native `bitLen`: ceil(log2(value)), with zero and one both yielding zero.
pub fn native_quantized_bit_len(value: u32) -> u32 {
    if value <= 1 {
        0
    } else {
        32 - (value - 1).leading_zeros()
    }
}

/// Read three equal-width midpoint-quantized components. A bounded reader fails
/// atomically on truncation; widths above 64 retain only the low 64 bits, as in
/// the reference. Native padded readers retain their
/// explicit zero-tail convention. Bounds and widths are supplied by the caller.
pub fn read_native_quantized_vec3(
    reader: &mut FilmBitReader<'_>,
    width: u32,
    bounds: [[f32; 2]; 3],
) -> Option<[f32; 3]> {
    let mut cursor = *reader;
    let scale = 1_u64.checked_shl(width).unwrap_or(0) as f32;
    let mut out = [0.; 3];
    for (axis, [min, max]) in bounds.into_iter().enumerate() {
        if width > 64 {
            cursor.skip((width - 64) as usize)?;
        }
        let q = cursor.read(width.min(64) as usize)? as f32;
        let step = (max - min) / scale;
        out[axis] = q.mul_add(step, min) + step * 0.5;
    }
    *reader = cursor;
    Some(out)
}

/// Read native `readQuantStat`: category 1 has a probe selecting category 4;
/// the two tail bits occupy bits 31:30. Category bases are deliberately not added,
/// matching the pinned parser. Truncation leaves a bounded reader unchanged.
pub fn read_native_quant_stat(reader: &mut FilmBitReader<'_>, mut category: i32) -> Option<u32> {
    let mut cursor = *reader;
    if category == 1 && cursor.bit()? {
        category = 4;
    }
    let width = match category {
        2 | 3 | 5 => 8,
        4 | 6 => 9,
        _ => 13,
    };
    let value = cursor.read(width)? as u32;
    let tail = cursor.read(2)? as u32;
    *reader = cursor;
    Some((tail << 30) | value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        data: String,
        start: usize,
        width: u32,
        bounds: [[u32; 2]; 3],
        vector: [u32; 3],
        end: usize,
        category: i32,
        stat: u32,
        stat_end: usize,
        n: u32,
        bit_len: u32,
    }
    #[test]
    fn native_generic_quantization() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/quantize-v41.json.zlib")[..])
            .read_to_end(&mut bytes)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 4096);
        for (index, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = c
                .data
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
                .collect();
            let mut r = FilmBitReader::new_padded(&data, c.start);
            let bounds = c.bounds.map(|a| a.map(f32::from_bits));
            let vector = read_native_quantized_vec3(&mut r, c.width, bounds).unwrap();
            for (a, b) in vector.into_iter().zip(c.vector) {
                if f32::from_bits(b).is_nan() {
                    assert!(a.is_nan(), "case {index}");
                } else {
                    assert_eq!(a.to_bits(), b, "case {index}");
                }
            }
            assert_eq!(r.position, c.end, "vector end {index}");
            let mut r = FilmBitReader::new_padded(&data, c.start);
            assert_eq!(
                read_native_quant_stat(&mut r, c.category),
                Some(c.stat),
                "stat {index}"
            );
            assert_eq!(r.position, c.stat_end, "stat end {index}");
            assert_eq!(native_quantized_bit_len(c.n), c.bit_len, "bit len {index}");
            for (needed, stat) in [(c.end, false), (c.stat_end, true)] {
                if let Some(mut r) = FilmBitReader::new(&data, c.start) {
                    let success = if stat {
                        read_native_quant_stat(&mut r, c.category).is_some()
                    } else {
                        read_native_quantized_vec3(&mut r, c.width, bounds).is_some()
                    };
                    assert_eq!(success, needed <= data.len() * 8, "bounded {index}");
                    assert_eq!(r.position, if success { needed } else { c.start });
                }
            }
        }
    }
}
