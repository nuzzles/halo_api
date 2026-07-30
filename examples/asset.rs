mod common;

use halo_api::clients::hi::models::UgcAssetKind;

fn parse_kind(value: &str) -> Result<UgcAssetKind, common::ExampleError> {
    match value {
        "Map" | "Maps" => Ok(UgcAssetKind::Map),
        "Playlist" | "Playlists" => Ok(UgcAssetKind::Playlist),
        "GameMode" | "UgcGameVariant" | "UgcGameVariants" => Ok(UgcAssetKind::GameMode),
        "MapModePair" | "MapModePairs" => Ok(UgcAssetKind::MapModePair),
        "Film" | "Films" => Ok(UgcAssetKind::Film),
        "Prefab" | "Prefabs" => Ok(UgcAssetKind::Prefab),
        "EngineGameVariant" | "EngineGameVariants" => Ok(UgcAssetKind::EngineGameVariant),
        other => Err(format!("unrecognized asset kind \"{other}\"").into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let kind = common::value(
        "HALO_ASSET_KIND",
        "Asset kind (Map, Playlist, GameMode, MapModePair, Film, Prefab, or EngineGameVariant)",
    )?;
    let kind = parse_kind(&kind)?;
    let asset = common::value("HALO_ASSET_ID", "Asset ID")?;
    println!("{:#?}", halo.asset(kind, &asset).await?);
    Ok(())
}
