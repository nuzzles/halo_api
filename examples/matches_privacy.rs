mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    println!("{:#?}", halo.matches_privacy(&common::xuid()?).await?);
    Ok(())
}
