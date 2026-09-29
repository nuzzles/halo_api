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
    Unknown(i32),
}

/// One decoded chunk with its original transport input and canonical payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Chunk<T> {
    pub source: FilmChunk,
    pub source_position: usize,
    pub data: Vec<u8>,
    pub transport: ChunkTransport,
    pub body: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChunkTransport {
    Clear,
    ZlibComplete,
    ZlibPartial,
    ZlibRejected,
}
impl TryFrom<i32> for ChunkKind {
    type Error = ParseError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Registry),
            2 => Ok(Self::Replication),
            3 => Ok(Self::Summary),
            other => Ok(Self::Unknown(other)),
        }
    }
}
