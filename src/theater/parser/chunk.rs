//! Validated chunk kinds and the inflated input passed to section readers.
use crate::clients::hi::models::RawChunkKind;
use crate::theater::film::ParseError;

/// Recognized film chunk category after input validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    /// Type 1: bootstrap data and the entity-component registry.
    Registry,
    /// Type 2: ordered replay packets and replication state changes.
    Replication,
    /// Type 3: terminal summary/highlights packets.
    Summary,
}

impl From<ChunkKind> for RawChunkKind {
    fn from(value: ChunkKind) -> Self {
        match value {
            ChunkKind::Registry => Self::REGISTRY,
            ChunkKind::Replication => Self::REPLICATION,
            ChunkKind::Summary => Self::SUMMARY,
        }
    }
}

/// One chunk after inflation and before kind-specific parsing.
/// Its downloaded byte buffer is no longer retained alongside `data`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InflatedFilmChunk {
    /// Recognized chunk category established from the raw manifest code.
    pub kind: ChunkKind,
    /// Manifest index carried from the raw chunk.
    pub index: usize,
    /// Manifest start-time offset in milliseconds.
    pub start_ms: i64,
    /// Inflated chunk bytes used by the section reader.
    pub data: Vec<u8>,
}

impl TryFrom<RawChunkKind> for ChunkKind {
    type Error = ParseError;
    fn try_from(value: RawChunkKind) -> Result<Self, Self::Error> {
        match value.raw() {
            1 => Ok(Self::Registry),
            2 => Ok(Self::Replication),
            3 => Ok(Self::Summary),
            other => Err(ParseError::UnsupportedChunkKind(other)),
        }
    }
}
