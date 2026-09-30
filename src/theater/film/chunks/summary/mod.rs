//! Recorded summary data.
use super::*;
pub type SummaryChunk = Chunk<PacketStream<SummaryPacketBody>>;

/// Packet envelope used by type-3 summary chunks.
pub type SummaryPacket = Packet<SummaryPacketBody>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummaryPacketBody {
    /// Recorded event count followed by the complete unparsed record stream.
    /// No marker search or assumed identity-to-tail distance establishes records.
    Events {
        declared_events: u32,
        records: SummaryRecordStream,
    },
}

impl PacketBody for SummaryPacketBody {
    fn source_regions(&self) -> Vec<SourceRegion> {
        match self {
            Self::Events { records, .. } => vec![
                SourceRegion {
                    source: BitRange { start: 0, end: 32 },
                    kind: SourceRegionKind::Fields,
                },
                SourceRegion {
                    source: records.source,
                    kind: SourceRegionKind::Opaque,
                },
            ],
        }
    }

    fn packet_type(&self) -> u16 {
        match self {
            Self::Events { .. } => 9,
        }
    }
}

impl SummaryChunk {
    pub fn payload(&self, packet: &SummaryPacket) -> Option<&[u8]> {
        packet.payload(&self.data)
    }
}

pub mod models;
pub use models::SummaryRecordStream;
