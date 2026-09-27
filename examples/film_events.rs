//! Download a v41 film and print its recorded summaries through the new API.
mod common;

use halo_api::theater::resolved::{EventFilter, EventKind, Record};
use halo_api::theater::{Film, FilmSource, FilmSourceMetadata, film::ParseOptions};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let manifest = halo.match_film(&match_id).await?;
    if manifest.custom_data.film_major_version != 41 {
        return Err("only v41 films are supported".into());
    }
    let chunks = halo.film_chunks(&manifest).await?;
    let metadata: Vec<_> = chunks
        .iter()
        .map(|chunk| FilmSourceMetadata {
            index: i64::from(chunk.metadata.index),
            chunk_type: i64::from(chunk.metadata.chunk_type),
            start_ms: chunk.metadata.start_time_offset_ms,
        })
        .collect();
    let bytes: Vec<_> = chunks.iter().map(|chunk| chunk.data.as_slice()).collect();
    let source = FilmSource::load(&bytes, &metadata)?;
    let film = Film::parse(&source, ParseOptions::default())?;
    let resolved = film.resolve();
    for event in resolved.query(EventFilter {
        kind: Some(EventKind::Summary),
        ..Default::default()
    }) {
        if let Some(Record::Summary(summary)) = resolved.record(event.source) {
            println!(
                "{:.3}s {} {:?}",
                event.timestamp_us as f64 / 1_000_000.0,
                summary.name,
                summary.kind
            );
        }
    }
    Ok(())
}
