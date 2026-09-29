//! Inspects native entity state through Film::parse and Film::resolve.

mod common;

use halo_api::theater::{
    Film,
    film::{ChunkKind, FilmChunk as InputChunk},
};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let input = if let Ok(directory) = std::env::var("HALO_FILM_DIR") {
        local_chunks(&directory)?
    } else {
        let (_, halo) = common::halo_infinite_client()?;
        let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
        let film = halo.match_film(&match_id).await?;
        halo.film_chunks(&film).await?
    };
    let film = Film::parse(input)?;
    let mut resolved = film.resolve();
    println!(
        "{} chunks, {} indexed records",
        1 + film.replication.chunks.len() + film.summaries.chunks.len(),
        resolved.events().len()
    );
    if let Some(end) = resolved.events().last().map(|e| e.timestamp_us) {
        for (key, entity) in &resolved.advance_to(end).entities {
            println!(
                "{key:?}: id {:?}, archetype {}, {} known component updates",
                entity.id,
                entity.archetype,
                entity.components.len()
            );
        }
    }
    Ok(())
}

fn local_chunks(directory: &str) -> Result<Vec<InputChunk>, common::ExampleError> {
    let mut chunks = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name
            .strip_prefix("chunk-")
            .and_then(|name| name.strip_suffix(".bin"))
        else {
            continue;
        };
        let Some((index, chunk_type)) = stem.split_once("-type-") else {
            continue;
        };
        let index = index.parse::<i32>()?;
        let chunk_type = chunk_type.parse::<i32>()?;
        chunks.push(InputChunk {
            kind: ChunkKind::try_from(chunk_type)?,
            index: Some(i64::from(index)),
            // Filenames provide order and category, but no recorded start time.
            start_ms: None,
            data: std::fs::read(entry.path())?,
        });
    }
    chunks.sort_by_key(|chunk| chunk.index);
    Ok(chunks)
}
