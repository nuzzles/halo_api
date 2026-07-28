//! Downloads a Theater film and prints its decoded player-event timeline.

mod common;

use halo_api::clients::hi::film::{
    FilmEventKind, decode_diagnostics, decode_events, decode_players,
};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let film = halo.match_film(&match_id).await?;
    let chunks = halo.film_chunks(&film).await?;
    let players = decode_players(&chunks);
    let events = decode_events(&chunks, &players);

    println!("Decoder diagnostics: {:?}", decode_diagnostics(&chunks));

    println!("Players:");
    for player in &players {
        println!("  {} ({})", player.gamertag, player.xuid);
    }
    println!("Timeline:");
    let mut kills = std::collections::BTreeMap::<String, usize>::new();
    let mut deaths = std::collections::BTreeMap::<String, usize>::new();
    for event in events {
        let seconds = event.timestamp_ms as f64 / 1_000.0;
        match event.kind {
            FilmEventKind::Kill | FilmEventKind::Death | FilmEventKind::Mode => println!(
                "  {seconds:>8.3}s  {:<16} {:?} (metadata {})",
                event.gamertag, event.kind, event.metadata
            ),
            FilmEventKind::Medal => {
                let medal = event.medal().expect("medal event has medal metadata");
                println!(
                    "  {seconds:>8.3}s  {:<16} {} (film medal {})",
                    event.gamertag,
                    medal.name(),
                    event.metadata
                );
            }
            FilmEventKind::Other(_) => {}
        }
        match event.kind {
            FilmEventKind::Kill => *kills.entry(event.gamertag).or_default() += 1,
            FilmEventKind::Death => *deaths.entry(event.gamertag).or_default() += 1,
            _ => {}
        }
    }
    println!("Kills:  {kills:?}");
    println!("Deaths: {deaths:?}");
    Ok(())
}
