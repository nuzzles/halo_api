//! Canonical film input and the three source-preserving recording sections.
use serde::{Deserialize, Serialize};

/// Structurally decoded canonical recording. Interpretations belong to ResolvedFilm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Film {
    pub registry: RegistryChunk,
    pub replication: ReplicationStream,
    pub summaries: SummaryEvents,
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
    #[error("unsupported film chunk kind {0}")]
    ChunkKind(i32),
    #[error(transparent)]
    Registry(#[from] FilmRegistryReadError),
    #[error("unable to load film chunks: {0}")]
    Source(String),
}

#[cfg(test)]
mod tests;

pub mod chunks;
pub use chunks::registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, RegistryBlockRead,
    RegistrySlotRead,
};
pub use chunks::replication::{
    AnticipatedDeclaration, ContinuationStatePolicy, DatumEntry, DatumTable, EventContinuation,
    EventField, EventFieldStage, EventFieldValue, EventListRead, EventListStop, EventRecord,
    FilmViewAdmission, FramePacket, KillEventFields, NewBindingRefusal,
    ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame, RecordHeader, RecordKind,
    ReplicationStream, ReplicationStreamPacket, ReplicationStreamPacketBody,
};
pub use chunks::summary::{
    FilmMedalDefinition, MedalAward, SummaryEvents, SummaryPacket, SummaryPacketBody,
};
pub use chunks::{
    ChunkKind, FilmChunk, FilmPacketHeader, RegistryChunk, ReplicationStreamChunk, SourceSpan,
    SummaryChunk,
};

pub use chunks::replication::components::{
    AbilityNonPredictedState, ActionBlock, BindingOrigin, CamoState, ControlEntry,
    DecodedFrameView, EntityComponentAttempt, EntityComponentSpan, EntityRecord, EntityViewStop,
    EquipmentCreationField, EquipmentField, FilmComponentObservation, FilmReadDiagnostics,
    FrameViewStop, GameEngineField, KeyframeChainAttempt, KeyframeChainStop, KeyframeComponentSpan,
    KeyframeRecord, KeyframeStop, KeyframeTable, ManagedObjectField, ManagedPropertyField,
    MovementComponent, MppField, NavpointField, ObjectParentState, ObjectiveField,
    PlayerStateField, ProbeComponent, ReadOperation, ReadRefusal, UnitEquipmentEntry,
    UnitEquipmentRead, UnitReference, UnitReferenceKind, WidthAdjustment, WidthPurpose,
    WidthRefusal,
};

pub use chunks::replication::components::position::PositionKind;

pub use chunks::replication::components::field::ComponentField;

pub use chunks::summary::{SummaryEvent, SummaryKind};
