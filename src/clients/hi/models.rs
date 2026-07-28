use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Response body from the playlist CSR endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecords {
    #[serde(rename = "Value")]
    pub records: Vec<CsrRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecord {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "Result")]
    pub result: CsrRecordResult,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecordResult {
    #[serde(rename = "Current")]
    pub current: CsrRecordRanking,
    #[serde(default, rename = "SeasonMax")]
    pub season_max: CsrRecordRanking,
    #[serde(rename = "AllTimeMax")]
    pub peak: CsrRecordRanking,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CsrRecordRanking {
    /// Numeric CSR value, or `-1` if the player is unranked in this playlist.
    #[serde(rename = "Value")]
    pub value: i32,
    #[serde(rename = "MeasurementMatchesRemaining")]
    pub measurement_matches_remaining: i32,
    #[serde(rename = "Tier")]
    pub tier: String,
    #[serde(rename = "TierStart")]
    pub tier_start: i32,
    /// 0-indexed sub-tier within `tier`. Not meaningful for Onyx, which reports `value`.
    #[serde(rename = "SubTier")]
    pub sub_tier: i32,
    #[serde(rename = "NextTier")]
    pub next_tier: String,
    #[serde(rename = "NextTierStart")]
    pub next_tier_start: i32,
    #[serde(rename = "NextSubTier")]
    pub next_sub_tier: i32,
    #[serde(rename = "InitialMeasurementMatches")]
    pub initial_measurement_matches: i32,
    #[serde(default, rename = "DemotionProtectionMatchesRemaining")]
    pub demotion_protection_matches_remaining: i32,
    #[serde(default, rename = "InitialDemotionProtectionMatches")]
    pub initial_demotion_protection_matches: i32,
}

impl CsrRecordRanking {
    pub fn is_unranked(&self) -> bool {
        self.value == -1
    }
}

/// A page of a player's match history.
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerMatchHistory {
    #[serde(rename = "Results")]
    pub results: Vec<MatchHistoryEntry>,
    #[serde(rename = "ResultCount")]
    pub result_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchHistoryEntry {
    #[serde(rename = "MatchId")]
    pub match_id: String,
    #[serde(rename = "MatchInfo")]
    pub info: MatchInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchInfo {
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "EndTime")]
    pub end_time: DateTime<Utc>,
    #[serde(rename = "Playlist")]
    pub playlist: Option<MatchPlaylist>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchPlaylist {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
}

/// Response body from the matchmade service record endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServiceRecord {
    #[serde(rename = "Subqueries")]
    pub subqueries: ServiceRecordSubqueries,
    #[serde(rename = "TimePlayed")]
    pub time_played: String,
    #[serde(rename = "MatchesCompleted")]
    pub matches_completed: i32,
    #[serde(rename = "Wins")]
    pub wins: i32,
    #[serde(rename = "Losses")]
    pub losses: i32,
    #[serde(rename = "Ties")]
    pub ties: i32,
    #[serde(rename = "CoreStats")]
    pub core_stats: CoreStats,
    #[serde(rename = "BombStats")]
    pub bomb_stats: serde_json::Value,
    #[serde(rename = "CaptureTheFlagStats")]
    pub capture_the_flag_stats: serde_json::Value,
    #[serde(rename = "EliminationStats")]
    pub elimination_stats: serde_json::Value,
    #[serde(rename = "ExtractionStats")]
    pub extraction_stats: serde_json::Value,
    #[serde(rename = "InfectionStats")]
    pub infection_stats: serde_json::Value,
    #[serde(rename = "OddballStats")]
    pub oddball_stats: serde_json::Value,
    #[serde(rename = "ZonesStats")]
    pub zones_stats: serde_json::Value,
    #[serde(rename = "StockpileStats")]
    pub stockpile_stats: serde_json::Value,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServiceRecordSubqueries {
    #[serde(rename = "SeasonIds")]
    pub season_ids: Vec<String>,
    #[serde(rename = "GameVariantCategories")]
    pub game_variant_categories: Vec<i32>,
    #[serde(rename = "IsRanked")]
    pub is_ranked: Vec<bool>,
    #[serde(rename = "PlaylistAssetIds")]
    pub playlist_asset_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CoreStats {
    #[serde(rename = "Score")]
    pub score: i32,
    #[serde(rename = "PersonalScore")]
    pub personal_score: i32,
    #[serde(rename = "RoundsWon")]
    pub rounds_won: i32,
    #[serde(rename = "RoundsLost")]
    pub rounds_lost: i32,
    #[serde(rename = "RoundsTied")]
    pub rounds_tied: i32,
    #[serde(rename = "Kills")]
    pub kills: i32,
    #[serde(rename = "Deaths")]
    pub deaths: i32,
    #[serde(rename = "Assists")]
    pub assists: i32,
    #[serde(rename = "AverageKDA")]
    pub kda: f32,
    #[serde(rename = "Suicides")]
    pub suicides: i32,
    #[serde(rename = "Betrayals")]
    pub betrayals: i32,
    #[serde(rename = "GrenadeKills")]
    pub grenade_kills: i32,
    #[serde(rename = "HeadshotKills")]
    pub headshot_kills: i32,
    #[serde(rename = "MeleeKills")]
    pub melee_kills: i32,
    #[serde(rename = "PowerWeaponKills")]
    pub power_weapon_kills: i32,
    #[serde(rename = "ShotsFired")]
    pub shots_fired: i32,
    #[serde(rename = "ShotsHit")]
    pub shots_hit: i32,
    #[serde(rename = "Accuracy")]
    pub accuracy: f32,
    #[serde(rename = "DamageDealt")]
    pub damage_dealt: i32,
    #[serde(rename = "DamageTaken")]
    pub damage_taken: i32,
    #[serde(rename = "CalloutAssists")]
    pub callout_assists: i32,
    #[serde(rename = "VehicleDestroys")]
    pub vehicle_destroys: i32,
    #[serde(rename = "DriverAssists")]
    pub driver_assists: i32,
    #[serde(rename = "Hijacks")]
    pub hijacks: i32,
    #[serde(rename = "EmpAssists")]
    pub emp_assists: i32,
    #[serde(rename = "MaxKillingSpree")]
    pub max_killing_spree: i32,
    #[serde(rename = "Medals")]
    pub medals: Vec<StatAward>,
    #[serde(rename = "PersonalScores")]
    pub personal_scores: Vec<StatAward>,
    #[serde(rename = "Spawns")]
    pub spawns: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatAward {
    #[serde(rename = "NameId")]
    pub name_id: i64,
    #[serde(rename = "Count")]
    pub count: i32,
    #[serde(rename = "TotalPersonalScoreAwarded")]
    pub total_personal_score_awarded: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserInfo {
    pub xuid: String,
    pub gamertag: String,
    pub gamerpic: Gamerpic,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Gamerpic {
    pub small: String,
    pub medium: String,
    pub large: String,
    pub xlarge: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrSeasonCalendar {
    #[serde(rename = "Seasons")]
    pub seasons: Vec<CsrSeason>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonCalendar {
    #[serde(rename = "Seasons")]
    pub seasons: Vec<Season>,
    #[serde(rename = "Events")]
    pub events: Vec<SeasonEvent>,
    #[serde(rename = "CareerRank")]
    pub career_rank: CareerRank,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Season {
    #[serde(rename = "CsrSeasonFilePath")]
    pub csr_season_file_path: String,
    #[serde(rename = "OperationTrackPath")]
    pub operation_track_path: String,
    #[serde(rename = "SeasonMetadata")]
    pub season_metadata: String,
    #[serde(rename = "StartDate")]
    pub start_date: ApiDate,
    #[serde(rename = "EndDate")]
    pub end_date: ApiDate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonEvent {
    #[serde(rename = "RewardTrackPath")]
    pub reward_track_path: String,
    #[serde(rename = "StartDate")]
    pub start_date: ApiDate,
    #[serde(rename = "EndDate")]
    pub end_date: ApiDate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CareerRank {
    #[serde(rename = "RewardTrackPath")]
    pub reward_track_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrSeason {
    #[serde(rename = "CsrSeasonFilePath")]
    pub csr_season_file_path: String,
    #[serde(rename = "StartDate")]
    pub start_date: ApiDate,
    #[serde(rename = "EndDate")]
    pub end_date: ApiDate,
}

impl CsrSeasonCalendar {
    pub fn current(&self, at: DateTime<Utc>) -> Option<&CsrSeason> {
        self.seasons
            .iter()
            .filter(|season| {
                season.start_date.iso8601_date <= at && at < season.end_date.iso8601_date
            })
            .max_by_key(|season| season.start_date.iso8601_date)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiDate {
    #[serde(rename = "ISO8601Date")]
    pub iso8601_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistMetadata {
    #[serde(rename = "NameHint")]
    pub name_hint: String,
    #[serde(rename = "UgcPlaylistVersion")]
    pub ugc_playlist_version: String,
    #[serde(rename = "HasCsr")]
    pub has_csr: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetLink {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
    #[serde(rename = "VersionId")]
    pub version_id: String,
    #[serde(rename = "PublicName")]
    pub public_name: String,
    #[serde(rename = "Description")]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "RotationEntries")]
    pub rotation_entries: Vec<RotationEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RotationEntry {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "Metadata")]
    pub metadata: RotationMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RotationMetadata {
    #[serde(rename = "Weight")]
    pub weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapModePairAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "MapLink")]
    pub map: AssetLink,
    #[serde(rename = "UgcGameVariantLink")]
    pub mode: AssetLink,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "CustomData")]
    pub custom_data: MapCustomData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapCustomData {
    #[serde(rename = "NumOfObjectsOnMap")]
    pub object_count: i64,
    #[serde(rename = "TagLevelId")]
    pub tag_level_id: i64,
    #[serde(rename = "IsBaked")]
    pub is_baked: bool,
    #[serde(rename = "HasNodeGraph")]
    pub has_node_graph: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameVariantAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "CustomData")]
    pub custom_data: GameVariantCustomData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameVariantCustomData {
    #[serde(rename = "KeyValues")]
    pub key_values: serde_json::Value,
    #[serde(rename = "HasNodeGraph")]
    pub has_node_graph: bool,
}

#[derive(Debug, Clone)]
pub struct RankedArenaMapMode {
    pub weight: f64,
    pub pair: MapModePairAsset,
    pub map: MapAsset,
    pub mode: GameVariantAsset,
}

#[derive(Debug, Clone)]
pub struct RankedArenaSeason {
    pub season: CsrSeason,
    pub map_modes: Vec<RankedArenaMapMode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranking(value: i32) -> CsrRecordRanking {
        CsrRecordRanking {
            value,
            tier: "Platinum".to_string(),
            sub_tier: 2,
            ..CsrRecordRanking::default()
        }
    }

    #[test]
    fn negative_one_is_unranked() {
        assert!(ranking(-1).is_unranked());
    }

    #[test]
    fn positive_value_is_ranked() {
        assert!(!ranking(1500).is_unranked());
    }

    #[test]
    fn deserializes_full_csr_response_shape() {
        let json = serde_json::json!({
            "Value": [{
                "Id": "xuid(123)",
                "ResultCode": 0,
                "Result": {
                    "Current": {
                        "Value": 1500,
                        "MeasurementMatchesRemaining": 0,
                        "Tier": "Platinum",
                        "TierStart": 1200,
                        "SubTier": 2,
                        "NextTier": "Diamond",
                        "NextTierStart": 1500,
                        "NextSubTier": 0,
                        "InitialMeasurementMatches": 5,
                        "DemotionProtectionMatchesRemaining": 2,
                        "InitialDemotionProtectionMatches": 3
                    },
                    "SeasonMax": { "Value": 1550, "Tier": "Diamond", "SubTier": 0 },
                    "AllTimeMax": { "Value": 1600, "Tier": "Diamond", "SubTier": 0 },
                }
            }]
        });
        let records: CsrRecords = serde_json::from_value(json).unwrap();
        assert_eq!(records.records.len(), 1);
        assert_eq!(records.records[0].result.current.value, 1500);
        assert_eq!(records.records[0].result.current.tier_start, 1200);
        assert_eq!(records.records[0].result.current.next_tier, "Diamond");
        assert_eq!(
            records.records[0]
                .result
                .current
                .initial_measurement_matches,
            5
        );
        assert_eq!(records.records[0].result.season_max.value, 1550);
        assert_eq!(records.records[0].result.peak.tier, "Diamond");
    }

    #[test]
    fn current_csr_season_requires_time_inside_date_range() {
        let calendar: CsrSeasonCalendar = serde_json::from_value(serde_json::json!({
            "Seasons": [{
                "CsrSeasonFilePath": "past.json",
                "StartDate": { "ISO8601Date": "2026-01-01T00:00:00Z" },
                "EndDate": { "ISO8601Date": "2026-02-01T00:00:00Z" }
            }, {
                "CsrSeasonFilePath": "overlapping-older.json",
                "StartDate": { "ISO8601Date": "2026-02-20T00:00:00Z" },
                "EndDate": { "ISO8601Date": "2026-04-01T00:00:00Z" }
            }, {
                "CsrSeasonFilePath": "current.json",
                "StartDate": { "ISO8601Date": "2026-03-01T00:00:00Z" },
                "EndDate": { "ISO8601Date": "2026-04-01T00:00:00Z" }
            }]
        }))
        .unwrap();

        let during = "2026-03-15T00:00:00Z".parse().unwrap();
        let gap = "2026-02-15T00:00:00Z".parse().unwrap();
        let at_end = "2026-04-01T00:00:00Z".parse().unwrap();

        assert_eq!(
            calendar.current(during).unwrap().csr_season_file_path,
            "current.json"
        );
        assert!(calendar.current(gap).is_none());
        assert!(calendar.current(at_end).is_none());
    }
}
