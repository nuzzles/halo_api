//! Signed reference traversal cursor. Slice addresses remain checked at access.
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::header::decode_reference_record_header;
use crate::theater::parser::v41::reference_bits::FilmBits;
#[derive(Clone, Copy)]
pub(crate) struct ComponentCursor<'a> {
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
    /// Retain the supplied coordinate; source guards validate it before reading.
    pub(crate) fn guarded(data: &'a [u8], position: i64) -> Self {
        Self::signed(data, position, None)
    }
    pub(crate) fn source_bits(&self) -> usize {
        self.data.len().saturating_mul(8)
    }
    pub(crate) fn fits_source(&self, width: i64) -> bool {
        let Some(end) = self.position.checked_add(width) else {
            return false;
        };
        self.position >= 0
            && width >= 0
            && usize::try_from(end).is_ok_and(|end| end <= self.source_bits())
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
    ) -> crate::theater::film::RecordHeader {
        let mut reader = FilmBits::new(self.data);
        reader.set_position(self.position);
        struct CopyBack<'r, 'd> {
            reader: FilmBits<'d>,
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
        decode_reference_record_header(&mut guard.reader, width, base)
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentCursor;

    #[test]
    fn source_guard_requires_a_nonnegative_nonoverflowing_range() {
        let bytes = [0u8; 2];
        for (start, width) in [(-1, 1), (0, -1), (i64::MAX, 1), (15, 2), (17, 0)] {
            assert!(!ComponentCursor::signed(&bytes, start, None).fits_source(width));
        }
        for (start, width) in [(0, 16), (15, 1), (16, 0)] {
            assert!(ComponentCursor::signed(&bytes, start, None).fits_source(width));
        }
    }
}
