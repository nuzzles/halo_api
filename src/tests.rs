use std::io::Write;
use std::sync::Arc;

use crate::auth::endpoints::AuthEndpoints;
use crate::auth::{AuthError, ClearanceTokenSource, HaloAuthClient, SpartanTokenSource};
use crate::clients::hi::endpoints::HaloEndpoints;
use crate::clients::hi::models::{
    CustomizationItemMetadata, EmblemImageAssets, MatchHistoryType, MatchType, PlaylistId,
    ServiceRecordFilter,
};
use crate::clients::hi::{HaloInfiniteClient, InfiniteClientError, Player};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xbox::auth::XblAuthProvider;
use xbox::cache::CachedToken;
use xbox::{XboxClient, XboxEndpoints, XboxError};

/// Hands out a fixed, never-expiring user token without touching the network.
struct FakeAuthProvider;

struct FailingSpartanTokenSource;
struct UnusedClearanceTokenSource;

#[async_trait]
impl SpartanTokenSource for FailingSpartanTokenSource {
    async fn spartan_token(&self) -> Result<CachedToken<String>, AuthError> {
        Err(AuthError::SpartanTokenProvider(
            "Xbox login failed".to_string(),
        ))
    }
}

#[async_trait]
impl ClearanceTokenSource for UnusedClearanceTokenSource {
    async fn clearance_token(
        &self,
        _spartan_token: &str,
    ) -> Result<CachedToken<String>, AuthError> {
        unreachable!("clearance is not requested when Spartan-token acquisition fails")
    }
}

#[async_trait]
impl XblAuthProvider for FakeAuthProvider {
    async fn user_token(&self) -> Result<CachedToken<String>, XboxError> {
        Ok(CachedToken::new(
            "fake-user-token".to_string(),
            Utc::now() + Duration::hours(1),
        ))
    }
}

fn xsts_body() -> serde_json::Value {
    serde_json::json!({
        "Token": "fake-xsts-token",
        "NotAfter": (Utc::now() + Duration::hours(1)).to_rfc3339(),
        "DisplayClaims": { "xui": [{ "uhs": "fake-uhs" }] }
    })
}

fn spartan_token_body(expires_at: chrono::DateTime<Utc>) -> serde_json::Value {
    serde_json::json!({
        "SpartanToken": "fake-spartan-token",
        "ExpiresUtc": { "ISO8601Date": expires_at.to_rfc3339() }
    })
}

fn csr_body(value: i32) -> serde_json::Value {
    serde_json::json!({
        "Value": [{
            "Id": "xuid(123456789)",
            "ResultCode": 0,
            "Result": {
                "Current": { "Value": value, "Tier": "Platinum", "SubTier": 2 },
                "AllTimeMax": { "Value": value, "Tier": "Platinum", "SubTier": 2 },
            }
        }]
    })
}

/// Sets up a mock server with XSTS + spartan-token mocks, and returns a `HaloInfiniteClient` wired to
/// it plus the shared `XboxClient` (for XUID resolution) — everything an end-to-end test needs.
async fn test_client(
    server: &MockServer,
) -> (HaloInfiniteClient, Arc<XboxClient<FakeAuthProvider>>) {
    Mock::given(method("POST"))
        .and(path("/xsts/authorize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(xsts_body()))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(
            "/oban/flight-configurations/titles/hi/audiences/retail/players/xuid(123456789)/active",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "FlightConfigurationId": "fake-clearance"
        })))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "xuid": "123456789",
            "notificationsReadDate": "2026-01-01T00:00:00Z"
        })))
        .mount(server)
        .await;

    Mock::given(method("POST"))
        .and(path("/spartan-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(spartan_token_body(Utc::now() + Duration::hours(1))),
        )
        .mount(server)
        .await;

    let xbox_client = Arc::new(XboxClient::with_endpoints(
        FakeAuthProvider,
        reqwest::Client::new(),
        XboxEndpoints {
            xsts_authorize_url: format!("{}/xsts/authorize", server.uri()),
            peoplehub_base_url: server.uri(),
        },
    ));

    let auth_endpoints = AuthEndpoints {
        spartan_token_url: format!("{}/spartan-token", server.uri()),
        clearance_url: format!(
            "{}/oban/flight-configurations/titles/hi/audiences/retail/players",
            server.uri()
        ),
        current_user_url: format!("{}/users/me", server.uri()),
    };
    let endpoints = HaloEndpoints {
        skill_base_url: server.uri(),
        halostats_base_url: server.uri(),
        current_user_url: format!("{}/users/me", server.uri()),
        profile_base_url: server.uri(),
        game_cms_base_url: server.uri(),
        ugc_base_url: server.uri(),
        settings_base_url: server.uri(),
        ban_base_url: server.uri(),
        economy_base_url: server.uri(),
    };
    let auth =
        HaloAuthClient::from_xbox_client_with_endpoints(xbox_client.clone(), &auth_endpoints);
    let halo_infinite_client = HaloInfiniteClient::with_endpoints(auth, endpoints);

    (halo_infinite_client, xbox_client)
}

