//! Source-linked interpretations of replication packet payloads.

use crate::theater::film::{ComponentField, FilmPacketHeader};

pub mod event_heads;
pub use event_heads::{
    DecodedHeadEvent, EventReference, EventReferenceValue, HeadEventPayload, HeadEventStop,
};

pub mod native_zoom;
pub use native_zoom::NativeZoomRead;

pub mod native_pickups;
pub use native_pickups::{NativePickupOutcome, NativePickupRead};

pub mod translocator;
pub use translocator::{
    NativeTranslocatorEvent, TeleportPosition, TranslocatorEvent, TranslocatorStop,
};

pub mod weapon_hits;
pub use weapon_hits::WeaponDamage;

pub mod weapon_hit_scan;
pub use weapon_hit_scan::WeaponDamageRead;

pub mod native_weapon_damage;
pub use native_weapon_damage::{NativeWeaponDamageField, NativeWeaponDamageRead};

pub mod native_packet_heads;
pub use native_packet_heads::NativePacketHeadRead;
