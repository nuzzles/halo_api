//! Ordered replication stream and packet envelope models.

use super::{Packet, ReplicationStreamChunk};
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::*;

/// The replication stream is an ordered sequence of differently typed packets.
/// The packet header’s packet_type determines how its payload is decoded.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStream {
    pub chunks: Vec<ReplicationStreamChunk>,
}

/// One framed entry from a replication-stream chunk.
///
/// The enclosing chunk retains the exact payload bytes. `body` is the v41
/// decoder's interpretation and can fail without discarding those bytes.
pub type ReplicationStreamPacket = Packet<ReplicationStreamPacketBody>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Successfully decoded body of a [`ReplicationStreamPacket`].
///
/// The packet header selects one of these layouts by its numeric `packet_type`.
/// The enclosing [`ReplicationStreamChunk`] retains the source bytes and exposes
/// them through [`ReplicationStreamChunk::payload`].
pub enum ReplicationStreamPacketBody {
    /// Packet type 0: incremental events, entity records, and controls in a
    /// [`FramePacket`].
    FramePacketBody(Box<FramePacket>),
    /// Packet type 1: datum/reference entries in a [`DatumTable`].
    DatumsPacketBody(Box<DatumTable>),
    /// Packet type 2: entity baselines in a [`KeyframeTable`].
    KeyframesPacketBody(Box<KeyframeTable>),
    /// Packet type 8: a known roster packet with no established v41 body model.
    RosterPacketBody,
    /// A packet type without a v41 body decoder. Inspect the packet payload.
    UnknownPacketBody,
}
