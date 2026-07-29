//! Downloads a Theater film and prints its decoded player-event timeline.

mod common;

use halo_api::clients::hi::film::FilmEventKind;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let events = halo.match_highlight_events(&match_id).await?;

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
            FilmEventKind::Other(code) => println!(
                "  {seconds:>8.3}s  {:<16} unclassified event code {code} (metadata {})",
                event.gamertag, event.metadata
            ),
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
