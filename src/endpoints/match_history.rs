use std::time::Duration;

use reqwest::Client;

use crate::error::HaloApiError;
use crate::models::PlayerMatchHistory;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Gets a page of a player's match history. `player_id` should be a wrapped XUID (see
/// [`xbox::util::wrap_xuid`]).
pub(crate) async fn get_player_matches(
    http: &Client,
    halostats_base_url: &str,
    spartan_token: &str,
    player_id: &str,
    start: u32,
    count: u32,
) -> Result<PlayerMatchHistory, HaloApiError> {
    let url =
        format!("{halostats_base_url}/hi/players/{player_id}/matches?start={start}&count={count}");

    let response = http
        .get(&url)
        .header("X-343-Authorization-Spartan", spartan_token)
        .header("Accept", "application/json")
        .timeout(DEFAULT_TIMEOUT)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(HaloApiError::HttpStatus {
            url,
            status: response.status(),
        });
    }

    Ok(response.json::<PlayerMatchHistory>().await?)
}
