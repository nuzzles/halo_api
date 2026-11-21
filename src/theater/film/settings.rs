//! Recorded match settings. Absent optional fields stay absent; no game defaults are applied.
use super::{TagId, WeaponTagId};
use serde::{Deserialize, Serialize};

/// Core metadata must decode successfully; genuinely omitted variant options stay optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSettings {
    pub build: String,
    pub start_time_unix_seconds: u32,
    /// First language of the recorded variant string table.
    pub name: String,
    pub description: String,
    pub engine_type: i64,
    pub score_to_win: Option<i64>,
    pub time_limit_seconds: Option<i64>,
    pub minimum_respawn_seconds: i64,
    pub overtime_seconds: Option<i64>,
    pub last_chance_enabled: Option<bool>,
    pub last_chance_seconds: Option<i64>,
    /// All recorded starting loadouts, retaining their order, including unknown slots.
    pub starting_loadouts: Vec<StartingLoadout>,
    pub options: Vec<MatchOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartingLoadout {
    pub weapons: Vec<StartingWeapon>,
    /// No equipment tag was recorded when absent; no game default is inferred.
    pub equipment: Option<TagId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartingWeapon {
    Real(WeaponTagId),
    Random,
    /// Neither a tag nor enabled random selection was recorded. Not a resolved empty slot.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchOption {
    pub name_hash: u32,
    /// Recorded bytes, including unknown types and nonfinite float bit patterns.
    pub raw_value: [u8; 8],
    pub value: MatchOptionValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchOptionValue {
    Boolean(bool),
    /// Exact IEEE-754 bits, avoiding lossy/nonfinite JSON floats.
    NumberBits(u32),
    /// An unrecognized discriminator; the exact payload remains in `MatchOption::raw_value`.
    Unknown,
}