#[tokio::test]
async fn gets_playlist_csr_end_to_end() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/hi/playlist/edfef3ac-9cbe-4fa2-b949-8f29deafd483/csrs",
        ))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(csr_body(1500)))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let csr = halo
        .playlist_csr(PlaylistId::RANKED_ARENA, &player)
        .await
        .unwrap();

    assert_eq!(csr.records.len(), 1);
    assert_eq!(csr.records[0].result.current.value, 1500);
}

#[tokio::test]
async fn supplied_halo_tokens_are_used_without_xbox_authentication() {
    let server = MockServer::start().await;
    let auth = HaloAuthClient::from_tokens("provided-spartan", "provided-clearance");
    let halo = HaloInfiniteClient::with_endpoints(
        auth,
        HaloEndpoints {
            skill_base_url: server.uri(),
            halostats_base_url: server.uri(),
            current_user_url: format!("{}/users/me", server.uri()),
            profile_base_url: server.uri(),
            game_cms_base_url: server.uri(),
            ugc_base_url: server.uri(),
            settings_base_url: server.uri(),
            ban_base_url: server.uri(),
            economy_base_url: server.uri(),
        },
    );
    Mock::given(method("GET"))
        .and(path(
            "/hi/playlist/edfef3ac-9cbe-4fa2-b949-8f29deafd483/csrs",
        ))
        .and(header("X-343-Authorization-Spartan", "provided-spartan"))
        .and(header("343-Clearance", "provided-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(csr_body(1500)))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let csr = halo
        .playlist_csr(PlaylistId::RANKED_ARENA, &player)
        .await
        .unwrap();

    assert_eq!(csr.records[0].result.current.value, 1500);
}

#[tokio::test]
async fn player_matches_uses_supported_query_parameters() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/matches"))
        .and(query_param("start", "0"))
        .and(query_param("count", "1"))
        .and(query_param("type", "all"))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "Results": [], "ResultCount": 0 })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let history = halo.player_matches(&player, 0, 1).await.unwrap();

    assert!(history.results.is_empty());
    assert_eq!(history.result_count, 0);
}

#[tokio::test]
async fn player_matches_of_type_sends_type_parameter() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/matches"))
        .and(query_param("type", "matchmaking"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "Results": [], "ResultCount": 0 })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    halo.player_matches_of_type(&player, 0, 25, MatchHistoryType::Matchmaking)
        .await
        .unwrap();
}

#[tokio::test]
async fn service_record_defaults_to_matchmade_without_filters() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/gt(Player)"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "xuid": "123456789",
            "gamertag": "Player",
            "gamerpic": { "small": "", "medium": "", "large": "", "xlarge": "" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/Matchmade/servicerecord"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .expect(1)
        .mount(&server)
        .await;

    halo.service_record(&Player::gamertag("Player"))
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    let request = requests
        .iter()
        .find(|request| request.url.path().ends_with("/servicerecord"))
        .unwrap();
    // No filters set → no query string.
    assert!(request.url.query().is_none());
}

