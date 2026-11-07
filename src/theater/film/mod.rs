//! Canonical film input and the three source-preserving recording sections.
use serde::{Deserialize, Serialize};

/// Structurally decoded canonical recording. Interpretations belong to TheaterRuntime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Film {
    /// The recording's sole registry/bootstrap chunk.
    pub registry: RegistryChunk,
    /// Replication chunks in their supplied order.
    pub replication_stream: Vec<ReplicationStreamChunk>,
    /// The recording's sole summary chunk.
    pub summary: SummaryChunk,
}
impl Film {
    /// Read the registry header, select a supported parser, and decode the chunks.
    ///
    /// Requires exactly one registry (first) and one summary (last). Unknown chunk kinds
    /// are rejected. Each chunk retains its original bytes and source position.
    pub fn parse(chunks: impl IntoIterator<Item = FilmChunk>) -> Result<Self, ParseError> {
        crate::theater::parser::ChunkReader::read(chunks)
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
    #[error("film requires a summary chunk")]
    MissingSummary,
    #[error("film contains multiple summary chunks")]
    MultipleSummaries,
    #[error("the last film chunk must be the summary chunk")]
    SummaryNotLast,
    #[error("unsupported film chunk kind {0}")]
    UnsupportedChunkKind(i32),
    #[error("truncated registry version header")]
    TruncatedRegistryHeader,
    #[error("unsupported film version {0}; supported: 41")]
    UnsupportedVersion(u32),
    #[error(transparent)]
    Registry(#[from] FilmRegistryReadError),
}

#[cfg(test)]
mod tests;

pub mod chunks;
pub mod maps;
pub mod weapon;
pub use chunks::registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, RegistryComponent,
    RegistryStop,
};
pub use chunks::replication::{
    ComponentReadStatus, ControlEntry, DatumComponentMask, DatumDecodeError, DatumEntry,
    DatumTable, DecodedFrameView, DefaultState, DefaultStateStatus, EntityComponentRead,
    EntityRecord, EntityViewStop, EventField, EventFieldStage, EventFieldValue, EventListRead,
    EventListStop, EventRecord, FrameDecodeError, FramePacket, FrameRead, FrameViewStop,
    KeyframeChainAttempt, KeyframeChainStop, KeyframeComponentRead, KeyframeRecord, KeyframeStop,
    KeyframeTable, ProductionEntityEnd, ProductionFrame, RecordHeader, RecordKind,
    ReplicationStreamPacket, ReplicationStreamPacketBody,
};
pub use chunks::summary::{SummaryPacket, SummaryPacketBody};
pub use chunks::{
    BitRange, ByteRange, Chunk, ChunkKind, ChunkTransport, FilmChunk, FilmPacketHeader, Packet,
    PacketBody, PacketDecodeError, PacketRead, PacketSource, PacketStream, RegistryChunk,
    ReplicationStreamChunk, SourceRegion, SourceRegionKind, SummaryChunk,
};
pub use weapon::{KnownWeapon, Weapon, WeaponDefinition, WeaponId, known_weapons};

pub use chunks::replication::components::field::{ComponentField, RawBits};
