/// Bounded MSB-first views; no allocation or expanded bit strings.
#[derive(Clone, Copy)]
pub(crate) struct Bits<'a>(pub &'a [u8]);
impl<'a> Bits<'a> {
    pub(crate) fn len(self) -> usize {
        self.0.len().saturating_mul(8)
    }
    pub(crate) fn read(
        self,
        offset: impl TryInto<usize>,
        width: impl TryInto<usize>,
    ) -> Option<u64> {
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
    pub(crate) fn windows(self) -> Windows<'a> {
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

/// Sequential bounded reader shared by the complete record grammar.
/// Default construction rejects truncation. The internal reference constructor explicitly
/// opts into the reference's zero-tail convention; its callers retain padding provenance.
#[derive(Clone, Copy)]
pub(crate) struct Cursor<'a> {
    bits: Bits<'a>,
    padded: bool,
    pub position: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn source_bits(&self) -> usize {
        self.bits.len()
    }

    pub(crate) fn new(bytes: &'a [u8], position: impl TryInto<usize>) -> Option<Self> {
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

    /// Bounded reads accept at most 64 bits. Reference padded reads consume any
    /// representable width and return its low 64 bits; address overflow refuses
    /// the read without changing the cursor.
    pub(crate) fn read(&mut self, width: impl TryInto<usize>) -> Option<u64> {
        let width = width.try_into().ok()?;
        if width > 64 {
            if !self.padded {
                return None;
            }
            // Reference ReadBits shifts into a u64: only the final 64 bits
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

    pub(crate) fn bit(&mut self) -> Option<bool> {
        Some(self.read(1)? != 0)
    }

    pub(crate) fn skip(&mut self, width: usize) -> Option<()> {
        let end = self.position.checked_add(width)?;
        if !self.padded && end > self.bits.len() {
            return None;
        }
        self.position = end;
        Some(())
    }
}
pub(crate) struct Windows<'a> {
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

/// Synthetic tail accounting is zero for negative reference cursor positions.
/// The reference signed endpoint remains authoritative on narrower targets.
pub(crate) fn padded_from_native(bit: i64, source_bits: usize) -> usize {
    if bit < 0 {
        return 0;
    }
    usize::try_from((bit as u64).saturating_sub(source_bits as u64)).unwrap_or(usize::MAX)
}
