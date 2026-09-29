//! Recorded summary data.
use super::*;
pub type SummaryChunk = Chunk<PacketStream<SummaryPacketBody>>;

/// Packet envelope used by type-3 summary chunks.
pub type SummaryPacket = Packet<SummaryPacketBody>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummaryPacketBody {
    /// Guarded captured-layout reads in wire order; intervening state is opaque.
    Events {
        declared_events: u32,
        segments: Vec<SummarySegment>,
    },
}

impl PacketBody for SummaryPacketBody {
    fn packet_type(&self) -> u16 {
        match self {
            Self::Events { .. } => 9,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummarySegment {
    Event(SummaryEvent),
    Opaque { start_bit: usize, end_bit: usize },
}
impl SummaryChunk {
    pub fn payload(&self, packet: &SummaryPacket) -> Option<&[u8]> {
        let range = packet.source.payload;
        self.data.get(range.start..range.end)
    }
}

pub mod models;
pub use models::SummaryEvent;
