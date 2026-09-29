//! Structural v41 decoding. No candidate searches or inferred runtime settings.
pub(crate) use super::{bits, transport};
use crate::theater::film::*;

/// Version-specific chunk reader selected by [`super::ChunkReader`].
pub struct V41ChunkReader;

/// Reader for a v41 component-registry chunk.
pub struct V41RegistryChunkReader;

/// Reader for a v41 replication-stream chunk.
pub struct V41ReplicationStreamChunkReader;

/// Reader for a v41 summary chunk.
pub struct V41SummaryChunkReader;

#[cfg(test)]
pub(crate) fn test_chunks(source: &FixtureFilmSource) -> Vec<FilmChunk> {
    source
        .original_chunks()
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            let metadata = source.metadata().get(i);
            FilmChunk {
                kind: match metadata
                    .map(|m| m.chunk_type)
                    .unwrap_or(if i == 0 { 1 } else { 2 })
                {
                    1 => ChunkKind::Registry,
                    3 => ChunkKind::Summary,
                    _ => ChunkKind::Replication,
                },
                index: metadata.map(|m| m.index),
                start_ms: metadata.map(|m| m.start_ms),
                data: bytes.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;

pub(crate) mod anticipated_bindings;

pub(crate) mod components;

pub(crate) mod config;
pub(crate) use config::*;

pub(crate) mod datums;

pub(crate) mod kill_decode;

pub(crate) mod kill_event_chain;

pub(crate) mod reader;

pub(crate) mod production_frame;

mod diagnostics;
mod observations;

pub(crate) mod records;

pub(crate) mod registry;

pub(crate) mod source_bits;

pub(crate) mod summary;

pub(crate) mod types;

pub(crate) use crate::theater::film::chunks::replication::components::unit_equipment;

pub(crate) use crate::theater::film::chunks::replication::components::unit_references;

pub(crate) mod world;

pub(crate) mod position_capture;

pub(crate) mod packets;

pub(crate) mod chunks;

pub(crate) use anticipated_bindings::*;
pub(crate) use components::*;
pub(crate) use datums::*;
pub(crate) use kill_decode::*;
pub(crate) use kill_event_chain::*;
pub(crate) use position_capture::*;
pub(crate) use reader::*;
pub(crate) use records::*;
pub(crate) use source_bits::*;
pub(crate) use types::*;
pub(crate) use world::*;

pub(crate) use components::{ComponentField, EntityRecord, EntityViewStop};
pub(crate) use diagnostics::{
    FilmReadDiagnostics, ReadOperation, ReadRefusal, WidthAdjustment, WidthPurpose, WidthRefusal,
};
pub(crate) use observations::{
    AbilityNonPredictedState, CamoState, EquipmentCreationField, EquipmentField,
    FilmComponentObservation, GameEngineField, ManagedObjectField, ManagedPropertyField,
    MovementComponent, MppField, NavpointField, ObjectParentState, ObjectiveField,
    PlayerStateField, ProbeComponent,
};
pub(crate) use production_frame::ProductionFrame;
pub(crate) use records::{RecordHeader, RecordKind};
pub(crate) use registry::FilmRegistry;
#[cfg(test)]
pub(crate) use transport::{FixtureChunkMetadata, FixtureFilmSource};
pub(crate) use unit_equipment::UnitEquipmentRead;
pub(crate) use unit_references::{UnitReference, UnitReferenceKind};
