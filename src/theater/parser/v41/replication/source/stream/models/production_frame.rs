//! Reference data models.
use crate::theater::parser::v41::replication::source::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionEntityEnd {
    Marker(RecordHeader),
    /// Admission refused after the recorded DELTA header. No body boundary is known.
    Rejected {
        header: RecordHeader,
        reason: DeltaRejectionReason,
    },
    Failure(EntityViewStop),
    Truncated,
    PayloadBoundary,
    RecordLimit,
}

/// Parser binding-state reason for refusing a DELTA body. This is diagnostic
/// context, not an additional field read from the recording.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DeltaRejectionReason {
    /// No rejection reason was retained, so none can be recovered.
    #[default]
    Unknown,
    Unbound,
    OtherView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionFrame {
    pub messages: Option<DecodedFrameView>,
    pub records: Vec<EntityRecord>,
    pub controls: Option<DecodedFrameView>,
    pub entity_end: Option<ProductionEntityEnd>,
    pub end_bit: i64,
    /// Externally supplied record-local schemas used to decode DELTA bodies.
    /// These are parser context, never archetypes recorded in the body.
    pub external_delta_schemas: Vec<ExternalDeltaSchemaUse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalDeltaSchemaUse {
    pub start_bit: i64,
    pub id: u32,
    pub archetype: u32,
}
