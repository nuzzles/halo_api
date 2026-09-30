//! Download a v41 film and print typed, source-linked summary interpretations.
mod common;
use halo_api::theater::{Film, TheaterRuntime};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let manifest = halo.match_film(&match_id).await?;
    let runtime = TheaterRuntime::load(Film::parse(halo.film_chunks(&manifest).await?)?);
    for event in runtime.summary_events() {
        println!(
            "{event} [{:?}; {:?}]",
            event.derivation, event.actor.roster_link
        );
    }
    for report in runtime.summary_reports() {
        if report.count_matches() != Some(true) {
            println!(
                "summary coverage {:?}: {:?} declared, {} candidates",
                report.source, report.declared_events, report.candidate_events
            );
        }
    }
    Ok(())
}
