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

    pub fn packets(self) -> &'a [NativeFilmPacket] {
        match self {
            Self::Registry(_) => &[],
            Self::Replication(chunk) => &chunk.packets,
            Self::Summary(chunk) => &chunk.packets,
        }
    }

    pub fn payload(self, packet: &NativeFilmPacket) -> Option<&'a [u8]> {
        let header = packet.header;
        self.data()
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
