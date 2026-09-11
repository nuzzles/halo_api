//! Save CURRENT account customization and referenced item metadata for film research.
//! This is reference data, not historical match state.
//! Usage: OUTPUT_DIRECTORY [XUID] [EXTRA_ITEM_PATHS_JSON]
mod common;

use halo_api::auth::HaloAuthClient;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::PathBuf, time::Duration};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let output = PathBuf::from(
        args.first()
            .ok_or("Usage: download_customization_reference OUTPUT_DIRECTORY [XUID]")?,
    );
    let xbox = common::xbox_client()?;
    let xuid = match args.get(1) {
        Some(xuid) => xuid.clone(),
        None => common::logged_in_player(&xbox).await?.1.to_string(),
    };
    if !xuid.bytes().all(|b| b.is_ascii_digit()) || xuid.is_empty() {
        return Err("Invalid XUID".into());
    }
    let timeout = Duration::from_secs(120);
    let auth = HaloAuthClient::from_xbox_client_with_timeout(xbox, timeout);
    let http = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    fs::create_dir_all(output.join("items"))?;
    let url = format!("https://economy.svc.halowaypoint.com/hi/customization?players=xuid({xuid})");
    let raw = halo_api::fetch_waypoint_json(&auth, &http, &url).await?;
    fs::write(
        output.join("current-customization.json"),
        serde_json::to_vec_pretty(&raw)?,
    )?;
    fs::write(
        output.join("provenance.json"),
        serde_json::to_vec_pretty(&json!({
            "url":url,"retrieved_at":chrono::Utc::now(),"scope":"Current account customization, NOT historical film state"
        }))?,
    )?;
    let result = &raw["PlayerCustomizations"][0]["Result"];
    let core = result["ArmorCores"]["ArmorCores"]
        .as_array()
        .ok_or("Missing armor cores")?
        .iter()
        .find(|c| c["IsEquipped"] == true)
        .ok_or("No equipped core")?;
    let theme = core["Themes"]
        .as_array()
        .ok_or("Missing themes")?
        .iter()
        .find(|t| t["IsEquipped"] == true)
        .ok_or("No equipped theme")?;
    println!("Current equipped core: {}", core["CorePath"]);
    let mut paths = BTreeSet::new();
    for object in [core, theme] {
        for (key, value) in object.as_object().ok_or("Expected object")? {
            if key.ends_with("Path")
                && let Some(path) = value.as_str().filter(|p| !p.is_empty())
            {
                paths.insert(path.to_string());
            }
        }
    }
    if let Some(file) = args.get(2) {
        let extra: Vec<String> = serde_json::from_slice(&fs::read(file)?)?;
        paths.extend(extra);
    }
    let mut items = Vec::new();
    for (i, path) in paths.iter().enumerate() {
        let url = format!("https://gamecms-hacs.svc.halowaypoint.com/hi/progression/file/{path}");
        let item: Value = halo_api::fetch_waypoint_json(&auth, &http, &url).await?;
        let file = format!("items/{i:03}.json");
        fs::write(output.join(&file), serde_json::to_vec_pretty(&item)?)?;
        println!("{path}: {}", item["CommonData"]["Title"]["value"]);
        items.push(json!({"path":path,"file":file,"url":url}));
    }
    fs::write(
        output.join("items.json"),
        serde_json::to_vec_pretty(&items)?,
    )?;
    Ok(())
}
