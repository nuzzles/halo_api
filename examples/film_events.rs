//! Download a v41 film and print its recorded summaries through the new API.
mod common;

use halo_api::theater::resolved::{EventFilter, EventKind, Record};
use halo_api::theater::{
    Film,
    film::{ChunkKind, FilmChunk as InputChunk},
};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let manifest = halo.match_film(&match_id).await?;
    if manifest.custom_data.film_major_version != 41 {
        return Err("only v41 films are supported".into());
    }
    let chunks = halo.film_chunks(&manifest).await?;
    let input = chunks
        .into_iter()
        .map(|c| {
            Ok(InputChunk {
                kind: ChunkKind::try_from(c.metadata.chunk_type)?,
                index: Some(i64::from(c.metadata.index)),
                start_ms: Some(c.metadata.start_time_offset_ms),
                data: c.data,
            })
        })
        .collect::<Result<Vec<_>, halo_api::theater::film::ParseError>>()?;
    let film = Film::parse(input)?;
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
