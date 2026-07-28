//! Fetches a game mode using a named game-mode ID.

mod common;

use halo_api::clients::hi::models::GameModeId;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let mode = halo.mode(GameModeId::RANKED_SLAYER).await?;
    println!("{}", mode.asset.public_name);
    println!("  description: {}", mode.asset.description);
    println!("  asset ID:    {}", mode.asset.asset_id);
    println!("  version ID:  {}", mode.asset.version_id);
    println!("  node graph:  {}", mode.custom_data.has_node_graph);
    if mode.custom_data.key_values != serde_json::Value::Object(Default::default()) {
        println!("  key values:  {:#}", mode.custom_data.key_values);
    }
    if let Some(url) = mode.hero_url() {
        println!("  hero: {url}");
    }
    if let Some(url) = mode.thumbnail_url() {
        println!("  thumbnail: {url}");
    }
    for url in mode.screenshot_urls() {
        println!("  screenshot: {url}");
    }
    Ok(())
}
