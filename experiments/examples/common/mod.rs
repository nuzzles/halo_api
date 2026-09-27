#![allow(dead_code)]

use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use halo_api::auth::HaloAuthClient;
use halo_api::clients::hi::{HaloInfiniteClient, Player};
use xbox::{RelyingParty, XboxClient, auth::LegacyPasswordProvider, models::Xuid};

pub type ExampleError = Box<dyn std::error::Error>;
pub type ExampleXboxClient = XboxClient<LegacyPasswordProvider>;

fn prompt_line(label: &str) -> Result<String, ExampleError> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn value(name: &str, label: &str) -> Result<String, ExampleError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(_) => prompt_line(label),
    }
}

pub fn comma_separated(name: &str, label: &str) -> Result<Vec<String>, ExampleError> {
    let values = value(name, label)?
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Err(format!("{name} must contain at least one value").into());
    }
    Ok(values)
}

pub async fn player_xuid(halo: &HaloInfiniteClient) -> Result<(String, Player), ExampleError> {
    let gamertag = value("HALO_GAMERTAG", "Gamertag")?;
    let user = halo.user(&Player::gamertag(gamertag)).await?;
    Ok((user.gamertag, Player::xuid(user.xuid)))
}

pub fn xbox_client() -> Result<Arc<ExampleXboxClient>, ExampleError> {
    let username = value("XBOX_USERNAME", "Xbox Live email")?;
    let password = match env::var("XBOX_PASSWORD") {
        Ok(password) => password,
        Err(_) => rpassword::prompt_password("Xbox Live password: ")?,
    };
    Ok(Arc::new(XboxClient::new(LegacyPasswordProvider::new(
        username, password,
    ))))
}

/// Uses a 60-second Halo API/auth timeout, overridable with HALO_TIMEOUT_SECS.
/// Xbox sign-in and XSTS timeouts are controlled separately by the xbox crate.
pub fn halo_infinite_client() -> Result<(Arc<ExampleXboxClient>, HaloInfiniteClient), ExampleError>
{
    let timeout_secs = match env::var("HALO_TIMEOUT_SECS") {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|_| "HALO_TIMEOUT_SECS must be a positive integer in seconds")?,
        Err(env::VarError::NotPresent) => 60,
        Err(error) => return Err(error.into()),
    };
    if timeout_secs == 0 {
        return Err("HALO_TIMEOUT_SECS must be greater than zero".into());
    }
    let timeout = Duration::from_secs(timeout_secs);
    let xbox = xbox_client()?;
    let auth = HaloAuthClient::from_xbox_client_with_timeout(xbox.clone(), timeout);
    let halo = HaloInfiniteClient::builder().timeout(timeout).build(auth);
    Ok((xbox, halo))
}

pub async fn logged_in_player(xbox: &ExampleXboxClient) -> Result<(String, Xuid), ExampleError> {
    let ticket = xbox.xsts_ticket(RelyingParty::XBOX).await?;
    let gamertag = ticket.gamertag().unwrap_or("<unknown>").to_string();
    let xuid = ticket.xuid().ok_or("Xbox ticket did not contain an XUID")?;
    Ok((gamertag, xuid))
}
