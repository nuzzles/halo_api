//! Reference data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProductionEntityEnd {
    Marker(RecordHeader),
    Rejected { header: RecordHeader },
    Failure(EntityViewStop),
    Truncated,
    PayloadBoundary,
    RecordLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProductionFrame {
    /// Optional 32-bit per-record words, in read order, including End/rejected
    /// headers. Signed ranges are payload-relative; negative positions identify
    /// decoder cursor states outside the recorded payload.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub record_prefixes: Vec<ComponentField>,
    pub messages: Option<DecodedFrameView>,
    pub records: Vec<EntityRecord>,
    pub controls: Option<DecodedFrameView>,
    pub entity_end: Option<ProductionEntityEnd>,
    pub views_completed: usize,
    pub end_bit: i64,
}
