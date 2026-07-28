//! Resolves multiple gamertags and fetches their public customization in one request.

mod common;

use xbox::models::Xuid;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertags = common::comma_separated("HALO_GAMERTAGS", "Gamertags (comma-separated)")?;

    let mut xuids = Vec::with_capacity(gamertags.len());
    for gamertag in &gamertags {
        let user = halo.user(gamertag).await?;
        println!("{} -> xuid({})", user.gamertag, user.xuid);
        xuids.push(Xuid::from(user.xuid));
    }

    let customizations = halo.player_customizations(&xuids).await?;
    println!("{customizations:#?}");
    Ok(())
}
