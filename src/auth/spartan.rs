use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use xbox::XboxClient;
use xbox::auth::{RelyingParty, XblAuthProvider};
use xbox::cache::CachedToken;

use crate::auth::AuthError;

use super::endpoints::AuthEndpoints;

const SPARTAN_TOKEN_TIMEOUT: Duration = Duration::from_secs(10);
const HALO_RELYING_PARTY: RelyingParty = RelyingParty::new("https://prod.xsts.halowaypoint.com/");

#[async_trait]
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

pub(crate) struct XboxSpartanTokenProvider<P: XblAuthProvider> {
    xbox: Arc<XboxClient<P>>,
    http: Client,
    spartan_token_url: String,
}

impl<P: XblAuthProvider> XboxSpartanTokenProvider<P> {
    pub(crate) fn with_endpoints(xbox: Arc<XboxClient<P>>, endpoints: &AuthEndpoints) -> Self {
        Self {
            xbox,
            http: Client::new(),
            spartan_token_url: endpoints.spartan_token_url.clone(),
        }
    }
}

#[async_trait]
impl<P: XblAuthProvider> SpartanTokenSource for XboxSpartanTokenProvider<P> {
    async fn spartan_token(&self) -> Result<CachedToken<String>, AuthError> {
        let xsts = self
            .xbox
            .xsts_ticket(HALO_RELYING_PARTY)
            .await
            .map_err(|err| AuthError::SpartanTokenProvider(err.to_string()))?;

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
