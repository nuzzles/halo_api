//! Parsed film with setup, timestamped replication, and guarded highlights.
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::parser::{FilmParseContext, ParseLimits};
use serde::{Deserialize, Serialize};

/// A structurally decoded recording assembled from its three chunk kinds.
/// Summary highlights are guarded candidates until their sequential wire
/// grammar is established.
/// Game state at a point in time belongs to [`crate::theater::FilmRuntime`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Film {
    /// Setup: recorded version, component schemas, game options, and map identity.
    pub registry: Registry,
    /// Timestamped entity, component, control, keyframe, and event observations.
    pub replication_stream: ReplicationStream,
    /// Guarded, typed highlights in recorded order.
    pub summary: Summary,
}
impl Film {
    /// Read the registry header, select a supported parser, and decode the chunks.
    ///
    /// Requires exactly one registry and at least one replication chunk.
    /// Chunks are ordered by manifest index before dispatch. The registry must
    /// be first, and the summary must be last.
    /// Unknown chunk kinds, damaged compressed inputs, and registries without
    /// readable map IDs or game options are rejected. Registry game modes other
    /// than Multiplayer are rejected. Source packets are
    /// consumed into typed observations and guarded highlights. Manifest indices
    /// must be contiguous from zero, and replay tick evidence must be complete.
    pub fn parse(chunks: impl IntoIterator<Item = RawFilmChunk>) -> Result<Self, ParseError> {
        crate::theater::parser::ChunkReader::read(
            chunks,
            ParseLimits::default(),
            FilmParseContext::default(),
        )
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("cannot inflate film chunk at index {index}: {cause}")]
    Inflate {
        index: usize,
        #[source]
        cause: InflateError,
    },
    #[error("film chunk at index {index} exceeds packet limit {limit}")]
    PacketLimit { index: usize, limit: usize },
    #[error("supplied DELTA entity schema {schema} is absent from the film registry")]
    InvalidDeltaContextSchema { schema: u32 },
    #[error("supplied script-event prefix contradicts recorded recorder simulation")]
    ContradictoryScriptPrefixContext,
    #[error("film requires a registry chunk")]
    MissingRegistry,
    #[error("a registry chunk must be first")]
    RegistryNotFirst,
    #[error("film requires a replication chunk")]
    MissingReplication,
    #[error("film requires a summary chunk")]
    MissingSummary,
    #[error("a summary chunk must be last")]
    SummaryNotLast,
    #[error("unsupported film chunk kind {0}")]
    UnsupportedChunkKind(i32),
    #[error("film contains invalid negative manifest chunk index {0}")]
    InvalidIndex(i32),
    #[error("truncated registry version header")]
    TruncatedRegistryHeader,
    #[error("registry map identity could not be read")]
    MissingRegistryMap,
    #[error("registry game options could not be fully read")]
    IncompleteRegistryOptions,
    #[error("unsupported registry game mode {0}; supported: 2 (Multiplayer)")]
    UnsupportedGameMode(u8),
    #[error("unsupported film version {0}; supported: 41")]
    UnsupportedVersion(u32),
    #[error("film contains duplicate manifest chunk index {index}")]
    DuplicateIndex { index: usize },
    #[error("film is missing manifest chunk index {index}")]
    MissingIndex { index: usize },
    #[error("no replay chunk records a tick-message marker")]
    NoTicks,
    #[error("a replay tick cannot be ruled out because replay bytes were not framed")]
    TickEvidenceIncomplete,
    #[error(transparent)]
    Registry(#[from] FilmRegistryReadError),
}

/// Failure before a structural registry read can be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FilmRegistryReadError {
    #[error("registry chunk is still compressed")]
    StillCompressed,
}

pub mod catalog;
mod ids;
mod name;
pub mod registry;
pub mod replication;
pub mod summary;
pub use crate::theater::parser::chunk::{ChunkKind, InflatedFilmChunk};
pub use crate::theater::parser::compression::InflateError;
pub use ids::{
    ComponentSlot, ControlPlayerIndex, EntityId, EntitySchemaId, KeyframeId, MapAssetId,
    MapVersionId, TagId, WeaponId, WeaponVariantId, Xuid,
};
pub use name::FilmName;
pub use registry::{
    EntitySchema, EntitySchemaKind, Registry, RegistryComponentDefinition, RegistryGameMode,
    RegistryMap, RegistryMapKind, RegistryOptions, RegistryVersion,
};
pub use replication::{
    BitValue, DamageAmount, DamageCause, DamageReference, FiringReference, ProjectileReference,
    ReplicationChange, ReplicationComponentChange, ReplicationControl, ReplicationEvent,
    ReplicationEventHandle, ReplicationEventKind, ReplicationField, ReplicationObservation,
    ReplicationStream, ReplicationValue, TagKind, TagLink, TagReference,
};
pub use summary::medals::Medal;
pub use summary::{Summary, SummaryHighlight, SummaryHighlightPayload};
