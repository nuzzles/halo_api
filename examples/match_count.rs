mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::player_xuid(&halo).await?;
    println!("Match counts for {gamertag}:");
    println!("{:#?}", halo.player_match_count(&xuid).await?);
    Ok(())
}
