//! Resolves multiple gamertags and fetches their public customization in one request.

mod common;

use halo_api::clients::hi::Player;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertags = common::comma_separated("HALO_GAMERTAGS", "Gamertags (comma-separated)")?;

    let players = gamertags
        .iter()
        .map(|gamertag| Player::gamertag(gamertag.clone()))
        .collect::<Vec<_>>();
    let customizations = halo.player_customizations(&players).await?;
    let emblem_mapping = halo.emblem_mapping().await?;
    for player in customizations.player_customizations {
        let data = player.result;
        println!("{} ({})", player.id, player.result_code);
        println!("  service tag: {}", data.appearance.service_tag);
        println!("  AI cores: {}", data.ai_cores.cores.len());
        println!("  armor cores: {}", data.armor_cores.cores.len());
        println!("  vehicle cores: {}", data.vehicle_cores.cores.len());
        println!("  weapon cores: {}", data.weapon_cores.cores.len());

        if let Some(configuration) = &data.appearance.emblem {
            let metadata = halo.emblem_metadata(&configuration.emblem_path).await?;
            println!("  emblem: {}", metadata.common_data.title.value);
            if let Some(images) = emblem_mapping.resolve(configuration).or_else(|| {
                emblem_mapping.resolve_metadata(&metadata, configuration.configuration_id)
            }) {
                std::fs::write("player-emblem.png", halo.emblem_image(images).await?)?;
                std::fs::write("player-nameplate.png", halo.emblem_nameplate(images).await?)?;
                println!("    saved player-emblem.png and player-nameplate.png");
            } else if let Some(image) = halo.customization_image(&metadata).await? {
                std::fs::write("player-emblem.png", image)?;
                println!("    saved player-emblem.png");
            }
        }

        if let Some(armor) = data.armor_cores.equipped() {
            let core = halo.customization_metadata(&armor.core_path).await?;
            println!("  equipped armor core: {}", core.common_data.title.value);
            if let Some(image) = halo.customization_image(&core).await? {
                std::fs::write("armor-core.png", image)?;
                println!("    saved armor-core.png");
            }
            if let Some(theme) = armor.equipped_theme() {
                let coating = halo.customization_metadata(&theme.coating_path).await?;
                let helmet = halo.customization_metadata(&theme.helmet_path).await?;
                let chest = if theme.chest_attachment_path.is_empty() {
                    None
                } else {
                    Some(
                        halo.customization_metadata(&theme.chest_attachment_path)
                            .await?,
                    )
                };
                let left_shoulder = halo
                    .customization_metadata(&theme.left_shoulder_pad_path)
                    .await?;
                let right_shoulder = halo
                    .customization_metadata(&theme.right_shoulder_pad_path)
                    .await?;

                println!("    coating: {}", coating.common_data.title.value);
                println!("    helmet: {}", helmet.common_data.title.value);
                if let Some(chest) = &chest {
                    println!("    chest: {}", chest.common_data.title.value);
                } else {
                    println!("    chest: none");
                }
                println!(
                    "    left shoulder: {}",
                    left_shoulder.common_data.title.value
                );
                println!(
                    "    right shoulder: {}",
                    right_shoulder.common_data.title.value
                );
                if let Some(image) = halo.customization_image(&helmet).await? {
                    std::fs::write("helmet.png", image)?;
                    println!("    saved helmet.png");
                }
                if let Some(image) = halo.customization_image(&coating).await? {
                    std::fs::write("coating.png", image)?;
                    println!("    saved coating.png");
                }
                if let Some(chest) = &chest
                    && let Some(image) = halo.customization_image(chest).await?
                {
                    std::fs::write("chest.png", image)?;
                    println!("    saved chest.png");
                }
                if let Some(image) = halo.customization_image(&left_shoulder).await? {
                    std::fs::write("left-shoulder.png", image)?;
                    println!("    saved left-shoulder.png");
                }
                if let Some(image) = halo.customization_image(&right_shoulder).await? {
                    std::fs::write("right-shoulder.png", image)?;
                    println!("    saved right-shoulder.png");
                }
                for (index, emblem) in theme.emblems.iter().enumerate() {
                    let metadata = halo.customization_metadata(&emblem.path).await?;
                    let file_name = format!("armor-emblem-{index}.png");
                    if let Some(images) = emblem_mapping
                        .resolve_customization_emblem(emblem)
                        .or_else(|| {
                            emblem_mapping.resolve_metadata(&metadata, emblem.configuration_id)
                        })
                    {
                        std::fs::write(&file_name, halo.emblem_image(images).await?)?;
                        println!("    saved {file_name}");
                    } else if let Some(image) = halo.customization_image(&metadata).await? {
                        std::fs::write(&file_name, image)?;
                        println!("    saved {file_name}");
                    }
                }
            }
        }

        if let Some(body) = data.spartan_body {
            println!("  body type: {}", body.body_type);
            println!("  voice: {}", body.voice_path);
        }
    }
    Ok(())
}
