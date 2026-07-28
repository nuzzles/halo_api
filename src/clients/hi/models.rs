use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct MatchesPrivacy {
    #[serde(rename = "MatchmadeGames")]
    pub matchmade_games: i32,
    #[serde(rename = "OtherGames")]
    pub other_games: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacySetting {
    Show,
    Hide,
    Unknown(i32),
}

impl PrivacySetting {
    pub fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Show,
            2 => Self::Hide,
            other => Self::Unknown(other),
        }
    }
}

impl MatchesPrivacy {
    pub fn matchmade_setting(&self) -> PrivacySetting {
        PrivacySetting::from_code(self.matchmade_games)
    }

    pub fn other_setting(&self) -> PrivacySetting {
        PrivacySetting::from_code(self.other_games)
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
    #[serde(rename = "LastTeamId")]
    pub last_team_id: i32,
    #[serde(rename = "Outcome")]
    pub outcome: i32,
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(rename = "PresentAtEndOfMatch")]
    pub present_at_end: bool,
    #[serde(rename = "MatchInfo")]
    pub info: MatchInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchInfo {
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "EndTime")]
    pub end_time: DateTime<Utc>,
    #[serde(rename = "Duration")]
    pub duration: String,
    #[serde(rename = "GameVariantCategory")]
    pub game_variant_category: i32,
    #[serde(rename = "MapVariant")]
    pub map_variant: Option<MatchAssetLink>,
    #[serde(rename = "UgcGameVariant")]
    pub ugc_game_variant: Option<MatchAssetLink>,
    #[serde(rename = "Playlist")]
    pub playlist: Option<MatchPlaylist>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchPlaylist {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchAssetLink {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
    #[serde(rename = "VersionId")]
    pub version_id: String,
    #[serde(rename = "AssetKind")]
    pub asset_kind: i32,
}

/// Detailed scoreboard returned for one match.
#[derive(Debug, Clone, Deserialize)]
pub struct MatchStats {
    #[serde(rename = "MatchId")]
    pub match_id: String,
    #[serde(rename = "MatchInfo")]
    pub info: MatchInfo,
    #[serde(rename = "Players")]
    pub players: Vec<MatchPlayer>,
    #[serde(rename = "Teams")]
    pub teams: Vec<MatchTeam>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchPlayer {
    #[serde(rename = "PlayerId")]
    pub player_id: String,
    #[serde(rename = "PlayerType")]
    pub player_type: i32,
    #[serde(rename = "LastTeamId")]
    pub last_team_id: i32,
    #[serde(rename = "Outcome")]
    pub outcome: i32,
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(rename = "PlayerTeamStats")]
    pub team_stats: Vec<PlayerTeamStats>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerTeamStats {
    #[serde(rename = "TeamId")]
    pub team_id: i32,
    #[serde(rename = "Stats")]
    pub stats: MatchStatsBlock,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchTeam {
    #[serde(rename = "TeamId")]
    pub team_id: i32,
    #[serde(rename = "Outcome")]
    pub outcome: i32,
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(rename = "Stats")]
    pub stats: MatchStatsBlock,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchStatsBlock {
    #[serde(rename = "CoreStats")]
    pub core: MatchCoreStats,
    #[serde(flatten)]
    pub mode_stats: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MatchCoreStats {
    #[serde(rename = "Score")]
    pub score: i32,
    #[serde(rename = "PersonalScore")]
    pub personal_score: i32,
    #[serde(rename = "Kills")]
    pub kills: i32,
    #[serde(rename = "Deaths")]
    pub deaths: i32,
    #[serde(rename = "Assists")]
    pub assists: i32,
    #[serde(rename = "KDA")]
    pub kda: f64,
    #[serde(rename = "Accuracy")]
    pub accuracy: f64,
    #[serde(rename = "DamageDealt")]
    pub damage_dealt: i64,
    #[serde(rename = "DamageTaken")]
    pub damage_taken: i64,
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
    pub matches_completed: i64,
    #[serde(rename = "Wins")]
    pub wins: i64,
    #[serde(rename = "Losses")]
    pub losses: i64,
    #[serde(rename = "Ties")]
    pub ties: i64,
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
    #[serde(
        default,
        rename = "SeasonIds",
        deserialize_with = "deserialize_null_default"
    )]
    pub season_ids: Vec<String>,
    #[serde(
        default,
        rename = "GameVariantCategories",
        deserialize_with = "deserialize_null_default"
    )]
    pub game_variant_categories: Vec<i32>,
    #[serde(
        default,
        rename = "IsRanked",
        deserialize_with = "deserialize_null_default"
    )]
    pub is_ranked: Vec<bool>,
    #[serde(
        default,
        rename = "PlaylistAssetIds",
        deserialize_with = "deserialize_null_default"
    )]
    pub playlist_asset_ids: Vec<String>,
    #[serde(rename = "GameplayInteractions")]
    pub gameplay_interactions: serde_json::Value,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CoreStats {
    #[serde(rename = "Score")]
    pub score: i64,
    #[serde(rename = "PersonalScore")]
    pub personal_score: i64,
    #[serde(rename = "RoundsWon")]
    pub rounds_won: i64,
    #[serde(rename = "RoundsLost")]
    pub rounds_lost: i64,
    #[serde(rename = "RoundsTied")]
    pub rounds_tied: i64,
    #[serde(rename = "Kills")]
    pub kills: i64,
    #[serde(rename = "Deaths")]
    pub deaths: i64,
    #[serde(rename = "Assists")]
    pub assists: i64,
    #[serde(rename = "AverageKDA")]
    pub kda: f64,
    #[serde(rename = "Suicides")]
    pub suicides: i64,
    #[serde(rename = "Betrayals")]
    pub betrayals: i64,
    #[serde(rename = "GrenadeKills")]
    pub grenade_kills: i64,
    #[serde(rename = "HeadshotKills")]
    pub headshot_kills: i64,
    #[serde(rename = "MeleeKills")]
    pub melee_kills: i64,
    #[serde(rename = "PowerWeaponKills")]
    pub power_weapon_kills: i64,
    #[serde(rename = "ShotsFired")]
    pub shots_fired: i64,
    #[serde(rename = "ShotsHit")]
    pub shots_hit: i64,
    #[serde(rename = "Accuracy")]
    pub accuracy: f64,
    #[serde(rename = "DamageDealt")]
    pub damage_dealt: i64,
    #[serde(rename = "DamageTaken")]
    pub damage_taken: i64,
    #[serde(rename = "CalloutAssists")]
    pub callout_assists: i64,
    #[serde(rename = "VehicleDestroys")]
    pub vehicle_destroys: i64,
    #[serde(rename = "DriverAssists")]
    pub driver_assists: i64,
    #[serde(rename = "Hijacks")]
    pub hijacks: i64,
    #[serde(rename = "EmpAssists")]
    pub emp_assists: i64,
    #[serde(rename = "MaxKillingSpree")]
    pub max_killing_spree: i64,
    #[serde(rename = "Medals")]
    pub medals: Vec<StatAward>,
    #[serde(rename = "PersonalScores")]
    pub personal_scores: Vec<StatAward>,
    #[serde(rename = "Spawns")]
    pub spawns: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatAward {
    #[serde(rename = "NameId")]
    pub name_id: i64,
    #[serde(rename = "Count")]
    pub count: i64,
    #[serde(rename = "TotalPersonalScoreAwarded")]
    pub total_personal_score_awarded: i64,
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
pub struct AppearanceCustomization {
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Appearance")]
    pub appearance: PlayerAppearance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerCustomizationCollection {
    #[serde(rename = "PlayerCustomizations")]
    pub player_customizations: Vec<PlayerCustomization>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerCustomization {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: String,
    #[serde(rename = "Result")]
    pub result: PlayerCustomizationData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerCustomizationData {
    #[serde(rename = "Appearance")]
    pub appearance: PlayerAppearance,
    #[serde(flatten)]
    pub other_customization: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerAppearance {
    #[serde(rename = "LastModifiedDateUtc")]
    pub last_modified: Option<ApiDate>,
    #[serde(rename = "ActionPosePath")]
    pub action_pose_path: Option<String>,
    #[serde(rename = "StancePath")]
    pub stance_path: Option<String>,
    #[serde(rename = "BackdropImagePath")]
    pub backdrop_image_path: Option<String>,
    #[serde(rename = "Emblem")]
    pub emblem: Option<EmblemConfiguration>,
    #[serde(rename = "ServiceTag")]
    pub service_tag: String,
    #[serde(rename = "IntroEmotePath")]
    pub intro_emote_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmblemConfiguration {
    #[serde(rename = "EmblemPath")]
    pub emblem_path: String,
    #[serde(rename = "ConfigurationId")]
    pub configuration_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanSummary {
    #[serde(rename = "Results")]
    pub results: Vec<BanSummaryResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanSummaryResult {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "Result")]
    pub result: BanResult,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanResult {
    #[serde(rename = "BansInEffect")]
    pub bans_in_effect: Vec<BanInEffect>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanInEffect {
    #[serde(rename = "Type")]
    pub ban_type: i32,
    #[serde(rename = "Scope")]
    pub scope: i32,
    #[serde(rename = "EnforceUntilUtc")]
    pub enforce_until: ApiDate,
    #[serde(rename = "BanMessagePath")]
    pub message_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanMessage {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "DisplayMessage")]
    pub display_message: LocalizedText,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalizedText {
    pub status: String,
    pub value: String,
    pub translations: std::collections::BTreeMap<String, String>,
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

    #[test]
    fn deserializes_discord_bot_response_contracts() {
        let privacy: MatchesPrivacy = serde_json::from_value(serde_json::json!({
            "MatchmadeGames": 1,
            "OtherGames": 2
        }))
        .unwrap();
        assert_eq!(privacy.matchmade_setting(), PrivacySetting::Show);
        assert_eq!(privacy.other_setting(), PrivacySetting::Hide);

        let service_record: ServiceRecord = serde_json::from_value(serde_json::json!({
            "MatchesCompleted": 3_000_000_000_i64,
            "CoreStats": {
                "Score": 4_000_000_000_i64,
                "DamageDealt": 5_000_000_000_i64
            }
        }))
        .unwrap();
        assert_eq!(service_record.matches_completed, 3_000_000_000);
        assert_eq!(service_record.core_stats.damage_dealt, 5_000_000_000);

        let empty_record: ServiceRecord = serde_json::from_value(serde_json::json!({
            "Subqueries": {
                "SeasonIds": null,
                "GameVariantCategories": null,
                "IsRanked": null,
                "PlaylistAssetIds": null,
                "GameplayInteractions": null
            },
            "CoreStats": { "Medals": [], "PersonalScores": [] }
        }))
        .unwrap();
        assert!(empty_record.subqueries.season_ids.is_empty());
        assert!(empty_record.subqueries.playlist_asset_ids.is_empty());

        let appearance: AppearanceCustomization = serde_json::from_value(serde_json::json!({
            "Status": "Success",
            "Appearance": {
                "LastModifiedDateUtc": { "ISO8601Date": "2026-01-01T00:00:00Z" },
                "ActionPosePath": "pose.json",
                "StancePath": null,
                "BackdropImagePath": "backdrop.json",
                "Emblem": { "EmblemPath": "emblem.json", "ConfigurationId": 42 },
                "ServiceTag": "117",
                "IntroEmotePath": null
            }
        }))
        .unwrap();
        assert_eq!(appearance.appearance.service_tag, "117");

        let public: PlayerCustomizationCollection = serde_json::from_value(serde_json::json!({
            "PlayerCustomizations": [{
                "Id": "xuid(123)",
                "ResultCode": "Success",
                "Result": {
                    "Appearance": {
                        "LastModifiedDateUtc": null,
                        "ActionPosePath": "pose.json",
                        "StancePath": null,
                        "BackdropImagePath": "backdrop.json",
                        "Emblem": null,
                        "ServiceTag": "117",
                        "IntroEmotePath": null
                    },
                    "ArmorCores": {}
                }
            }]
        }))
        .unwrap();
        assert_eq!(public.player_customizations[0].result_code, "Success");

        let bans: BanSummary = serde_json::from_value(serde_json::json!({
            "Results": [{
                "Id": "xuid(123)",
                "ResultCode": 0,
                "Result": { "BansInEffect": [{
                    "Type": 1,
                    "Scope": 1,
                    "EnforceUntilUtc": { "ISO8601Date": "2026-08-01T00:00:00Z" },
                    "BanMessagePath": "Banning/example.json"
                }] }
            }]
        }))
        .unwrap();
        assert_eq!(bans.results[0].result.bans_in_effect.len(), 1);

        let message: BanMessage = serde_json::from_value(serde_json::json!({
            "Title": "HI: Admin Matchmaking Ban Cheating",
            "DisplayMessage": {
                "status": "Ready",
                "value": "Suspended until {0}",
                "translations": { "de-DE": "Gesperrt bis {0}" }
            }
        }))
        .unwrap();
        assert_eq!(message.display_message.translations.len(), 1);

        let scoreboard: MatchStats = serde_json::from_value(serde_json::json!({
            "MatchId": "match",
            "MatchInfo": {
                "StartTime": "2026-01-01T00:00:00Z",
                "EndTime": "2026-01-01T00:10:00Z",
                "Duration": "PT10M",
                "GameVariantCategory": 6,
                "MapVariant": { "AssetKind": 2, "AssetId": "map", "VersionId": "v1" },
                "UgcGameVariant": { "AssetKind": 6, "AssetId": "mode", "VersionId": "v2" },
                "Playlist": { "AssetId": "playlist" }
            },
            "Teams": [],
            "Players": [{
                "PlayerId": "xuid(123)",
                "PlayerType": 1,
                "LastTeamId": 0,
                "Outcome": 2,
                "Rank": 1,
                "PlayerTeamStats": [{
                    "TeamId": 0,
                    "Stats": { "CoreStats": {
                        "PersonalScore": 2500, "Kills": 20, "Deaths": 10, "Assists": 5
                    }}
                }]
            }]
        }))
        .unwrap();
        assert_eq!(scoreboard.players[0].team_stats[0].stats.core.kills, 20);
    }
}
