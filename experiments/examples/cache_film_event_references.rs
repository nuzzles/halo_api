//! Cache official medal definitions and match stats for offline event validation.
//! Usage: cargo run --release --example cache_film_event_references -- group/slug ...
mod common;

use halo_api::auth::HaloAuthClient;
use std::{fs, path::PathBuf, time::Duration};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("films");
    let selected: Vec<_> = std::env::args().skip(1).collect();
    if selected.is_empty() {
        return Err("Specify downloaded film folders as group/slug".into());
    }
    let mut requests = vec![(
        root.join("analysis/summary-events/medal-metadata.json"),
        "https://gamecms-hacs.svc.halowaypoint.com/hi/Waypoint/file/medals/metadata.json"
            .to_owned(),
    )];
    for selected in selected {
        let parts: Vec<_> = selected.split('/').collect();
        if parts.len() != 2
            || parts
                .iter()
                .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
        {
            return Err("Expected a catalog group/slug".into());
        }
        let directory = root.join(selected);
        let film: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("film.json"))?)?;
        let id = film["match_id"].as_str().ok_or("Missing match ID")?;
        if id.len() != 36 || !id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
            return Err("Invalid match ID".into());
        }
        requests.push((
            directory.join("settings/match-stats.json"),
            format!("https://halostats.svc.halowaypoint.com/hi/matches/{id}/stats"),
        ));
    }
    let timeout = Duration::from_secs(120);
    let auth = HaloAuthClient::from_xbox_client_with_timeout(common::xbox_client()?, timeout);
    let http = reqwest::Client::builder().timeout(timeout).build()?;
    for (path, url) in requests {
        if path.exists() {
            println!("Cached {}", path.display());
            continue;
        }
        let value = halo_api::fetch_waypoint_json(&auth, &http, &url).await?;
        fs::create_dir_all(path.parent().ok_or("Missing parent")?)?;
        fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
        println!("Saved {}", path.display());
    }
    Ok(())
}
