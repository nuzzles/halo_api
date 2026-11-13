//! Download a v41 film and print its typed observations and guarded highlights.
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

    let runtime = TheaterRuntime::load(film);
    println!(
        "Film contains {} observations and {} summary highlights.",
        runtime.observations().len(),
        runtime.summary_highlights().len()
    );
    println!("Latest 20 summary highlights:");
    for event in runtime.summary_highlights().iter().rev().take(20) {
        println!(
            "{} at {} us: {:?} ({})",
            event.gamertag, event.timestamp_us, event.payload, event.xuid
        );
    }
    Ok(())
}
