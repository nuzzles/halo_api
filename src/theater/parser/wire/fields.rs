//! Exact bit values retained from an established source range.
use smallvec::SmallVec;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawBits {
    pub bit_len: usize,
    /// MSB-first bytes; unused low bits in the final byte are zero.
    pub bytes: SmallVec<[u8; 4]>,
}

impl RawBits {
    pub fn from_low(value: u64, bit_len: usize) -> Self {
        let mut bytes = SmallVec::from_elem(0, bit_len.div_ceil(8));
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
            if bit_len == 0 {
                return Self {
                    bit_len,
                    bytes: SmallVec::new(),
                };
            }
            if bit_len <= 8 {
                let first = start_bit / 8;
                let offset = start_bit % 8;
                let byte = if offset == 0 {
                    data[first]
                } else {
                    (data[first] << offset)
                        | (data.get(first + 1).copied().unwrap_or(0) >> (8 - offset))
                };
                let bytes =
                    SmallVec::from_buf_and_len([byte & (u8::MAX << (8 - bit_len)), 0, 0, 0], 1);
                return Self { bit_len, bytes };
            }
            let byte_len = bit_len.div_ceil(8);
            let first = start_bit / 8;
            let offset = start_bit % 8;
            let mut bytes = if offset == 0 {
                SmallVec::from_slice(&data[first..first + byte_len])
            } else if bit_len <= 32 {
                let mut window = [0u8; 8];
                let source_len = (byte_len + 1).min(data.len() - first);
                window[..source_len].copy_from_slice(&data[first..first + source_len]);
                let shifted = (u64::from_be_bytes(window) << offset).to_be_bytes();
                SmallVec::from_slice(&shifted[..byte_len])
            } else {
                let mut bytes = SmallVec::from_elem(0, byte_len);
                let paired = byte_len.min(data.len() - first - 1);
                for ((output, &high), &low) in bytes[..paired]
                    .iter_mut()
                    .zip(&data[first..first + paired])
                    .zip(&data[first + 1..first + paired + 1])
                {
                    *output = (high << offset) | (low >> (8 - offset));
                }
                if paired < byte_len {
                    bytes[paired] = data[first + paired] << offset;
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
        if self.bytes.len() == 1 {
            return u64::from(self.bytes[0] >> ((8 - self.bit_len % 8) % 8));
        }
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
    fn bulk_unaligned_fields_preserve_bytes_and_tail_padding() {
        let data = (0..1025).map(|i| (i % 251) as u8).collect::<Vec<_>>();
        for start in 0..8 {
            for len in [
                33usize, 63, 64, 65, 127, 128, 129, 255, 256, 257, 511, 1024, 4096, 8192,
            ] {
                let mut expected = vec![0; len.div_ceil(8)];
                for bit in 0..len {
                    expected[bit / 8] |=
                        ((data[(start + bit) / 8] >> (7 - (start + bit) % 8)) & 1) << (7 - bit % 8);
                }
                for source in [&data[..], &data[..(start + len).div_ceil(8)]] {
                    let raw = RawBits::from_source(source, start, len).unwrap();
                    assert_eq!(raw.bytes.as_slice(), expected, "start {start}, len {len}");
                }
            }
        }
    }

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
                assert_eq!(raw.bytes.as_slice(), expected, "start {start}, len {len}");
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

    #[test]
    fn unaligned_word_path_retains_every_source_bit() {
        let data = [0x72, 0xa1, 0x05, 0xef, 0xc3, 0x49, 0x18, 0xb6, 0x2d];
        for start in 0..data.len() * 8 {
            for len in 9..=32.min(data.len() * 8 - start) {
                let raw = RawBits::from_source(&data, start, len).unwrap();
                let mut expected = vec![0; len.div_ceil(8)];
                for bit in 0..len {
                    expected[bit / 8] |=
                        ((data[(start + bit) / 8] >> (7 - (start + bit) % 8)) & 1) << (7 - bit % 8);
                }
                assert_eq!(raw.bytes.as_slice(), expected, "start {start}, len {len}");
            }
        }
    }
}