#[tokio::test]
async fn service_record_with_filter_sends_lowercase_query_parameters() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/gt(Player)"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "xuid": "123456789",
            "gamertag": "Player",
            "gamerpic": { "small": "", "medium": "", "large": "", "xlarge": "" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/Matchmade/servicerecord"))
        .and(query_param("seasonid", "Csr/Seasons/CsrSeason5-1.json"))
        .and(query_param("gamevariantcategory", "6"))
        .and(query_param("isranked", "True"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let filter = ServiceRecordFilter::for_season("Csr/Seasons/CsrSeason5-1.json")
        .game_variant_category(6)
        .ranked(true);
    halo.service_record_with(&Player::gamertag("Player"), MatchType::Matchmade, &filter)
        .await
        .unwrap();
}

#[tokio::test]
async fn match_skill_parses_typed_skill_results() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/matches/test-match/skill"))
        .and(query_param("players", "xuid(123456789)"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Value": [{
                "Id": "xuid(123456789)",
                "ResultCode": 0,
                "Result": {
                    "TeamMmr": 1500.5,
                    "TeamId": 0,
                    "TeamMmrs": { "0": 1500.5, "1": 1480.0 },
                    "RankRecap": {
                        "PreMatchCsr": { "Value": 1490, "Tier": "Platinum", "SubTier": 2 },
                        "PostMatchCsr": { "Value": 1500, "Tier": "Platinum", "SubTier": 3 }
                    },
                    "StatPerformances": {
                        "Kills": { "Count": 20.0, "Expected": 15.0, "StdDev": 4.0 },
                        "Deaths": { "Count": 10.0, "Expected": 12.0, "StdDev": 3.0 }
                    },
                    "Counterfactuals": {
                        "SelfCounterfactuals": { "Kills": 15.0, "Deaths": 12.0 },
                        "TierCounterfactuals": {
                            "Platinum": { "Kills": 14.0, "Deaths": 12.5 }
                        }
                    }
                }
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let skill = halo.match_skill("test-match", &[player]).await.unwrap();

    assert_eq!(skill.results.len(), 1);
    let result = &skill.results[0];
    assert_eq!(result.id, "xuid(123456789)");
    assert_eq!(result.result.team_mmr, 1500.5);
    assert_eq!(result.result.rank_recap.post_match_csr.value, 1500);
    let performances = result.result.stat_performances.as_ref().unwrap();
    assert_eq!(performances.kills.count, 20.0);
    let counterfactuals = result.result.counterfactuals.as_ref().unwrap();
    assert_eq!(counterfactuals.by_tier["Platinum"].kills, 14.0);
}

#[tokio::test]
async fn match_skill_tolerates_social_match_null_fields() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    // Social matches return ResultCode 1 with empty/null skill fields; deserialization must not
    // choke on `"Counterfactuals": null` or `"StatPerformances": {}`.
    Mock::given(method("GET"))
        .and(path("/hi/matches/social/skill"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Value": [{
                "Id": "xuid(123456789)",
                "ResultCode": 1,
                "Result": {
                    "TeamMmr": 0.0,
                    "RankRecap": {
                        "PreMatchCsr": { "Value": 0, "Tier": "" },
                        "PostMatchCsr": { "Value": 0, "Tier": "" }
                    },
                    "StatPerformances": {},
                    "TeamId": 0,
                    "TeamMmrs": {},
                    "RankedRewards": null,
                    "Counterfactuals": null
                }
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let skill = halo.match_skill("social", &[player]).await.unwrap();

    assert_eq!(skill.results[0].result_code, 1);
    assert!(skill.results[0].result.counterfactuals.is_none());
}

#[tokio::test]
async fn career_rank_uses_clearance_and_default_track() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/hi/players/xuid(123456789)/rewardtracks/careerranks/careerRank1",
        ))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "CurrentProgress": {
                "Rank": 42,
                "PartialProgress": 1200,
                "HasReachedMaxRank": false
            },
            "SpartanId": "spartan-123"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let career = halo.career_rank(&player).await.unwrap();

    assert_eq!(career.current_progress.rank, 42);
    assert_eq!(career.current_progress.partial_progress, 1200);
    assert_eq!(career.spartan_id.as_deref(), Some("spartan-123"));
}

#[tokio::test]
async fn career_ranks_works_for_arbitrary_players() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/careerranks/careerRank1"))
        .and(query_param("players", "xuid(123456789)"))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "RewardTracks": [{
                "Id": "xuid(123456789)",
                "ResultCode": "Success",
                "Result": {
                    "CurrentProgress": {
                        "Rank": 271,
                        "PartialProgress": 500,
                        "HasReachedMaxRank": false
                    },
                    "SpartanId": "spartan-1234"
                }
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let career = halo.career_rank_of(&player).await.unwrap();

    assert_eq!(career.current_progress.rank, 271);
    assert_eq!(career.current_progress.partial_progress, 500);
    assert_eq!(career.spartan_id.as_deref(), Some("spartan-1234"));
}

#[tokio::test]
async fn player_match_count_parses_typed_counts() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/matches/count"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "CustomMatchesPlayedCount": 10,
            "MatchesPlayedCount": 100,
            "MatchmadeMatchesPlayedCount": 85,
            "LocalMatchesPlayedCount": 5
        })))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let count = halo.player_match_count(&player).await.unwrap();

    assert_eq!(count.total, 100);
    assert_eq!(count.matchmade, 85);
    assert_eq!(count.custom, 10);
    assert_eq!(count.local, 5);
}

