//! Native keyframe table fields and retained historical comparison models.
use super::KeyframeRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyframeChainStop {
    End,
    Header,
    Desync,
    Slot,
    Budget,
    InvalidEncoding,
    /// Payload remains after all 8,191 native slots and byte alignment.
    TrailingData,
    /// A required slot header or packet prefix extends beyond the payload.
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeChainAttempt {
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    pub archetype: u32,
    /// Absent for a no-archetype entry in historical reference reads. Native
    /// vacant entries are retained in `KeyframeTable::slots.headers` instead.
    pub record: Option<KeyframeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Entity baselines decoded from a type-2
/// [`ReplicationStreamPacket`](crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacket).
pub struct KeyframeTable {
    pub records: Vec<KeyframeChainAttempt>,
    pub stop: KeyframeChainStop,
    /// Native table framing. Absent only on legacy reference-model reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slots: Option<KeyframeSlotsRead>,
}

/// Exact fixed slot headers in physical table order. No-archetype entries have
/// no body but still occupy a slot. An ID of all ones is not a terminator.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeSlotsRead {
    pub configuration: Option<bool>,
    pub headers: Vec<KeyframeSlotHeader>,
    pub end_bit: usize,
    pub tail: crate::theater::parser::packet::BitRange,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeSlotHeader {
    pub start_bit: usize,
    pub id: u32,
    pub archetype: u32,
    pub word: u32,
    pub flags: u8,
    pub byte: u8,
}
