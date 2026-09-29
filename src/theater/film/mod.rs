//! Canonical film input and the three source-preserving recording sections.
use serde::{Deserialize, Serialize};

/// Structurally decoded canonical recording. Interpretations belong to ResolvedFilm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Film {
    pub registry: RegistryChunk,
    pub chunks: Vec<FilmDataChunk>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilmDataChunk {
    Replication(ReplicationStreamChunk),
    Summary(SummaryChunk),
    Unknown(Chunk<()>),
}
impl Film {
    /// Read the registry header, select a supported parser, and decode the chunks.
    /// The first chunk must be the only registry chunk. Remaining input order is retained.
    pub fn parse(chunks: impl IntoIterator<Item = FilmChunk>) -> Result<Self, ParseError> {
        crate::theater::parser::ChunkReader::read(chunks)
    }

    pub fn resolve(&self) -> crate::theater::ResolvedFilm<'_> {
        crate::theater::ResolvedFilm::from_film(self)
    }
    pub fn replication_chunks(&self) -> impl Iterator<Item = &ReplicationStreamChunk> {
        self.chunks.iter().filter_map(|chunk| match chunk {
            FilmDataChunk::Replication(chunk) => Some(chunk),
            FilmDataChunk::Summary(_) => None,
            FilmDataChunk::Unknown(_) => None,
        })
    }
    pub fn summary_chunks(&self) -> impl Iterator<Item = &SummaryChunk> {
        self.chunks.iter().filter_map(|chunk| match chunk {
            FilmDataChunk::Summary(chunk) => Some(chunk),
            FilmDataChunk::Replication(_) => None,
            FilmDataChunk::Unknown(_) => None,
        })
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("film requires a registry chunk")]
    MissingRegistry,
    #[error("the first film chunk must be a registry chunk")]
    RegistryNotFirst,
    #[error("film contains multiple registry chunks")]
    MultipleRegistries,
    #[error("truncated registry version header")]
    TruncatedRegistryHeader,
    #[error("unsupported film version {0}; supported: 41")]
    UnsupportedVersion(u32),
    #[error(transparent)]
    Registry(#[from] FilmRegistryReadError),
    #[error("film contains more chunks than this platform can index")]
    TooManyChunks,
}

#[cfg(test)]
mod tests;

pub mod chunks;
pub use chunks::registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, RegistryBlockRead,
    RegistrySlotRead, RegistryStop,
};
pub use chunks::replication::{
    ContinuationStatePolicy, ControlEntry, DatumDecodeError, DatumEntry, DatumTable,
    DecodedFrameView, EntityComponentRead, EntityRecord, EntityViewStop, EventContinuation,
    EventField, EventFieldStage, EventFieldValue, EventListRead, EventListStop, EventRecord,
    FrameDecodeError, FramePacket, FrameRead, FrameViewStop, KeyframeChainAttempt,
    KeyframeChainStop, KeyframeComponentRead, KeyframeRecord, KeyframeStop, KeyframeTable,
    KillEventFields, ProductionEntityEnd, ProductionFrame, RecordHeader, RecordKind,
    ReplicationStreamPacket, ReplicationStreamPacketBody,
};
pub use chunks::summary::{SummaryPacket, SummaryPacketBody, SummarySegment};
pub use chunks::{
    BitRange, ByteRange, Chunk, ChunkKind, ChunkTransport, FilmChunk, FilmPacketHeader, Packet,
    PacketBody, PacketDecodeError, PacketRead, PacketSource, PacketStream, RegistryChunk,
    ReplicationStreamChunk, SummaryChunk,
};

pub use chunks::replication::components::{
    ActionBlock, UnitEquipmentEntry, UnitEquipmentRead, UnitReference, UnitReferenceKind,
};

pub use chunks::replication::components::position::PositionKind;

pub use chunks::replication::components::field::{ComponentField, RawBits};

pub use chunks::summary::SummaryEvent;
