//! Decode one downloaded film with the upstream, wasm-compatible Theater module.
//! Usage: cargo run --release --manifest-path experiments/Cargo.toml \
//!   --example decode_theater_film -- <chunk-folder> [output.json] [--compact]
use halo_api::clients::hi::models::{FilmChunk, FilmChunkData};
use halo_api_upstream as halo_api;
use serde::Deserialize;
type ExampleError = Box<dyn std::error::Error>;

use halo_api::theater::{DecodeOptions, Film, Velocity};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

fn main() -> Result<(), ExampleError> {
    let mut args = std::env::args().skip(1);
    let directory = PathBuf::from(
        args.next()
            .ok_or("usage: decode_theater_film <chunk-folder> [output.json] [--compact]")?,
    );
    let mut output = directory.join("decoded-film.json");
    let mut retain_coverage = true;
    let mut output_set = false;
    for arg in args {
        if arg == "--compact" {
            retain_coverage = false;
        } else if !arg.starts_with('-') && !output_set {
            output = PathBuf::from(arg);
            output_set = true;
        } else {
            return Err(format!("unexpected argument: {arg}").into());
        }
    }
    let load = Instant::now();
    let input = load_film(&directory)?;
    let load_time = load.elapsed();
    let start = Instant::now();
    let film = Film::try_from_chunks(
        &input.chunks,
        DecodeOptions {
            major_version: input.film_major_version,
            match_id: Some(input.match_id),
            duration_us: Some(
                u64::try_from(input.film_length)?
                    .checked_mul(1000)
                    .ok_or("film duration overflow")?,
            ),
            retain_coverage,
        },
    )?;
    let decode_time = start.elapsed();
    let export = Instant::now();
    let mut writer = BufWriter::new(File::create(&output)?);
    serde_json::to_writer(&mut writer, &film)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    // Small, seekable evidence table: the inspector can read one chunk without
    // loading a long match's entire typed Film or scanning for byte signatures.
    let velocity_output = output.with_extension("velocity.csv");
    let mut evidence = BufWriter::new(File::create(&velocity_output)?);
    writeln!(
        evidence,
        "player,life,chunk,payload_byte,bit,end_bit,direction_code,magnitude_code"
    )?;
    for player in &film.players {
        for sample in &player.velocities {
            let s = sample.source;
            write!(
                evidence,
                "{},{},{},{},{},{},",
                player.id, sample.life, s.chunk, s.payload_byte, s.bit, s.end_bit
            )?;
            match sample.value {
                Velocity::Stationary => writeln!(evidence, ",")?,
                Velocity::Directed {
                    direction_code,
                    magnitude_code,
                    ..
                } => writeln!(evidence, "{direction_code},{magnitude_code}")?,
            }
        }
    }
    evidence.flush()?;
    let projectile_output = output.with_extension("projectile.csv");
    let mut evidence = csv::Writer::from_path(&projectile_output)?;
    evidence.write_record([
        "track",
        "player",
        "life",
        "chunk",
        "payload_byte",
        "bit",
        "end_bit",
        "kind",
        "x",
        "y",
        "z",
        "direction_code",
        "magnitude_code",
        "rest",
        "x_bits",
        "y_bits",
        "z_bits",
    ])?;
    for (track, projectile) in film.projectiles.iter().enumerate() {
        let life = film
            .players
            .iter()
            .find(|p| p.id == projectile.player)
            .and_then(|p| p.lives.iter().find(|l| l.id == projectile.life))
            .ok_or("Unbound projectile")?;
        let widths = life.layout.axis_bits();
        let row = |source: halo_api::theater::SourceSpan, kind: &str| {
            let mut row = vec![
                track.to_string(),
                projectile.player.to_string(),
                projectile.life.to_string(),
                source.chunk.to_string(),
                source.payload_byte.to_string(),
                source.bit.to_string(),
                source.end_bit.to_string(),
                kind.to_owned(),
            ];
            row.extend([
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ]);
            row.extend(widths.map(|w| w.to_string()));
            row
        };
        for (i, sample) in projectile.positions.iter().enumerate() {
            let mut row = row(sample.source, if i == 0 { "spawn" } else { "position" });
            for (cell, value) in row[8..11].iter_mut().zip(sample.value) {
                *cell = value.to_string();
            }
            evidence.write_record(row)?;
        }
        for sample in &projectile.velocities {
            let mut row = row(sample.source, "velocity");
            if let Velocity::Directed {
                direction_code,
                magnitude_code,
                ..
            } = sample.value
            {
                row[11] = direction_code.to_string();
                row[12] = magnitude_code.to_string();
            }
            evidence.write_record(row)?;
        }
        for sample in &projectile.at_rest {
            let mut row = row(sample.source, "rest");
            row[13] = u8::from(sample.value).to_string();
            evidence.write_record(row)?;
        }
    }
    evidence.flush()?;
    let export_time = export.elapsed();
    let counts = serde_json::json!({
        "summary_events": film.summary_events.len(),
        "medal_awards": film.summary_events.iter().filter(|e| e.medal.is_some()).count(),
        "players": film.players.len(),
        "appearance": film.players.iter().map(|p|p.appearance.len()).sum::<usize>(),
        "lives": film.players.iter().map(|p|p.lives.len()).sum::<usize>(),
        "positions": film.players.iter().map(|p|p.positions.len()).sum::<usize>(),
        "velocities": film.players.iter().map(|p|p.velocities.len()).sum::<usize>(),
        "aim": film.players.iter().map(|p|p.aim.len()).sum::<usize>(),
        "inputs": film.players.iter().map(|p|p.inputs.len()).sum::<usize>(),
        "crouch_input": film.players.iter().map(|p|p.crouch_input.len()).sum::<usize>(),
        "firing": film.players.iter().map(|p|p.firing.len()).sum::<usize>(),
        "damage": film.players.iter().map(|p|p.damage.len()).sum::<usize>(),
        "melee": film.players.iter().map(|p|p.melee.len()).sum::<usize>(),
        "grenades": film.players.iter().map(|p|p.grenades.len()).sum::<usize>(),
        "reloads": film.players.iter().map(|p|p.reloads.len()).sum::<usize>(),
        "magazines": film.players.iter().map(|p|p.magazines.len()).sum::<usize>(),
        "selections": film.players.iter().map(|p|p.selections.len()).sum::<usize>(),
        "zoom": film.players.iter().map(|p|p.zoom.len()).sum::<usize>(),
        "body": film.players.iter().map(|p|p.body.len()).sum::<usize>(),
        "shields": film.players.iter().map(|p|p.shields.len()).sum::<usize>(),
        "projectiles": film.projectiles.len(),
    });
    println!(
        "{}",
        serde_json::json!({"film":format!("{}/{}",input.group,input.slug),"match_id":film.match_id,"output":output,"velocity_evidence":velocity_output,"projectile_evidence":projectile_output,"counts":counts,"load_ms":load_time.as_secs_f64()*1000.,"decode_ms":decode_time.as_secs_f64()*1000.,"export_ms":export_time.as_secs_f64()*1000.})
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
