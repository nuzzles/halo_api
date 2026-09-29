//! Reference data models.
use super::replication::{DatumDecodeError, FrameDecodeError};
/// Original location in a decompressed chunk; bits are MSB-first and half-open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceSpan {
    /// Chunk index.
    pub chunk: i32,
    /// Packet payload byte offset in that chunk (zero for non-packet data).
    pub payload_byte: usize,
    /// First bit relative to the payload.
    pub bit: usize,
    /// First bit after the checked window; does not imply a complete record.
    pub end_bit: usize,
}

/// One checked byte-aligned replication packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmPacketHeader {
    pub chunk_index: i32,
    pub packet_type: u16,
    pub byte_2: u8,
    pub byte_3: u8,
    pub payload_offset: usize,
    pub payload_size: usize,
    pub timestamp_us: u64,
}

/// Why a structurally recognized packet could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum PacketDecodeError {
    #[error("packet type {packet_type} has no supported v41 layout")]
    UnsupportedLayout { packet_type: u16 },
    #[error("packet type {packet_type} has an invalid datum table: {reason}")]
    InvalidDatumTable {
        packet_type: u16,
        reason: DatumDecodeError,
    },
    #[error("packet type {packet_type} has an invalid keyframe profile: {reason}")]
    InvalidKeyframeProfile {
        packet_type: u16,
        reason: FrameDecodeError,
    },
    #[error("packet type {packet_type} has a truncated summary event count")]
    TruncatedSummaryCount { packet_type: u16 },
}
