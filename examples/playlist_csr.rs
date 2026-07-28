mod common;
use halo_api::clients::hi::models::PlaylistId;
#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (_, xuid) = common::player_xuid(&halo).await?;
    println!(
        "{:#?}",
        halo.playlist_csr(PlaylistId::RANKED_ARENA, &xuid).await?
    );
    Ok(())
}
