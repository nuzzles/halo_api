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
    /// Timestamped entity, component, control, keyframe, event, and roster observations.
    pub replication_stream: ReplicationStream,
    /// Guarded, typed highlights in recorded order.
    pub summary: Summary,
}

/// Registry and replication observations, without summary highlights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmReplication {
    pub registry: Registry,
    pub replication_stream: ReplicationStream,
}

impl AsRef<ReplicationStream> for Film {
    fn as_ref(&self) -> &ReplicationStream {
        &self.replication_stream
    }
}

impl AsRef<ReplicationStream> for FilmReplication {
    fn as_ref(&self) -> &ReplicationStream {
        &self.replication_stream
    }
}

/// Registry and guarded summary highlights, without replication observations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmSummary {
    pub registry: Registry,
    pub summary: Summary,
}

/// Controls how film chunks are decoded. The default uses multiple threads on
/// native targets when parallel work is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FilmParseOptions {
    pub threading: Threading,
}

/// Whether film decoding may spawn worker threads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Threading {
    /// Decode on the calling thread only.
    Single,
    /// Allow parallel decoding on native targets when it is beneficial.
    #[default]
    Multi,
}

impl Film {
    /// Decode only the registry and replication stream.
    ///
    /// Requires one registry at manifest index zero and at least one replication
    /// chunk, with contiguous indices starting at one and complete replay tick
    /// evidence. Summary chunks are ignored without inflation or validation.
    /// Other validation is the same as [`Self::parse`].
    pub fn parse_replication(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
    ) -> Result<FilmReplication, ParseError> {
        Self::parse_replication_with_options(chunks, FilmParseOptions::default())
    }

    /// Decode registry and replication with an explicit threading choice.
    /// See [`Self::parse_replication`] for input requirements and
    /// [`Self::parse_with_options`] for threading behavior.
    pub fn parse_replication_with_options(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        options: FilmParseOptions,
    ) -> Result<FilmReplication, ParseError> {
        crate::theater::parser::ChunkReader::read_replication(
            chunks,
            ParseLimits::default(),
            FilmParseContext::default(),
            options.threading,
        )
    }

    /// Decode only the registry and summary highlights.
    ///
    /// Requires one registry at manifest index zero and one summary at its
    /// original positive manifest index. Replication chunks are ignored without
    /// inflation or validation; gaps before the summary and absent replay ticks
    /// are allowed. Registry validation is the same as [`Self::parse`].
    /// Highlights remain guarded candidates and may be incomplete.
    pub fn parse_summary(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
    ) -> Result<FilmSummary, ParseError> {
        Self::parse_summary_with_options(chunks, FilmParseOptions::default())
    }

    /// Decode registry and summary with an explicit threading choice.
    /// See [`Self::parse_summary`] for input requirements. Summary decoding
    /// currently runs on the calling thread in either mode.
    pub fn parse_summary_with_options(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        options: FilmParseOptions,
    ) -> Result<FilmSummary, ParseError> {
        crate::theater::parser::ChunkReader::read_summary(
            chunks,
            ParseLimits::default(),
            options.threading,
        )
    }

    /// Read the registry header, select a supported parser, and decode the chunks.
    ///
    /// Requires exactly one registry and at least one replication chunk.
    /// Chunks are ordered by manifest index before dispatch. The registry must
    /// be first, and the summary must be last.
    /// Unknown chunk kinds, damaged compressed inputs, and registries without
    /// readable map IDs, game options, or an initial player table are rejected.
    /// Registry game modes other than Multiplayer are rejected. Source packets are
    /// consumed into typed observations and guarded highlights. Manifest indices
    /// must be contiguous from zero, and replay tick evidence must be complete.
    pub fn parse(chunks: impl IntoIterator<Item = RawFilmChunk>) -> Result<Self, ParseError> {
        Self::parse_with_options(chunks, FilmParseOptions::default())
    }

    /// Decode a film with an explicit threading choice. `Single` never spawns
    /// parser workers; `Multi` may do so on native targets. WASM decodes on
    /// the calling thread in either mode.
    pub fn parse_with_options(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        options: FilmParseOptions,
    ) -> Result<Self, ParseError> {
        crate::theater::parser::ChunkReader::read(
            chunks,
            ParseLimits::default(),
            FilmParseContext::default(),
            options.threading,
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
    #[error("registry initial player table could not be fully decoded")]
    IncompleteRegistryPlayers,
    #[error("recorded player customization has an unsupported or invalid layout")]
    InvalidCustomization,
    #[error("recorded match setting {field} is missing or invalid")]
    InvalidMatchSettings { field: &'static str },
    #[error("recorded event field {field} is required but absent")]
    MissingRequiredEventField { field: &'static str },
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
pub mod players;
pub use players::*;
pub mod settings;
pub use catalog::options::KnownMatchOption;
pub use settings::*;
mod ids;
pub mod registry;
pub mod replication;
pub mod summary;
pub use crate::theater::parser::chunk::{ChunkKind, InflatedFilmChunk};
pub use crate::theater::parser::compression::InflateError;
pub use ids::{
    BotAbsoluteIndex, BotId, BotSlot, ComponentSlot, ControlPlayerIndex, EntityId, EntitySchemaId,
    EquipmentTagId, EventPlayerSlot, InitialPlayerSlot, KeyframeId, MapAssetId, MapVersionId,
    TagId, WeaponId, WeaponTagId, WeaponVariantId, Xuid,
};
pub use registry::{
    EntitySchema, EntitySchemaKind, Registry, RegistryComponentDefinition, RegistryGameMode,
    RegistryMap, RegistryMapKind, RegistryOptions, RegistryVersion,
};
pub use replication::{
    BitValue, DamageAmount, DamageCause, DamageReference, FiringReference, ProjectileReference,
    ReplicationChange, ReplicationComponentChange, ReplicationControl, ReplicationEvent,
    ReplicationEventKind, ReplicationEventReference, ReplicationField, ReplicationObservation,
    ReplicationStream, ReplicationValue, TagKind, TagLink, TagReference,
};
pub use summary::medals::Medal;
pub use summary::{Summary, SummaryHighlight, SummaryHighlightPayload};
