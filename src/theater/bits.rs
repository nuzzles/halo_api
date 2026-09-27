/// Bounded MSB-first views; no allocation or expanded bit strings.
#[derive(Clone, Copy)]
pub(super) struct Bits<'a>(pub &'a [u8]);
impl<'a> Bits<'a> {
    pub fn len(self) -> usize {
        self.0.len().saturating_mul(8)
    }
    pub fn read(self, offset: impl TryInto<usize>, width: impl TryInto<usize>) -> Option<u64> {
        let width = width.try_into().ok()?;
        let offset = offset.try_into().ok()?;
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

/// Sequential bounded reader shared by the complete record grammar.
/// Default construction rejects truncation. The internal native constructor explicitly
/// opts into the reference's zero-tail convention; its callers retain padding provenance.
#[derive(Clone, Copy)]
pub struct Cursor<'a> {
    bits: Bits<'a>,
    padded: bool,
    pub position: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn source_bits(&self) -> usize {
        self.bits.len()
    }

    pub(crate) fn native_reader(self) -> super::NativeFilmBits<'a> {
        let mut reader = super::NativeFilmBits::new(self.bits.0);
        reader.set_position(self.position as i64);
        reader
    }

    pub fn new(bytes: &'a [u8], position: impl TryInto<usize>) -> Option<Self> {
        let position = position.try_into().ok()?;
        let bits = Bits(bytes);
        (position <= bits.len()).then_some(Self {
            bits,
            position,
            padded: false,
        })
    }

    /// Reference-only padding mode. Callers must expose the number of synthetic tail bits.
    pub(crate) fn new_padded(bytes: &'a [u8], position: usize) -> Self {
        Self {
            bits: Bits(bytes),
            position,
            padded: true,
        }
    }

    /// Bounded reads accept at most 64 bits. Native padded reads consume any
    /// representable width and return its low 64 bits; address overflow refuses
    /// the read without changing the cursor.
    pub fn read(&mut self, width: impl TryInto<usize>) -> Option<u64> {
        let width = width.try_into().ok()?;
        if width > 64 {
            if !self.padded {
                return None;
            }
            // Native ReadBits shifts into a u64: only the final 64 bits
            // survive, while the entire field advances the cursor.
            let end = self.position.checked_add(width)?;
            let mut tail = *self;
            tail.position = end - 64;
            let value = tail.read(64)?;
            self.position = end;
            return Some(value);
        }
        let end = self.position.checked_add(width)?;
        let value = if self.padded && end > self.bits.len() {
            let available = self.bits.len().saturating_sub(self.position).min(width);
            if available == 0 {
                0
            } else {
                self.bits.read(self.position, available)? << (width - available)
            }
        } else {
            self.bits.read(self.position, width)?
        };
        self.position += width;
        Some(value)
    }

    pub fn bit(&mut self) -> Option<bool> {
        Some(self.read(1)? != 0)
    }

    pub fn skip(&mut self, width: usize) -> Option<()> {
        let end = self.position.checked_add(width)?;
        if !self.padded && end > self.bits.len() {
            return None;
        }
        self.position = end;
        Some(())
    }

    pub fn gated(&mut self, width: usize, polarity: bool) -> Option<Option<u64>> {
        if self.bit()? == polarity {
            Some(Some(self.read(width)?))
        } else {
            Some(None)
        }
    }

    /// Engine signed variable integer: selector chooses 8, 16, 32 or 64 bits.
    pub fn signed_variable(&mut self) -> Option<i32> {
        let width = 8 << self.read(2)?;
        let value = self.read(width)? as u32;
        Some(match width {
            8 => value as i8 as i32,
            16 => value as i16 as i32,
            _ => value as i32,
        })
    }

    /// Raw categorical handle value and generation; category bases are not added.
    /// Matches LevelUp readVarWidthInt, including category-1's selector for category 4.
    pub fn handle(&mut self, mut category: u8) -> Option<(u32, u8)> {
        if category == 1 && self.bit()? {
            category = 4;
        }
        let width = match category {
            2 | 3 | 5 => 8,
            4 | 6 => 9,
            _ => 13,
        };
        Some((self.read(width)? as u32, self.read(2)? as u8))
    }

    /// Mask gate: clear = count3 and count times index6; set = dense64.
    pub fn mask(&mut self) -> Option<u64> {
        if self.bit()? {
            return self.read(64);
        }
        let count = self.read(3)?;
        let mut mask = 0;
        for _ in 0..count {
            mask |= 1 << self.read(6)?;
        }
        Some(mask)
    }
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

/// Checked projection used only when a caller is about to address source data.
/// Signed native trace coordinates themselves must not use this projection.
pub(crate) fn native_address(bit: impl TryInto<usize>) -> usize {
    bit.try_into()
        .ok()
        .expect("native cursor outside source address space")
}
/// Synthetic tail accounting is zero for negative native cursor positions.
/// The native signed endpoint remains authoritative on narrower targets.
pub(crate) fn padded_from_native(bit: i64, source_bits: usize) -> usize {
    if bit < 0 {
        return 0;
    }
    usize::try_from((bit as u64).saturating_sub(source_bits as u64)).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod padding_tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_padding_matches_go_without_changing_bounded_reads() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/native-padding-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        let mut count = 0;
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let start = row["start"].as_u64().unwrap() as usize;
            for read in row["reads"].as_array().unwrap() {
                let width = read["width"].as_u64().unwrap() as usize;
                let expected = read["value"].as_u64().unwrap();
                let mut native = Cursor::new_padded(&data, start);
                assert_eq!(native.read(width), Some(expected));
                assert_eq!(native.position, read["end"].as_u64().unwrap() as usize);
                if let Some(mut bounded) = Cursor::new(&data, start) {
                    let actual = bounded.read(width);
                    if start + width <= data.len() * 8 {
                        assert_eq!(actual, Some(expected));
                    } else {
                        assert_eq!(actual, None);
                        assert_eq!(bounded.position, start);
                    }
                }
                count += 1;
            }
        }
        assert_eq!(count, 89505);
    }
}
