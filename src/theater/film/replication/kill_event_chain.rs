//! Native data models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KillEventFields {
    pub killer: i32,
    pub victim: i32,
    pub assist: i32,
    /// Raw integer percentage; values above 100 are retained.
    pub killer_pct: u32,
    /// Meaningful as a damage share only when assist is present.
    pub assist_pct: u32,
    /// Unresolved one-bit field, retained without interpretation.
    pub flag: u8,
    /// -1 means a mandatory field exceeded the packet bounds.
    pub end: i64,
}

/// Read order and source ranges from the native event-layout walker. Scalar
/// values remain unnamed unless their semantics are established elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NativeEventFieldStage {
    Header,
    References,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NativeEventFieldValue {
    Scalar(u64),
    /// A supported layout skipped this region without interpreting its fields.
    Opaque,
    /// The requested read exceeded the source; no zero value is substituted.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeEventField {
    pub bit: usize,
    pub width: usize,
    pub stage: NativeEventFieldStage,
    pub value: NativeEventFieldValue,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeEventRecord {
    pub start_bit: usize,
    /// Parent reader position on return. A failed nested code-85 read leaves
    /// this at body_start_bit; its attempted fields still remain in field_range.
    pub end_bit: usize,
    pub code: Option<u8>,
    pub body_start_bit: Option<usize>,
    pub field_range: [usize; 2],
    /// The layout reached its end; opaque fields can still be present.
    pub layout_complete: bool,
    /// Direct code-85 indices and integer shares, without player resolution.
    pub kill_fields: Option<KillEventFields>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NativeEventListStop {
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
pub struct NativeEventListRead {
    pub start_bit: usize,
    pub end_bit: usize,
    pub gate15: Option<bool>,
    pub records: Vec<NativeEventRecord>,
    pub fields: Vec<NativeEventField>,
    pub stop: NativeEventListStop,
}
