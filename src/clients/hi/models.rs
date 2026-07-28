use std::{borrow::Cow, collections::BTreeMap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
}

/// A Halo Infinite matchmaking playlist asset ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaylistId(Cow<'static, str>);

impl PlaylistId {
    pub const RANKED_ARENA: Self = Self(Cow::Borrowed("edfef3ac-9cbe-4fa2-b949-8f29deafd483"));
    pub const RANKED_DOUBLES: Self = Self(Cow::Borrowed("fa5aa2a3-2428-4912-a023-e1eeea7b877c"));
    pub const RANKED_SLAYER: Self = Self(Cow::Borrowed("dcb2e24e-05fb-4390-8076-32a0cdb4326e"));

    /// Creates an ID for a playlist that is not included among the named constants.
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PlaylistId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A Halo Infinite UGC game-mode asset ID paired with an immutable version ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameModeId {
    asset_id: Cow<'static, str>,
    version_id: Cow<'static, str>,
}

impl GameModeId {
    pub const FIESTA_SLAYER: Self = Self::from_static(
        "aca7bbf8-7a18-4aae-8785-1bd3f58275fd",
        "3685f6b2-2860-4e98-9d13-513087edb465",
    );
    pub const RANKED_ONE_FLAG_CTF: Self = Self::from_static(
        "18ac247d-7f86-4a59-9b47-9e74a6384ac2",
        "6dbe8411-cc9b-44ca-b680-32847677536a",
    );
    pub const RANKED_STRONGHOLDS: Self = Self::from_static(
        "22b8a0eb-0d02-4eb3-8f56-5f63fc254f83",
        "7a6d2582-284c-4728-bec9-118e32cd0cc0",
    );
    pub const RANKED_ODDBALL: Self = Self::from_static(
        "751bcc9d-aace-45a1-8d71-358f0bc89f7e",
        "227d4ffc-d67f-449a-8315-a1f82854d2ed",
    );
    pub const RANKED_SLAYER: Self = Self::from_static(
        "c2d20d44-8606-4669-b894-afae15b3524f",
        "0091d411-f90d-44a7-aac3-ccc7ff2b131f",
    );
    pub const RANKED_KING_OF_THE_HILL: Self = Self::from_static(
        "88c22b1f-2d64-48b9-bab1-26fe4721fb23",
        "43e75f3a-eee5-4147-b9d3-19782fac58f8",
    );
    pub const RANKED_CTF_3_CAPTURES: Self = Self::from_static(
        "4cb279b7-a064-4df6-9058-02cdc6825d93",
        "1392d27e-e7e3-41d9-93f9-420c66cff577",
    );
    pub const RANKED_CTF_5_CAPTURES: Self = Self::from_static(
        "507191c6-a492-4331-b2ae-a172101eb23e",
        "ffd0ef4e-da75-42b0-93bb-ab44d4e6905b",
    );
    pub const RANKED_ATTRITION: Self = Self::from_static(
        "0bc630bf-2ee3-4eae-b272-b68d4ab80be7",
        "b6b22432-f3d9-468c-9359-b82a72791030",
    );
    pub const RANKED_DOUBLES_ODDBALL: Self = Self::from_static(
        "9beb95dc-9fa2-4c6e-889f-d717f2adfe49",
        "75c45183-df50-405c-8fbc-bccc0f7eb375",
    );
    pub const RANKED_DOUBLES_SLAYER: Self = Self::from_static(
        "b0c65df9-0b2c-4040-b018-ad3e1baab832",
        "9e8f9dae-007d-4eb4-a131-4e5d526d9130",
    );

    /// Creates a game-mode ID from an asset GUID and immutable version GUID.
    pub fn new(asset_id: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            asset_id: Cow::Owned(asset_id.into()),
            version_id: Cow::Owned(version_id.into()),
        }
    }

    const fn from_static(asset_id: &'static str, version_id: &'static str) -> Self {
        Self {
            asset_id: Cow::Borrowed(asset_id),
            version_id: Cow::Borrowed(version_id),
        }
    }

    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

/// A Halo Infinite map asset ID paired with an immutable version ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapId {
    asset_id: Cow<'static, str>,
    version_id: Cow<'static, str>,
}

