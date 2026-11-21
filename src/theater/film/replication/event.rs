use super::ReplicationEventKind;
use crate::theater::film::{EventPlayerSlot, TagId, WeaponTagId, WeaponVariantId};

/// A recorded reference with a known role in its event. The index is relative
/// to `kind`'s reference domain, not an `EntityId` or a player slot. Resolving it
/// against runtime entities belongs to consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ReplicationEventReference {
    pub kind: u8,
    pub index: u16,
    pub generation: u8,
}

/// One event from the replication stream. Player numbers are the recorded
/// five-bit event-body slots, not resolved player identities. Event references
/// are not used to infer which individual shot produced a hit.
/// Required fields are enforced for supported captures even when the wire has
/// a presence gate; an absent required value fails parsing instead of defaulting.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationEvent {
    WeaponReload {
        player: EventPlayerSlot,
    },
    Throw {
        player: EventPlayerSlot,
    },
    Melee {
        weapon: WeaponTagId,
        variant: WeaponVariantId,
        player: EventPlayerSlot,
    },
    Pickup {
        item: TagId,
    },
    Zoom {
        level: u8,
    },
    ProjectileDetonation {
        position: EventPosition,
    },
    ProjectileImpact {
        position: EventPosition,
    },
    /// `damage_aftermath`: victim/attacker references and recorded hit fields.
    DamageAftermath {
        /// First recorded reference: the victim.
        victim: ReplicationEventReference,
        /// Second recorded reference: the attacker, when present.
        attacker: Option<ReplicationEventReference>,
        effect: TagId,
        /// Separately recorded body slot; not derived from `attacker`.
        attacker_player: Option<EventPlayerSlot>,
        /// Raw 19-bit packed direction, not an angle or vector.
        packed_direction: u32,
    },
    /// `action_weapon_fire`: the shooter's body slot and the recorded weapon
    /// tag and variant. Neither a hit nor a kill is implied.
    WeaponFire {
        /// First recorded reference: the shooter's unit.
        shooter: ReplicationEventReference,
        player: EventPlayerSlot,
        weapon: WeaponTagId,
        variant: WeaponVariantId,
    },
    /// A structurally complete event without a supported public payload.
    /// Keeps the event occurrence and kind, not its uninterpreted bytes.
    Other {
        kind: ReplicationEventKind,
    },
}

/// Exact codes and their widths from the point's selected encoding.
/// Map bounds and application-specific coordinate conversion belong to consumers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventPosition {
    DefaultBox {
        codes: [u32; 3],
        axis_bits: u8,
    },
    MapRegion {
        region: u32,
        codes: [u32; 3],
        axis_bits: [u8; 3],
    },
}