#[tokio::test]
async fn match_highlight_events_downloads_and_decodes_film() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&[]).unwrap();
    let compressed_chunk = encoder.finish().unwrap();

    Mock::given(method("GET"))
        .and(path("/hi/films/matches/test-match/spectate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "FilmStatusBond": 1,
            "CustomData": {
                "FilmLength": 1000,
                "Chunks": [{
                    "Index": 0,
                    "ChunkStartTimeOffsetMilliseconds": 0,
                    "DurationMilliseconds": 1000,
                    "ChunkSize": compressed_chunk.len(),
                    "FileRelativePath": "film/highlights.bin",
                    "ChunkType": 3
                }],
                "HasGameEnded": true,
                "ManifestRefreshSeconds": 60,
                "MatchId": "test-match",
                "FilmMajorVersion": 1
            },
            "BlobStoragePathPrefix": server.uri(),
            "AssetId": "film-asset"
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/film/highlights.bin"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(compressed_chunk))
        .expect(1)
        .mount(&server)
        .await;

    let events = halo.match_highlight_events("test-match").await.unwrap();

    assert!(events.is_empty());
}

#[tokio::test]
async fn match_highlight_event_validation_fetches_film_and_stats_once() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&[]).unwrap();
    let compressed_chunk = encoder.finish().unwrap();

    Mock::given(method("GET"))
        .and(path("/hi/films/matches/test-match/spectate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "FilmStatusBond": 1,
            "CustomData": {
                "FilmLength": 1000,
                "Chunks": [{
                    "Index": 0,
                    "ChunkStartTimeOffsetMilliseconds": 0,
                    "DurationMilliseconds": 1000,
                    "ChunkSize": compressed_chunk.len(),
                    "FileRelativePath": "film/highlights.bin",
                    "ChunkType": 3
                }],
                "HasGameEnded": true,
                "ManifestRefreshSeconds": 60,
                "MatchId": "test-match",
                "FilmMajorVersion": 41
            },
            "BlobStoragePathPrefix": server.uri(),
            "AssetId": "film-asset"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/film/highlights.bin"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(compressed_chunk))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/hi/matches/test-match/stats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "MatchId": "test-match",
            "MatchInfo": {
                "StartTime": "2026-01-01T00:00:00Z",
                "EndTime": "2026-01-01T00:01:00Z",
                "Duration": "PT1M",
                "GameVariantCategory": 6,
                "MapVariant": null,
                "UgcGameVariant": null,
                "Playlist": null
            },
            "Players": [],
            "Teams": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let report = halo
        .match_highlight_events_with_validation("test-match")
        .await
        .unwrap();

    assert!(report.events.is_empty());
    assert!(report.validation.matches_stats());
}

#[tokio::test]
async fn spartan_token_is_cached_across_calls() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/hi/playlist/edfef3ac-9cbe-4fa2-b949-8f29deafd483/csrs",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(csr_body(1500)))
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    halo.playlist_csr(PlaylistId::RANKED_ARENA, &player)
        .await
        .unwrap();
    halo.playlist_csr(PlaylistId::RANKED_ARENA, &player)
        .await
        .unwrap();

    // The mock server tracks received requests; assert /spartan-token was hit exactly once
    // across both CSR lookups above, proving the spartan token was cached rather than
    // re-fetched on every call.
    let received = server.received_requests().await.unwrap();
    let spartan_hits = received
        .iter()
        .filter(|r| r.url.path() == "/spartan-token")
        .count();
    let clearance_hits = received
        .iter()
        .filter(|r| {
            r.url.path()
                == "/oban/flight-configurations/titles/hi/audiences/retail/players/xuid(123456789)/active"
        })
        .count();

    assert_eq!(
        spartan_hits, 1,
        "spartan token should only be fetched once across two CSR lookups"
    );
    assert_eq!(
        clearance_hits, 1,
        "clearance should be fetched once and cached across CSR lookups"
    );
}

