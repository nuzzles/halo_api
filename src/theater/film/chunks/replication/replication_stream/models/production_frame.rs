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
    pub messages: Option<DecodedFrameView>,
    pub records: Vec<EntityRecord>,
    pub controls: Option<DecodedFrameView>,
    pub entity_end: Option<ProductionEntityEnd>,
    pub end_bit: i64,
}
