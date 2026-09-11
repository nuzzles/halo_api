//! Cache the exact map revisions referenced by the tracked films' match history.
//! Run from any directory; uses films.csv and films/ beside this experiment crate.
mod common;

use halo_api::auth::HaloAuthClient;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

#[derive(serde::Deserialize)]
struct Entry {
    group: String,
    slug: String,
    match_id: String,
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let entries: Vec<Entry> = csv::Reader::from_path(root.join("films.csv"))?
        .deserialize()
        .collect::<Result<_, _>>()?;
    let mut wanted = BTreeMap::new();
    for e in &entries {
        for segment in [&e.group, &e.slug] {
            if !segment
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            {
                return Err("Invalid catalog directory".into());
            }
        }
        let dir = root.join("films").join(&e.group).join(&e.slug);
        if dir.join("film.json").exists() {
            wanted.insert(e.match_id.clone(), dir);
        }
    }
    let timeout = Duration::from_secs(120);
    let xbox = common::xbox_client()?;
    let xuid = common::logged_in_player(&xbox).await?.1.to_string();
    let auth = HaloAuthClient::from_xbox_client_with_timeout(xbox, timeout);
    let http = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let mut history = BTreeMap::<String, Value>::new();
    for (id, dir) in &wanted {
        let file = dir.join("settings/match-history-entry.json");
        if file.exists() {
            let entry: Value = serde_json::from_slice(&fs::read(file)?)?;
            if entry["MatchId"].as_str() != Some(id) {
                return Err("Cached match mismatch".into());
            }
            history.insert(id.clone(), entry);
        }
    }
    for start in (0..1000).step_by(25) {
        if history.len() == wanted.len() {
            break;
        }
        println!("Reading match history {start}..{}", start + 25);
        let url = format!(
            "https://halostats.svc.halowaypoint.com/hi/players/xuid({xuid})/matches?start={start}&count=25&type=all"
        );
        let page = halo_api::fetch_waypoint_json(&auth, &http, &url).await?;
        let rows = page["Results"].as_array().ok_or("Missing Results")?;
        for r in rows {
            if let Some(id) = r["MatchId"].as_str()
                && wanted.contains_key(id)
            {
                let dir = wanted[id].join("settings");
                fs::create_dir_all(&dir)?;
                fs::write(
                    dir.join("match-history-entry.json"),
                    serde_json::to_vec_pretty(r)?,
                )?;
                history.insert(id.to_owned(), r.clone());
            }
        }
        if rows.len() < 25 {
            break;
        }
    }
    let mut maps = BTreeMap::<String, Value>::new();
    for (id, dir) in &wanted {
        let Some(entry) = history.get(id) else {
            println!("No cached/history reference for {id}");
            continue;
        };
        let map = &entry["MatchInfo"]["MapVariant"];
        let asset = map["AssetId"].as_str().ok_or("Missing map asset")?;
        let version = map["VersionId"].as_str().ok_or("Missing map revision")?;
        let url = format!(
            "https://discovery-infiniteugc.svc.halowaypoint.com/hi/maps/{asset}/versions/{version}"
        );
        let file = dir.join("settings/map-variant.json");
        let raw = if file.exists() {
            serde_json::from_slice(&fs::read(&file)?)?
        } else if let Some(raw) = maps.get(&url) {
            raw.clone()
        } else {
            halo_api::fetch_waypoint_json(&auth, &http, &url).await?
        };
        if raw["AssetId"].as_str() != Some(asset) || raw["VersionId"].as_str() != Some(version) {
            return Err(format!("Map revision mismatch for {id}").into());
        }
        fs::write(&file, serde_json::to_vec_pretty(&raw)?)?;
        fs::write(
            dir.join("settings/map-provenance.json"),
            serde_json::to_vec_pretty(&json!({
                "match_id":id,"level_id":entry["MatchInfo"]["LevelId"],"map_url":url,
                "retrieved_at":chrono::Utc::now(),"scope":"Exact referenced map revision; does not imply map geometry or bounds are in the film"
            }))?,
        )?;
        println!(
            "{}: {} · level {}",
            dir.display(),
            raw["PublicName"],
            entry["MatchInfo"]["LevelId"]
        );
        maps.insert(url, raw);
    }
    println!(
        "Cached exact map references for {} of {} downloaded films; missing entries may be older than the first 1000 history results.",
        history.len(),
        wanted.len()
    );
    Ok(())
}