#[tokio::test]
async fn gamertag_not_found_maps_to_typed_error() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/gt(NoSuchGamer)"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let err = halo
        .service_record(&Player::gamertag("NoSuchGamer"))
        .await
        .unwrap_err();
    assert!(matches!(err, InfiniteClientError::GamertagNotFound(gt) if gt == "NoSuchGamer"));
}

#[tokio::test]
async fn infinite_client_preserves_auth_errors() {
    let halo = HaloInfiniteClient::new(HaloAuthClient::with_sources(
        Arc::new(FailingSpartanTokenSource),
        Arc::new(UnusedClearanceTokenSource),
    ));
    let error = halo.match_stats("unused").await.unwrap_err();

    assert!(matches!(
        error,
        InfiniteClientError::Auth(AuthError::SpartanTokenProvider(message))
            if message == "Xbox login failed"
    ));
}

#[tokio::test]
async fn ban_summary_uses_spartan_auth_without_clearance() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/bansummary"))
        .and(query_param("auth", "st"))
        .and(query_param("targets", "xuid(123456789),xuid(987654321)"))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "Results": [] })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let players = [Player::xuid("123456789"), Player::xuid("987654321")];
    halo.ban_summary(&players).await.unwrap();

    let requests = server.received_requests().await.unwrap();
    let request = requests
        .iter()
        .find(|request| request.url.path() == "/hi/bansummary")
        .unwrap();
    assert!(request.headers.get("343-clearance").is_none());
}

#[tokio::test]
async fn ugc_playlist_and_map_mode_pair_send_clearance_header_and_query() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;
    let asset = serde_json::json!({
        "AssetId": "asset",
        "VersionId": "version",
        "PublicName": "Ranked Arena",
        "Description": ""
    });

    Mock::given(method("GET"))
        .and(path("/hi/playlists/asset/versions/version"))
        .and(query_param("clearanceId", "fake-clearance"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut body = asset.clone();
            body["RotationEntries"] = serde_json::json!([]);
            body
        }))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/hi/mapModePairs/asset/versions/version"))
        .and(query_param("clearanceId", "fake-clearance"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut body = asset.clone();
            body["MapLink"] = asset.clone();
            body["UgcGameVariantLink"] = asset;
            body
        }))
        .expect(1)
        .mount(&server)
        .await;

    halo.playlist("asset", "version").await.unwrap();
    halo.map_mode_pair("asset", "version").await.unwrap();
}

#[tokio::test]
async fn csr_season_file_uses_pc_user_agent_and_spartan_auth() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/Progression/file/Csr/Seasons/CsrSeason13-3.json"))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header(
            "User-Agent",
            "SHIVA-2043073184/6.10021.18539.0 (release; PC)",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .expect(1)
        .mount(&server)
        .await;

    halo.csr_season_file("Csr/Seasons/CsrSeason13-3.json")
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    let request = requests
        .iter()
        .find(|request| request.url.path().ends_with("CsrSeason13-3.json"))
        .unwrap();
    assert!(request.headers.get("343-clearance").is_none());
    assert!(request.url.query().is_none());
}

#[tokio::test]
async fn emblem_image_download_uses_spartan_auth_and_clearance() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/Waypoint/file/images/emblems/example.png"))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes([1, 2, 3]))
        .expect(1)
        .mount(&server)
        .await;

    let assets = EmblemImageAssets {
        emblem_cms_path: "images/emblems/example.png".to_string(),
        nameplate_cms_path: "images/nameplates/example.png".to_string(),
        text_color: "#FFFFFF".to_string(),
    };
    assert_eq!(
        halo.emblem_image(&assets).await.unwrap().as_ref(),
        [1, 2, 3]
    );

    Mock::given(method("GET"))
        .and(path(
            "/hi/images/file/Progression/Inventory/Armor/Helmets/example.png",
        ))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes([4, 5, 6]))
        .expect(1)
        .mount(&server)
        .await;
    let metadata: CustomizationItemMetadata =
        serde_json::from_value(serde_json::json!({ "CommonData": {
            "Title": { "status": "Ready", "value": "Helmet", "translations": {} },
            "DisplayPath": { "Media": { "MediaUrl": {
                "Path": "Progression/Inventory/Armor/Helmets/example.png"
            }}}
        }}))
        .unwrap();
    assert_eq!(
        halo.customization_image(&metadata).await.unwrap().unwrap(),
        [4, 5, 6]
    );
}

