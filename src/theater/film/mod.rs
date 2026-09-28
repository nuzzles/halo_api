//! Native film input and the three source-preserving recording sections.
use serde::{Deserialize, Serialize};

/// Structurally decoded native recording. Interpretations belong to ResolvedFilm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Film {
    pub registry: Registry,
    pub replication: ReplicationStream,
    pub summaries: SummaryEvents,
}
impl Film {
    /// Read the registry header, select a supported parser, and decode the chunks.
    /// The first chunk must be the only registry chunk. Remaining input order is retained.
    pub fn parse(chunks: impl IntoIterator<Item = FilmChunk>) -> Result<Self, ParseError> {
        let mut chunks = chunks.into_iter();
        let first = chunks.next().ok_or(ParseError::MissingRegistry)?;
        let registry = crate::theater::parser::registry_chunk::RegistryChunkParser::parse(first)?;
        match registry.definition.registry.major_version {
            41 => crate::theater::parser::v41::V41ChunkParser::parse(registry, chunks),
            version => Err(ParseError::UnsupportedVersion(version)),
        }
    }

    pub fn resolve(&self) -> crate::theater::ResolvedFilm<'_> {
        crate::theater::ResolvedFilm::from_film(self)
    }
    /// Sections retain input positions, allowing a lookup without copying bytes.
    pub(crate) fn chunk(&self, position: usize) -> Option<&ParsedChunk> {
        if self.registry.chunk.source_position == position {
            return Some(&self.registry.chunk);
        }
        for chunks in [&self.replication.chunks, &self.summaries.chunks] {
            if let Ok(i) = chunks.binary_search_by_key(&position, |c| c.source_position) {
                return Some(&chunks[i]);
            }
        }
        None
    }
    pub(crate) fn chunks(&self) -> impl Iterator<Item = &ParsedChunk> {
        std::iter::once(&self.registry.chunk)
            .chain(&self.replication.chunks)
            .chain(&self.summaries.chunks)
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

pub mod chunk;
pub mod registry;
pub mod replication;
pub mod summary;
pub use chunk::{ChunkKind, FilmChunk, ParsedChunk};
pub use registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, NativeRegistryBlockRead,
    NativeRegistrySlotRead, Registry,
};
pub use replication::{
    AnticipatedDeclaration, DatumEntry, DatumTable, DecodedHeadEvent, EventReference,
    EventReferenceValue, FilmPacket, FilmViewAdmission, HeadEventPayload, HeadEventStop,
    KillEventFields, NativeContinuationStatePolicy, NativeEventContinuation, NativeEventField,
    NativeEventFieldStage, NativeEventFieldValue, NativeEventListRead, NativeEventListStop,
    NativeEventRecord, NativeFilmPacket, NativeFilmPacketBody, NativeNewBindingRefusal,
    NativePacketHeadRead, NativePickupOutcome, NativePickupRead, NativeRosterRead,
    NativeTranslocatorEvent, NativeWeaponDamageField, NativeWeaponDamageRead, NativeZoomRead,
    ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame, RecordHeader, RecordKind,
    ReplicationStream, RosterEntry, RosterReport, RosterUpdate, SourceSpan, TeleportPosition,
    TranslocatorEvent, TranslocatorStop, WeaponDamage, WeaponDamageRead,
};
pub use summary::{FilmMedalDefinition, MedalAward, SummaryEvents};

pub mod components;
pub use components::{
    BindingOrigin, DecodedFrameView, EntityComponentAttempt, EntityComponentSpan, EntityRecord,
    EntityViewStop, FilmComponentObservation, FilmMapBounds, FilmMppWidths, FilmQuantizationRange,
    FilmReadDiagnostics, FrameViewStop, KeyframeChainAttempt, KeyframeChainStop,
    KeyframeComponentSpan, KeyframeRecord, KeyframeStop, NativeAbilityNonPredictedState,
    NativeActionBlock, NativeCamoState, NativeControlEntry, NativeEquipmentCreationField,
    NativeEquipmentField, NativeGameEngineField, NativeKeyframeLayout, NativeKeyframeTable,
    NativeManagedObjectField, NativeManagedPropertyField, NativeMovementComponent,
    NativeMovementProfile, NativeMppField, NativeNavpointField, NativeObjectParentState,
    NativeObjectiveField, NativePlayerStateField, NativePrecisionDescriptor, NativeProbeComponent,
    NativeReadOperation, NativeReadRefusal, NativeScanGrammar, NativeScanProfile,
    NativeSharedWidths, NativeUnitReference, NativeUnitReferenceKind, NativeWidthAdjustment,
    NativeWidthPurpose, NativeWidthRefusal, UnitEquipmentEntry, UnitEquipmentRead,
};

pub use replication::player::{
    NativePlayerSlotRead, NativeSlotField, NativeSlotValue, PlayerTableShorts, PlayerTableSlot,
};

pub use components::position::NativePositionKind;

pub use components::read_diagnostics::ChainInferenceOutcome;

pub use components::field::ComponentField;

pub use summary::{SummaryEvent, SummaryKind};
