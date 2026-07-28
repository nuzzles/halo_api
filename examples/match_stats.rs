mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let id = common::value("HALO_MATCH_ID", "Match ID")?;
    println!("{:#?}", halo.match_stats(&id).await?);
    Ok(())
}
