//! Reference data models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatumEntry {
    pub flags: u8,
    pub generation_tag: u8,
    pub generation: u32,
    pub view_mask: u64,
    pub component_mask: [u64; 4],
}

/// Decoded body of a type-1
/// [`ReplicationStreamPacket`](crate::theater::film::chunks::replication::ReplicationStreamPacket).
///
/// Every wire slot remains present and ordered, including default-valued slots.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatumTable {
    pub entries: Vec<DatumEntry>,
    pub tail_words: [u32; 5],
    pub consumed_bits: usize,
    pub padding_bits: usize,
    pub padding: u8,
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
