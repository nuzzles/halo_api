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
