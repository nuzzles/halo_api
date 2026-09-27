//! Prints the latest match ID and start/end times using only match history.
//!
//! Uses the same authentication and gamertag lookup as `player_matches`.
//! Set HALO_GAMERTAG or enter it when prompted. Times are printed in UTC.

mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, player) = common::player_xuid(&halo).await?;
    let history = halo.player_matches(&player, 0, 1).await?;
    let latest = history.results.first().ok_or("No recent matches found")?;

    println!("Latest match for {gamertag}:");
    println!("Match ID: {}", latest.match_id);
    println!(
        "Start:    {}",
        latest.info.start_time.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "End:      {}",
        latest.info.end_time.format("%Y-%m-%d %H:%M:%S UTC")
    );
    Ok(())
}
