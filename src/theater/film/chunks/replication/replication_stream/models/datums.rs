//! Reference data models.
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatumEntry {
    pub flags: u8,
    pub generation_tag: u8,
    pub generation: u32,
    pub view_mask: u64,
    pub component_mask: [u64; 4],
}

/// Lossless sparse representation: absent entries equal `default_entry`, not unknown.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatumTable {
    pub slot_count: usize,
    pub default_entry: DatumEntry,
    pub entries: BTreeMap<usize, DatumEntry>,
    pub tail_words: [u32; 5],
    pub consumed_bits: usize,
    pub padding_bits: usize,
    pub padding: u8,
}
