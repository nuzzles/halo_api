//! Resolves multiple gamertags and prints their public customization response.

mod common;

use halo_api::clients::hi::Player;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    common::ensure_player_customizations_auth().await?;
    let (_, halo) = common::halo_infinite_client()?;
    let gamertags = common::comma_separated("HALO_GAMERTAGS", "Gamertags (comma-separated)")?;

    let players = gamertags
        .iter()
        .map(|gamertag| Player::gamertag(gamertag.clone()))
        .collect::<Vec<_>>();
    let customizations = halo.player_customizations(&players).await?;
    println!("{}", serde_json::to_string_pretty(&customizations)?);
    Ok(())
}
