mod common;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let id = common::value("HALO_PLAYLIST_ID", "Playlist ID")?;
    println!("{:#?}", halo.playlist_metadata(&id).await?);
    Ok(())
}
