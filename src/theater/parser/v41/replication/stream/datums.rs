//! Packet type 1: datum/reference table entries.
use crate::theater::parser::v41::packet::PacketDecodeError;
use crate::theater::parser::v41::replication::source::ReplicationStreamPacketBody;

pub(crate) fn decode(payload: &[u8]) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    decode_datum_table(payload)
        .map(Box::new)
        .map(ReplicationStreamPacketBody::DatumsPacketBody)
        .map_err(|error| PacketDecodeError::InvalidDatumTable {
            packet_type: 1,
            reason: error,
        })
}

use crate::theater::parser::v41::replication::source::stream::models::datums::{
    DatumComponentMask, DatumDecodeError, DatumEntry, DatumTable,
};
use crate::theater::parser::wire::bits::Cursor;

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
            source: crate::theater::parser::v41::packet::BitRange {
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
        let source = crate::theater::parser::v41::packet::BitRange {
            start: r.position,
            end: r.position + 256,
        };
        let mut words = [0; 4];
        for word in &mut words {
            *word = r.read(64).ok_or_else(invalid)?;
        }
        component_masks.push(DatumComponentMask { source, words });
    }
    let tail_source = crate::theater::parser::v41::packet::BitRange {
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
        padding_source: crate::theater::parser::v41::packet::BitRange {
            start: consumed_bits,
            end: total,
        },
        padding,
    })
}
