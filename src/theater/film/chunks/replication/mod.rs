//! Reference packet stream, records, and read results.
use super::*;

pub mod components;
pub use components::*;

pub mod replication_stream;
pub use replication_stream::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// One type-2 film chunk after transport decompression and packet framing.
pub struct ReplicationStreamChunk {
    /// Original input bytes and manifest metadata.
    pub source: FilmChunk,
    /// Position in the complete input sequence passed to `Film::parse`.
    pub source_position: usize,
    /// Complete decompressed chunk bytes, including unparsed regions.
    pub data: Vec<u8>,
    /// Packets in their recorded order.
    pub packets: Vec<ReplicationStreamPacket>,
}
impl ReplicationStreamChunk {
    /// Return the packet's retained raw payload when its recorded size agrees.
    pub fn payload(&self, packet: &ReplicationStreamPacket) -> Option<&[u8]> {
        let header = packet.header;
        self.data
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
