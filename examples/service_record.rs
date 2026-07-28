mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let player = common::value("HALO_PLAYER", "Gamertag or XUID")?;
    println!("{:#?}", halo.service_record(&player).await?);
    Ok(())
}
