use super::*;

/// A summary event decoded independently from replication records.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryEvent {
    pub xuid: u64,
    /// Recorded UTF-16 code units, excluding the zero terminator.
    pub gamertag_utf16: Vec<u16>,
    /// Recorded film-relative timestamp in milliseconds.
    pub timestamp_ms: u32,
    /// Raw metadata byte (medal code for medal events).
    pub metadata: u8,
    /// Raw medal flag.
    pub medal_flag: u8,
    pub type_code: u8,
    /// Exact 60-byte event tail; intervening identity state is not decoded.
    pub source: BitRange,
    /// Recorded XUID field, independently of the roster and gamertag.
    pub identity_source: BitRange,
}
