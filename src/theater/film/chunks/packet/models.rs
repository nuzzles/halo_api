//! Models shared by replication-stream and summary packets.

use super::super::replication::{DatumDecodeError, FrameDecodeError};

/// One framed packet whose body type is selected by its enclosing chunk kind.
///
/// The header locates the original payload in the enclosing decompressed chunk.
/// A failed body decode therefore preserves the packet envelope and source bytes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Packet<T> {
    pub header: FilmPacketHeader,
    pub body: Result<T, PacketDecodeError>,
}

/// Original location in a decompressed chunk; bits are MSB-first and half-open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceSpan {
    pub chunk: i32,
    pub payload_byte: usize,
    pub bit: usize,
    pub end_bit: usize,
}

/// One checked byte-aligned film packet header.
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
