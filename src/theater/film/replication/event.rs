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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationEvent {
    WeaponReload {
        player: Option<EventPlayerSlot>,
    },
    Throw {
        /// Asset tag when the tagged throw branch records one.
        item: Option<TagId>,
        player: Option<EventPlayerSlot>,
    },
    Melee {
        weapon: Option<WeaponTagId>,
        variant: WeaponVariantId,
        player: Option<EventPlayerSlot>,
    },
    Pickup {
        item: Option<TagId>,
    },
    Zoom {
        level: u8,
    },
    Teleport {
        /// Quantized source positions, not reconstructed world coordinates.
        from: EventPosition,
        to: EventPosition,
        tags: [Option<TagId>; 2],
    },
    ProjectileDetonation {
        data: ProjectileDetonation,
    },
    ProjectileImpact {
        data: ProjectileImpact,
    },
    /// `damage_aftermath`: victim/attacker references and recorded hit fields.
    DamageAftermath {
        /// First recorded reference: the victim, when present.
        victim: Option<ReplicationEventReference>,
        /// Second recorded reference: the attacker, when present.
        attacker: Option<ReplicationEventReference>,
        effect: Option<TagId>,
        /// Separately recorded body slot; not derived from `attacker`.
        attacker_player: Option<EventPlayerSlot>,
        /// Raw 19-bit packed direction, not an angle or vector.
        packed_direction: u32,
    },
    /// `action_weapon_fire`: the shooter's body slot and the recorded weapon
    /// tag and variant. Neither a hit nor a kill is implied.
    WeaponFire {
        /// First recorded reference: the shooter's unit, when present.
        shooter: Option<ReplicationEventReference>,
        player: Option<EventPlayerSlot>,
        weapon: Option<WeaponTagId>,
        variant: WeaponVariantId,
    },
    /// `PlayerKilledEvent`: recorded player slots and damage-share words. The
    /// event itself does not identify a weapon or damage-effect tag.
    Kill {
        victim: Option<EventPlayerSlot>,
        killer: Option<EventPlayerSlot>,
        killer_percent: u32,
        assistant: Option<EventPlayerSlot>,
        assistant_percent: u32,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectileVariant {
    pub tag: Option<TagId>,
    pub variant: WeaponVariantId,
}

/// Established detonation asset identifiers and position. Uninterpreted wire
/// fields are consumed by the parser but not retained in the finished film.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectileDetonation {
    pub variant: Option<ProjectileVariant>,
    pub tag: Option<TagId>,
    pub position: EventPosition,
}

/// Established impact asset identifiers and position, without uninterpreted fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectileImpact {
    pub variant: Option<ProjectileVariant>,
    pub position: EventPosition,
}
