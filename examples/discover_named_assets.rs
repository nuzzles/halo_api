//! Emits named IDs for current playlists and every map/mode in their rotations.

mod common;

use std::collections::{BTreeMap, HashSet};

use halo_api::clients::hi::models::{AssetLink, PlaylistId};

fn constant_name(name: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            if separator && !output.is_empty() {
                output.push('_');
            }
            output.push(character.to_ascii_uppercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    if output.is_empty() {
        output.push_str("UNNAMED");
    } else if output.as_bytes()[0].is_ascii_digit() {
        output.insert_str(0, "ASSET_");
    }
    output
}

fn print_constants(type_name: &str, assets: Vec<AssetLink>, versioned: bool) {
    let mut seen = HashSet::new();
    let mut named = BTreeMap::<String, Vec<AssetLink>>::new();
    for asset in assets {
        let key = if versioned {
            (asset.asset_id.clone(), asset.version_id.clone())
        } else {
            (asset.asset_id.clone(), String::new())
        };
        if seen.insert(key) {
            named
                .entry(constant_name(&asset.public_name))
                .or_default()
                .push(asset);
        }
    }

    println!("// {type_name}");
    for (base_name, duplicates) in named {
        let has_duplicates = duplicates.len() > 1;
        for asset in duplicates {
            let name = if has_duplicates {
                format!("{base_name}_{}", asset.asset_id[..8].to_ascii_uppercase())
            } else {
                base_name.clone()
            };
            if versioned {
                println!(
                    "pub const {name}: Self = Self::from_static(\"{}\", \"{}\"); // {}",
                    asset.asset_id, asset.version_id, asset.public_name
                );
            } else {
                println!(
                    "pub const {name}: Self = Self(Cow::Borrowed(\"{}\")); // {}",
                    asset.asset_id, asset.public_name
                );
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let playlist_ids = if std::env::var_os("HALO_PLAYLIST_IDS").is_some() {
        common::comma_separated(
            "HALO_PLAYLIST_IDS",
            "Current playlist IDs (comma-separated)",
        )?
    } else {
        [
            PlaylistId::BIG_TEAM_BATTLE,
            PlaylistId::SQUAD_BATTLE,
            PlaylistId::TEAM_DOUBLES,
        ]
        .into_iter()
        .map(|id| id.as_str().to_owned())
        .collect()
    };

    let mut playlists = Vec::new();
    let mut maps = Vec::new();
    let mut modes = Vec::new();
    for playlist_id in playlist_ids {
        let playlist_id = PlaylistId::new(playlist_id);
        let metadata = halo.playlist_metadata(playlist_id.as_str()).await?;
        let playlist = halo
            .playlist(playlist_id.as_str(), &metadata.ugc_playlist_version)
            .await?;
        playlists.push(playlist.asset);

        for rotation in playlist.rotation_entries {
            let pair = halo
                .map_mode_pair(&rotation.asset.asset_id, &rotation.asset.version_id)
                .await?;
            maps.push(pair.map);
            modes.push(pair.mode);
        }
    }

    print_constants("PlaylistId", playlists, false);
    print_constants("MapId", maps, true);
    print_constants("GameModeId", modes, true);
    Ok(())
}
