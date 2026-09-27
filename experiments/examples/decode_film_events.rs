//! Decode cached footer chunks and optionally validate against cached match stats.
//! Usage: cargo run --release --example decode_film_events -- <film-folder> ...
use halo_api_upstream::clients::hi::models::{FilmChunk, FilmChunkData, MatchStats};
use halo_api_upstream::theater::{
    decode_summary_events, film_medal_definition, validate_summary_events,
};
use serde::Deserialize;
use std::{fs, path::PathBuf, time::Instant};

#[derive(Deserialize)]
struct Manifest {
    film_major_version: i32,
    match_id: String,
    chunks: Vec<Chunk>,
}
#[derive(Deserialize)]
struct Chunk {
    index: i32,
    chunk_type: i32,
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let stdout = args.iter().any(|a| a == "--stdout");
    let folders: Vec<_> = args
        .iter()
        .filter(|a| a.as_str() != "--stdout")
        .map(PathBuf::from)
        .collect();
    if folders.is_empty() {
        return Err("Supply one or more downloaded film folders".into());
    }
    if stdout && folders.len() != 1 {
        return Err("--stdout requires exactly one film folder".into());
    }
    for folder in folders {
        let manifest: Manifest = serde_json::from_slice(&fs::read(folder.join("film.json"))?)?;
        let mut chunks = Vec::new();
        let start = Instant::now();
        for entry in manifest.chunks.iter().filter(|c| c.chunk_type == 3) {
            if PathBuf::from(&entry.file).components().count() != 1 {
                return Err("Chunk filename must be local to the film folder".into());
            }
            chunks.push(FilmChunkData {
                metadata: FilmChunk {
                    index: entry.index,
                    chunk_type: entry.chunk_type,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: 0,
                    file_relative_path: entry.file.clone(),
                },
                data: fs::read(folder.join(&entry.file))?,
            });
        }
        let report = decode_summary_events(&chunks, manifest.film_major_version)?;
        // Read-only bridge for the inspector: current native decoding, no stale exports,
        // stats lookup, or file writes. Partial reports keep their count diagnostics.
        if stdout {
            println!(
                "{}",
                serde_json::json!({
                    "match_id": manifest.match_id, "major_version": manifest.film_major_version,
                    "summary": report
                })
            );
            continue;
        }
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.;
        let stats_path = folder.join("settings/match-stats.json");
        let validation = if stats_path.exists() {
            let raw: serde_json::Value = serde_json::from_slice(&fs::read(stats_path)?)?;
            if raw["MatchId"].as_str() != Some(&manifest.match_id) {
                return Err("Cached match ID mismatch".into());
            }
            let stats: MatchStats = serde_json::from_value(raw)?;
            Some(validate_summary_events(&report, &stats))
        } else {
            None
        };
        let medals = report
            .events
            .iter()
            .filter_map(|e| e.medal.as_ref())
            .count();
        let unknown_medals = report
            .events
            .iter()
            .filter_map(|e| e.medal.as_ref())
            .filter(|m| m.name.is_none())
            .count();
        let sorting_weight_mismatches = report
            .events
            .iter()
            .filter(|e| {
                e.medal.is_some()
                    && film_medal_definition(e.metadata)
                        .and_then(|m| m.sorting_weight)
                        .zip(e.type_code)
                        .is_some_and(|(expected, recorded)| expected != u16::from(recorded))
            })
            .count();
        let output = folder.join("summary-events.json");
        fs::write(&output, serde_json::to_vec(&report)?)?;
        if let Some(validation) = &validation {
            fs::write(
                folder.join("summary-events-validation.json"),
                serde_json::to_vec_pretty(validation)?,
            )?;
        }
        println!(
            "{}",
            serde_json::json!({
                "match_id":manifest.match_id, "output":output, "events":report.events.len(),
                "medals":medals, "unknown_medals":unknown_medals,
                "declared_counts_match":report.matches_declared_counts(),
                "match_stats_match":validation.as_ref().map(|v| v.matches_stats()),
                "sorting_weight_mismatches":sorting_weight_mismatches, "load_decode_ms":elapsed_ms
            })
        );
        if !report.matches_declared_counts()
            || validation.as_ref().is_some_and(|v| !v.matches_stats())
        {
            return Err("Summary event validation failed; see exported diagnostics".into());
        }
    }
    Ok(())
}
