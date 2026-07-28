//! Looks up a gamertag and prints its equipped Halo Infinite appearance.

mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertag = common::value("HALO_GAMERTAG", "Gamertag")?;
    println!("{:#?}", halo.appearance_by_gamertag(&gamertag).await?);
    Ok(())
}
