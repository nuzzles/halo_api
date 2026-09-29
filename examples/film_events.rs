//! Download a v41 film and print its recorded summaries through the new API.
mod common;

use halo_api::theater::Film;
use halo_api::theater::resolved::{EventFilter, EventKind, Record};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let manifest = halo.match_film(&match_id).await?;
    if manifest.custom_data.film_major_version != 41 {
        return Err("only v41 films are supported".into());
    }
    let chunks = halo.film_chunks(&manifest).await?;
    let film = Film::parse(chunks)?;
    let resolved = film.resolve();
    for event in resolved.query(EventFilter {
        kind: Some(EventKind::Summary),
        ..Default::default()
    }) {
        if let Some(Record::Summary(summary)) = resolved.record(event.source) {
            println!(
                "{:.3}s {} type={} medal={}",
                event.timestamp_us as f64 / 1_000_000.0,
                String::from_utf16_lossy(&summary.gamertag_utf16),
                summary.type_code,
                summary.medal_flag,
            );
        }
    }
    Ok(())
}
