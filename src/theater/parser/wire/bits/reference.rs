//! Reference bit conventions, kept separate from the bounded production cursor.

/// Read one MSB-first bit; positions outside either boundary yield zero.
pub(crate) fn reference_bit_at(data: &[u8], position: i64) -> u8 {
    let Ok(p) = usize::try_from(position) else {
        return 0;
    };
    data.get(p / 8).map_or(0, |b| (b >> (7 - p % 8)) & 1)
}

/// Zero-pad both boundaries. Negative widths read nothing; widths above 64
/// retain the last 64 bits, as in the reference's unsigned accumulator.
pub(crate) fn reference_bits_tolerant(data: &[u8], position: i64, width: i64) -> u64 {
    if let (Ok(start), Ok(width)) = (usize::try_from(position), usize::try_from(width))
        && width <= 64
        && let Some(end) = start.checked_add(width)
        && end <= data.len().saturating_mul(8)
    {
        let mut value = 0u64;
        let mut bit = start;
        while bit < end {
            let byte_end = (bit | 7).saturating_add(1).min(end);
            let count = byte_end - bit;
            let shift = 8 - bit % 8 - count;
            let mask = (1u16 << count) - 1;
            value = (value << count) | u64::from((u16::from(data[bit / 8] >> shift)) & mask);
            bit = byte_end;
        }
        return value;
    }
    let mut value = 0;
    for i in width.saturating_sub(64).max(0)..width {
        value = (value << 1) | u64::from(reference_bit_at(data, position.wrapping_add(i)));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::reference_bits_tolerant;

    #[test]
    fn byte_path_matches_bitwise_reference_at_all_boundaries() {
        let data = [0x72, 0xa1, 0x05, 0xef, 0xc3, 0x49, 0x18, 0xb6, 0x2d];
        for position in -8..=data.len() as i64 * 8 + 8 {
            for width in -1i64..=80 {
                let expected = (width.saturating_sub(64).max(0)..width).fold(0u64, |value, i| {
                    let p = position.wrapping_add(i);
                    let bit = usize::try_from(p)
                        .ok()
                        .and_then(|p| data.get(p / 8).map(|b| (b >> (7 - p % 8)) & 1))
                        .unwrap_or(0);
                    (value << 1) | u64::from(bit)
                });
                assert_eq!(
                    reference_bits_tolerant(&data, position, width),
                    expected,
                    "position {position}, width {width}"
                );
            }
        }
    }
}

/// Read a scalar with zero tail padding.
///
/// # Panics
/// Requires a nonnegative position and a width no greater than 64.
pub(crate) fn reference_bits_at(data: &[u8], position: i64, width: u32) -> u64 {
    assert!(position >= 0 && width <= 64);
    reference_bits_tolerant(data, position, i64::from(width))
}

/// Borrowed reference sequential reader. Tail bits are synthetic zeros, and
/// `remaining()` can be negative. Consumers must retain padding provenance.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FilmBits<'a> {
    data: &'a [u8],
    position: i64,
}
impl<'a> FilmBits<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }
    pub(crate) fn position(&self) -> i64 {
        self.position
    }
    pub(crate) fn set_position(&mut self, position: i64) {
        self.position = position;
    }
    /// Reference unbounded signed movement, including backward skips.
    pub(crate) fn skip(&mut self, width: i64) {
        self.position = self.position.wrapping_add(width);
    }
    /// Reference zero-tail read with a 32-bit width. See `read_wide` for the
    /// full pinned 64-bit Go uint domain and panic/cursor behavior.
    pub(crate) fn read(&mut self, width: u32) -> u64 {
        self.read_wide(u64::from(width))
    }
    /// Reference zero-tail read. Wide reads retain the last 64 bits without
    /// allocating or iterating over synthetic padding. Cursor arithmetic uses
    /// the pinned reference's signed 64-bit int domain on every target.
    ///
    /// # Panics
    /// A nonempty read at a negative position panics. A read wider than 64 bits
    /// that wraps before its final bit panics at i64::MIN; its consumed prefix
    /// remains reflected in the cursor. Reads through 64 bits use the reference
    /// word path and wrap the endpoint without reading at the wrapped position.
    pub(crate) fn read_wide(&mut self, width: u64) -> u64 {
        assert!(width == 0 || self.position >= 0);
        if width > 64 {
            let until_negative = (i64::MAX - self.position) as u64 + 1;
            if width > until_negative {
                self.position = i64::MIN;
                panic!("reference bit read reached a negative byte index");
            }
        }
        let retained = width.min(64);
        let start = self.position.wrapping_add((width - retained) as i64);
        let result = reference_bits_tolerant(self.data, start, retained as i64);
        self.skip(width as i64);
        result
    }
    pub(crate) fn read_bit(&mut self) -> bool {
        self.read(1) != 0
    }
}