#[tokio::test]
async fn career_reward_track_exposes_rank_titles_and_icons() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/hi/Progression/file/RewardTracks/CareerRanks/careerRank1.json",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "TrackId": "careerRank1",
            "XpPerRank": 1000,
            "Ranks": [{
                "Rank": 271,
                "XpRequiredForRank": 45000,
                "RankTitle": { "status": "Ready", "value": "Cadet", "translations": {} },
                "RankSubTitle": { "status": "Ready", "value": "1", "translations": {} },
                "RankTier": { "status": "Ready", "value": "Onyx", "translations": {} },
                "TierType": "TierOnyx",
                "RankIcon": "Progression/RewardTracks/CareerRanks/careerRank1/271_icon.png",
                "RankLargeIcon": "Progression/RewardTracks/CareerRanks/careerRank1/271_large.png",
                "RankAdornmentIcon": "Progression/RewardTracks/CareerRanks/careerRank1/271_adornment.png",
                "RankGrade": 1
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let track = halo.career_reward_track().await.unwrap();
    let rank = track.rank(271).unwrap();

    assert_eq!(rank.display_title().as_deref(), Some("Cadet 1"));
    assert_eq!(rank.rank_tier.as_ref().unwrap().value, "Onyx");
    assert_eq!(rank.tier_type.as_deref(), Some("TierOnyx"));
    assert_eq!(rank.rank_grade, Some(1));
    assert!(track.rank(9999).is_none());

    Mock::given(method("GET"))
        .and(path(
            "/hi/images/file/Progression/RewardTracks/CareerRanks/careerRank1/271_adornment.png",
        ))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes([7, 8, 9]))
        .expect(1)
        .mount(&server)
        .await;

    let icon_path = rank.rank_adornment_icon.as_deref().unwrap();
    assert_eq!(halo.rank_icon_image(icon_path).await.unwrap(), [7, 8, 9]);
}

#[tokio::test]
async fn challenge_decks_parses_typed_deck_and_reward_track_shape() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/decks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "AssignedDecks": [{
                "Id": "deck-1",
                "Path": "Progression/Decks/deck-1.json",
                "ActiveChallenges": [{
                    "Path": "Progression/Challenges/active.json",
                    "Progress": 1,
                    "Id": "active-challenge",
                    "CanReroll": true
                }],
                "UpcomingChallenges": [{
                    "Path": "Progression/Challenges/upcoming.json",
                    "Progress": 0,
                    "Id": "upcoming-challenge",
                    "CanReroll": false,
                    "Difficulty": "Medium",
                    "TypeIconPath": "icon.png",
                    "IsUserEvent": false,
                    "Category": "Weekly",
                    "Description": { "status": "Ready", "value": "Win 3 matches", "translations": {} },
                    "Title": { "status": "Ready", "value": "Winner", "translations": {} },
                    "ThresholdForSuccess": 3,
                    "Reward": {
                        "InventoryItems": ["item-1"],
                        "SoftExperience": 500,
                        "OperationExperience": 0
                    }
                }],
                "CompletedChallenges": [],
                "Expiration": { "ISO8601Date": "2026-08-01T00:00:00Z" }
            }],
            "ClearanceId": "fake-clearance",
            "ActiveRewardTrack": {
                "RewardTrackPath": "Progression/RewardTracks/weekly.json",
                "TrackType": "Challenge",
                "CurrentProgress": 100,
                "PreviousProgress": 50,
                "IsOwned": true,
                "BaseXp": 100,
                "BoostXp": 0,
                "HasReachedMaxRank": false
            },
            "ScheduledRewardTrack": {
                "RewardTrackPath": "Progression/RewardTracks/next-week.json",
                "TrackType": "Challenge",
                "CurrentProgress": 0,
                "PreviousProgress": 0,
                "IsOwned": false,
                "BaseXp": 0,
                "BoostXp": 0,
                "HasReachedMaxRank": false
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let player = Player::xuid("123456789");
    let decks = halo.challenge_decks(&player).await.unwrap();

    assert_eq!(decks.clearance_id, "fake-clearance");
    assert_eq!(decks.assigned_decks.len(), 1);
    let deck = &decks.assigned_decks[0];
    assert_eq!(deck.active_challenges[0].progress, 1);
    assert!(deck.active_challenges[0].description.is_none());
    let upcoming = &deck.upcoming_challenges[0];
    assert_eq!(upcoming.title.as_ref().unwrap().value, "Winner");
    assert_eq!(upcoming.reward.as_ref().unwrap().soft_experience, 500);
    assert_eq!(decks.active_reward_track.current_progress, 100);
    assert!(!decks.scheduled_reward_track.is_owned);
}

