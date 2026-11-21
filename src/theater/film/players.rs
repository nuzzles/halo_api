//! Player records and appearance identifiers recorded in a film. No API lookup is performed.
use super::{BotAbsoluteIndex, BotId, BotSlot, InitialPlayerSlot, Xuid};
use serde::{Deserialize, Serialize};

/// Decoded recorded appearance. Film parsing fails if the layout cannot be decoded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmCustomization {
    pub spartan: SpartanCustomization,
}

/// Hashed names from the recorded model configuration, not inventory paths or tag IDs.
/// Zero and `u32::MAX` are preserved; no universal missing-value rule is imposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRegionPermutation {
    pub region: u32,
    pub permutation: u32,
}

/// Named fields of the v41 HI_1_12_0/HI_1_13_0 appearance layout.
/// These select game assets; they do not contain meshes, textures, or animation clips.
/// Unmodeled bytes are not retained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpartanCustomization {
    pub regions: Vec<ModelRegionPermutation>,
    pub variant_name: u32,
    pub style_name: u32,
    pub model_theme_name: u32,
    pub model_coating_name: u32,
    pub model_marker_name: u32,
    /// All 24 recorded entries, including empty entries, in source order.
    /// An array index is not asserted to be an equipment slot such as helmet.
    pub attachments: Vec<CustomizationAttachment>,
    /// All seven recorded objects, including empty entries, in source order.
    pub objects: Vec<CustomizationObject>,
    pub theme_name: u32,
    pub coating_name: u32,
    pub action_pose: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomizationAttachment {
    pub variant_name: u32,
    pub style_name: u32,
    pub theme_name: u32,
    pub coating_name: u32,
    pub marker_name: u32,
    pub region_override_name: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomizationObject {
    pub variant_name: u32,
    pub style_name: u32,
    pub theme_name: u32,
    pub coating_name: u32,
    pub marker_name: u32,
    /// Eight recorded pairs; zero pairs are retained.
    pub regions: Vec<ModelRegionPermutation>,
}

/// Shared body of an initial player, session-roster entry, or bot record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRecord {
    /// Display name decoded from UTF-16; malformed units use the replacement character.
    pub gamertag: String,
    pub customization: FilmCustomization,
}

/// One of the 32 initial slots. Vacant slots are retained to preserve slot numbering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialPlayer {
    pub slot: InitialPlayerSlot,
    pub xuid: Xuid,
    pub player: PlayerRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterPlayer {
    pub xuid: Xuid,
    pub player: PlayerRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotPlayer {
    pub absolute_index: BotAbsoluteIndex,
    pub slot: BotSlot,
    pub bot_id: BotId,
    pub player: PlayerRecord,
}
