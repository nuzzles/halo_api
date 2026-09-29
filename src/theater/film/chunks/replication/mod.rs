//! Native packet stream, records, and read results.
use super::*;

pub mod components;
pub use components::*;

pub mod replication_stream;
pub use replication_stream::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStreamChunk {
    pub source: FilmChunk,
    pub source_position: usize,
    pub data: Vec<u8>,
    pub packets: Vec<NativeFilmPacket>,
    /// Start of the unwalked suffix. Nested packet bit offsets address `data`.
    pub packet_walk_end_byte: usize,
}
impl ReplicationStreamChunk {
    pub fn payload(&self, packet: &NativeFilmPacket) -> Option<&[u8]> {
        let header = packet.header;
        self.data
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
pub mod native_packet_heads;
pub use native_packet_heads::NativePacketHeadRead;

pub mod native_zoom;
pub use native_zoom::NativeZoomRead;

pub mod event_heads;
pub use event_heads::{
    DecodedHeadEvent, EventReference, EventReferenceValue, HeadEventPayload, HeadEventStop,
};

pub mod types;
pub use types::{FilmPacket, SourceSpan};

pub mod translocator;
pub use translocator::{
    NativeTranslocatorEvent, TeleportPosition, TranslocatorEvent, TranslocatorStop,
};

pub mod records;
pub use records::{RecordHeader, RecordKind};

pub mod native_pickups;
pub use native_pickups::{NativePickupOutcome, NativePickupRead};

pub mod roster_updates;
pub use roster_updates::{NativeRosterRead, RosterEntry, RosterReport, RosterUpdate};

pub mod datums;
pub use datums::{DatumEntry, DatumTable};

pub mod weapon_hit_scan;
pub use weapon_hit_scan::WeaponDamageRead;

pub mod production_frame;
pub use production_frame::{ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame};

pub mod kill_event_chain;
pub use kill_event_chain::{
    KillEventFields, NativeEventField, NativeEventFieldStage, NativeEventFieldValue,
    NativeEventListRead, NativeEventListStop, NativeEventRecord,
};

pub mod world;
pub use world::{FilmViewAdmission, NativeNewBindingRefusal};

pub mod native_weapon_damage;
pub use native_weapon_damage::{NativeWeaponDamageField, NativeWeaponDamageRead};

pub mod anticipated_bindings;
pub use anticipated_bindings::AnticipatedDeclaration;

pub mod weapon_hits;
pub use weapon_hits::WeaponDamage;

pub mod player;
pub use player::{
    NativePlayerSlotRead, NativeSlotField, NativeSlotValue, PlayerTableShorts, PlayerTableSlot,
};
