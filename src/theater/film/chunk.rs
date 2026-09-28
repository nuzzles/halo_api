//! Input chunks and preserved source storage.
use super::*;
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
/// One ordered input chunk. Bytes may be zlib-compressed or already decompressed.
/// Manifest identity and timing are optional; supplied values are retained exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmChunk {
    pub kind: ChunkKind,
    pub index: Option<i64>,
    pub start_ms: Option<i64>,
    pub data: Vec<u8>,
}
impl FilmChunk {
    pub fn new(kind: ChunkKind, data: impl Into<Vec<u8>>) -> Self {
        Self {
            kind,
            index: None,
            start_ms: None,
            data: data.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedChunk {
    /// Original input, including exact transport bytes and supplied metadata.
    pub source: FilmChunk,
    /// Position in the input list, independent of optional manifest numbering.
    pub source_position: usize,
    pub data: Vec<u8>,
    pub packets: Vec<NativeFilmPacket>,
    /// Start of the unwalked suffix. Nested packet bit offsets address `data`.
    pub packet_walk_end_byte: usize,
}
impl ParsedChunk {
    /// Borrow a packet payload from its retained decompressed source.
    pub fn payload(&self, packet: &NativeFilmPacket) -> Option<&[u8]> {
        let header = packet.header;
        self.data
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
