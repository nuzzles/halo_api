//! Record headers and registry-driven delta traversal. The caller supplies a checked
//! boundary and entity binding; this layer never searches for a plausible header.
use crate::theater::film::chunks::replication::replication_stream::models::records::{
    RecordHeader, RecordKind,
};
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::reference_bits::FilmBits;
use serde::{Deserialize, Serialize};

/// Entity ID widths belong to the replication view, not a universal film constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordIdLayout {
    pub low_bits: usize,
    pub base: u32,
}

/// Decode a reference header with signed cursor coordinates and reference ID widths.
/// Nonpositive ID widths omit the low-ID read. Positive widths use the pinned
/// 64-bit width domain, independent of the host pointer size. A reference read
/// panic leaves the caller's reader at its reached position.
pub(crate) fn decode_reference_record_header(
    reader: &mut FilmBits<'_>,
    width: i64,
    base: u32,
) -> RecordHeader {
    let start_bit = reader.position();
    let kind = if reader.read_bit() {
        RecordKind::Delta
    } else {
        match reader.read(2) {
            0 => RecordKind::End,
            1 => RecordKind::New,
            2 => RecordKind::Delete,
            _ => RecordKind::Delta,
        }
    };
    let id = if kind == RecordKind::End {
        None
    } else {
        let low = if width > 0 {
            reader.read_wide(width as u64) as u32
        } else {
            0
        };
        let low = low.wrapping_add(base) & 0x3fff_ffff;
        let tag = reader.read(2) as u32;
        Some((tag << 30) | low)
    };
    RecordHeader {
        prefix: None,
        kind,
        id,
        start_bit,
        end_bit: reader.position(),
    }
}

pub(crate) fn decode_frame_header_signed(
    data: &[u8],
    bit: i64,
    encoding: &RecordLayout,
) -> Option<RecordHeader> {
    if usize::try_from(bit).ok()? >= data.len().checked_mul(8)? {
        return None;
    }
    let width = encoding
        .reference_id_low_bits
        .or_else(|| i64::try_from(encoding.ids.low_bits).ok())?;
    let mut reader = FilmBits::new(data);
    reader.set_position(bit);
    let header = decode_reference_record_header(&mut reader, width, encoding.ids.base);
    (usize::try_from(header.end_bit).ok()? <= data.len().checked_mul(8)?).then_some(header)
}
