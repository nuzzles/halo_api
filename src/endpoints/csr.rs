use std::time::Duration;

use reqwest::Client;

use crate::error::HaloApiError;
use crate::models::CsrRecords;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Gets Competitive Skill Rank (CSR) for one or more players in a given playlist.
///
/// `player_ids` should be XUIDs in wrapped form (see [`xbox::util::wrap_xuid`]).
pub(crate) async fn get_playlist_csr(
    http: &Client,
    skill_base_url: &str,
    spartan_token: &str,
    playlist_id: &str,
    player_ids: &[String],
) -> Result<CsrRecords, HaloApiError> {
    let players = player_ids.join(",");
    let url = format!("{skill_base_url}/hi/playlist/{playlist_id}/csrs?players={players}");

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

    Ok(response.json::<CsrRecords>().await?)
}
