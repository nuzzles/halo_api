//! Reference bit conventions, kept separate from the bounded production cursor.

/// Native `ReadBitsAtForDiag`: MSB-first read with a wrapping 32-bit accumulator.
/// Nonpositive widths return zero. Wider reads retain the low 32 bits.
///
/// # Panics
/// Any requested bit outside the buffer panics, including negative positions.
/// This diagnostic contract deliberately differs from zero-padded native readers.
pub fn native_bits_for_diagnostics(data: &[u8], position: i64, width: i64) -> u32 {
    let mut value = 0_u32;
    for i in 0..width {
        let p = position.wrapping_add(i);
        value = (value << 1) | u32::from((data[(p >> 3) as usize] >> (7 - (p & 7))) & 1);
    }
    value
}

/// Read one MSB-first bit; positions outside either boundary yield zero.
pub fn native_bit_at(data: &[u8], position: i64) -> u8 {
    let Ok(p) = usize::try_from(position) else {
        return 0;
    };
    data.get(p / 8).map_or(0, |b| (b >> (7 - p % 8)) & 1)
}

/// Zero-pad both boundaries. Negative widths read nothing; widths above 64
/// retain the last 64 bits, as in the reference's unsigned accumulator.
pub fn native_bits_tolerant(data: &[u8], position: i64, width: i64) -> u64 {
    let mut value = 0;
    for i in width.saturating_sub(64).max(0)..width {
        value = (value << 1) | u64::from(native_bit_at(data, position.wrapping_add(i)));
    }
    value
}

/// Read a scalar with zero tail padding.
///
/// # Panics
/// Requires a nonnegative position and a width no greater than 64.
pub fn native_bits_at(data: &[u8], position: i64, width: u32) -> u64 {
    assert!(position >= 0 && width <= 64);
    native_bits_tolerant(data, position, i64::from(width))
}

/// Read only the available bits, right-aligned, without padding the tail.
/// Negative widths read nothing. Wide values retain their last 64 bits.
///
/// # Panics
/// Matches the native fallback's negative byte-index panic. Starts -7..-1
/// instead yield leading zeros because native signed division truncates toward zero.
pub fn native_bits_truncated(data: &[u8], position: i64, width: i64) -> u64 {
    let mut value = 0;
    for i in 0..width {
        let p = position.wrapping_add(i);
        let byte = p / 8;
        if byte >= data.len() as i64 {
            break;
        }
        let shift = (7 - p % 8) as u32;
        value = (value << 1) | u64::from(data[byte as usize].checked_shr(shift).unwrap_or(0) & 1);
    }
    value
}

