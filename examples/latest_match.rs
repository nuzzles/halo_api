//! Prints a gamertag's latest match ID and a short summary.
//! Set HALO_GAMERTAG or enter the gamertag when prompted.

mod common;

use halo_api::clients::hi::models::{GameModeId, MapId};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, player) = common::player_xuid(&halo).await?;
    // History is newest first. Request one entry across all match types.
    let history = halo.player_matches(&player, 0, 1).await?;
    let Some(latest) = history.results.first() else {
        println!("No matches found for {gamertag}.");
        return Ok(());
    };

    println!("Latest match for {gamertag}");
    println!("Match ID: {}", latest.match_id);
    println!("Started (UTC): {}", latest.info.start_time.to_rfc3339());
    println!("Ended (UTC): {}", latest.info.end_time.to_rfc3339());
    println!("Duration (ISO 8601): {}", latest.info.duration);
    println!("Outcome: {}", latest.outcome);
    println!("Rank: {} | Team: {}", latest.rank, latest.last_team_id);
    println!("Present at end: {}", latest.present_at_end);

    // Metadata is optional; a failed lookup should not hide the match ID.
    if let Some(link) = &latest.info.map_variant {
        match halo.map(MapId::new(&link.asset_id, &link.version_id)).await {
            Ok(map) => println!("Map: {}", map.asset.public_name),
            Err(error) => eprintln!("Map metadata unavailable: {error}"),
        }
    }
    if let Some(link) = &latest.info.ugc_game_variant {
        match halo
            .mode(GameModeId::new(&link.asset_id, &link.version_id))
            .await
        {
            Ok(mode) => println!("Mode: {}", mode.asset.public_name),
            Err(error) => eprintln!("Mode metadata unavailable: {error}"),
        }
    }
    Ok(())
}
