use std::time::Duration;

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::Client;
use serde::Deserialize;
use xbox::cache::CachedToken;

use crate::auth::AuthError;

const TIMEOUT: Duration = Duration::from_secs(10);

/// Produces the Waypoint flight clearance required alongside a Spartan token.
#[async_trait]
pub trait ClearanceTokenSource: Send + Sync {
    async fn clearance_token(&self, spartan_token: &str) -> Result<CachedToken<String>, AuthError>;
}

/// Fetches clearance from Halo Waypoint's OBAN flight-configuration endpoint.
pub struct WaypointClearanceProvider {
    http: Client,
    current_user_url: String,
    clearance_base_url: String,
}

impl WaypointClearanceProvider {
    pub fn new(current_user_url: impl Into<String>, clearance_base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            current_user_url: current_user_url.into(),
            clearance_base_url: clearance_base_url.into(),
        }
    }
}

#[derive(Deserialize)]
struct CurrentUserResponse {
    xuid: String,
}

#[derive(Deserialize)]
struct ClearanceResponse {
    #[serde(rename = "FlightConfigurationId")]
    flight_configuration_id: String,
}

#[async_trait]
impl ClearanceTokenSource for WaypointClearanceProvider {
    async fn clearance_token(&self, spartan_token: &str) -> Result<CachedToken<String>, AuthError> {
        let current_user_response = self
            .http
            .get(&self.current_user_url)
            .header("X-343-Authorization-Spartan", spartan_token)
            .header("Accept", "application/json")
            .timeout(TIMEOUT)
            .send()
            .await?;
        if !current_user_response.status().is_success() {
            return Err(AuthError::HttpStatus {
                url: self.current_user_url.clone(),
                status: current_user_response.status(),
            });
        }
        let current_user = current_user_response.json::<CurrentUserResponse>().await?;
        let url = format!(
            "{}/xuid({})/active",
            self.clearance_base_url, current_user.xuid
        );
        let response = self
            .http
            .get(&url)
            .header("X-343-Authorization-Spartan", spartan_token)
            .header("Accept", "application/json")
            .timeout(TIMEOUT)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(AuthError::HttpStatus {
                url,
                status: response.status(),
            });
        }
        let body = response.json::<ClearanceResponse>().await?;
        // OBAN does not publish an expiry. Refresh hourly and immediately after any 401.
        Ok(CachedToken::new(
            body.flight_configuration_id,
            Utc::now() + ChronoDuration::hours(1),
        ))
    }
}
