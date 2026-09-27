//! Inspects native entity state through Film::parse and Film::resolve.

mod common;

use halo_api::clients::hi::models::{FilmChunk, FilmChunkData};
use halo_api::theater::{Film, FilmSource, FilmSourceMetadata, film::ParseOptions};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (chunks, film_major_version) = if let Ok(directory) = std::env::var("HALO_FILM_DIR") {
        let version = std::env::var("HALO_FILM_MAJOR_VERSION")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(41);
        (local_chunks(&directory)?, version)
    } else {
        let (_, halo) = common::halo_infinite_client()?;
        let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
        let film = halo.match_film(&match_id).await?;
        let version = film.custom_data.film_major_version;
        (halo.film_chunks(&film).await?, version)
    };
    if film_major_version != 41 {
        return Err("only v41 films are supported".into());
    }
    let metadata: Vec<_> = chunks
        .iter()
        .map(|c| FilmSourceMetadata {
            index: i64::from(c.metadata.index),
            chunk_type: i64::from(c.metadata.chunk_type),
            start_ms: c.metadata.start_time_offset_ms,
        })
        .collect();
    let bytes: Vec<_> = chunks.iter().map(|c| c.data.as_slice()).collect();
    let source = FilmSource::load(&bytes, &metadata)?;
    let film = Film::parse(&source, ParseOptions::default())?;
    let mut resolved = film.resolve();
    println!(
        "{} chunks, {} indexed records",
        film.chunks.len(),
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

fn local_chunks(directory: &str) -> Result<Vec<FilmChunkData>, common::ExampleError> {
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
        chunks.push(FilmChunkData {
            metadata: FilmChunk {
                index,
                start_time_offset_ms: i64::from(index.saturating_sub(1)) * 20_000,
                duration_ms: 0,
                size: 0,
                file_relative_path: name,
                chunk_type,
            },
            data: std::fs::read(entry.path())?,
        });
    }
    chunks.sort_by_key(|chunk| chunk.metadata.index);
    Ok(chunks)
}
