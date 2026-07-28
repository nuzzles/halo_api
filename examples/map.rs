//! Fetches a map using a named map ID.

mod common;

use halo_api::clients::hi::models::MapId;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let map = halo.map(MapId::LIVE_FIRE).await?;
    println!("{}", map.asset.public_name);
    println!("  description: {}", map.asset.description);
    println!("  asset ID:    {}", map.asset.asset_id);
    println!("  version ID:  {}", map.asset.version_id);
    println!("  objects:     {}", map.custom_data.object_count);
    println!("  tag level:   {}", map.custom_data.tag_level_id);
    println!("  baked:       {}", map.custom_data.is_baked);
    println!("  node graph:  {}", map.custom_data.has_node_graph);
    if let Some(url) = map.hero_url() {
        println!("  hero: {url}");
    }
    if let Some(url) = map.thumbnail_url() {
        println!("  thumbnail: {url}");
    }
    for url in map.screenshot_urls() {
        println!("  screenshot: {url}");
    }
    Ok(())
}
