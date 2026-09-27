//! Native byte primitives. Offset units and tail conventions are explicit.

/// Read a complete unaligned byte, MSB first. A byte crossing either boundary is zero.
/// Unlike the native scalar bit reader, this never pads a partial byte.
pub fn native_octet_at_bit(data: &[u8], position: i64) -> u8 {
    let Ok(position) = usize::try_from(position) else {
        return 0;
    };
    super::bits::Bits(data).read(position, 8).unwrap_or(0) as u8
}

/// Assemble eight unaligned bytes little-endian. Each incomplete byte is zero;
/// complete bytes remain available even when the full word crosses a boundary.
pub fn native_u64_le_at_bit(data: &[u8], position: i64) -> u64 {
    (0..8).fold(0, |value, i| {
        value
            | position
                .checked_add(i * 8)
                .map_or(0, |p| u64::from(native_octet_at_bit(data, p)))
                << (i * 8)
    })
}

/// First fully bounded 64-bit MSB-first match, in increasing bit order.
/// A little-endian identity must be converted to its wire byte pattern first.
pub fn native_find_pattern64(data: &[u8], target: u64) -> Option<usize> {
    for (i, bytes) in data.windows(8).enumerate() {
        let word = u64::from_be_bytes(bytes.try_into().unwrap());
        if word == target {
            return Some(i * 8);
        }
        if let Some(&next) = data.get(i + 8) {
            for shift in 1..8 {
                if word << shift | u64::from(next) >> (8 - shift) == target {
                    return Some(i * 8 + shift);
                }
            }
        }
    }
    None
}

/// Read a native LE integer at a byte offset.
///
/// # Panics
/// Panics if the complete integer is not in the buffer, matching the native primitive.
pub fn native_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(data[offset..][..2].try_into().unwrap())
}

/// Read a native LE integer at a byte offset.
///
/// # Panics
/// Panics if the complete integer is not in the buffer, matching the native primitive.
pub fn native_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..][..4].try_into().unwrap())
}

/// Read a native LE integer at a byte offset.
///
/// # Panics
/// Panics if the complete integer is not in the buffer, matching the native primitive.
pub fn native_u64_le(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..][..8].try_into().unwrap())
}

/// Read a native BE integer at a byte offset.
///
/// # Panics
/// Panics if the complete integer is not in the buffer, matching the native primitive.
pub fn native_u32_be(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(data[offset..][..4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn native_octets_and_patterns_match_reference() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/source-octets-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(rows.len(), 96);
        for row in rows {
            let data: Vec<u8> = serde_json::from_value(row["data"].clone()).unwrap();
            for r in row["reads"].as_array().unwrap() {
                let p = r[0].as_i64().unwrap();
                assert_eq!(native_octet_at_bit(&data, p) as u64, r[1].as_u64().unwrap());
                assert_eq!(native_u64_le_at_bit(&data, p), r[2].as_u64().unwrap());
            }
            for r in row["integers"].as_array().unwrap() {
                let p = r[0].as_u64().unwrap() as usize;
                assert_eq!(u64::from(native_u16_le(&data, p)), r[1].as_u64().unwrap());
                assert_eq!(u64::from(native_u32_le(&data, p)), r[2].as_u64().unwrap());
                assert_eq!(native_u64_le(&data, p), r[3].as_u64().unwrap());
                assert_eq!(u64::from(native_u32_be(&data, p)), r[4].as_u64().unwrap());
            }
            for r in row["patterns"].as_array().unwrap() {
                let expected = r[2]
                    .as_bool()
                    .unwrap()
                    .then(|| r[1].as_u64().unwrap() as usize);
                assert_eq!(
                    native_find_pattern64(&data, r[0].as_u64().unwrap()),
                    expected
                );
            }
        }
        assert_eq!(native_octet_at_bit(&[0xff], 1), 0);
        assert_eq!(native_u64_le_at_bit(&[0xff], -8), 0xff00);
        assert_eq!(native_u64_le_at_bit(&[0xff], i64::MAX), 0);
    }
}