#[tokio::test]
async fn settings_parses_hipc_manifest_as_json() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/settings/hipc/e2a0a7c6-6efe-42af-9283-c2ab73250c48"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Authorities": {
                "authority-1": {
                    "AuthorityId": "authority-1",
                    "Scheme": 2,
                    "Hostname": "example.svc.halowaypoint.com",
                    "Port": 443,
                    "AuthenticationMethods": [1, 2]
                }
            },
            "RetryPolicies": {
                "policy-1": {
                    "RetryPolicyId": "policy-1",
                    "TimeoutMs": 5000,
                    "RetryOptions": {
                        "MaxRetryCount": 3,
                        "RetryDelayMs": 100,
                        "RetryGrowth": 2.0,
                        "RetryJitterMs": 50,
                        "RetryIfNotFound": false
                    }
                }
            },
            "Settings": { "SomeFlag": "true" },
            "Endpoints": {
                "endpoint-1": {
                    "AuthorityId": "authority-1",
                    "Path": "/hi/example",
                    "QueryString": null,
                    "RetryPolicyId": "policy-1",
                    "TopicName": null,
                    "AcknowledgementTypeId": 0,
                    "AuthenticationLifetimeExtensionSupported": false,
                    "ClearanceAware": true
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let settings = halo.settings().await.unwrap();

    assert_eq!(
        settings.authorities["authority-1"].hostname,
        "example.svc.halowaypoint.com"
    );
    assert_eq!(settings.retry_policies["policy-1"].timeout_ms, 5000);
    assert_eq!(settings.settings["SomeFlag"], "true");
    assert!(settings.endpoints["endpoint-1"].clearance_aware);
}

#[tokio::test]
async fn asset_fetches_typed_envelope_for_arbitrary_kind() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/films/some-film"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "AssetId": "some-film",
            "VersionId": "v1",
            "PublicName": "My Film",
            "Description": ""
        })))
        .expect(1)
        .mount(&server)
        .await;

    let asset = halo.film_asset("some-film").await.unwrap();
    assert_eq!(asset.asset.public_name, "My Film");
}

#[tokio::test]
async fn match_history_pager_walks_pages_until_short_page() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    let full_page = (0..25)
        .map(|index| {
            serde_json::json!({
                "MatchId": format!("match-{index}"),
                "MatchInfo": {
                    "StartTime": "2026-01-01T00:00:00Z",
                    "EndTime": "2026-01-01T00:10:00Z",
                    "Duration": "PT10M",
                    "GameVariantCategory": 6,
                    "MapVariant": null,
                    "UgcGameVariant": null,
                    "Playlist": null
                },
                "LastTeamId": 0,
                "Outcome": 2,
                "Rank": 1,
                "PresentAtEndOfMatch": true
            })
        })
        .collect::<Vec<_>>();

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/matches"))
        .and(query_param("start", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Results": full_page,
            "ResultCount": 25
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/matches"))
        .and(query_param("start", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Results": [full_page[0].clone(), full_page[1].clone(), full_page[2].clone()],
            "ResultCount": 3
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut pager = halo.player_matches_pager(Player::xuid("123456789"), MatchHistoryType::All);

    let first = pager.next_page().await.unwrap();
    assert_eq!(first.len(), 25);
    assert!(!pager.is_done());

    let second = pager.next_page().await.unwrap();
    assert_eq!(second.len(), 3);
    assert!(pager.is_done());

    // Exhausted — no further request is made (mocks above each `.expect(1)`).
    let third = pager.next_page().await.unwrap();
    assert!(third.is_empty());
}

#[tokio::test]
async fn match_history_pager_stops_immediately_on_empty_first_page() {
    let server = MockServer::start().await;
    let (halo, _xbox) = test_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/hi/players/xuid(123456789)/matches"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "Results": [], "ResultCount": 0 })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mut pager = halo.player_matches_pager(Player::xuid("123456789"), MatchHistoryType::All);

    let page = pager.next_page().await.unwrap();
    assert!(page.is_empty());
    assert!(pager.is_done());

    let again = pager.next_page().await.unwrap();
    assert!(again.is_empty());
}
