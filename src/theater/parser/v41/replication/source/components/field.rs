//! Source-backed component fields.
use crate::theater::parser::wire::bits::Bits;
use std::sync::Arc;

/// Field width and scalar tail retained until ordered projection copies its
/// exact bits from the packet payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldBits {
    pub bit_len: usize,
    low: u64,
}

impl FieldBits {
    pub fn from_source(data: &[u8], start_bit: usize, bit_len: usize) -> Option<Self> {
        let end = start_bit.checked_add(bit_len)?;
        (end <= data.len().checked_mul(8)?).then_some(())?;
        let low = if bit_len == 0 {
            0
        } else if bit_len <= 8 {
            let first = start_bit / 8;
            let offset = start_bit % 8;
            let window = (u16::from(data[first]) << 8)
                | u16::from(data.get(first + 1).copied().unwrap_or(0));
            let shift = 16 - offset - bit_len;
            u64::from((window >> shift) & ((1u16 << bit_len) - 1))
        } else {
            let retained = bit_len.min(64);
            Bits(data).read(end - retained, retained)?
        };
        Some(Self { bit_len, low })
    }

    pub fn low_u64(self) -> u64 {
        self.low
    }
}
/// One source-backed component field with its exact payload position and bits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentField {
    pub name: Arc<str>,
    pub bit: usize,
    /// The read width is `raw.bit_len`. Zero is an explicit schema-selected,
    /// zero-bit read, distinct from an omitted field.
    pub raw: FieldBits,
}

impl ComponentField {
    pub fn materialize(
        &self,
        payload: &[u8],
    ) -> Option<crate::theater::parser::wire::fields::RawBits> {
        crate::theater::parser::wire::fields::RawBits::from_source(
            payload,
            self.bit,
            self.raw.bit_len,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentField, FieldBits};
    use crate::theater::parser::wire::fields::RawBits;

    #[test]
    fn deferred_bits_match_exact_source_values_at_all_boundaries() {
        let data = [
            0x72, 0xa1, 0x05, 0xef, 0xc3, 0x49, 0x18, 0xb6, 0x2d, 0xf0, 0x97,
        ];
        for start in 0..=data.len() * 8 {
            for len in 0..=data.len() * 8 - start {
                let expected = RawBits::from_source(&data, start, len).unwrap();
                let field = ComponentField {
                    name: "test".into(),
                    bit: start,
                    raw: FieldBits::from_source(&data, start, len).unwrap(),
                };
                assert_eq!(
                    field.raw.low_u64(),
                    expected.low_u64(),
                    "start {start}, len {len}"
                );
                assert_eq!(
                    field.materialize(&data),
                    Some(expected),
                    "start {start}, len {len}"
                );
            }
        }
        assert!(FieldBits::from_source(&data, data.len() * 8, 1).is_none());
    }
}
