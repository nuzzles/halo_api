//! Type-1 entity datum table decoding.
use super::bits::Cursor;
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::datums::{
    DatumDecodeError, DatumEntry, DatumTable,
};

impl Default for DatumEntry {
    fn default() -> Self {
        Self {
            flags: 0,
            generation_tag: 0,
            generation: 1,
            view_mask: 0,
            component_mask: [0; 4],
        }
    }
}

/// Decode one complete decompressed type-1 packet payload. The layout has 79 bits
/// per datum, a separate 256-bit bitmap per slot, five tail words and byte alignment.
pub(crate) fn decode_datum_table(data: &[u8]) -> Result<DatumTable, DatumDecodeError> {
    let invalid = || DatumDecodeError::Truncated;
    let total = data.len().checked_mul(8).ok_or_else(invalid)?;
    if total < 160 {
        return Err(DatumDecodeError::NoEntries { bytes: data.len() });
    }
    let count = (total - 160 + 7) / 335;
    if count == 0 {
        return Err(DatumDecodeError::NoEntries { bytes: data.len() });
    }
    if count > 8191 {
        return Err(DatumDecodeError::AboveCapacity { slots: count });
    }
    let consumed_bits = count * 335 + 160;
    let remainder = total as i64 - consumed_bits as i64;
    if !(0..=7).contains(&remainder) {
        return Err(DatumDecodeError::Misaligned {
            total_bits: total,
            slots: count,
            remainder,
        });
    }
    let padding_bits = remainder as usize;
    let mut r = Cursor::new(data, 0).ok_or_else(invalid)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(DatumEntry {
            flags: r.read(6).ok_or_else(invalid)? as u8,
            generation_tag: r.read(8).ok_or_else(invalid)? as u8,
            generation: r.read(32).ok_or_else(invalid)? as u32,
            view_mask: r.read(33).ok_or_else(invalid)?.reverse_bits() >> 31,
            component_mask: [0; 4],
        });
    }
    for entry in &mut entries {
        for word in &mut entry.component_mask {
            *word = r.read(64).ok_or_else(invalid)?.reverse_bits();
        }
    }
    let mut tail_words = [0; 5];
    for word in &mut tail_words {
        *word = r.read(32).ok_or_else(invalid)? as u32;
    }
    let padding = r.read(padding_bits).ok_or_else(invalid)? as u8;
    Ok(DatumTable {
        entries,
        tail_words,
        consumed_bits,
        padding_bits,
        padding,
    })
}
