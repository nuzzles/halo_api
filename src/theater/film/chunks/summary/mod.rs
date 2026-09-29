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
    pub packets: Vec<NativeFilmPacket>,
    /// Start of the unwalked suffix. Nested packet bit offsets address `data`.
    pub packet_walk_end_byte: usize,
}
impl SummaryChunk {
    pub fn payload(&self, packet: &NativeFilmPacket) -> Option<&[u8]> {
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
                NativeFilmPacketBody::Summary { events, .. } => events.as_slice(),
                _ => &[],
            })
    }
}

pub mod medals;
pub use medals::{FilmMedalDefinition, MedalAward};

pub mod models;
pub use models::{SummaryEvent, SummaryKind};
