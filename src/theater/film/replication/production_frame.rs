//! Native data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProductionEntityEnd {
    Marker(RecordHeader),
    Rejected {
        header: RecordHeader,
        reason: FilmViewAdmission,
    },
    Failure(EntityViewStop),
    Truncated,
    PayloadBoundary,
    RecordLimit,
}

/// Native entity-view admission counters for one production frame.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProductionAdmissionDiagnostics {
    pub rejected_unbound: u64,
    pub rejected_other_view: u64,
    pub anticipated_bindings: std::collections::BTreeMap<u32, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProductionFrame {
    /// Failed native header reads and refused NEW binding mutations.
    #[serde(default, skip_serializing_if = "FilmReadDiagnostics::is_empty")]
    pub header_diagnostics: FilmReadDiagnostics,
    /// None in older exports that did not retain these native counters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admission_diagnostics: Option<ProductionAdmissionDiagnostics>,
    /// Optional 32-bit per-record words, in read order, including End/rejected
    /// headers. Signed ranges are payload-relative; negative positions identify
    /// absent head bits, and padded_bits identifies the absent tail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub record_prefixes: Vec<ComponentField>,
    pub messages: Option<DecodedFrameView>,
    pub records: Vec<EntityRecord>,
    pub controls: Option<DecodedFrameView>,
    pub entity_end: Option<ProductionEntityEnd>,
    pub views_completed: usize,
    pub end_bit: i64,
    /// Number of synthetic zero tail bits consumed by the native reader.
    #[serde(default)]
    pub padded_bits: usize,
}
