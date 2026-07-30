mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (_, player) = common::player_xuid(&halo).await?;
    println!("{:#?}", halo.player_match_count(&player).await?);
    Ok(())
}
