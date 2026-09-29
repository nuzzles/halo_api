//! Recorded summary data.
use super::*;
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SummaryEvents {
    pub chunks: Vec<SummaryChunk>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryChunk {
    pub source: FilmChunk,
    pub source_position: usize,
    pub data: Vec<u8>,
    pub packets: Vec<SummaryPacket>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryPacket {
    pub header: FilmPacketHeader,
    pub body: SummaryPacketBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummaryPacketBody {
    /// Guarded captured-layout reads in wire order; intervening state is opaque.
    Events {
        declared_events: u32,
        events: Vec<SummaryEvent>,
    },
    /// The summary reader could not start. The payload remains in its chunk.
    Refused { message: String },
    /// No summary reader selected; this does not imply an empty payload.
    Opaque,
}
impl SummaryChunk {
    pub fn payload(&self, packet: &SummaryPacket) -> Option<&[u8]> {
        let header = packet.header;
        self.data
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
impl SummaryEvents {
    /// Recorded summary entries in source order, without inferred player linkage.
    pub fn events(&self) -> impl Iterator<Item = &SummaryEvent> {
        self.chunks
            .iter()
            .flat_map(|c| &c.packets)
            .flat_map(|p| match &p.body {
                SummaryPacketBody::Events { events, .. } => events.as_slice(),
                _ => &[],
            })
    }
}

pub mod medals;
pub use medals::{FilmMedalDefinition, MedalAward};

pub mod models;
pub use models::{SummaryEvent, SummaryKind};
