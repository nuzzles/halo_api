mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let asset = common::value("HALO_ASSET_ID", "Mode asset ID")?;
    let version = common::value("HALO_VERSION_ID", "Mode version ID")?;
    println!("{:#?}", halo.mode(&asset, &version).await?);
    Ok(())
}
