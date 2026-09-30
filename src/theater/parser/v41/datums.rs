//! Type-1 entity datum table decoding.
use super::bits::Cursor;
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::datums::{
    DatumComponentMask, DatumDecodeError, DatumEntry, DatumTable,
};

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
            source: crate::theater::film::BitRange {
                start: r.position,
                end: r.position + 79,
            },
            flags: r.read(6).ok_or_else(invalid)? as u8,
            generation_tag: r.read(8).ok_or_else(invalid)? as u8,
            generation: r.read(32).ok_or_else(invalid)? as u32,
            view_mask: r.read(33).ok_or_else(invalid)?,
        });
    }
    let mut component_masks = Vec::with_capacity(count);
    for _ in 0..count {
        let source = crate::theater::film::BitRange {
            start: r.position,
            end: r.position + 256,
        };
        let mut words = [0; 4];
        for word in &mut words {
            *word = r.read(64).ok_or_else(invalid)?;
        }
        component_masks.push(DatumComponentMask { source, words });
    }
    let tail_source = crate::theater::film::BitRange {
        start: r.position,
        end: r.position + 160,
    };
    let mut tail_words = [0; 5];
    for word in &mut tail_words {
        *word = r.read(32).ok_or_else(invalid)? as u32;
    }
    let padding = r.read(padding_bits).ok_or_else(invalid)? as u8;
    Ok(DatumTable {
        entries,
        component_masks,
        tail_words,
        tail_source,
        padding_source: crate::theater::film::BitRange {
            start: consumed_bits,
            end: total,
        },
        padding,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_slots_keep_separate_wire_tables_and_unreversed_mask_words() {
        // Independently specified layout: two 79-bit headers, two 256-bit
        // masks, five 32-bit tail words, and two alignment bits (830+2 bits).
        let mut bytes = [0u8; 104];
        let mut bit = 0;
        let mut put = |width: usize, value: u64| {
            for shift in (0..width).rev() {
                bytes[bit / 8] |= (((value >> shift) & 1) as u8) << (7 - bit % 8);
                bit += 1;
            }
        };
        for (flags, tag, generation, mask) in
            [(5, 0x12, 0x34567890, 1u64 << 32), (6, 0x98, 0x76543210, 3)]
        {
            for (width, value) in [(6, flags), (8, tag), (32, generation), (33, mask)] {
                put(width, value);
            }
        }
        let masks = [[1u64 << 63, 2, 3, 4], [5, 6, 7, 0x0123456789abcdef]];
        for words in masks {
            for word in words {
                put(64, word);
            }
        }
        for word in [9, 10, 11, 12, 13] {
            put(32, word);
        }
        put(2, 3);
        let table = decode_datum_table(&bytes).unwrap();
        assert_eq!(table.entries.len(), 2);
        assert_eq!(
            table.entries[0].source,
            crate::theater::film::BitRange { start: 0, end: 79 }
        );
        assert_eq!(
            table.entries[1].source,
            crate::theater::film::BitRange {
                start: 79,
                end: 158
            }
        );
        assert_eq!(table.entries[0].view_mask, 1u64 << 32);
        assert_eq!(table.entries[1].view_mask, 3);
        assert_eq!(
            table.component_masks[0].source,
            crate::theater::film::BitRange {
                start: 158,
                end: 414
            }
        );
        assert_eq!(
            table.component_masks[1].source,
            crate::theater::film::BitRange {
                start: 414,
                end: 670
            }
        );
        assert_eq!(table.component_masks[0].words, masks[0]);
        assert_eq!(table.component_masks[1].words, masks[1]);
        assert_eq!(
            table.tail_source,
            crate::theater::film::BitRange {
                start: 670,
                end: 830
            }
        );
        assert_eq!(table.tail_words, [9, 10, 11, 12, 13]);
        assert_eq!(
            table.padding_source,
            crate::theater::film::BitRange {
                start: 830,
                end: 832
            }
        );
        assert_eq!(table.padding, 3);
        assert!(decode_datum_table(&bytes[..103]).is_err());
    }
}
