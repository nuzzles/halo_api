mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, player) = common::player_xuid(&halo).await?;
    println!("Matches for {gamertag}:");
    println!("{:#?}", halo.player_matches(&player, 0, 25).await?);
    Ok(())
}
