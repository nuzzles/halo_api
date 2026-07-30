//! Looks up a gamertag and prints its equipped Halo Infinite appearance.

mod common;

use halo_api::clients::hi::Player;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertag = common::value("HALO_GAMERTAG", "Gamertag")?;
    println!("{:#?}", halo.appearance(&Player::gamertag(gamertag)).await?);
    Ok(())
}
