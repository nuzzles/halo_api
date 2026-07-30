//! Prints the logged-in player's match-history privacy settings.

mod common;

use halo_api::clients::hi::Player;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (xbox, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::logged_in_player(&xbox).await?;
    let privacy = halo.matches_privacy(&Player::from(&xuid)).await?;

    println!("{gamertag}");
    println!("  matchmade games: {:?}", privacy.matchmade_setting());
    println!("  other games:     {:?}", privacy.other_setting());
    Ok(())
}
