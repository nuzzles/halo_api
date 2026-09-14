//! Prints recorded lives and positions from the current typed Theater decoder.

mod common;

use halo_api::clients::hi::models::{FilmChunk, FilmChunkData};
use halo_api::theater::{DecodeOptions, Film};

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
    let film = Film::try_from_chunks(
        &chunks,
        DecodeOptions {
            major_version: film_major_version,
            ..DecodeOptions::v41()
        },
    )?;
    println!(
        "{} packets, {} players, {} summary events",
        film.packets.len(),
        film.players.len(),
        film.summary_events.len()
    );
    for player in &film.players {
        println!(
            "player {:<16} xuid {:?} -> roster {}: {} lives, {} positions",
            player.name,
            player.xuid,
            player.id,
            player.lives.len(),
            player.positions.len()
        );
        for life in player.lives.iter().take(25) {
            let position = player.positions.iter().find(|p| p.life == life.id);
            println!(
                "  life {} spawn {:.3}s, death {:?}s, first position {:?}",
                life.id,
                life.start_us as f64 / 1_000_000.,
                life.death_us.map(|t| t as f64 / 1_000_000.),
                position.map(|p| &p.value)
            );
        }
    }
    for limitation in &film.diagnostics.limitations {
        eprintln!("Limit: {limitation}");
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
