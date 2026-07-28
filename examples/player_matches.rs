mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::player_xuid(&halo).await?;
    println!("Matches for {gamertag}:");
    println!("{:#?}", halo.player_matches(&xuid, 0, 25).await?);
    Ok(())
}
