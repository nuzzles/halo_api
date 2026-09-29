//! Native data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameViewStop {
    Complete,
    Truncated,
    Unsupported { reason: String },
    RecordLimit,
}

/// One completely read native kind-0 control payload. Optional fields remain
/// absent when their gates are closed. Partial payload fields stay in the view.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeControlEntry {
    pub start_bit: i64,
    pub end_bit: i64,
    pub index: u8,
    pub baseline: Option<u8>,
    pub short: Option<u8>,
    pub analog: Option<[u8; 2]>,
    pub third_analog: Option<u8>,
    pub extra: Option<u8>,
    pub flags: Option<u8>,
    pub action: Option<crate::theater::film::NativeActionBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecodedFrameView {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_entries: Vec<NativeControlEntry>,
    /// Includes the precise failed bounded read or grouped guard, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Box<crate::theater::film::FilmReadDiagnostics>>,
    pub start_bit: i64,
    /// First unread bit, including when an unsupported branch is encountered.
    pub end_bit: i64,
    /// Synthetic bits consumed by native action decoding beyond the source.
    /// A padded view remains truncated rather than becoming a complete view.
    #[serde(default, skip_serializing_if = "no_padding")]
    pub padded_bits: usize,
    pub kinds: Vec<u8>,
    pub fields: Vec<ComponentField>,
    pub stop: FrameViewStop,
}

fn no_padding(bits: &usize) -> bool {
    *bits == 0
}