impl MapId {
    pub const FORGE_SPACE: Self = Self::from_static(
        "76669255-697d-48c9-a802-7ff2ec8257f1",
        "b8abf687-4e95-4846-83c7-33e779eed33e",
    );
    pub const FRAGMENTATION: Self = Self::from_static(
        "4f196016-0101-4844-8358-2504f7c44656",
        "645e4e35-a573-4362-a59c-1c7867622891",
    );
    pub const STREETS: Self = Self::from_static(
        "f0a1760f-0d4a-4bcc-ac7a-e8f9aee331dc",
        "7cfb2ea5-2f69-4ed5-9d54-73988bebe8d7",
    );
    pub const DEADLOCK: Self = Self::from_static(
        "08607bf4-6abe-4a5b-9547-290a6cc1433e",
        "866fbb48-fc87-459e-a3fe-a69d764c0256",
    );
    pub const HIGHPOWER: Self = Self::from_static(
        "c494ef7c-d203-42a9-9c0f-b3f576334501",
        "aeedf79f-ae5a-4de6-bb29-b14f66baf64b",
    );
    pub const LAUNCH_SITE: Self = Self::from_static(
        "56a11b8c-64d1-4537-8893-a9241e4d5b93",
        "1cd21d5a-a57e-4d42-997d-ff95ca0e32fc",
    );
    pub const LIVE_FIRE: Self = Self::from_static(
        "b6aca0c7-8ba7-4066-bf91-693571374c3c",
        "67bf316f-e891-4e85-8f3d-b129ef5fcb2e",
    );
    pub const BEHEMOTH: Self = Self::from_static(
        "53136ad9-0fd6-4271-8752-31d114b9561e",
        "d79b7333-90bf-4cbf-8e44-3b39da651202",
    );
    pub const BAZAAR: Self = Self::from_static(
        "298d5036-cd43-47b3-a4bd-31e127566593",
        "5546a6ec-841d-4955-be7a-5f32c3ac0428",
    );
    pub const AQUARIUS: Self = Self::from_static(
        "33c0766c-ef15-48f8-b298-34aba5bff3b4",
        "711c83cf-c952-46cf-80fa-57e62af2bd38",
    );
    pub const RECHARGE: Self = Self::from_static(
        "8420410b-044d-44d7-80b6-98a766c8c39f",
        "3195263c-ef0f-49da-99a1-54839a1a64a0",
    );
    pub const BREAKER: Self = Self::from_static(
        "e6cbfe01-665b-4a8c-bf3a-d63a65a7c890",
        "635c65fe-d207-47e5-b30d-dce1ec680c51",
    );
    pub const CATALYST: Self = Self::from_static(
        "e859cf75-9b8a-429a-91be-2376681c8537",
        "463a6db1-b0a7-4477-b20b-0c51a7916d4f",
    );
    pub const CHASM: Self = Self::from_static(
        "fc1ced39-128b-439d-9b44-4710225090f3",
        "e03dfd3a-804f-48d0-a1f2-e5963f036dbe",
    );
    pub const CLIFFHANGER: Self = Self::from_static(
        "81274d6f-6a94-425a-a16e-3bdb1e2eea9d",
        "2cbfa179-2bd2-499d-a5bd-74bf2d14d05b",
    );
    pub const FORBIDDEN: Self = Self::from_static(
        "ea51a3dd-2125-4e5b-872d-25f1835edd29",
        "dd6b48a2-cdb7-4197-af60-706f4ca10f69",
    );
    pub const FOREST: Self = Self::from_static(
        "619bea21-f1e6-461f-8a7d-2bb4f905d0ca",
        "ed9e777f-4c8a-441c-b970-e66cc5c2dd9a",
    );
    pub const HOUSE_OF_RECKONING: Self = Self::from_static(
        "eaf608f0-6ec3-444f-a51a-9c1de5d0bf5c",
        "681d9ead-df2c-45d6-a828-a7d9e2e582cd",
    );
    pub const ILLUSION: Self = Self::from_static(
        "86ef3b1c-2f39-4c29-8c19-65ab84a704c2",
        "db6a631d-1465-4b86-b970-78d800e5cfd3",
    );
    pub const OASIS: Self = Self::from_static(
        "6aa0a116-66a6-4242-a1b3-41aa417d6dc6",
        "32465034-7f98-4dea-b178-8058d3d39c8e",
    );
    pub const PRISM: Self = Self::from_static(
        "92d23264-d3b9-462e-adbc-8ddb44e81966",
        "c9a6a0ef-03b6-46d7-b0c0-6e18d105f0de",
    );
    pub const SCARR: Self = Self::from_static(
        "247637f8-1ed2-47de-8ff0-fd4b68f50888",
        "b1bbed19-ba61-4cfd-8b1e-14a986cf75c1",
    );

