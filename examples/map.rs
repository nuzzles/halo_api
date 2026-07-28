mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let asset = common::value("HALO_ASSET_ID", "Map asset ID")?;
    let version = common::value("HALO_VERSION_ID", "Map version ID")?;
    println!("{:#?}", halo.map(&asset, &version).await?);
    Ok(())
}
