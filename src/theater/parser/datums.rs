//! Type-1 entity datum tables, ported from LevelUp `type1_datums.go`.
use super::{DecodeError, bits::Cursor};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatumEntry {
    pub flags: u8,
    pub generation_tag: u8,
    pub generation: u32,
    pub view_mask: u64,
    pub component_mask: [u64; 4],
}

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

/// Lossless sparse representation: absent entries equal `default_entry`, not unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatumTable {
    pub slot_count: usize,
    pub default_entry: DatumEntry,
    pub entries: BTreeMap<usize, DatumEntry>,
    pub tail_words: [u32; 5],
    pub consumed_bits: usize,
    pub padding_bits: usize,
    pub padding: u8,
}

/// Native datum-block size refusals, preserving the quantities in each error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub(crate) enum DatumTableError {
    #[error("bloc de datums : {bytes} octets ne portent aucune entree")]
    NoEntries { bytes: usize },
    #[error("bloc de datums : {slots} entrees derivees au-dela du cap 8191")]
    AboveCapacity { slots: usize },
    #[error("bloc de datums : {total_bits} bits pour {slots} entrees, reste {remainder}")]
    Misaligned {
        total_bits: usize,
        slots: usize,
        remainder: i64,
    },
}

/// Decode one complete decompressed type-1 packet payload. The layout has 79 bits
/// per datum, a separate 256-bit bitmap per slot, five tail words and byte alignment.
pub(crate) fn decode_datum_table(data: &[u8]) -> Result<DatumTable, DecodeError> {
    let invalid = || DecodeError::Inconsistent("invalid type-1 datum table length".into());
    let total = data.len().checked_mul(8).ok_or_else(invalid)?;
    if total < 160 {
        return Err(DatumTableError::NoEntries { bytes: data.len() }.into());
    }
    let count = (total - 160 + 7) / 335;
    if count == 0 {
        return Err(DatumTableError::NoEntries { bytes: data.len() }.into());
    }
    if count > 8191 {
        return Err(DatumTableError::AboveCapacity { slots: count }.into());
    }
    let consumed_bits = count * 335 + 160;
    let remainder = total as i64 - consumed_bits as i64;
    if !(0..=7).contains(&remainder) {
        return Err(DatumTableError::Misaligned {
            total_bits: total,
            slots: count,
            remainder,
        }
        .into());
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
    let default_entry = DatumEntry::default();
    let entries = entries
        .into_iter()
        .enumerate()
        .filter(|(_, e)| *e != default_entry)
        .collect();
    Ok(DatumTable {
        slot_count: count,
        default_entry,
        entries,
        tail_words,
        consumed_bits,
        padding_bits,
        padding,
    })
}
