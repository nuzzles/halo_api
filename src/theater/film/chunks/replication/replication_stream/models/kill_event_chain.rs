//! Reference data models.
/// Read order and source ranges from the reference event-layout walker. Scalar
/// values remain unnamed unless their semantics are established elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventFieldStage {
    Header,
    References,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventFieldValue {
    Scalar(u64),
    /// A supported layout skipped this region without interpreting its fields.
    Opaque,
    /// The requested read exceeded the source; no zero value is substituted.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventField {
    /// Payload-relative start of the read or read attempt.
    pub bit: usize,
    /// Requested width. For `Unavailable`, this is not a recorded source range.
    pub width: usize,
    pub stage: EventFieldStage,
    pub value: EventFieldValue,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventRecord {
    pub start_bit: usize,
    /// End of the furthest source-backed field, excluding unavailable attempts.
    /// Nested partial reads remain inside this extent even if their reader
    /// returned without advancing the parent cursor.
    pub end_bit: usize,
    pub code: Option<u8>,
    pub body_start_bit: Option<usize>,
    /// Header, reference, and body fields in their recorded read order.
    pub fields: Vec<EventField>,
    /// The layout reached its end; opaque fields can still be present.
    pub layout_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventListStop {
    Terminator,
    SourceBoundary,
    Truncated,
    UnsupportedCode,
    UnsupportedReferences,
    UnsupportedBody,
    MissingRuntimeGate15,
    RecordLimit,
    InvalidStart,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventListRead {
    pub start_bit: usize,
    pub end_bit: usize,
    pub gate15: Option<bool>,
    pub records: Vec<EventRecord>,
    /// Recorded list terminator, if reached.
    pub terminator: Option<EventField>,
    pub stop: EventListStop,
}
