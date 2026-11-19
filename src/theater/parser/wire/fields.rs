//! Exact bit values retained from an established source range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawBits {
    pub bit_len: usize,
    /// MSB-first bytes; unused low bits in the final byte are zero.
    pub bytes: Vec<u8>,
}

impl RawBits {
    pub fn from_low(value: u64, bit_len: usize) -> Self {
        let mut bytes = vec![0; bit_len.div_ceil(8)];
        let retained = bit_len.min(64);
        for index in 0..retained {
            let bit = (value >> (retained - 1 - index)) & 1;
            let target = bit_len - retained + index;
            bytes[target / 8] |= (bit as u8) << (7 - target % 8);
        }
        Self { bit_len, bytes }
    }
    pub fn from_source(data: &[u8], start_bit: usize, bit_len: usize) -> Option<Self> {
        let end = start_bit.checked_add(bit_len)?;
        (end <= data.len().checked_mul(8)?).then(|| {
            let byte_len = bit_len.div_ceil(8);
            let first = start_bit / 8;
            let offset = start_bit % 8;
            let mut bytes = if offset == 0 {
                data[first..first + byte_len].to_vec()
            } else {
                let mut bytes = Vec::with_capacity(byte_len);
                for index in 0..byte_len {
                    bytes.push(
                        (data[first + index] << offset)
                            | data.get(first + index + 1).copied().unwrap_or(0) >> (8 - offset),
                    );
                }
                bytes
            };
            if !bit_len.is_multiple_of(8) {
                *bytes.last_mut().expect("nonzero bit length") &= u8::MAX << (8 - bit_len % 8);
            }
            Self { bit_len, bytes }
        })
    }

    pub fn low_u64(&self) -> u64 {
        let trailing_padding = (8 - self.bit_len % 8) % 8;
        let last_eight = self.bytes.len().saturating_sub(8);
        let low = self.bytes[last_eight..]
            .iter()
            .fold(0u64, |value, &byte| (value << 8) | u64::from(byte));
        let preceding = if last_eight > 0 && trailing_padding != 0 {
            u64::from(self.bytes[last_eight - 1]) << (64 - trailing_padding)
        } else {
            0
        };
        (low >> trailing_padding) | preceding
    }
}

#[cfg(test)]
mod tests {
    use super::RawBits;

    #[test]
    fn source_ranges_preserve_msb_order_and_padding() {
        let data = [0b1010_1101, 0b0110_0011, 0b1101_0000];
        for start in 0..=data.len() * 8 {
            for len in 0..=data.len() * 8 - start {
                let raw = RawBits::from_source(&data, start, len).unwrap();
                let mut expected = vec![0; len.div_ceil(8)];
                for i in 0..len {
                    expected[i / 8] |=
                        ((data[(start + i) / 8] >> (7 - (start + i) % 8)) & 1) << (7 - i % 8);
                }
                assert_eq!(raw.bytes, expected, "start {start}, len {len}");
                let expected_low = (len.saturating_sub(64)..len).fold(0u64, |value, i| {
                    (value << 1) | u64::from((expected[i / 8] >> (7 - i % 8)) & 1)
                });
                assert_eq!(raw.low_u64(), expected_low, "start {start}, len {len}");
            }
        }
        assert!(RawBits::from_source(&data, 24, 1).is_none());
        assert!(RawBits::from_source(&data, usize::MAX, 2).is_none());
    }

    #[test]
    fn low_u64_across_nine_bytes() {
        let data = [
            0x72, 0xa1, 0x05, 0xef, 0xc3, 0x49, 0x18, 0xb6, 0x2d, 0xf0, 0x97, 0x5c,
        ];
        for start in 0..=data.len() * 8 {
            for len in 0..=data.len() * 8 - start {
                let raw = RawBits::from_source(&data, start, len).unwrap();
                let expected = (len.saturating_sub(64)..len).fold(0u64, |value, i| {
                    let source = start + i;
                    (value << 1) | u64::from((data[source / 8] >> (7 - source % 8)) & 1)
                });
                assert_eq!(raw.low_u64(), expected, "start {start}, len {len}");
            }
        }
    }
}
