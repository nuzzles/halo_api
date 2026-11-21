//! Player records and appearance identifiers recorded in a film. No API lookup is performed.
use super::{BitValue, Xuid};
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
    pub gamertag: String,
    /// Original units including the terminating NUL when recorded.
    pub gamertag_utf16: Vec<u16>,
    pub presence_mask: BitValue,
    pub byte_list: Vec<u8>,
    pub word_list: Vec<u32>,
    pub block104: Vec<u8>,
    pub block16: Vec<u8>,
    pub desired_representation: u32,
    /// Exact byte-copy key. Kept independent of any separately recorded XUID.
    pub join_key: [u8; 8],
    /// Six raw values with widths 10, 14, 6, 8, 7, and 1. No team assignment inferred.
    pub short_fields: [u16; 6],
    pub customization: FilmCustomization,
    /// Build-specific tail, including a second name and possible bot team data.
    pub block44: Vec<u8>,
}

/// One of the 32 initial slots. Vacant slots are retained to preserve slot numbering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialPlayer {
    pub slot: u8,
    pub flags: [bool; 3],
    pub header_word: u32,
    pub header_value: u8,
    pub session_token: u64,
    pub xuid: Xuid,
    pub player: PlayerRecord,
    pub tail: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterPlayer {
    pub xuid: Xuid,
    /// Optional five-bit value; its relationship to player/entity indices is not established.
    pub option: Option<u8>,
    pub player: PlayerRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotPlayer {
    pub absolute_index: u32,
    pub slot: u32,
    pub bot_id: u32,
    pub player: PlayerRecord,
}
