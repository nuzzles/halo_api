//! Models shared by replication-stream and summary packets.

use super::super::replication::{DatumDecodeError, FrameDecodeError};

/// One framed packet whose body type is selected by its enclosing chunk kind.
///
/// The header locates the original payload in the enclosing decompressed chunk.
/// A failed body decode therefore preserves the packet envelope and source bytes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Packet<T> {
    pub source: PacketSource,
    pub header: FilmPacketHeader,
    pub body: PacketRead<T>,
}

/// Associates a decoded body variant with its recorded packet type code.
pub trait PacketBody {
    fn packet_type(&self) -> u16;
}

impl<T: PacketBody> Packet<T> {
    /// Checks that a decoded body agrees with the wire header. Opaque bodies
    /// carry no typed variant and therefore cannot conflict.
    pub fn body_type_matches_header(&self) -> bool {
        match &self.body {
            PacketRead::Complete(body) => body.packet_type() == self.header.packet_type,
            PacketRead::Opaque { .. } => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PacketStream<T> {
    pub packets: Vec<Packet<T>>,
    /// Source byte ranges that framing did not classify as packets.
    pub opaque: Vec<ByteRange>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PacketRead<T> {
    Complete(T),
    Opaque { reason: PacketDecodeError },
}

impl<T> PacketRead<T> {
    pub fn complete(&self) -> Option<&T> {
        match self {
            Self::Complete(value) => Some(value),
            Self::Opaque { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BitRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PacketSource {
    pub header: ByteRange,
    pub payload: ByteRange,
}

/// One checked byte-aligned film packet header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmPacketHeader {
    pub packet_type: u16,
    pub unknown_2: [u8; 2],
    pub payload_size: u32,
    pub timestamp_us: u64,
}

/// Why a structurally recognized packet could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum PacketDecodeError {
    #[error("packet type {packet_type} has no established v41 layout")]
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
