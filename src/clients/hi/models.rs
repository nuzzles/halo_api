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
    #[serde(rename = "Result")]
    pub result: CsrRecordResult,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecordResult {
    #[serde(rename = "Current")]
    pub current: CsrRecordRanking,
    #[serde(rename = "AllTimeMax")]
    pub peak: CsrRecordRanking,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecordRanking {
    /// Numeric CSR value, or `-1` if the player is unranked in this playlist.
    #[serde(rename = "Value")]
    pub value: i32,
    #[serde(rename = "Tier")]
    pub tier: String,
    /// 0-indexed sub-tier within `tier`. Not meaningful for Onyx, which reports `value`.
    #[serde(rename = "SubTier")]
    pub sub_tier: i32,
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
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceRecord {
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoreStats {
    #[serde(rename = "Kills")]
    pub kills: i32,
    #[serde(rename = "Deaths")]
    pub deaths: i32,
    #[serde(rename = "Assists")]
    pub assists: i32,
    #[serde(rename = "AverageKDA")]
    pub kda: f32,
    #[serde(rename = "Accuracy")]
    pub accuracy: f32,
    #[serde(rename = "DamageDealt")]
    pub damage_dealt: i32,
    #[serde(rename = "DamageTaken")]
    pub damage_taken: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranking(value: i32) -> CsrRecordRanking {
        CsrRecordRanking {
            value,
            tier: "Platinum".to_string(),
            sub_tier: 2,
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
                    "Current": { "Value": 1500, "Tier": "Platinum", "SubTier": 2 },
                    "AllTimeMax": { "Value": 1600, "Tier": "Diamond", "SubTier": 0 },
                }
            }]
        });
        let records: CsrRecords = serde_json::from_value(json).unwrap();
        assert_eq!(records.records.len(), 1);
        assert_eq!(records.records[0].result.current.value, 1500);
        assert_eq!(records.records[0].result.peak.tier, "Diamond");
    }
}
