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
    /// 0-indexed sub-tier within `tier` (e.g. "Platinum 1" is `sub_tier == 0`). Not meaningful
    /// for the Onyx tier, which reports `value` directly instead.
    #[serde(rename = "SubTier")]
    pub sub_tier: i32,
}

impl CsrRecordRanking {
    /// Whether this ranking represents "unranked" (the API's `-1` sentinel), as opposed to a
    /// real CSR value.
    pub fn is_unranked(&self) -> bool {
        self.value == -1
    }
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
    fn deserializes_full_response_shape() {
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
