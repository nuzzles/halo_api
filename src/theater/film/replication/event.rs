use super::{BitValue, ReplicationEventKind};
use crate::theater::film::{TagId, WeaponVariantId};

/// One recorded event reference. `kind` selects the handle's index domain;
/// `index` is kept in that domain rather than assumed to be a live entity ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationEventHandle {
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
        handles: [Option<ReplicationEventHandle>; 3],
        mode: u8,
        player: Option<u8>,
    },
    Throw {
        handles: [Option<ReplicationEventHandle>; 3],
        selection: ThrowSelection,
        player: Option<u8>,
    },
    Melee {
        handles: [Option<ReplicationEventHandle>; 3],
        kind: u8,
        extra: Option<u8>,
        mode: u8,
        weapon: Option<TagId>,
        variant: WeaponVariantId,
        player: Option<u8>,
    },
    Pickup {
        handles: [Option<ReplicationEventHandle>; 3],
        kind: u8,
        item: Option<TagId>,
    },
    Zoom {
        handles: [Option<ReplicationEventHandle>; 3],
        level: u8,
    },
    Teleport {
        handles: [Option<ReplicationEventHandle>; 3],
        /// Quantized source positions, not reconstructed world coordinates.
        from: EventPosition,
        to: EventPosition,
        tags: [Option<TagId>; 2],
    },
    ProjectileDetonation {
        handles: [Option<ReplicationEventHandle>; 3],
        data: ProjectileDetonation,
    },
    ProjectileImpact {
        handles: [Option<ReplicationEventHandle>; 3],
        data: ProjectileImpact,
    },
    /// `damage_aftermath`: a hit's effect tag, attacker slot, direction, and
    /// first five-bit damage scale. The victim is not named by these body fields.
    DamageAftermath {
        /// First recorded reference, interpreted as the victim when present.
        victim: Option<ReplicationEventHandle>,
        /// Second recorded reference, interpreted as the attacker when present.
        attacker: Option<ReplicationEventHandle>,
        /// Optional third recorded reference (kind 7). Its gameplay meaning is
        /// unknown, so this handle is preserved without assigning it a role.
        third_reference: Option<ReplicationEventHandle>,
        effect: Option<TagId>,
        attacker_player: Option<u8>,
        /// Raw 19-bit packed direction, not an angle or vector.
        packed_direction: u32,
        /// First raw five-bit scale word, not a complete damage amount.
        raw_scale: u8,
    },
    /// `action_weapon_fire`: the shooter's body slot and the recorded weapon
    /// tag and variant. Neither a hit nor a kill is implied.
    WeaponFire {
        /// First reference is the shooter's unit when present.
        handles: [Option<ReplicationEventHandle>; 3],
        player: Option<u8>,
        weapon: Option<TagId>,
        variant: WeaponVariantId,
    },
    /// `PlayerKilledEvent`: recorded player slots and damage-share words. The
    /// event itself does not identify a weapon or damage-effect tag.
    Kill {
        handles: [Option<ReplicationEventHandle>; 3],
        victim: Option<u8>,
        killer: Option<u8>,
        killer_percent: u32,
        /// Recorded one-bit flag, without imposing a gameplay interpretation.
        flag: bool,
        assistant: Option<u8>,
        assistant_percent: u32,
    },
    /// A structurally complete event whose body is not projected yet. Bits
    /// start after the presence flag and seven-bit kind code, and include
    /// references and any guard trailer.
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
