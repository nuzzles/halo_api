//! Native v41 record models and parser support.
//! Use `Film::parse` from the parent module to parse a recording.
pub mod anticipated_bindings;
pub(crate) use anticipated_bindings::*;
pub(crate) mod bits;
pub mod bootstrap;
pub(crate) use bootstrap::*;
pub mod bot_metadata;
pub(crate) use bot_metadata::*;
pub mod chain_inference;
pub mod components;
pub(crate) use components::*;
pub mod datums;
pub(crate) use datums::*;
pub mod event_heads;
pub(crate) use event_heads::*;
pub mod fire_events;
pub(crate) use fire_events::*;
pub mod head_observations;
pub(crate) use head_observations::*;
pub mod highlight_events;
pub(crate) use highlight_events::*;
pub mod kill_decode;
pub(crate) use kill_decode::*;
pub mod kill_event_chain;
pub(crate) use kill_event_chain::*;
pub mod medals;
pub mod native_event_gate;
pub(crate) use native_event_gate::*;
pub mod native_identity;
pub(crate) use native_identity::*;
pub mod native_packet_heads;
pub(crate) use native_packet_heads::*;
pub mod native_pickups;
pub(crate) use native_pickups::*;
pub mod native_profile;
pub mod native_reader;
pub(crate) use native_reader::*;
pub mod native_scan_profile;
pub mod native_weapon_damage;
pub(crate) use native_weapon_damage::*;
pub mod native_zoom;
pub(crate) use native_zoom::*;
pub mod objective_extract;
pub(crate) use objective_extract::*;
pub mod player_table;
pub(crate) use player_table::*;
pub mod position_capture;
pub(crate) use position_capture::*;
pub mod production_frame;
pub mod profile;
pub(crate) use profile::*;
pub mod profile_values;
pub(crate) use profile_values::*;
pub mod read_diagnostics;
pub mod records;
pub(crate) use records::*;
pub mod registry;
pub(crate) use registry::*;
pub mod roster_updates;
pub(crate) use roster_updates::*;
pub(crate) mod source;
pub mod source_bits;
pub(crate) use source_bits::*;
pub mod summary;
pub mod translocator;
pub(crate) use translocator::*;
pub mod types;
pub(crate) use types::*;
pub mod unit_equipment;
pub mod unit_references;
pub mod weapon_hit_scan;
pub mod weapon_hits;
pub mod world;
pub(crate) use world::*;

pub use anticipated_bindings::AnticipatedDeclaration;
pub use bootstrap::FilmIdentity;
pub use bot_metadata::{FilmBotEntry, NativeBotCandidate, NativeBotMetadataRead};
pub use chain_inference::ChainInferenceOutcome;
pub use components::{
    BindingOrigin, ComponentField, DecodedFrameView, EntityComponentAttempt, EntityComponentSpan,
    EntityRecord, EntityViewStop, FrameViewStop, KeyframeChainAttempt, KeyframeChainStop,
    KeyframeComponentSpan, KeyframeRecord, KeyframeStop, NativeActionBlock, NativeControlEntry,
    NativeKeyframeTable,
};
pub use datums::{DatumEntry, DatumTable};
pub use event_heads::{
    DecodedHeadEvent, EventReference, EventReferenceValue, HeadEventPayload, HeadEventStop,
};
pub use fire_events::{
    FilmFireEvent, FireUnitReference, NativeFireAimAttempt, NativeFireAimMethod, NativeFireAimStop,
    NativeFireField, NativeFireHeaderStop, NativeFireRead,
};
pub use highlight_events::{
    NativeHighlightEvent, NativeHighlightIdentityRead, NativeHighlightScan, NativeHighlightTailRead,
};
pub use kill_event_chain::{
    KillEventFields, NativeEventField, NativeEventFieldStage, NativeEventFieldValue,
    NativeEventListRead, NativeEventListStop, NativeEventRecord,
};
pub use medals::MedalAward;
pub use native_event_gate::{NativeEventGate15Policy, NativeEventGate15Selection};
pub use native_identity::{NativeIdentityField, NativeIdentityRead, NativeIdentityValue};
pub use native_packet_heads::NativePacketHeadRead;
pub use native_pickups::{NativePickupOutcome, NativePickupRead};
pub use native_profile::{NativeMovementProfile, NativePrecisionDescriptor};
pub use native_scan_profile::{
    NativeKeyframeLayout, NativeScanGrammar, NativeScanProfile, NativeSharedWidths,
};
pub use native_weapon_damage::{NativeWeaponDamageField, NativeWeaponDamageRead};
pub use native_zoom::NativeZoomRead;
pub use objective_extract::ObjectiveFooterEvent;
pub use player_table::{
    NativePlayerSlotRead, NativeSlotField, NativeSlotValue, PlayerTable, PlayerTableError,
    PlayerTableReport, PlayerTableShorts, PlayerTableSlot,
};
pub use position_capture::NativePositionKind;
pub use production_frame::{ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame};
pub use profile::FilmMapBounds;
pub use profile_values::{FilmMppWidths, FilmQuantizationRange};
pub use read_diagnostics::{
    FilmComponentObservation, FilmReadDiagnostics, NativeAbilityNonPredictedState, NativeCamoState,
    NativeEquipmentCreationField, NativeEquipmentField, NativeGameEngineField,
    NativeManagedObjectField, NativeManagedPropertyField, NativeMovementComponent, NativeMppField,
    NativeNavpointField, NativeObjectParentState, NativeObjectiveField, NativePlayerStateField,
    NativeProbeComponent, NativeReadOperation, NativeReadRefusal, NativeWidthAdjustment,
    NativeWidthPurpose, NativeWidthRefusal,
};
pub use records::{RecordHeader, RecordKind};
pub use registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, NativeRegistryBlockRead,
    NativeRegistrySlotRead,
};
pub use roster_updates::{NativeRosterRead, RosterEntry, RosterReport, RosterUpdate};
pub(crate) use source::{FilmSource, FilmSourceMetadata};
pub use translocator::{
    NativeTranslocatorEvent, TeleportPosition, TranslocatorEvent, TranslocatorStop,
};
pub use types::{FilmPacket, SourceSpan, SummaryEvent, SummaryKind};
pub use unit_equipment::{UnitEquipmentEntry, UnitEquipmentRead};
pub use unit_references::{NativeUnitReference, NativeUnitReferenceKind};
pub use weapon_hit_scan::WeaponDamageRead;
pub use weapon_hits::WeaponDamage;
pub use world::{FilmViewAdmission, NativeNewBindingRefusal};

pub(crate) mod v41;