    /// Creates a map ID from an asset GUID and immutable version GUID.
    pub fn new(asset_id: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            asset_id: Cow::Owned(asset_id.into()),
            version_id: Cow::Owned(version_id.into()),
        }
    }

    const fn from_static(asset_id: &'static str, version_id: &'static str) -> Self {
        Self {
            asset_id: Cow::Borrowed(asset_id),
            version_id: Cow::Borrowed(version_id),
        }
    }

    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UgcAssetKind {
    Map,
    Playlist,
    GameMode,
}

impl UgcAssetKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Playlist => "Playlist",
            Self::GameMode => "UgcGameVariant",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UgcSearchResults {
    #[serde(rename = "EstimatedTotal")]
    pub estimated_total: u32,
    #[serde(rename = "Start")]
    pub start: u32,
    #[serde(rename = "ResultCount")]
    pub result_count: u32,
    #[serde(rename = "Results")]
    pub results: Vec<UgcSearchResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UgcSearchResult {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
    #[serde(rename = "AssetVersionId")]
    pub version_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "AssetKind")]
    pub asset_kind: i32,
    #[serde(rename = "Tags")]
    pub tags: Vec<String>,
    /// The catalog that owns the asset. Halo-owned assets use home `2`.
    #[serde(default, rename = "AssetHome")]
    pub asset_home: Option<i32>,
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

/// Waypoint image assets indexed by emblem identifier and configuration ID.
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct EmblemMapping {
    pub emblems: BTreeMap<String, BTreeMap<i64, EmblemImageAssets>>,
}

impl EmblemMapping {
    pub fn get(&self, emblem_id: &str, configuration_id: i64) -> Option<&EmblemImageAssets> {
        self.emblems.get(emblem_id)?.get(&configuration_id)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmblemImageAssets {
    pub emblem_cms_path: String,
    pub nameplate_cms_path: String,
    pub text_color: String,
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
    #[serde(rename = "Files")]
    pub files: Option<AssetFiles>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetFiles {
    #[serde(rename = "Prefix")]
    pub prefix: String,
    #[serde(rename = "FileRelativePaths")]
    pub relative_paths: Vec<String>,
}

impl AssetFiles {
    pub fn url(&self, relative_path: &str) -> String {
        format!(
            "{}/{}",
            self.prefix.trim_end_matches('/'),
            relative_path.trim_start_matches('/')
        )
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.relative_paths
            .iter()
            .filter(|path| {
                let path = path.to_ascii_lowercase();
                path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg")
            })
            .map(|path| self.url(path))
    }

    fn named_image_url(&self, file_name: &str) -> Option<String> {
        self.relative_paths
            .iter()
            .find(|path| {
                path.rsplit('/')
                    .next()
                    .is_some_and(|name| name.eq_ignore_ascii_case(file_name))
            })
            .map(|path| self.url(path))
    }

    fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.relative_paths
            .iter()
            .filter(|path| {
                let Some(name) = path.rsplit('/').next() else {
                    return false;
                };
                let name = name.to_ascii_lowercase();
                name.starts_with("screenshot")
                    && (name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg"))
            })
            .map(|path| self.url(path))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "RotationEntries")]
    pub rotation_entries: Vec<RotationEntry>,
}

impl PlaylistAsset {
    pub fn hero_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("hero.png"))
    }

    pub fn thumbnail_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("thumbnail.png"))
    }

    pub fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset
            .files
            .iter()
            .flat_map(AssetFiles::screenshot_urls)
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset.files.iter().flat_map(AssetFiles::image_urls)
    }
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

