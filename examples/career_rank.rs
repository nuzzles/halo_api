mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, player) = common::player_xuid(&halo).await?;
    println!("Career rank for {gamertag}:");
    println!("{:#?}", halo.career_rank(&player).await?);
    Ok(())
}
