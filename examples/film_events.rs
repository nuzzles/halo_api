//! Download a v41 film and print typed, source-linked summary interpretations.
mod common;
use std::time::Instant;

use halo_api::theater::{Film, TheaterRuntime};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    println!("Downloading film {match_id}...");
    let download_started = Instant::now();
    let download = async {
        let manifest = halo.match_film(&match_id).await?;
        halo.film_chunks(&manifest).await
    }
    .await;
    let chunks = match download {
        Ok(chunks) => chunks,
        Err(error) => {
            eprintln!(
                "Download stopped after {:.3}s: {error}",
                download_started.elapsed().as_secs_f64()
            );
            return Err(error.into());
        }
    };
    let bytes: usize = chunks.iter().map(|chunk| chunk.data.len()).sum();
    println!(
        "Download complete: {} chunks, {bytes} bytes in {:.3}s.",
        chunks.len(),
        download_started.elapsed().as_secs_f64()
    );

    println!("Parsing film...");
    let parse_started = Instant::now();
    let film = Film::parse(chunks)?;
    println!(
        "Film parsed in {:.3}s.",
        parse_started.elapsed().as_secs_f64()
    );

    println!("Resolving film...");
    let resolve_started = Instant::now();
    let runtime = TheaterRuntime::load(film);
    println!(
        "Film resolved in {:.3}s: {} events, {} summary events.",
        resolve_started.elapsed().as_secs_f64(),
        runtime.events().len(),
        runtime.summary_events().len()
    );
    println!("Latest 20 summary events:");
    for event in runtime.summary_events().iter().rev().take(20) {
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
