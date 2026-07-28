//! Prints a Discord-ready report for a gamertag's most recent match.

mod common;

use std::collections::HashMap;

use halo_api::clients::hi::models::{GameModeId, MapId, MatchType};
use xbox::models::Xuid;

fn bare_xuid(player_id: &str) -> &str {
    player_id
        .strip_prefix("xuid(")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(player_id)
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let gamertag = common::value("HALO_GAMERTAG", "Gamertag")?;
    let (_, halo) = common::halo_infinite_client()?;
    let user = halo.user(&gamertag).await?;
    let xuid = Xuid::from(user.xuid.clone());

    let history = halo
        .player_matches_by_type(&xuid, MatchType::Custom, 0, 1)
        .await?;
    let latest = history
        .results
        .first()
        .ok_or("No recent custom matches found")?;
    let stats = halo.match_stats(&latest.match_id).await?;

    let player = stats
        .players
        .iter()
        .find(|player| bare_xuid(&player.player_id) == user.xuid)
        .ok_or("Player was not present in the match stats")?;
    let core = player
        .team_stats
        .first()
        .map(|team| &team.stats.core)
        .ok_or("Player did not have match stats")?;

    let map = if let Some(link) = &stats.info.map_variant {
        halo.map(MapId::new(&link.asset_id, &link.version_id))
            .await
            .ok()
    } else {
        None
    };
    let mode = if let Some(link) = &stats.info.ugc_game_variant {
        halo.mode(GameModeId::new(&link.asset_id, &link.version_id))
            .await
            .ok()
    } else {
        None
    };

    let human_xuids = stats
        .players
        .iter()
        .filter(|player| player.player_type == 1)
        .map(|player| Xuid::from(bare_xuid(&player.player_id)))
        .collect::<Vec<_>>();
    let names = halo
        .users(&human_xuids)
        .await?
        .into_iter()
        .map(|user| (user.xuid, user.gamertag))
        .collect::<HashMap<_, _>>();

    let map_name = map
        .as_ref()
        .map(|map| map.asset.public_name.as_str())
        .unwrap_or("Unknown map");
    let mode_name = mode
        .as_ref()
        .map(|mode| mode.asset.public_name.as_str())
        .unwrap_or("Unknown mode");
    let kd = if core.deaths == 0 {
        core.kills as f64
    } else {
        core.kills as f64 / core.deaths as f64
    };

    println!("## {} — {}", latest.outcome, user.gamertag);
    println!("**{map_name} · {mode_name}**");
    println!("{} · {}", stats.info.start_time, stats.info.duration);
    println!();
    println!(
        "**K/D/A:** {}/{}/{}  ·  **K/D:** {:.2}  ·  **Accuracy:** {:.1}%",
        core.kills, core.deaths, core.assists, kd, core.accuracy
    );
    println!(
        "**Score:** {}  ·  **Damage:** {} dealt / {} taken  ·  **Rank:** {}",
        core.personal_score, core.damage_dealt, core.damage_taken, latest.rank
    );
    println!();
    println!("```text");
    println!("RK  TEAM  PLAYER                    SCORE   OBJ   K   D   A");
    let mut players = stats.players.iter().collect::<Vec<_>>();
    players.sort_by_key(|player| player.rank);
    for player in players {
        let Some(core) = player.team_stats.first().map(|team| &team.stats.core) else {
            continue;
        };
        let name = names
            .get(bare_xuid(&player.player_id))
            .map(String::as_str)
            .unwrap_or(&player.player_id);
        println!(
            "{:<3} {:<5} {:<24} {:>5} {:>5} {:>3} {:>3} {:>3}",
            player.rank,
            player.last_team_id,
            name.chars().take(24).collect::<String>(),
            core.personal_score,
            core.score,
            core.kills,
            core.deaths,
            core.assists
        );
    }
    println!("```");
    if let Some(hero) = map.as_ref().and_then(|map| map.hero_url()) {
        println!("Map image: {hero}");
    }
    println!("Match ID: {}", stats.match_id);

    Ok(())
}
