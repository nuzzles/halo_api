mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::player_xuid(&halo).await?;
    println!("Operation passes for {gamertag}:");
    println!("{:#?}", halo.reward_track_operations(&xuid).await?);
    Ok(())
}
