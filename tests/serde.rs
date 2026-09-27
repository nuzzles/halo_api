#![cfg(feature = "serde")]

use halo_api::clients::hi::models::{MatchOutcome, PlayerMatchHistory};
use serde_json::json;

#[test]
fn match_history_preserves_nested_metadata_and_wire_types() {
    let history: PlayerMatchHistory = serde_json::from_value(json!({
        "ResultCount": 1,
        "Results": [{
            "MatchId": "match-id", "LastTeamId": 1, "Outcome": 2,
            "Rank": 3, "PresentAtEndOfMatch": true,
            "MatchInfo": {
                "StartTime": "2026-09-01T12:00:00Z", "EndTime": "2026-09-01T12:10:00Z",
                "Duration": "PT10M", "GameVariantCategory": 1,
                "MapVariant": { "AssetId": "map", "VersionId": "version", "AssetKind": 2 },
                "Playlist": { "AssetId": "playlist" }, "TeamsEnabled": true
            }
        }]
    }))
    .unwrap();
    let encoded = serde_json::to_value(&history).unwrap();
    let entry = &encoded["Results"][0];
    assert_eq!(entry["Outcome"], 2);
    assert_eq!(entry["PresentAtEndOfMatch"], true);
    assert_eq!(entry["MatchInfo"]["StartTime"], "2026-09-01T12:00:00Z");
    assert_eq!(entry["MatchInfo"]["MapVariant"]["AssetId"], "map");
    assert_eq!(entry["MatchInfo"]["Playlist"]["AssetId"], "playlist");
    let decoded: PlayerMatchHistory = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
}

#[test]
fn outcomes_round_trip_as_numeric_codes_including_unknown_values() {
    for code in [1, 2, 3, 4, 0, -1, 99, i32::MAX] {
        let outcome: MatchOutcome = serde_json::from_value(json!(code)).unwrap();
        assert_eq!(serde_json::to_value(outcome).unwrap(), json!(code));
        assert_eq!(outcome.code(), code);
    }
}

#[test]
fn flattened_mode_stats_keep_halo_field_names() {
    use halo_api::clients::hi::models::MatchStatsBlock;
    let stats: MatchStatsBlock = serde_json::from_value(json!({
        "CoreStats": { "Kills": 7 }, "CaptureTheFlagStats": {}
    }))
    .unwrap();
    let encoded = serde_json::to_value(&stats).unwrap();
    assert_eq!(encoded["CoreStats"]["Kills"], 7);
    assert!(encoded["CaptureTheFlagStats"].is_object());
    assert!(encoded.get("mode_stats").is_none());
    let decoded: MatchStatsBlock = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
}
