//! Recorded match settings. Absent optional fields stay absent; no game defaults are applied.
use super::TagId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSettings {
    pub build: Option<String>,
    pub start_time_unix_seconds: Option<u32>,
    /// Whether a complete, unambiguous variant was available for named projection.
    pub variant_decoded: bool,
    /// First language of the recorded variant string table.
    pub name: Option<String>,
    pub description: Option<String>,
    pub engine_type: Option<i64>,
    pub killcam_enabled: Option<bool>,
    pub play_of_game_enabled: Option<bool>,
    pub score_to_win: Option<i64>,
    pub time_limit_seconds: Option<i64>,
    pub minimum_respawn_seconds: Option<i64>,
    pub overtime_seconds: Option<i64>,
    pub last_chance_enabled: Option<bool>,
    pub last_chance_seconds: Option<i64>,
    /// All recorded starting loadouts, retaining their order and absent fields.
    pub starting_loadouts: Option<Vec<StartingLoadout>>,
    pub options: Option<Vec<MatchOption>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartingLoadout {
    pub weapons: Option<Vec<StartingWeapon>>,
    pub equipment: Option<TagId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartingWeapon {
    pub tag: Option<TagId>,
    pub random: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchOption {
    pub name_hash: Option<u32>,
    /// Recorded bytes, including unknown types and nonfinite float bit patterns.
    pub raw_value: Option<Vec<u8>>,
    pub value: Option<MatchOptionValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchOptionValue {
    Boolean(bool),
    /// Exact IEEE-754 bits, avoiding lossy/nonfinite JSON floats.
    NumberBits(u32),
}
