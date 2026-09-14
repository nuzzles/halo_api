//! Downloads only the version-41 footer and prints the recorded event timeline.
mod common;

use halo_api::theater::SummaryKind;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let report = halo.match_summary_events_with_validation(&match_id).await?;
    println!("Timeline:");
    for event in &report.summary.events {
        let seconds = event.time_us as f64 / 1_000_000.;
        match &event.medal {
            Some(medal) => println!(
                "  {seconds:>8.3}s  {:<16} {} (film code {}, NameId {:?})",
                event.name,
                medal.name.as_deref().unwrap_or("Unknown medal"),
                medal.film_id,
                medal.name_id,
            ),
            None => println!(
                "  {seconds:>8.3}s  {:<16} {:?} (type {:?}, metadata {})",
                event.name, event.kind, event.type_code, event.metadata,
            ),
        }
    }
    let count = |kind| {
        report
            .summary
            .events
            .iter()
            .filter(|e| e.kind == kind)
            .count()
    };
    println!(
        "Kills: {}, deaths: {}, medals: {}, mode events: {}",
        count(SummaryKind::Kill),
        count(SummaryKind::Death),
        count(SummaryKind::Medal),
        count(SummaryKind::Mode)
    );
    println!(
        "Matches declared footer count: {}",
        report.summary.matches_declared_counts()
    );
    println!(
        "Matches per-player stats and medal identities: {}",
        report.validation.matches_stats()
    );
    for player in report
        .validation
        .players
        .iter()
        .filter(|p| !p.matches_stats())
    {
        println!("Mismatch: {player:?}");
    }
    Ok(())
}
