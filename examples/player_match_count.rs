mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    println!("{:#?}", halo.player_match_count(&common::xuid()?).await?);
    Ok(())
}
