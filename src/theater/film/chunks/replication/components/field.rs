//! Reference data models.
/// One source-backed component field with its exact payload position and bits.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ComponentField {
    pub name: String,
    pub bit: usize,
    pub width: usize,
    pub raw: RawBits,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
            let mut bytes = vec![0; bit_len.div_ceil(8)];
            for index in 0..bit_len {
                let source = start_bit + index;
                let bit = (data[source / 8] >> (7 - source % 8)) & 1;
                bytes[index / 8] |= bit << (7 - index % 8);
            }
            Self { bit_len, bytes }
        })
    }

    pub fn low_u64(&self) -> u64 {
        let start = self.bit_len.saturating_sub(64);
        (start..self.bit_len).fold(0, |value, index| {
            (value << 1) | u64::from((self.bytes[index / 8] >> (7 - index % 8)) & 1)
        })
    }
}
