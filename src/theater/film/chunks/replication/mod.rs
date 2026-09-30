//! Reference packet stream, records, and read results.
use super::*;

pub mod components;
pub use components::*;

pub mod replication_stream;
pub use replication_stream::*;

/// One type-2 film chunk after transport decompression and packet framing.
pub type ReplicationStreamChunk = Chunk<PacketStream<ReplicationStreamPacketBody>>;
impl ReplicationStreamChunk {
    /// Return the packet's retained raw payload when its recorded size agrees.
    pub fn payload(&self, packet: &ReplicationStreamPacket) -> Option<&[u8]> {
        packet.payload(&self.data)
    }
}
