//! Summarizes the logged-in player's career record and five most recent matches.

mod common;

use halo_api::clients::hi::Player;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (xbox, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::logged_in_player(&xbox).await?;
    let player = Player::from(&xuid);
    let (record, history) = tokio::try_join!(
        halo.service_record(&player),
        halo.player_matches(&player, 0, 5),
    )?;

    println!("{gamertag} (xuid {xuid})");
    println!(
        "Career: {}-{}-{} | K/D/A: {}/{}/{} | accuracy: {:.2}% | played: {}",
        record.wins,
        record.losses,
        record.ties,
        record.core_stats.kills,
        record.core_stats.deaths,
        record.core_stats.assists,
        record.core_stats.accuracy,
        record.time_played,
    );
    println!("Recent matches:");
    for entry in history.results {
        println!(
            "{} | outcome {} | rank {} | team {} | {}",
            entry.info.start_time, entry.outcome, entry.rank, entry.last_team_id, entry.match_id,
        );
    }
    Ok(())
}
