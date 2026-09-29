//! Input chunks and preserved source storage.
use super::*;

/// One ordered input chunk. Bytes may be zlib-compressed or already decompressed.
/// Manifest identity and timing are optional; supplied values are retained exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmChunk {
    pub kind: ChunkKind,
    pub index: Option<i64>,
    pub start_ms: Option<i64>,
    pub data: Vec<u8>,
}

/// Transport chunk category supplied by the film manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkKind {
    Registry,
    Replication,
    Summary,
}
impl TryFrom<i32> for ChunkKind {
    type Error = ParseError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Registry),
            2 => Ok(Self::Replication),
            3 => Ok(Self::Summary),
            other => Err(ParseError::ChunkKind(other)),
        }
    }
}
/// Borrowed access to any decoded chunk without erasing its concrete type.
#[derive(Debug, Clone, Copy)]
pub enum FilmChunkRef<'a> {
    Registry(&'a RegistryChunk),
    Replication(&'a ReplicationStreamChunk),
    Summary(&'a SummaryChunk),
}

/// Borrowed access to either concrete packet type without erasing its origin.
#[derive(Debug, Clone, Copy)]
pub enum PacketRef<'a> {
    Replication(&'a ReplicationStreamPacket),
    Summary(&'a SummaryPacket),
}

impl PacketRef<'_> {
    pub fn header(self) -> FilmPacketHeader {
        match self {
            Self::Replication(packet) => packet.header,
            Self::Summary(packet) => packet.header,
        }
    }
}

impl<'a> FilmChunkRef<'a> {
    pub fn source(self) -> &'a FilmChunk {
        match self {
            Self::Registry(chunk) => &chunk.source,
            Self::Replication(chunk) => &chunk.source,
            Self::Summary(chunk) => &chunk.source,
        }
    }

    pub fn source_position(self) -> usize {
        match self {
            Self::Registry(chunk) => chunk.source_position,
            Self::Replication(chunk) => chunk.source_position,
            Self::Summary(chunk) => chunk.source_position,
        }
    }

    pub fn data(self) -> &'a [u8] {
        match self {
            Self::Registry(chunk) => &chunk.data,
            Self::Replication(chunk) => &chunk.data,
            Self::Summary(chunk) => &chunk.data,
        }
    }

    pub fn packets(self) -> impl Iterator<Item = PacketRef<'a>> {
        let replication = match self {
            Self::Replication(chunk) => Some(chunk.packets.as_slice()),
            _ => None,
        };
        let summaries = match self {
            Self::Summary(chunk) => Some(chunk.packets.as_slice()),
            _ => None,
        };
        replication
            .into_iter()
            .flatten()
            .map(PacketRef::Replication)
            .chain(summaries.into_iter().flatten().map(PacketRef::Summary))
    }

    pub fn packet(self, index: usize) -> Option<PacketRef<'a>> {
        match self {
            Self::Registry(_) => None,
            Self::Replication(chunk) => chunk.packets.get(index).map(PacketRef::Replication),
            Self::Summary(chunk) => chunk.packets.get(index).map(PacketRef::Summary),
        }
    }

    pub fn payload(self, packet: PacketRef<'a>) -> Option<&'a [u8]> {
        let header = packet.header();
        self.data()
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
