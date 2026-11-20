use std::sync::Arc;
use std::time::Duration;

use super::cache::CachedToken;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::auth::AuthError;

use super::endpoints::AuthEndpoints;

const SPARTAN_TOKEN_TIMEOUT: Duration = Duration::from_secs(10);

/// Supplies a valid XSTS ticket for `https://prod.xsts.halowaypoint.com/`.
/// The caller owns Xbox sign-in and ticket refresh. This method may be called
/// again whenever Halo credentials expire or a request is rejected.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait XstsTicketSource: Send + Sync {
    async fn xsts_ticket(&self) -> Result<String, AuthError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub(crate) trait SpartanTokenSource: Send + Sync {
    async fn spartan_token(&self) -> Result<CachedToken<String>, AuthError>;
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

pub(crate) struct WaypointSpartanTokenProvider {
    source: Arc<dyn XstsTicketSource>,
    http: Client,
    spartan_token_url: String,
}

impl WaypointSpartanTokenProvider {
    pub(crate) fn with_endpoints(
        source: Arc<dyn XstsTicketSource>,
        endpoints: &AuthEndpoints,
    ) -> Self {
        Self {
            source,
            http: Client::new(),
            spartan_token_url: endpoints.spartan_token_url.clone(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl SpartanTokenSource for WaypointSpartanTokenProvider {
    async fn spartan_token(&self) -> Result<CachedToken<String>, AuthError> {
        let xsts = self.source.xsts_ticket().await?;

        let response = self
            .http
            .post(&self.spartan_token_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&serde_json::json!({
                "Audience": "urn:343:s3:services",
                "MinVersion": "4",
                "Proof": [{
                    "Token": xsts,
                    "TokenType": "Xbox_XSTSv3",
                }],
            }))
            .timeout(SPARTAN_TOKEN_TIMEOUT)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AuthError::HttpStatus {
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
