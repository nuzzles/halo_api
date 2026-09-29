//! Native film input and the three source-preserving recording sections.
use serde::{Deserialize, Serialize};

/// Structurally decoded native recording. Interpretations belong to ResolvedFilm.
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
    /// Sections retain input positions, allowing a lookup without copying bytes.
    pub(crate) fn chunk(&self, position: usize) -> Option<FilmChunkRef<'_>> {
        if self.registry.source_position == position {
            return Some(FilmChunkRef::Registry(&self.registry));
        }
        if let Ok(i) = self
            .replication
            .chunks
            .binary_search_by_key(&position, |c| c.source_position)
        {
            return Some(FilmChunkRef::Replication(&self.replication.chunks[i]));
        }
        self.summaries
            .chunks
            .binary_search_by_key(&position, |c| c.source_position)
            .ok()
            .map(|i| FilmChunkRef::Summary(&self.summaries.chunks[i]))
    }
    pub(crate) fn chunks(&self) -> impl Iterator<Item = FilmChunkRef<'_>> {
        std::iter::once(FilmChunkRef::Registry(&self.registry))
            .chain(
                self.replication
                    .chunks
                    .iter()
                    .map(FilmChunkRef::Replication),
            )
            .chain(self.summaries.chunks.iter().map(FilmChunkRef::Summary))
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
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, NativeRegistryBlockRead,
    NativeRegistrySlotRead,
};
pub use chunks::replication::{
    AnticipatedDeclaration, DatumEntry, DatumTable, DecodedHeadEvent, EventReference,
    EventReferenceValue, FilmPacket, FilmPacketBody, FilmPacketHeader, FilmViewAdmission,
    HeadEventPayload, HeadEventStop, KillEventFields, NativeContinuationStatePolicy,
    NativeEventContinuation, NativeEventField, NativeEventFieldStage, NativeEventFieldValue,
    NativeEventListRead, NativeEventListStop, NativeEventRecord, NativeNewBindingRefusal,
    NativePacketHeadRead, NativePickupOutcome, NativePickupRead, NativeRosterRead,
    NativeTranslocatorEvent, NativeWeaponDamageField, NativeWeaponDamageRead, NativeZoomRead,
    ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame, RecordHeader, RecordKind,
    ReplicationStream, RosterEntry, RosterReport, RosterUpdate, SourceSpan, TeleportPosition,
    TranslocatorEvent, TranslocatorStop, WeaponDamage, WeaponDamageRead,
};
pub use chunks::summary::{FilmMedalDefinition, MedalAward, SummaryEvents};
pub use chunks::{
    ChunkKind, FilmChunk, FilmChunkRef, RegistryChunk, ReplicationStreamChunk, SummaryChunk,
};

pub use chunks::replication::components::{
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

pub use chunks::replication::player::{
    NativePlayerSlotRead, NativeSlotField, NativeSlotValue, PlayerTableShorts, PlayerTableSlot,
};

pub use chunks::replication::components::position::NativePositionKind;

pub use chunks::replication::components::read_diagnostics::ChainInferenceOutcome;

pub use chunks::replication::components::field::ComponentField;

pub use chunks::summary::{SummaryEvent, SummaryKind};
