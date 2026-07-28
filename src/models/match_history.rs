use chrono::{DateTime, Utc};
use serde::Deserialize;

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
