mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let id = common::value("HALO_MATCH_ID", "Match ID")?;
    let xuid = common::xuid()?;
    println!("{:#?}", halo.match_skill(&id, &[xuid]).await?);
    Ok(())
}
