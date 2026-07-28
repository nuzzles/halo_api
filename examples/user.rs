mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertag = common::value("HALO_GAMERTAG", "Gamertag")?;
    println!("{:#?}", halo.user(&gamertag).await?);
    Ok(())
}
