//! Prints the most recent match scoreboard for the logged-in player.

mod common;

use std::collections::HashMap;

use halo_api::clients::hi::Player;

fn bare_xuid(player_id: &str) -> &str {
    player_id
        .strip_prefix("xuid(")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(player_id)
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (xbox, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::logged_in_player(&xbox).await?;
    let history = halo.player_matches(&Player::from(&xuid), 0, 1).await?;
    let latest = history.results.first().ok_or("No recent matches found")?;
    let stats = halo.match_stats(&latest.match_id).await?;

    let players = stats
        .players
        .iter()
        .filter(|player| player.player_type == 1)
        .map(|player| Player::xuid(bare_xuid(&player.player_id)))
        .collect::<Vec<_>>();
    let names = halo
        .users(&players)
        .await?
        .into_iter()
        .map(|user| (user.xuid, user.gamertag))
        .collect::<HashMap<_, _>>();

    println!("Latest match for {gamertag}: {}", stats.match_id);
    println!("rank  team  player                       score   K   D   A");
    let mut players = stats.players.iter().collect::<Vec<_>>();
    players.sort_by_key(|player| player.rank);
    for player in players {
        let core = player.team_stats.first().map(|stats| &stats.stats.core);
        let name = names
            .get(bare_xuid(&player.player_id))
            .map(String::as_str)
            .unwrap_or(&player.player_id);
        println!(
            "{:<5} {:<5} {:<28} {:>5} {:>3} {:>3} {:>3}",
            player.rank,
            player.last_team_id,
            name,
            core.map_or(0, |stats| stats.personal_score),
            core.map_or(0, |stats| stats.kills),
            core.map_or(0, |stats| stats.deaths),
            core.map_or(0, |stats| stats.assists),
        );
    }
    Ok(())
}
