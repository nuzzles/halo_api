#![allow(dead_code)]

use std::env;
use std::io::{self, Write};

use halo_api::auth::HaloAuthClient;
use halo_api::clients::hi::{HaloInfiniteClient, Player};

pub type ExampleError = Box<dyn std::error::Error>;

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

pub fn halo_infinite_client() -> Result<((), HaloInfiniteClient), ExampleError> {
    let spartan = value("HALO_SPARTAN_TOKEN", "Halo Spartan token")?;
    let clearance = value("HALO_CLEARANCE_TOKEN", "Halo clearance token")?;
    let auth = HaloAuthClient::from_tokens(spartan, clearance);
    Ok(((), HaloInfiniteClient::new(auth)))
}

pub async fn logged_in_player(halo: &HaloInfiniteClient) -> Result<(String, String), ExampleError> {
    let user = halo.current_user().await?;
    let player = halo.user(&Player::xuid(user.xuid)).await?;
    Ok((player.gamertag, player.xuid))
}