/// Borrowed native sequential reader. Tail bits are synthetic zeros, and
/// `remaining()` can be negative. Consumers must retain padding provenance.
#[derive(Clone, Copy, Debug)]
pub struct NativeFilmBits<'a> {
    data: &'a [u8],
    position: i64,
}
impl<'a> NativeFilmBits<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }
    /// Original buffer without allocation or copying.
    pub fn octets(&self) -> &'a [u8] {
        self.data
    }
    pub fn bit_len(&self) -> i64 {
        (self.data.len() as i64).wrapping_mul(8)
    }
    pub fn position(&self) -> i64 {
        self.position
    }
    pub fn set_position(&mut self, position: i64) {
        self.position = position;
    }
    pub fn remaining(&self) -> i64 {
        self.bit_len().wrapping_sub(self.position)
    }
    /// Native unbounded signed movement, including backward skips.
    pub fn skip(&mut self, width: i64) {
        self.position = self.position.wrapping_add(width);
    }
    /// Native zero-tail read with a 32-bit width. See `read_wide` for the
    /// full pinned 64-bit Go uint domain and panic/cursor behavior.
    pub fn read(&mut self, width: u32) -> u64 {
        self.read_wide(u64::from(width))
    }
    /// Native zero-tail read. Wide reads retain the last 64 bits without
    /// allocating or iterating over synthetic padding. Cursor arithmetic uses
    /// the pinned reference's signed 64-bit int domain on every target.
    ///
    /// # Panics
    /// A nonempty read at a negative position panics. A read wider than 64 bits
    /// that wraps before its final bit panics at i64::MIN; its consumed prefix
    /// remains reflected in the cursor. Reads through 64 bits use the native
    /// word path and wrap the endpoint without reading at the wrapped position.
    pub fn read_wide(&mut self, width: u64) -> u64 {
        assert!(width == 0 || self.position >= 0);
        if width > 64 {
            let until_negative = (i64::MAX - self.position) as u64 + 1;
            if width > until_negative {
                self.position = i64::MIN;
                panic!("native bit read reached a negative byte index");
            }
        }
        let retained = width.min(64);
        let start = self.position.wrapping_add((width - retained) as i64);
        let result = native_bits_tolerant(self.data, start, retained as i64);
        self.skip(width as i64);
        result
    }
    pub fn read_bit(&mut self) -> bool {
        self.read(1) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_signed_cursor_continuation() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/source-bits-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for row in rows {
            let data: Vec<u8> = serde_json::from_value(row["data"].clone()).unwrap();
            for sequence in row["sequences"].as_array().unwrap() {
                let mut reader = NativeFilmBits::new(&data);
                reader.set_position(sequence["start"].as_i64().unwrap());
                for (i, step) in sequence["steps"].as_array().unwrap().iter().enumerate() {
                    let width = step["width"].as_i64().unwrap();
                    if step["op"] == "skip" {
                        reader.skip(width);
                    } else {
                        let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            reader.read(width.try_into().unwrap())
                        }));
                        assert_eq!(
                            value.is_err(),
                            step["panic"].as_bool().unwrap(),
                            "start={} step={i} width={width}",
                            sequence["start"]
                        );
                        if let Ok(value) = value {
                            assert_eq!(Some(value), step["value"].as_u64());
                        }
                    }
                    assert_eq!(
                        Some(reader.position()),
                        step["end"].as_i64(),
                        "start={} step={i} width={width}",
                        sequence["start"]
                    );
                    assert_eq!(Some(reader.remaining()), step["remaining"].as_i64());
                }
            }
        }
    }

    #[test]
    fn native_bit_conventions_match_reference() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/source-bits-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 24);
        for row in rows {
            let data: Vec<u8> = serde_json::from_value(row["data"].clone()).unwrap();
            for c in row["cases"].as_array().unwrap() {
                let p = c["p"].as_i64().unwrap();
                let n = c["n"].as_i64().unwrap();
                assert_eq!(
                    u64::from(native_bit_at(&data, p)),
                    c["bit"].as_u64().unwrap()
                );
                assert_eq!(
                    native_bits_tolerant(&data, p, n),
                    c["tolerant"].as_u64().unwrap()
                );
                let truncated = std::panic::catch_unwind(|| native_bits_truncated(&data, p, n));
                assert_eq!(truncated.is_err(), c["truncated_panic"].as_bool().unwrap());
                if let Ok(v) = truncated {
                    assert_eq!(v, c["truncated"].as_u64().unwrap());
                }
                if p >= 0 && (0..=64).contains(&n) {
                    assert_eq!(
                        native_bits_at(&data, p, n as u32),
                        c["tolerant"].as_u64().unwrap()
                    );
                }
                if n < 0 {
                    continue;
                }
                if p >= 0 {
                    let mut reader = crate::theater::NativeFilmReader::new(&data);
                    reader.set_bit_position(p as usize);
                    assert_eq!(
                        reader.read_bits(n as usize),
                        c["read"].as_u64(),
                        "live reader p={p} n={n}"
                    );
                    assert_eq!(reader.bit_position() as i64, c["end"].as_i64().unwrap());
                    assert_eq!(
                        reader.padded_bits(),
                        (p + n).saturating_sub(data.len() as i64 * 8).max(0) as usize
                    );
                }
                let mut r = NativeFilmBits::new(&data);
                assert_eq!(r.position(), 0);
                assert_eq!(r.octets().as_ptr(), data.as_ptr());
                r.set_position(p);
                let read =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| r.read(n as u32)));
                assert_eq!(read.is_err(), c["read_panic"].as_bool().unwrap());
                if let Ok(v) = read {
                    assert_eq!(v, c["read"].as_u64().unwrap());
                }
                assert_eq!(r.position(), c["end"].as_i64().unwrap());
                assert_eq!(r.remaining(), c["remaining"].as_i64().unwrap());
                r.skip(-3);
                assert_eq!(r.position(), c["end"].as_i64().unwrap() - 3);
            }
            let mut r = NativeFilmBits::new(&data);
            for p in 0..data.len() * 8 + 3 {
                assert_eq!(r.read_bit(), native_bit_at(&data, p as i64) != 0);
            }
        }
    }
}
