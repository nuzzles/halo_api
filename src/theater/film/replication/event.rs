use super::ReplicationEventKind;
use crate::theater::film::{EventPlayerSlot, TagId, WeaponTagId, WeaponVariantId};

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
    /// `damage_aftermath`: a hit's effect tag, attacker slot, and direction.
    /// The victim is not named by these body fields.
    DamageAftermath {
        effect: Option<TagId>,
        attacker_player: Option<EventPlayerSlot>,
        /// Raw 19-bit packed direction, not an angle or vector.
        packed_direction: u32,
    },
    /// `action_weapon_fire`: the shooter's body slot and the recorded weapon
    /// tag and variant. Neither a hit nor a kill is implied.
    WeaponFire {
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
