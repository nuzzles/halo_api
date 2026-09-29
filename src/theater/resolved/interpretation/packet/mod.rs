//! Source-linked interpretations of replication packet payloads.

use crate::theater::film::FilmPacketHeader;

pub mod event_heads;
pub use event_heads::{
    DecodedHeadEvent, EventReference, EventReferenceValue, HeadEventPayload, HeadEventStop,
};

pub mod zoom;
pub use zoom::ZoomRead;

pub mod pickups;
pub use pickups::{PickupOutcome, PickupRead};

pub mod translocator;
pub use translocator::{TeleportPosition, TranslocatorEvent, TranslocatorRead, TranslocatorStop};

pub mod weapon_hits;
pub use weapon_hits::WeaponDamage;

pub mod weapon_hit_scan;
pub use weapon_hit_scan::WeaponDamageRead;

pub mod weapon_damage;
pub use weapon_damage::{WeaponDamageField, WeaponDamageTrace};

pub mod packet_heads;
pub use packet_heads::PacketHeadRead;
