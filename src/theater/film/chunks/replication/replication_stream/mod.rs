//! Ordered replication stream and packet envelope models.

use super::{FilmPacketHeader, PacketDecodeError, ReplicationStreamChunk};
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::*;

/// The replication stream is an ordered sequence of differently typed packets.
/// The packet header’s packet_type determines how its payload is decoded.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStream {
    pub chunks: Vec<ReplicationStreamChunk>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// One framed entry from a replication-stream chunk.
///
/// The enclosing chunk retains the exact payload bytes. `body` is the v41
/// decoder's interpretation and can fail without discarding those bytes.
pub struct ReplicationStreamPacket {
    /// Recorded framing metadata and the payload's location in its chunk.
    pub header: FilmPacketHeader,
    /// Typed decoding, kept separate from the recorded bytes.
    pub body: Result<ReplicationStreamPacketBody, PacketDecodeError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
/// Successfully decoded replication packet payload.
pub enum ReplicationStreamPacketBody {
    /// Packet type 0: incremental events, entity records, and controls.
    Frame(Box<FramePacket>),
    /// Packet type 1: datum/reference table entries.
    Datums(DatumTable),
    /// Packet type 2: entity baseline records.
    Keyframes(KeyframeTable),
    /// A packet type without a v41 body decoder. Inspect the packet payload.
    Unknown,
}
