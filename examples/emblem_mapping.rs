//! Resolves one emblem configuration to its readable name and image URLs.

mod common;

use halo_api::clients::hi::models::EmblemConfiguration;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let configuration = EmblemConfiguration {
        emblem_path: "Inventory/Spartan/Emblems/104-001-samurai-thega-d83dae85.json".into(),
        configuration_id: 1_198_260_756,
    };

    let metadata = halo.emblem_metadata(&configuration.emblem_path).await?;
    let mapping = halo.emblem_mapping().await?;
    let images = mapping.resolve(&configuration).ok_or("emblem not mapped")?;

    let emblem = halo.emblem_image(images).await?;
    let nameplate = halo.emblem_nameplate(images).await?;
    std::fs::write("emblem.png", emblem)?;
    std::fs::write("nameplate.png", nameplate)?;

    println!("{}", metadata.common_data.title.value);
    println!("saved emblem.png and nameplate.png");
    println!("text color: {}", images.text_color);
    Ok(())
}
