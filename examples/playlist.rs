//! Resolves and fetches the current version of a named playlist.

mod common;

use halo_api::clients::hi::models::PlaylistId;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let playlist = PlaylistId::RANKED_ARENA;
    let metadata = halo.playlist_metadata(playlist.as_str()).await?;
    let playlist = halo
        .playlist(playlist.as_str(), &metadata.ugc_playlist_version)
        .await?;

    println!("{}", playlist.asset.public_name);
    println!("  description: {}", playlist.asset.description);
    println!("  asset ID:    {}", playlist.asset.asset_id);
    println!("  version ID:  {}", playlist.asset.version_id);
    println!("  name hint:   {}", metadata.name_hint);
    println!("  has CSR:     {}", metadata.has_csr);
    println!("  rotations:   {}", playlist.rotation_entries.len());
    if let Some(url) = playlist.hero_url() {
        println!("  hero: {url}");
    }
    if let Some(url) = playlist.thumbnail_url() {
        println!("  thumbnail: {url}");
    }
    for url in playlist.screenshot_urls() {
        println!("  screenshot: {url}");
    }
    println!("Rotation entries:");
    for entry in playlist.rotation_entries {
        println!(
            "  {} (weight {}, asset {}, version {})",
            entry.asset.public_name,
            entry.metadata.weight,
            entry.asset.asset_id,
            entry.asset.version_id
        );
    }
    Ok(())
}
