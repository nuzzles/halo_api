//! Type-1 packet sections in their recorded order.
use crate::theater::film::BitRange;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatumEntry {
    /// The 79-bit slot-header range, before the separate mask table.
    pub source: BitRange,
    pub flags: u8,
    pub generation_tag: u8,
    pub generation: u32,
    /// The recorded 33-bit word; the first wire bit is bit 32.
    pub view_mask: u64,
}

/// Decoded body of a type-1
/// [`ReplicationStreamPacket`](crate::theater::film::chunks::replication::ReplicationStreamPacket).
///
/// Every wire slot remains present and ordered, including default-valued slots.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatumTable {
    pub entries: Vec<DatumEntry>,
    /// Separate 256-bit masks in the same slot order, following all headers.
    pub component_masks: Vec<DatumComponentMask>,
    pub tail_words: [u32; 5],
    pub tail_source: BitRange,
    pub padding_source: BitRange,
    pub padding: u8,
}

/// One slot's recorded mask block in the separate component-mask table.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatumComponentMask {
    pub source: BitRange,
    /// Four MSB-first wire words, without bit reversal or semantic indexing.
    pub words: [u64; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum DatumDecodeError {
    #[error("{bytes} bytes cannot contain a datum entry")]
    NoEntries { bytes: usize },
    #[error("derived slot count {slots} exceeds the v41 capacity of 8191")]
    AboveCapacity { slots: usize },
    #[error("{total_bits} bits for {slots} slots leaves an invalid remainder of {remainder}")]
    Misaligned {
        total_bits: usize,
        slots: usize,
        remainder: i64,
    },
    #[error("datum table ended before all declared fields were read")]
    Truncated,
}
