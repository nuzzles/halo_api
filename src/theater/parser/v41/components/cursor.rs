//! Signed reference traversal cursor. Slice addresses remain checked at access.
#[derive(Clone, Copy)]
pub(super) struct ComponentCursor<'a> {
    data: &'a [u8],
    pub position: i64,
    mirror: Option<&'a std::cell::Cell<i64>>,
}
impl<'a> ComponentCursor<'a> {
    pub(crate) fn new(data: &'a [u8], position: usize) -> Option<Self> {
        (position <= data.len().checked_mul(8)?).then_some(Self {
            data,
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
            position,
            mirror,
        }
    }
    /// Reference view guards use signed wrapping position + width before reading.
    pub(crate) fn guarded(data: &'a [u8], position: i64) -> Self {
        Self::signed(data, position, None)
    }
    pub(crate) fn source_bits(&self) -> usize {
        self.data.len().saturating_mul(8)
    }
    pub(crate) fn fits_source(&self, width: i64) -> bool {
        self.position.wrapping_add(width) <= (self.data.len() as i64).wrapping_mul(8)
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
    pub(crate) fn raw_bits(&self, start: i64, width: u64) -> Option<crate::theater::film::RawBits> {
        crate::theater::film::RawBits::from_source(
            self.data,
            usize::try_from(start).ok()?,
            usize::try_from(width).ok()?,
        )
    }
    pub(crate) fn reference_header(
        &mut self,
        width: i64,
        base: u32,
    ) -> crate::theater::parser::v41::RecordHeader {
        let mut reader = crate::theater::parser::v41::FilmBits::new(self.data);
        reader.set_position(self.position);
        struct CopyBack<'r, 'd> {
            reader: crate::theater::parser::v41::FilmBits<'d>,
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
        crate::theater::parser::v41::decode_reference_record_header(&mut guard.reader, width, base)
    }
}
