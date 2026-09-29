//! Record headers and registry-driven delta traversal. The caller supplies a checked
//! boundary and entity binding; this layer never searches for a plausible header.
use super::bits::Cursor;
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::records::{
    RecordHeader, RecordKind,
};
use serde::{Deserialize, Serialize};

/// Entity ID widths belong to the replication view, not a universal film constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordIdLayout {
    pub low_bits: usize,
    pub base: u32,
}

/// Read the prefix-coded kind followed by the view's low ID and two generation bits.
pub(crate) fn decode_record_header(
    data: &[u8],
    bit: usize,
    layout: RecordIdLayout,
) -> Option<RecordHeader> {
    if layout.low_bits > 30 || layout.base > 0x3fff_ffff {
        return None;
    }
    decode_header_cursor(Cursor::new(data, bit)?, layout)
}

pub(crate) fn decode_header_cursor(
    mut r: Cursor<'_>,
    layout: RecordIdLayout,
) -> Option<RecordHeader> {
    let bit = r.position;
    let kind = if r.bit()? {
        RecordKind::Delta
    } else {
        match r.read(2)? {
            0 => RecordKind::End,
            1 => RecordKind::New,
            2 => RecordKind::Delete,
            _ => RecordKind::Delta,
        }
    };
    let id = if kind == RecordKind::End {
        None
    } else {
        let low = (r.read(layout.low_bits)? as u32).checked_add(layout.base)?;
        if low > 0x3fff_ffff {
            return None;
        }
        Some(((r.read(2)? as u32) << 30) | low)
    };
    Some(RecordHeader {
        kind,
        id,
        start_bit: bit as i64,
        end_bit: r.position as i64,
    })
}

/// Decode a reference header with signed cursor coordinates and reference ID widths.
/// Nonpositive ID widths omit the low-ID read. Positive widths use the pinned
/// 64-bit width domain, independent of the host pointer size. A reference read
/// panic leaves the caller's reader at its reached position.
pub(crate) fn decode_reference_record_header(
    reader: &mut super::FilmBits<'_>,
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
        kind,
        id,
        start_bit,
        end_bit: reader.position(),
    }
}

pub(crate) fn decode_frame_header_signed(
    data: &[u8],
    bit: i64,
    encoding: &super::FrameEncoding,
) -> Option<RecordHeader> {
    let width = encoding
        .reference_id_low_bits
        .or_else(|| i64::try_from(encoding.ids.low_bits).ok())?;
    let mut reader = super::FilmBits::new(data);
    reader.set_position(bit);
    Some(decode_reference_record_header(
        &mut reader,
        width,
        encoding.ids.base,
    ))
}
