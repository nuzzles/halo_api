use super::{BitValue, ReplicationEventKind};
use crate::theater::film::{EventPlayerSlot, TagId, WeaponTagId, WeaponVariantId};

/// One event from the replication stream. Player numbers are the recorded
/// five-bit event-body slots, not resolved player identities. Event references
/// are not used to infer which individual shot produced a hit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationEvent {
    WeaponReload {
        mode: u8,
        player: Option<EventPlayerSlot>,
    },
    Throw {
        selection: ThrowSelection,
        player: Option<EventPlayerSlot>,
    },
    Melee {
        kind: u8,
        extra: Option<u8>,
        mode: u8,
        weapon: Option<WeaponTagId>,
        variant: WeaponVariantId,
        player: Option<EventPlayerSlot>,
    },
    Pickup {
        kind: u8,
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
    /// `damage_aftermath`: a hit's effect tag, attacker slot, direction, and
    /// first five-bit damage scale. The victim is not named by these body fields.
    DamageAftermath {
        effect: Option<TagId>,
        attacker_player: Option<EventPlayerSlot>,
        /// Raw 19-bit packed direction, not an angle or vector.
        packed_direction: u32,
        /// First raw five-bit scale word, not a complete damage amount.
        raw_scale: u8,
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
        /// Recorded one-bit flag, without imposing a gameplay interpretation.
        flag: bool,
        assistant: Option<EventPlayerSlot>,
        assistant_percent: u32,
    },
    /// A structurally complete event whose body is not projected yet.
    /// Contains only body bits, without reference headers or guard trailers.
    Raw {
        kind: ReplicationEventKind,
        payload: BitValue,
    },
}

/// The two recorded throw branches. Codes are not mapped to equipment names.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThrowSelection {
    Tagged { tag: Option<TagId>, value: u8 },
    Index(u8),
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

/// Recorded detonation fields. Numeric suffixes denote wire widths where the
/// gameplay meaning is not established, rather than inventing units or roles.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectileDetonation {
    pub value6: u8,
    pub variant: Option<ProjectileVariant>,
    pub tag: Option<TagId>,
    pub position: EventPosition,
    pub value19: u32,
    pub value5: u8,
    pub flag: bool,
    pub value9: u16,
    pub optional18: Option<u32>,
    pub direction_bits: u8,
    pub direction: u32,
    pub value2: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectileImpact {
    pub variant: Option<ProjectileVariant>,
    pub value14: u16,
    pub first19: u32,
    pub position: EventPosition,
    pub second19: u32,
    pub value9: u16,
    pub flag: bool,
}
