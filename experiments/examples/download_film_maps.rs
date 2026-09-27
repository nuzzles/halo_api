//! Cache the exact map revisions referenced by the tracked films' match history.
//! Run from any directory; uses films.csv and films/ beside this experiment crate.
//! --assets also caches referenced .mvar files. HALO_PROBE_FILM selects group/slug.
//! --navmesh caches a referenced navmesh.blob when present, independently of --assets.
//! HALO_PLAYER_XUID selects a participant's history instead of the signed-in account.
mod common;

use halo_api::auth::HaloAuthClient;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

async fn download_assets(
    http: &reqwest::Client,
    metadata: &Value,
    settings: &std::path::Path,
    placements: bool,
    navmesh: bool,
) -> Result<(), common::ExampleError> {
    const LIMIT: usize = 64 * 1024 * 1024;
    let prefix = metadata["Files"]["Prefix"]
        .as_str()
        .ok_or("Missing asset prefix")?;
    let expected = format!(
        "https://blobs-infiniteugc.svc.halowaypoint.com/ugcstorage/map/{}/{}/",
        metadata["AssetId"].as_str().ok_or("Missing asset ID")?,
        metadata["VersionId"].as_str().ok_or("Missing version ID")?
    );
    if prefix != expected {
        return Err("Unexpected map asset host/revision path".into());
    }
    for name in metadata["Files"]["FileRelativePaths"]
        .as_array()
        .ok_or("Missing asset paths")?
        .iter()
        .filter_map(Value::as_str)
        .filter(|s| (placements && s.ends_with(".mvar")) || (navmesh && *s == "navmesh.blob"))
    {
        if !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err("Unexpected map asset filename".into());
        }
        let target = settings.join(name);
        if target.exists() {
            println!("Cached {}", target.display());
            continue;
        }
        let mut response = http
            .get(format!("{prefix}{name}"))
            .send()
            .await?
            .error_for_status()?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if chunk.len() > LIMIT - bytes.len() {
                return Err("Map asset exceeds 64 MiB experiment limit".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        if bytes.is_empty() {
            return Err("Empty map asset response".into());
        }
        fs::write(&target, &bytes)?;
        println!("Downloaded {} ({} bytes)", target.display(), bytes.len());
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct Entry {
    group: String,
    slug: String,
    match_id: String,
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a != "--assets" && a != "--navmesh") {
        return Err(
            "Usage: download_film_maps [--assets] [--navmesh]; HALO_PROBE_FILM=group/slug".into(),
        );
    }
    let assets = args.iter().any(|a| a == "--assets");
    let navmesh = args.iter().any(|a| a == "--navmesh");
    let selected = std::env::var("HALO_PROBE_FILM").ok();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let entries: Vec<Entry> = csv::Reader::from_path(root.join("films.csv"))?
        .deserialize()
        .collect::<Result<_, _>>()?;
    let mut wanted = BTreeMap::new();
    for e in &entries {
        if selected
            .as_ref()
            .is_some_and(|s| s != &format!("{}/{}", e.group, e.slug))
        {
            continue;
        }
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
    if wanted.is_empty() {
        return Err("No downloaded catalog films match the selection".into());
    }
    let timeout = Duration::from_secs(120);
    let xbox = common::xbox_client()?;
    let xuid = match std::env::var("HALO_PLAYER_XUID") {
        Ok(value) => {
            let parsed = value.parse::<u64>()?;
            if parsed == 0 {
                return Err("HALO_PLAYER_XUID must be a positive numeric XUID".into());
            }
            parsed.to_string()
        }
        Err(std::env::VarError::NotPresent) => common::logged_in_player(&xbox).await?.1.to_string(),
        Err(error) => return Err(error.into()),
    };
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
        if assets || navmesh {
            if navmesh
                && !raw["Files"]["FileRelativePaths"]
                    .as_array()
                    .is_some_and(|paths| paths.iter().any(|p| p.as_str() == Some("navmesh.blob")))
            {
                println!("No navmesh.blob listed for this map revision");
            }
            download_assets(&http, &raw, &dir.join("settings"), assets, navmesh).await?;
        }
        maps.insert(url, raw);
    }
    println!(
        "Cached exact map references for {} of {} downloaded films. Missing entries may belong to another player (set HALO_PLAYER_XUID) or fall outside the first 1000 history results.",
        history.len(),
        wanted.len()
    );
    Ok(())
}
