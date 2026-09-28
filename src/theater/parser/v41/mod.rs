//! Structural v41 decoding. No candidate searches or inferred runtime settings.
pub(crate) use super::{bits, transport};
use crate::theater::film::*;

/// Version-specific chunk decoder selected by `Film::parse` after registry validation.
/// Kept internal so callers use the single version-dispatching entry point.
pub(crate) struct V41ChunkParser;

#[cfg(test)]
pub(crate) fn test_chunks(source: &FilmSource) -> Vec<FilmChunk> {
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

pub(crate) mod datums;

pub(crate) mod event_heads;

pub(crate) mod kill_decode;

pub(crate) mod kill_event_chain;

pub(crate) mod medals;

pub(crate) mod native_packet_heads;

pub(crate) mod native_pickups;

pub(crate) mod native_reader;

pub(crate) use crate::theater::film::components::native_scan_profile;

pub(crate) mod native_weapon_damage;

pub(crate) mod native_zoom;

pub(crate) mod production_frame;

pub(crate) use crate::theater::film::components::profile;

pub(crate) use crate::theater::film::components::profile_values;

pub(crate) use crate::theater::film::components::read_diagnostics;

pub(crate) mod records;

pub(crate) mod registry;

pub(crate) mod roster_updates;

pub(crate) mod source_bits;

pub(crate) mod summary;

pub(crate) mod translocator;

pub(crate) mod types;

pub(crate) use crate::theater::film::components::unit_equipment;

pub(crate) use crate::theater::film::components::unit_references;

pub(crate) mod weapon_hit_scan;

pub(crate) mod world;

pub(crate) mod player_slot;

pub(crate) mod position_capture;
pub(crate) use crate::theater::resolved::interpretation::bootstrap;
pub(crate) use crate::theater::resolved::interpretation::bot_metadata;

pub(crate) use crate::theater::resolved::interpretation::fire_events;
pub(crate) use crate::theater::resolved::interpretation::head_observations;
pub(crate) use crate::theater::resolved::interpretation::highlight_events;
pub(crate) use crate::theater::resolved::interpretation::native_event_gate;
pub(crate) use crate::theater::resolved::interpretation::native_identity;
pub(crate) use crate::theater::resolved::interpretation::objective_extract;
pub(crate) use crate::theater::resolved::interpretation::player_table;
pub(crate) use anticipated_bindings::*;
pub(crate) use bootstrap::*;
pub(crate) use bot_metadata::*;
pub(crate) use components::*;
pub(crate) use datums::*;
pub(crate) use event_heads::*;
pub(crate) use fire_events::*;
pub(crate) use head_observations::*;
pub(crate) use highlight_events::*;
pub(crate) use kill_decode::*;
pub(crate) use kill_event_chain::*;
pub(crate) use native_event_gate::*;
pub(crate) use native_identity::*;
pub(crate) use native_packet_heads::*;
pub(crate) use native_pickups::*;
pub(crate) use native_reader::*;
pub(crate) use native_weapon_damage::*;
pub(crate) use native_zoom::*;
pub(crate) use objective_extract::*;
pub(crate) use player_table::*;
pub(crate) use position_capture::*;
pub(crate) use profile_values::*;
pub(crate) use records::*;
pub(crate) use roster_updates::*;
pub(crate) use source_bits::*;
pub(crate) use translocator::*;
pub(crate) use types::*;
pub(crate) use world::*;

pub(crate) use crate::theater::resolved::identity::PlayerTable;
pub(crate) use components::{ComponentField, EntityRecord, EntityViewStop};
pub(crate) use native_scan_profile::NativeScanProfile;

pub(crate) use production_frame::ProductionFrame;
pub(crate) use profile::FilmMapBounds;
pub(crate) use read_diagnostics::{
    FilmComponentObservation, FilmReadDiagnostics, NativeAbilityNonPredictedState,
    NativeEquipmentCreationField, NativeEquipmentField, NativeGameEngineField,
    NativeManagedObjectField, NativeMovementComponent, NativeMppField, NativeNavpointField,
    NativeObjectiveField, NativePlayerStateField, NativeProbeComponent, NativeWidthAdjustment,
    NativeWidthPurpose,
};
pub(crate) use records::{RecordHeader, RecordKind};
pub(crate) use registry::FilmRegistry;
#[cfg(test)]
pub(crate) use transport::{FilmSource, FilmSourceMetadata};
pub(crate) use unit_equipment::UnitEquipmentRead;
pub(crate) use unit_references::{NativeUnitReference, NativeUnitReferenceKind};

pub(crate) use crate::theater::film::PlayerTableSlot;

pub(crate) mod replication;
