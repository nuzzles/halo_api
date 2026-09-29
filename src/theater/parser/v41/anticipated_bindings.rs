//! Dated keyframe declarations used by the reference's later-keyframe fallback.
//! These are evidence for a binding, not decoded creation records.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AnticipatedDeclaration {
    pub chunk_index: i64,
    pub archetype: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AnticipatedBindings {
    /// Full entity ID, including the two high bits, to chronologically dated declarations.
    pub entries: BTreeMap<u32, Vec<AnticipatedDeclaration>>,
    pub declarations: usize,
    pub conflicts: usize,
    pub heads: BTreeMap<u8, usize>,
}
impl AnticipatedBindings {
    /// First declaration strictly after the current chunk. Earlier declarations do not qualify.
    pub(crate) fn after(&self, id: u32, chunk_index: i64) -> Option<AnticipatedDeclaration> {
        let values = self.entries.get(&id)?;
        values
            .get(values.partition_point(|d| d.chunk_index <= chunk_index))
            .copied()
    }
}
