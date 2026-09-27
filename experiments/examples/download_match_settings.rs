//! Capture the exact variant revision linked by match history, without losing unknown JSON fields.
//! Usage: cargo run --example download_match_settings -- MATCH_ID OUTPUT_DIRECTORY [PLAYER_XUID]
//! This captures variant defaults; lobby overrides are not established by the asset alone.

mod common;

use halo_api::auth::HaloAuthClient;
use serde_json::{Value, json};
use std::{fs, path::PathBuf, time::Duration};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err(
            "Usage: download_match_settings MATCH_ID OUTPUT_DIRECTORY [PLAYER_XUID]".into(),
        );
    }
    let match_id = &args[0];
    let output = PathBuf::from(&args[1]);
    fs::create_dir_all(&output)?;
    let timeout = Duration::from_secs(
        std::env::var("HALO_TIMEOUT_SECS")
            .unwrap_or("120".into())
            .parse()?,
    );
    if timeout.is_zero() {
        return Err("HALO_TIMEOUT_SECS must be positive".into());
    }
    let xbox = common::xbox_client()?;
    let xuid = match args.get(2) {
        Some(xuid) => xuid.clone(),
        None => common::logged_in_player(&xbox).await?.1.to_string(),
    };
    let auth = HaloAuthClient::from_xbox_client_with_timeout(xbox, timeout);
    let http = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let cache = output.join("match-history-entry.json");
    let entry: Value = if cache.exists() {
        let value: Value = serde_json::from_slice(&fs::read(&cache)?)?;
        if value["MatchId"].as_str() != Some(match_id) {
            return Err("Cached match ID differs".into());
        }
        value
    } else {
        let mut found = None;
        for start in (0..1000).step_by(25) {
            println!("Searching match history {start}..{}", start + 25);
            let page = halo_api::fetch_waypoint_json(&auth, &http, &format!("https://halostats.svc.halowaypoint.com/hi/players/xuid({xuid})/matches?start={start}&count=25&type=all")).await?;
            let rows = page["Results"].as_array().ok_or("Missing Results")?;
            if let Some(entry) = rows
                .iter()
                .find(|r| r["MatchId"].as_str() == Some(match_id))
            {
                found = Some(entry.clone());
                break;
            }
            if rows.len() < 25 {
                break;
            }
        }
        let entry =
            found.ok_or("Match not in first 1000 history entries; supply a participant's XUID")?;
        fs::write(&cache, serde_json::to_vec_pretty(&entry)?)?;
        entry
    };
    let variant = &entry["MatchInfo"]["UgcGameVariant"];
    let asset_id = variant["AssetId"]
        .as_str()
        .ok_or("No game variant asset ID")?;
    let version_id = variant["VersionId"]
        .as_str()
        .ok_or("No game variant version ID")?;
    let url = format!(
        "https://discovery-infiniteugc.svc.halowaypoint.com/hi/ugcGameVariants/{asset_id}/versions/{version_id}"
    );
    println!("Retrieving exact game variant {asset_id} / {version_id}");
    let raw = halo_api::fetch_waypoint_json(&auth, &http, &url).await?;
    fs::write(
        output.join("game-variant.json"),
        serde_json::to_vec_pretty(&raw)?,
    )?;
    fs::write(
        output.join("provenance.json"),
        serde_json::to_vec_pretty(&json!({
            "match_id": match_id, "variant_url": url, "retrieved_at": chrono::Utc::now(),
            "scope": "Exact referenced UGC variant revision; per-match lobby overrides unverified"
        }))?,
    )?;
    println!(
        "Name: {}",
        raw["PublicName"].as_str().unwrap_or("<unknown>")
    );
    println!(
        "Saved raw game-variant.json and match-history-entry.json to {}",
        output.display()
    );
    let mut csv = csv::Writer::from_path(output.join("variant-overrides.csv"))?;
    csv.write_record(["key", "raw_value_json", "asset_id", "version_id"])?;
    if let Some(values) = raw["CustomData"]["KeyValues"].as_object() {
        for (key, value) in values {
            csv.write_record([key.as_str(), &value.to_string(), asset_id, version_id])?;
        }
        println!(
            "Saved {} raw overrides to variant-overrides.csv",
            values.len()
        );
    }
    csv.flush()?;
    if let (Some(engine_id), Some(engine_version)) = (
        raw["EngineGameVariantLink"]["AssetId"].as_str(),
        raw["EngineGameVariantLink"]["VersionId"].as_str(),
    ) {
        let engine_url = format!(
            "https://discovery-infiniteugc.svc.halowaypoint.com/hi/engineGameVariants/{engine_id}/versions/{engine_version}"
        );
        println!("Retrieving exact engine variant {engine_id} / {engine_version}");
        let engine = halo_api::fetch_waypoint_json(&auth, &http, &engine_url).await?;
        fs::write(
            output.join("engine-variant.json"),
            serde_json::to_vec_pretty(&engine)?,
        )?;
        fs::write(
            output.join("engine-provenance.json"),
            serde_json::to_vec_pretty(&json!({
                "url": engine_url, "retrieved_at": chrono::Utc::now(),
                "source": "game-variant.json EngineGameVariantLink"
            }))?,
        )?;
        println!("Engine name: {}", engine["PublicName"]);
        // Download the engine binary and English menu/localization files only.
        // Blob requests have no authentication headers; file URLs come from this revision.
        if let (Some(prefix), Some(paths)) = (
            engine["Files"]["Prefix"].as_str(),
            engine["Files"]["FileRelativePaths"].as_array(),
        ) {
            let mut captured = Vec::new();
            for name in paths.iter().filter_map(Value::as_str) {
                if !(name.ends_with(".en")
                    || name.ends_with("_guid.txt")
                    || name.ends_with(".bin") && (!name.contains('/') || name.ends_with("_en.bin")))
                {
                    continue;
                }
                if !std::path::Path::new(name)
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_)))
                {
                    return Err("Unsafe asset relative path".into());
                }
                let file_url = format!("{}/{}", prefix.trim_end_matches('/'), name);
                let parsed = reqwest::Url::parse(&file_url)?;
                if parsed.scheme() != "https"
                    || parsed.host_str() != Some("blobs-infiniteugc.svc.halowaypoint.com")
                {
                    return Err("Unexpected engine file origin".into());
                }
                let bytes = http
                    .get(parsed)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?;
                let destination = output.join("engine-files").join(name);
                fs::create_dir_all(destination.parent().ok_or("Missing file parent")?)?;
                fs::write(&destination, &bytes)?;
                println!("Saved {name}: {} bytes", bytes.len());
                captured.push(json!({"file": name, "url": file_url, "bytes": bytes.len()}));
            }
            fs::write(
                output.join("engine-files.json"),
                serde_json::to_vec_pretty(&captured)?,
            )?;
        }
    }
    Ok(())
}
