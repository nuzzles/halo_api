//! Reference packet stream, records, and read results.
use super::*;

pub mod components;
pub use components::*;

pub mod replication_stream;
pub use replication_stream::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStreamChunk {
    pub source: FilmChunk,
    pub source_position: usize,
    pub data: Vec<u8>,
    pub packets: Vec<ReplicationStreamPacket>,
}
impl ReplicationStreamChunk {
    pub fn payload(&self, packet: &ReplicationStreamPacket) -> Option<&[u8]> {
        let header = packet.header;
        self.data
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
