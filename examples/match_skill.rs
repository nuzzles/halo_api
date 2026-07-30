mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let id = common::value("HALO_MATCH_ID", "Match ID")?;
    let (_, player) = common::player_xuid(&halo).await?;
    println!("{:#?}", halo.match_skill(&id, &[player]).await?);
    Ok(())
}
