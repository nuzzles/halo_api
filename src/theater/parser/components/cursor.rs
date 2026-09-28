//! Signed native traversal cursor. Slice addresses remain checked at access.
#[derive(Clone, Copy)]
pub(super) struct ComponentCursor<'a> {
    data: &'a [u8],
    padded: bool,
    guarded_native: bool,
    pub position: i64,
    mirror: Option<&'a std::cell::Cell<i64>>,
}
impl<'a> ComponentCursor<'a> {
    pub(crate) fn new(data: &'a [u8], position: usize) -> Option<Self> {
        (position <= data.len().checked_mul(8)?).then_some(Self {
            data,
            padded: false,
            guarded_native: false,
            mirror: None,
            position: i64::try_from(position).ok()?,
        })
    }
    pub(crate) fn signed(
        data: &'a [u8],
        position: i64,
        mirror: Option<&'a std::cell::Cell<i64>>,
    ) -> Self {
        Self {
            data,
            padded: true,
            guarded_native: false,
            position,
            mirror,
        }
    }
    /// Native view guards use signed wrapping position + width before reading.
    pub(crate) fn guarded(data: &'a [u8], position: i64) -> Self {
        Self {
            guarded_native: true,
            ..Self::signed(data, position, None)
        }
    }
    pub(crate) fn source_bits(&self) -> usize {
        self.data.len().saturating_mul(8)
    }
    pub(crate) fn fits_source(&self, width: i64) -> bool {
        self.position.wrapping_add(width) <= (self.data.len() as i64).wrapping_mul(8)
    }
    pub(crate) fn detached(&self) -> Self {
        Self {
            mirror: None,
            ..*self
        }
    }
    pub(crate) fn read_wide(&mut self, width: u64) -> Option<u64> {
        if self.guarded_native && !self.fits_source(width as i64) {
            return None;
        }
        if self.padded {
            let mut reader = crate::theater::parser::NativeFilmBits::new(self.data);
            reader.set_position(self.position);
            // Restore the native position even when a read unwinds on the host.
            struct CopyBack<'r, 'd> {
                reader: crate::theater::parser::NativeFilmBits<'d>,
                position: &'r mut i64,
                mirror: Option<&'r std::cell::Cell<i64>>,
            }
            impl Drop for CopyBack<'_, '_> {
                fn drop(&mut self) {
                    *self.position = self.reader.position();
                    if let Some(mirror) = self.mirror {
                        mirror.set(*self.position);
                    }
                }
            }
            let mut guard = CopyBack {
                reader,
                position: &mut self.position,
                mirror: self.mirror,
            };
            return Some(guard.reader.read_wide(width));
        }
        let bit = usize::try_from(self.position).ok()?;
        let value = crate::theater::parser::bits::Bits(self.data).read(bit, width)?;
        self.position = self.position.checked_add(i64::try_from(width).ok()?)?;
        Some(value)
    }
    pub(crate) fn skip_signed(&mut self, width: i64) {
        self.position = self.position.wrapping_add(width);
        if let Some(mirror) = self.mirror {
            mirror.set(self.position);
        }
    }
    pub(crate) fn remaining_source_bits(&self) -> usize {
        usize::try_from(self.position).ok().map_or(0, |bit| {
            self.data.len().saturating_mul(8).saturating_sub(bit)
        })
    }
    pub(crate) fn address(&self) -> Option<usize> {
        usize::try_from(self.position).ok()
    }
    pub(crate) fn native_header(
        &mut self,
        width: i64,
        base: u32,
    ) -> crate::theater::parser::RecordHeader {
        let mut reader = crate::theater::parser::NativeFilmBits::new(self.data);
        reader.set_position(self.position);
        struct CopyBack<'r, 'd> {
            reader: crate::theater::parser::NativeFilmBits<'d>,
            cursor: &'r mut ComponentCursor<'d>,
        }
        impl Drop for CopyBack<'_, '_> {
            fn drop(&mut self) {
                self.cursor.position = self.reader.position();
                if let Some(mirror) = self.cursor.mirror {
                    mirror.set(self.cursor.position);
                }
            }
        }
        let mut guard = CopyBack {
            reader,
            cursor: self,
        };
        crate::theater::parser::decode_native_record_header(&mut guard.reader, width, base)
    }
}
