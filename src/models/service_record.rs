use serde::Deserialize;

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
