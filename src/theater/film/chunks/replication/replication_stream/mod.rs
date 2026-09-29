//! Ordered replication stream and packet envelope models.

use super::Packet;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::*;

/// One framed entry from a replication-stream chunk.
///
/// The enclosing chunk retains the exact payload bytes. `body` is the v41
/// decoder's interpretation and can fail without discarding those bytes.
pub type ReplicationStreamPacket = Packet<ReplicationStreamPacketBody>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Successfully decoded body of a [`ReplicationStreamPacket`].
///
/// The packet header selects one of these layouts by its numeric `packet_type`.
/// The enclosing [`super::ReplicationStreamChunk`] retains the source bytes and
/// exposes them through [`super::ReplicationStreamChunk::payload`].
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
    /// Packet type 7: the recorded end-of-stream packet.
    EndPacketBody,
}

impl super::PacketBody for ReplicationStreamPacketBody {
    fn packet_type(&self) -> u16 {
        match self {
            Self::FramePacketBody(_) => 0,
            Self::DatumsPacketBody(_) => 1,
            Self::KeyframesPacketBody(_) => 2,
            Self::EndPacketBody => 7,
            Self::RosterPacketBody => 8,
        }
    }
}
