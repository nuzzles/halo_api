/// Bounded MSB-first views; no allocation or expanded bit strings.
#[derive(Clone, Copy)]
pub(super) struct Bits<'a>(pub &'a [u8]);
impl<'a> Bits<'a> {
    pub fn len(self) -> usize {
        self.0.len().saturating_mul(8)
    }
    pub fn read(self, offset: usize, width: usize) -> Option<u64> {
        if width > 64 || offset.checked_add(width)? > self.len() {
            return None;
        }
        if width == 0 {
            return Some(0);
        }
        let first = offset / 8;
        let count = ((offset % 8) + width).div_ceil(8);
        let mut word = 0u128;
        for &byte in self.0.get(first..first + count)? {
            word = (word << 8) | u128::from(byte);
        }
        let shift = count * 8 - offset % 8 - width;
        Some(((word >> shift) & ((1u128 << width) - 1)) as u64)
    }
    pub fn is(self, offset: usize, pattern: &str) -> bool {
        pattern
            .as_bytes()
            .chunks(64)
            .enumerate()
            .all(|(i, p)| self.read(offset + i * 64, p.len()) == Some(pattern_bytes(p)))
    }
    pub fn windows(self) -> Windows<'a> {
        let mut value = 0u64;
        for i in 0..8 {
            value = (value << 8) | u64::from(self.0.get(i).copied().unwrap_or(0));
        }
        Windows {
            bits: self,
            offset: 0,
            value,
        }
    }
}
const fn pattern_bytes(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    let mut i = 0;
    while i < bytes.len() {
        value = (value << 1) | (bytes[i] - b'0') as u64;
        i += 1;
    }
    value
}
pub(super) const fn pattern(value: &str) -> u64 {
    pattern_bytes(value.as_bytes())
}
pub(super) struct Windows<'a> {
    bits: Bits<'a>,
    offset: usize,
    value: u64,
}
impl Iterator for Windows<'_> {
    type Item = (usize, u64);
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.bits.len() {
            return None;
        }
        let result = (self.offset, self.value);
        let next = self.offset + 64;
        let bit = self
            .bits
            .0
            .get(next / 8)
            .map_or(0, |b| u64::from((b >> (7 - next % 8)) & 1));
        self.value = (self.value << 1) | bit;
        self.offset += 1;
        Some(result)
    }
}
