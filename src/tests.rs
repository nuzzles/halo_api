use std::sync::Arc;

use crate::auth::endpoints::AuthEndpoints;
use crate::auth::{AuthClient, AuthError, HaloAuth, HaloCredentials};
use crate::clients::hi::endpoints::HaloEndpoints;
use crate::clients::hi::models::PlaylistId;
use crate::clients::hi::{HaloInfiniteClient, InfiniteClientError};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xbox::auth::XblAuthProvider;
use xbox::cache::CachedToken;
use xbox::{XboxClient, XboxEndpoints, XboxError};

/// Hands out a fixed, never-expiring user token without touching the network.
struct FakeAuthProvider;

struct FailingHaloAuth;

#[async_trait]
impl HaloAuth for FailingHaloAuth {
    async fn credentials(&self, _require_clearance: bool) -> Result<HaloCredentials, AuthError> {
        Err(AuthError::SpartanTokenProvider(
            "Xbox login failed".to_string(),
        ))
    }

    async fn invalidate(&self) {}
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
    let auth = AuthClient::from_xbox_client_with_endpoints(xbox_client.clone(), &auth_endpoints);
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

    let xuid = "123456789".into();
    let csr = halo
        .playlist_csr(PlaylistId::RANKED_ARENA, &xuid)
        .await
        .unwrap();

    assert_eq!(csr.records.len(), 1);
    assert_eq!(csr.records[0].result.current.value, 1500);
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

    let xuid = "123456789".into();
    halo.playlist_csr(PlaylistId::RANKED_ARENA, &xuid)
        .await
        .unwrap();
    halo.playlist_csr(PlaylistId::RANKED_ARENA, &xuid)
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
        .and(path("/hi/players/NoSuchGamer/Matchmade/servicerecord"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let err = halo.service_record("NoSuchGamer").await.unwrap_err();
    assert!(matches!(err, InfiniteClientError::GamertagNotFound(gt) if gt == "NoSuchGamer"));
}

#[tokio::test]
async fn infinite_client_preserves_auth_errors() {
    let halo = HaloInfiniteClient::new(FailingHaloAuth);
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

    let xuids = ["123456789".into(), "987654321".into()];
    halo.ban_summary(&xuids).await.unwrap();

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
