use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use halo_api::constants::PlaylistId;
use halo_api::{HaloClient, HaloEndpoints};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xbox::auth::XblAuthProvider;
use xbox::cache::CachedToken;
use xbox::{XboxClient, XboxEndpoints, XboxError};

/// Hands out a fixed, never-expiring user token without touching the network.
struct FakeAuthProvider;

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

/// Sets up a mock server with XSTS + spartan-token mocks, and returns a `HaloClient` wired to
/// it plus the shared `XboxClient` (for XUID resolution) — everything an end-to-end test needs.
async fn test_client(server: &MockServer) -> (HaloClient, Arc<XboxClient<FakeAuthProvider>>) {
    Mock::given(method("POST"))
        .and(path("/xsts/authorize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(xsts_body()))
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

    let halo_client = HaloClient::from_xbox_client_with_endpoints(
        xbox_client.clone(),
        HaloEndpoints {
            skill_base_url: server.uri(),
            halostats_base_url: server.uri(),
            spartan_token_url: format!("{}/spartan-token", server.uri()),
        },
    );

    (halo_client, xbox_client)
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
        .respond_with(ResponseTemplate::new(200).set_body_json(csr_body(1500)))
        .expect(1)
        .mount(&server)
        .await;

    let xuid = "123456789".into();
    let csr = halo.playlist_csr(PlaylistId::Arena, &xuid).await.unwrap();

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
    halo.playlist_csr(PlaylistId::Arena, &xuid).await.unwrap();
    halo.playlist_csr(PlaylistId::Arena, &xuid).await.unwrap();

    // The mock server tracks received requests; assert /spartan-token was hit exactly once
    // across both CSR lookups above, proving the spartan token was cached rather than
    // re-fetched on every call.
    let received = server.received_requests().await.unwrap();
    let spartan_hits = received
        .iter()
        .filter(|r| r.url.path() == "/spartan-token")
        .count();

    assert_eq!(
        spartan_hits, 1,
        "spartan token should only be fetched once across two CSR lookups"
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
    assert!(matches!(err, halo_api::HaloApiError::GamertagNotFound(gt) if gt == "NoSuchGamer"));
}
