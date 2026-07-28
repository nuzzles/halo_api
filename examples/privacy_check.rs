//! Prints the logged-in player's match-history privacy settings.

mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (xbox, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::logged_in_player(&xbox).await?;
    let privacy = halo.matches_privacy(&xuid).await?;

    println!("{gamertag}");
    println!("  matchmade games: {:?}", privacy.matchmade_setting());
    println!("  other games:     {:?}", privacy.other_setting());
    Ok(())
}
