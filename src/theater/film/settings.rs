//! Recorded match settings. Absent optional fields stay absent; no game defaults are applied.
use super::catalog::{Catalog, options::OptionHash};
use super::{EquipmentTagId, KnownMatchOption, WeaponTagId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core metadata must decode successfully; genuinely omitted variant options stay optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSettings {
    pub build: String,
    /// Recorder's wall-clock start time in UTC, not necessarily the exact match start.
    pub recording_start_time: DateTime<Utc>,
    /// First language of the recorded variant string table.
    pub name: String,
    pub description: String,
    pub engine_type: GameEngineType,
    pub mode_options: ModeOptions,
}

/// Variant engine discriminator, distinct from the registry's broad game category.
/// No code-to-name mapping has been verified yet; exact codes are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i64", into = "i64")]
pub enum GameEngineType {
    Unknown(i64),
}

impl GameEngineType {
    pub const fn raw(self) -> i64 {
        match self {
            Self::Unknown(code) => code,
        }
    }
}

impl From<i64> for GameEngineType {
    fn from(code: i64) -> Self {
        Self::Unknown(code)
    }
}

impl From<GameEngineType> for i64 {
    fn from(value: GameEngineType) -> Self {
        value.raw()
    }
}

/// Recorded mode rules, starting loadouts, and script options. No game defaults are applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeOptions {
    pub score_to_win: Option<i64>,
    pub time_limit_seconds: Option<i64>,
    /// Recorded round limit; absence does not infer the game's default.
    pub round_limit: Option<i64>,
    /// Recorded round-win threshold (`earlyVictoryRounds` in the serializer).
    pub rounds_to_win: Option<i64>,
    /// Recorded round-draw threshold; omitted values remain `None`, not an inferred default.
    pub rounds_to_draw: Option<i64>,
    pub minimum_respawn_seconds: i64,
    /// Additional respawn delay after self-destruction.
    pub self_destruction_respawn_penalty_seconds: i64,
    /// Additional respawn delay after a betrayal.
    pub betrayal_respawn_penalty_seconds: i64,
    pub machinima_controls_enabled: Option<bool>,
    pub overtime_seconds: Option<i64>,
    /// Whether sudden death is enabled during overtime.
    pub sudden_death_enabled_in_overtime: Option<bool>,
    /// Recorded sudden-death duration; absence does not imply zero.
    pub sudden_death_seconds: Option<i64>,
    /// All recorded starting loadouts, retaining their order, including unknown slots.
    pub starting_loadouts: Vec<StartingLoadout>,
    pub options: Vec<MatchOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartingLoadout {
    pub weapons: Vec<StartingWeapon>,
    pub equipment: StartingEquipment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartingEquipment {
    /// A recorded equipment asset tag, even if it is not in the item catalogue.
    Real(EquipmentTagId),
    /// No equipment tag was recorded; no game default is inferred.
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartingWeapon {
    Real(WeaponTagId),
    Random,
    /// Neither a tag nor enabled random selection was recorded. Not a resolved empty slot.
    Unknown,
}

/// A recorded option with a catalogue-classified name; no default value is inferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchOption {
    pub name: MatchOptionName,
    pub value: MatchOptionValue,
}

/// A catalogued option name or the exact unrecognized hash recorded in the film.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchOptionName {
    Known(KnownMatchOption),
    Unknown { name_hash: u32 },
}

impl MatchOptionName {
    pub const fn hash(self) -> OptionHash {
        match self {
            Self::Known(option) => option.hash(),
            Self::Unknown { name_hash } => OptionHash(name_hash),
        }
    }

    pub const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Known(option) => Some(option.name()),
            Self::Unknown { .. } => None,
        }
    }
}

impl From<OptionHash> for MatchOptionName {
    fn from(hash: OptionHash) -> Self {
        Catalog.match_option(hash).map_or(
            Self::Unknown {
                name_hash: hash.raw(),
            },
            Self::Known,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchOptionValue {
    Boolean(bool),
    /// Exact IEEE-754 bits, avoiding lossy/nonfinite JSON floats.
    NumberBits(u32),
    /// The option was recorded, but its value type is not supported.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_codes_round_trip_without_inventing_classifications() {
        for code in [i64::MIN, -1, 0, 1, 2, 99, i64::MAX] {
            let engine = GameEngineType::from(code);
            assert_eq!(engine, GameEngineType::Unknown(code));
            assert_eq!(engine.raw(), code);
            let json = serde_json::to_string(&engine).unwrap();
            assert_eq!(json, code.to_string());
            assert_eq!(
                serde_json::from_str::<GameEngineType>(&json).unwrap(),
                engine
            );
        }
        for invalid in ["null", "\"Slayer\"", "1.5", "9223372036854775808"] {
            assert!(serde_json::from_str::<GameEngineType>(invalid).is_err());
        }
    }
}
