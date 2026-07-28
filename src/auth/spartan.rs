use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use xbox::XboxClient;
use xbox::auth::{RelyingParty, XblAuthProvider};
use xbox::cache::CachedToken;

use crate::endpoints::HaloEndpoints;
use crate::error::HaloApiError;

const SPARTAN_TOKEN_TIMEOUT: Duration = Duration::from_secs(10);

/// Something that can produce a fresh Halo "spartan token" on demand — the bearer credential
/// every Halo Waypoint API call requires.
///
/// This is deliberately its own trait, separate from anything in the `xbox` crate: a spartan
/// token can be sourced from anywhere (an `xbox` client, a shared cache elsewhere, a value read
/// from a secrets manager), so [`crate::HaloClient`] only depends on this trait, not on `xbox`'s
/// concrete types.
#[async_trait]
pub trait SpartanTokenSource: Send + Sync {
    async fn spartan_token(&self) -> Result<CachedToken<String>, HaloApiError>;
}

#[derive(Debug, Deserialize)]
struct SpartanTokenResponse {
    #[serde(rename = "SpartanToken")]
    spartan_token: String,
    #[serde(rename = "ExpiresUtc")]
    expires_utc: ExpiresUtc,
}

#[derive(Debug, Deserialize)]
struct ExpiresUtc {
    #[serde(rename = "ISO8601Date")]
    iso8601_date: DateTime<Utc>,
}

/// The default [`SpartanTokenSource`]: mints a spartan token from an Xbox Live XSTS ticket
/// (scoped to [`RelyingParty::Halo`]) obtained via a shared `xbox::XboxClient`.
///
/// Takes the `xbox::XboxClient` as an `Arc` so callers can keep their own handle to it (e.g.
/// for gamertag/XUID resolution) alongside the one wired into a [`crate::HaloClient`].
pub struct XboxSpartanTokenProvider<P: XblAuthProvider> {
    xbox: Arc<XboxClient<P>>,
    http: Client,
    spartan_token_url: String,
}

impl<P: XblAuthProvider> XboxSpartanTokenProvider<P> {
    pub fn new(xbox: Arc<XboxClient<P>>) -> Self {
        Self::with_endpoints(xbox, &HaloEndpoints::default())
    }

    /// Constructs a provider pointed at overridden endpoint URLs, e.g. to point at a mock
    /// server in tests.
    pub fn with_endpoints(xbox: Arc<XboxClient<P>>, endpoints: &HaloEndpoints) -> Self {
        Self {
            xbox,
            http: Client::new(),
            spartan_token_url: endpoints.spartan_token_url.clone(),
        }
    }
}

#[async_trait]
impl<P: XblAuthProvider> SpartanTokenSource for XboxSpartanTokenProvider<P> {
    async fn spartan_token(&self) -> Result<CachedToken<String>, HaloApiError> {
        let xsts = self
            .xbox
            .xsts_ticket(RelyingParty::Halo)
            .await
            .map_err(|err| HaloApiError::SpartanTokenProvider(err.to_string()))?;

        let response = self
            .http
            .post(&self.spartan_token_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&serde_json::json!({
                "Audience": "urn:343:s3:services",
                "MinVersion": "4",
                "Proof": [{
                    "Token": xsts.token,
                    "TokenType": "Xbox_XSTSv3",
                }],
            }))
            .timeout(SPARTAN_TOKEN_TIMEOUT)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HaloApiError::HttpStatus {
                url: self.spartan_token_url.clone(),
                status: response.status(),
            });
        }

        let body = response.json::<SpartanTokenResponse>().await?;
        Ok(CachedToken::new(
            body.spartan_token,
            body.expires_utc.iso8601_date,
        ))
    }
}