impl MapAsset {
    pub fn hero_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("hero.png"))
    }

    pub fn thumbnail_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("thumbnail.png"))
    }

    pub fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset
            .files
            .iter()
            .flat_map(AssetFiles::screenshot_urls)
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset.files.iter().flat_map(AssetFiles::image_urls)
    }
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

impl GameVariantAsset {
    pub fn hero_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("hero.png"))
    }

    pub fn thumbnail_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("thumbnail.png"))
    }

    pub fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset
            .files
            .iter()
            .flat_map(AssetFiles::screenshot_urls)
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset.files.iter().flat_map(AssetFiles::image_urls)
    }
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

    #[test]
    fn asset_ids_support_named_and_custom_values() {
        assert_eq!(PlaylistId::RANKED_ARENA.as_str().len(), 36);
        assert_eq!(PlaylistId::new("custom").as_str(), "custom");
        assert_eq!(GameModeId::RANKED_SLAYER.asset_id().len(), 36);
        assert_eq!(GameModeId::new("asset", "version").version_id(), "version");
        assert_eq!(MapId::LIVE_FIRE.asset_id().len(), 36);
        assert_eq!(MapId::new("asset", "version").version_id(), "version");
    }

    #[test]
    fn map_exposes_ugc_image_urls() {
        let map: MapAsset = serde_json::from_value(serde_json::json!({
            "AssetId": "asset",
            "VersionId": "version",
            "PublicName": "Live Fire",
            "Description": "",
            "Files": {
                "Prefix": "https://cdn.example/map/",
                "FileRelativePaths": [
                    "map.mvar",
                    "images/hero.png",
                    "images/screenshot.jpg"
                ]
            },
            "CustomData": {
                "NumOfObjectsOnMap": 0,
                "TagLevelId": 0,
                "IsBaked": true,
                "HasNodeGraph": false
            }
        }))
        .unwrap();

        assert_eq!(
            map.image_urls().collect::<Vec<_>>(),
            [
                "https://cdn.example/map/images/hero.png",
                "https://cdn.example/map/images/screenshot.jpg"
            ]
        );
        assert_eq!(
            map.hero_url().as_deref(),
            Some("https://cdn.example/map/images/hero.png")
        );
        assert!(map.thumbnail_url().is_none());
        assert_eq!(
            map.screenshot_urls().collect::<Vec<_>>(),
            ["https://cdn.example/map/images/screenshot.jpg"]
        );
    }

    #[test]
    fn game_mode_exposes_ugc_image_urls() {
        let mode: GameVariantAsset = serde_json::from_value(serde_json::json!({
            "AssetId": "asset",
            "VersionId": "version",
            "PublicName": "Ranked:Slayer",
            "Description": "",
            "Files": {
                "Prefix": "https://cdn.example/mode/",
                "FileRelativePaths": [
                    "images/hero.png",
                    "images/thumbnail.png",
                    "images/screenshot1.Png"
                ]
            },
            "CustomData": {
                "KeyValues": {},
                "HasNodeGraph": false
            }
        }))
        .unwrap();

        assert_eq!(
            mode.hero_url().as_deref(),
            Some("https://cdn.example/mode/images/hero.png")
        );
        assert_eq!(
            mode.thumbnail_url().as_deref(),
            Some("https://cdn.example/mode/images/thumbnail.png")
        );
        assert_eq!(
            mode.screenshot_urls().collect::<Vec<_>>(),
            ["https://cdn.example/mode/images/screenshot1.Png"]
        );
    }

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

        let emblem_mapping: EmblemMapping = serde_json::from_value(serde_json::json!({
            "104-001-reach-wrath-e-37d15c60": {
                "-1490538315": {
                    "emblemCmsPath": "images/emblems/wrath.png",
                    "nameplateCmsPath": "images/nameplates/wrath.png",
                    "textColor": "#000000"
                }
            }
        }))
        .unwrap();
        let emblem = emblem_mapping
            .get("104-001-reach-wrath-e-37d15c60", -1_490_538_315)
            .unwrap();
        assert_eq!(emblem.emblem_cms_path, "images/emblems/wrath.png");
        assert_eq!(emblem.text_color, "#000000");

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
