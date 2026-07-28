mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let kind = common::value("HALO_ASSET_KIND", "Asset kind (for example Maps)")?;
    let asset = common::value("HALO_ASSET_ID", "Asset ID")?;
    println!("{:#?}", halo.asset(&kind, &asset).await?);
    Ok(())
}
