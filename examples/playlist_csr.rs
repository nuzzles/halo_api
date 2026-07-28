mod common;
use halo_api::clients::hi::constants::PlaylistId;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let xuid = common::xuid()?;
    println!("{:#?}", halo.playlist_csr(PlaylistId::Arena, &xuid).await?);
    Ok(())
}
