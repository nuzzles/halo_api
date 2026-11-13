//! Reference data models.
use crate::theater::parser::v41::chunks::replication::source::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameViewStop {
    Complete,
    Truncated,
    Unsupported {
        reason: String,
    },
    /// Resource budget exhausted after a present continuation bit. This is not
    /// a claimed wire-format maximum; following kind/body bits remain unparsed.
    RecordLimit,
}

/// One completely read control payload, in wire order. Partial payload fields
/// remain in the enclosing view. Extents start after the recorded kind field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ControlEntry {
    pub start_bit: i64,
    pub end_bit: i64,
    pub baseline: Option<u8>,
    pub payload: ControlPayload,
}

impl ControlEntry {
    /// The explicitly recorded player index, when this kind carries one.
    pub fn index(&self) -> Option<u8> {
        match &self.payload {
            ControlPayload::Input(input) => Some(input.index),
            ControlPayload::Commands { index, .. } => Some(*index),
            ControlPayload::Kind2 { .. } => None,
        }
    }
}

/// The three supported control payload layouts. All gates and exact command /
/// secondary-block fields are retained in `DecodedFrameView::fields`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ControlPayload {
    Input(ControlInput),
    Commands {
        index: u8,
        /// None is a closed list gate; Some(empty) is a recorded zero count.
        /// Each range identifies one complete command in source order.
        records: Option<Vec<BitRange>>,
    },
    /// The meaning of this optional 13-bit field is not established.
    Kind2 {
        value: Option<u16>,
    },
}

/// Source-read projections of a kind-0 input payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ControlInput {
    pub index: u8,
    pub short: Option<u8>,
    pub analog: Option<[u8; 2]>,
    pub third_analog: Option<u8>,
    pub extra: Option<u8>,
    pub flags: Option<u8>,
    /// Complete secondary unit block, excluding its presence gate.
    pub secondary: Option<BitRange>,
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
