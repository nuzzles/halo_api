//! Emits named constants for every 343-tagged map, mode, and playlist in the live catalog.

mod common;

use std::collections::{BTreeMap, HashSet};

use halo_api::clients::hi::models::{UgcAssetKind, UgcSearchResult};

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

async fn official_assets(
    halo: &halo_api::HaloInfiniteClient,
    kind: UgcAssetKind,
) -> Result<Vec<UgcSearchResult>, common::ExampleError> {
    const PAGE_SIZE: u32 = 20;
    let mut start = 0;
    let mut assets = Vec::new();
    loop {
        let page = halo.search_assets(kind, start, PAGE_SIZE).await?;
        assets.extend(page.results.into_iter().filter(|asset| {
            asset.asset_home == Some(2)
                || asset
                    .tags
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case("343i"))
        }));
        start += page.result_count;
        if page.result_count == 0 || start >= page.estimated_total {
            break;
        }
    }
    Ok(assets)
}

fn print_constants(type_name: &str, assets: Vec<UgcSearchResult>, versioned: bool) {
    let mut named = BTreeMap::<String, Vec<UgcSearchResult>>::new();
    let mut seen = HashSet::new();
    for asset in assets {
        if !seen.insert((asset.asset_id.clone(), asset.version_id.clone())) {
            continue;
        }
        named
            .entry(constant_name(&asset.name))
            .or_default()
            .push(asset);
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
                    asset.asset_id, asset.version_id, asset.name
                );
            } else {
                println!(
                    "pub const {name}: Self = Self(Cow::Borrowed(\"{}\")); // {}",
                    asset.asset_id, asset.name
                );
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    print_constants(
        "MapId",
        official_assets(&halo, UgcAssetKind::Map).await?,
        true,
    );
    print_constants(
        "GameModeId",
        official_assets(&halo, UgcAssetKind::GameMode).await?,
        true,
    );
    print_constants(
        "PlaylistId",
        official_assets(&halo, UgcAssetKind::Playlist).await?,
        false,
    );
    Ok(())
}
