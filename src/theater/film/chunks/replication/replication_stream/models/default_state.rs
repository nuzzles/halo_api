//! Default-state sections preceding component updates in NEW/baseline records.
use crate::theater::film::{BitRange, ComponentField};

/// Source-backed default-state section. Guards and masks remain on the enclosing
/// record. This section owns its ordered body fields and any default-state check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DefaultState {
    pub source: BitRange,
    pub fields: Vec<ComponentField>,
    pub status: DefaultStateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefaultStateStatus {
    Complete,
    /// An established layout skips fields without decoding their structure.
    Opaque,
    Unsupported,
    Truncated,
}
