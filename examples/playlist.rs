mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let asset = common::value("HALO_ASSET_ID", "Playlist asset ID")?;
    let version = common::value("HALO_VERSION_ID", "Playlist version ID")?;
    println!("{:#?}", halo.playlist(&asset, &version).await?);
    Ok(())
}
