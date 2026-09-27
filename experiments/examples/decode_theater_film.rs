//! Export a source-faithful v41 Film and inspect its resolved record index.
//! Usage: cargo run --release --manifest-path experiments/Cargo.toml \
//!   --example decode_theater_film -- <chunk-folder> [output.json]
use halo_api::clients::hi::models::{FilmChunk, FilmChunkData};
use halo_api::theater::{Film, FilmSource, FilmSourceMetadata, film::ParseOptions};
use halo_api_upstream as halo_api;
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};
type ExampleError = Box<dyn std::error::Error>;

fn main() -> Result<(), ExampleError> {
    let mut args = std::env::args().skip(1);
    let directory = PathBuf::from(
        args.next()
            .ok_or("usage: decode_theater_film <chunk-folder> [output.json]")?,
    );
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| directory.join("decoded-film.json"));
    if args.next().is_some() || output.to_string_lossy().starts_with("--") {
        return Err("only a directory and optional output path are supported; native exports always retain source bytes".into());
    }
    let input = load_film(&directory)?;
    if input.film_major_version != 41 {
        return Err("only v41 films are supported".into());
    }
    let metadata: Vec<_> = input
        .chunks
        .iter()
        .map(|c| FilmSourceMetadata {
            index: i64::from(c.metadata.index),
            chunk_type: i64::from(c.metadata.chunk_type),
            start_ms: c.metadata.start_time_offset_ms,
        })
        .collect();
    let bytes: Vec<_> = input.chunks.iter().map(|c| c.data.as_slice()).collect();
    let source = FilmSource::load(&bytes, &metadata)?;
    let film = Film::parse(&source, ParseOptions::default())?;
    let resolved = film.resolve();
    let mut writer = BufWriter::new(File::create(&output)?);
    serde_json::to_writer(&mut writer, &film)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    println!(
        "{}",
        serde_json::json!({"film":format!("{}/{}",input.group,input.slug),
        "match_id":input.match_id,"duration_ms":input.film_length,"output":output,
        "chunks":film.chunks.len(),"indexed_records":resolved.events().len()})
    );
    Ok(())
}

#[derive(Deserialize)]
struct Download {
    film_major_version: i32,
    film_length: i64,
    match_id: String,
    #[serde(default)]
    group: String,
    #[serde(default)]
    slug: String,
    chunks: Vec<CachedChunk>,
}
#[derive(Deserialize)]
struct CachedChunk {
    index: i32,
    chunk_type: i32,
    start_time_offset_ms: i64,
    duration_ms: i64,
    compressed_size: i64,
    file: String,
}
struct Input {
    film_major_version: i32,
    film_length: i64,
    match_id: String,
    group: String,
    slug: String,
    chunks: Vec<FilmChunkData>,
}
fn load_film(directory: &std::path::Path) -> Result<Input, ExampleError> {
    let metadata: Download = serde_json::from_reader(File::open(directory.join("film.json"))?)?;
    let mut chunks = Vec::with_capacity(metadata.chunks.len());
    for c in metadata.chunks {
        if !std::path::Path::new(&c.file)
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
        {
            return Err("chunk file must be relative to the selected folder".into());
        }
        let data = std::fs::read(directory.join(&c.file))?;
        chunks.push(FilmChunkData {
            metadata: FilmChunk {
                index: c.index,
                chunk_type: c.chunk_type,
                start_time_offset_ms: c.start_time_offset_ms,
                duration_ms: c.duration_ms,
                size: c.compressed_size,
                file_relative_path: c.file,
            },
            data,
        });
    }
    Ok(Input {
        film_major_version: metadata.film_major_version,
        film_length: metadata.film_length,
        match_id: metadata.match_id,
        group: metadata.group,
        slug: metadata.slug,
        chunks,
    })
}
