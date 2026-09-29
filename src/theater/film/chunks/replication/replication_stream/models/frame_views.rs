//! Reference data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameViewStop {
    Complete,
    Truncated,
    Unsupported { reason: String },
    RecordLimit,
}

/// One completely read reference kind-0 control payload. Optional fields remain
/// absent when their gates are closed. Partial payload fields stay in the view.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ControlEntry {
    pub start_bit: i64,
    pub end_bit: i64,
    pub index: u8,
    pub baseline: Option<u8>,
    pub short: Option<u8>,
    pub analog: Option<[u8; 2]>,
    pub third_analog: Option<u8>,
    pub extra: Option<u8>,
    pub flags: Option<u8>,
    pub action: Option<crate::theater::film::ActionBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecodedFrameView {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_entries: Vec<ControlEntry>,
    pub start_bit: i64,
    /// First unread bit, including when an unsupported branch is encountered.
    pub end_bit: i64,
    pub kinds: Vec<u8>,
    pub fields: Vec<ComponentField>,
    pub stop: FrameViewStop,
}
