use std::time::Duration;

use reqwest::{Client, StatusCode};

use crate::error::HaloApiError;
use crate::models::ServiceRecord;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Gets a player's matchmade service record. `gamertag` may be a plain gamertag or a
/// wrapped XUID (see [`xbox::util::wrap_xuid`]).
pub(crate) async fn get_service_record(
    http: &Client,
    halostats_base_url: &str,
    spartan_token: &str,
    gamertag: &str,
) -> Result<ServiceRecord, HaloApiError> {
    let url = format!("{halostats_base_url}/hi/players/{gamertag}/Matchmade/servicerecord");

    let response = http
        .get(&url)
        .header("X-343-Authorization-Spartan", spartan_token)
        .header("Accept", "application/json")
        .timeout(DEFAULT_TIMEOUT)
        .send()
        .await?;

    let status = response.status();
    if status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND {
        return Err(HaloApiError::GamertagNotFound(gamertag.to_string()));
    }
    if !status.is_success() {
        return Err(HaloApiError::HttpStatus { url, status });
    }

    Ok(response.json::<ServiceRecord>().await?)
}
