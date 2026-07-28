mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    println!("{:#?}", halo.player_matches(&common::xuid()?, 0, 25).await?);
    Ok(())
}
