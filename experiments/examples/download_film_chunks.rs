//! Downloads tracked Theater films from the consolidated CSV catalog.
//!
//! Each film lands in `<output>/<group>/<slug>/` as decompressed
//! `chunk-NNN-type-T.bin` files alongside a `film.json` recording the real
//! manifest metadata, so the corpus can be re-read offline by the decoding
//! examples. Films whose `film.json` already exists are skipped, making reruns
//! cheap after a partial failure.
//!
//! ```text
//! HALO_PROBE_MANIFEST  catalog CSV (default: films.csv)
//! HALO_PROBE_CATEGORY  optional category, e.g. Ranked Arena gameplay
//! HALO_PROBE_FILM      optional group/slug selector
//! HALO_FILM_CORPUS     output directory (default: films)
//! ```

mod common;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use halo_api::clients::hi::HaloInfiniteClient;
use halo_api::clients::hi::models::FilmChunkData;

const DEFAULT_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/films.csv");
const DEFAULT_OUTPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/films");

/// One row of the probe manifest: a match ID plus the action it isolates.
#[derive(Debug, serde::Deserialize)]
struct ProbeFilm {
    category: String,
    group: String,
    slug: String,
    match_id: String,
    description: String,
    analysis_profile: String,
}

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let manifest_path = env_or("HALO_PROBE_MANIFEST", DEFAULT_MANIFEST);
    let output = PathBuf::from(env_or("HALO_FILM_CORPUS", DEFAULT_OUTPUT));
    let mut films = read_probe_manifest(Path::new(&manifest_path))?;
    if let Ok(category) = std::env::var("HALO_PROBE_CATEGORY") {
        films.retain(|film| film.category == category);
    }
    if let Ok(label) = std::env::var("HALO_PROBE_FILM") {
        films.retain(|film| format!("{}/{}", film.group, film.slug) == label);
    }
    if films.is_empty() {
        return Err("No catalog films match the selected category/label".into());
    }
    let pending = films
        .iter()
        .filter(|film| !directory_for(&output, film).join("film.json").exists())
        .collect::<Vec<_>>();
    println!(
        "{} films from {manifest_path} -> {}: {} cached, {} to download",
        films.len(),
        output.display(),
        films.len() - pending.len(),
        pending.len(),
    );
    if pending.is_empty() {
        println!("Corpus is complete; nothing to download.");
        return Ok(());
    }

    // Authenticate before the download loop so bad credentials fail once here
    // rather than being reported as a download failure on every film.
    let (xbox, halo) = common::halo_infinite_client()?;
    let (gamertag, xuid) = common::logged_in_player(&xbox).await?;
    println!("authenticated as {gamertag} (xuid {xuid})\n");

    let mut failures = Vec::new();
    for (position, film) in pending.iter().enumerate() {
        print!(
            "[{:>2}/{}] {:<30} ",
            position + 1,
            pending.len(),
            format!("{}/{}", film.group, film.slug),
        );
        io::stdout().flush()?;
        match download(&halo, film, &directory_for(&output, film)).await {
            Ok(chunks) => {
                let bytes = chunks.iter().map(|chunk| chunk.data.len()).sum::<usize>();
                println!("{} chunks, {} bytes decompressed", chunks.len(), bytes);
            }
            Err(error) => {
                println!("FAILED: {error}");
                failures.push((film, error));
            }
        }
    }

    if failures.is_empty() {
        println!("\nAll {} films downloaded.", pending.len());
        return Ok(());
    }
    println!("\n{} of {} films failed:", failures.len(), pending.len());
    for (film, error) in &failures {
        println!(
            "  {}/{} ({}): {error}",
            film.group, film.slug, film.match_id
        );
    }
    println!("Rerun to retry only the films that failed.");
    Err(format!("{} films could not be downloaded", failures.len()).into())
}

/// Fetches one film's chunks and writes them with a metadata sidecar.
///
/// `film.json` is written last so a partially downloaded directory is not
/// mistaken for a cached one on the next run.
async fn download(
    halo: &HaloInfiniteClient,
    probe: &ProbeFilm,
    directory: &Path,
) -> Result<Vec<FilmChunkData>, common::ExampleError> {
    let film = halo.match_film(&probe.match_id).await?;
    let chunks = halo.film_chunks(&film).await?;
    fs::create_dir_all(directory)?;

    let mut records = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        let name = format!(
            "chunk-{:03}-type-{}.bin",
            chunk.metadata.index, chunk.metadata.chunk_type
        );
        fs::write(directory.join(&name), &chunk.data)?;
        records.push(serde_json::json!({
            "file": name,
            "index": chunk.metadata.index,
            "chunk_type": chunk.metadata.chunk_type,
            "start_time_offset_ms": chunk.metadata.start_time_offset_ms,
            "duration_ms": chunk.metadata.duration_ms,
            "compressed_size": chunk.metadata.size,
            "decompressed_size": chunk.data.len(),
            "file_relative_path": chunk.metadata.file_relative_path,
        }));
    }

    let metadata = serde_json::json!({
        "category": probe.category,
        "group": probe.group,
        "slug": probe.slug,
        "description": probe.description,
        "analysis_profile": probe.analysis_profile,
        "match_id": film.custom_data.match_id,
        "asset_id": film.asset_id,
        "film_status_bond": film.status,
        "film_major_version": film.custom_data.film_major_version,
        "film_length": film.custom_data.film_length,
        "has_game_ended": film.custom_data.has_game_ended,
        "blob_storage_path_prefix": film.blob_storage_path_prefix,
        "chunks": records,
    });
    fs::write(
        directory.join("film.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    Ok(chunks)
}

/// Reads real CSV quoting, including descriptions containing commas.
fn read_probe_manifest(path: &Path) -> Result<Vec<ProbeFilm>, common::ExampleError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;
    let films = reader
        .deserialize()
        .collect::<Result<Vec<ProbeFilm>, _>>()?;
    let mut labels = std::collections::BTreeSet::new();
    let mut ids = std::collections::BTreeSet::new();
    for film in &films {
        let safe = |part: &str| {
            !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        };
        if !safe(&film.group)
            || !safe(&film.slug)
            || film.category.is_empty()
            || film.description.is_empty()
            || film.match_id.is_empty()
            || !labels.insert((&film.group, &film.slug))
            || !ids.insert(&film.match_id)
        {
            return Err(format!(
                "{}: empty, unsafe, or duplicate catalog entry",
                path.display()
            )
            .into());
        }
    }
    if films.is_empty() {
        return Err(format!("{} contained no films", path.display()).into());
    }
    Ok(films)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_preserves_quoted_descriptions_and_ranked_category() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("films.csv");
        let films = read_probe_manifest(&path).unwrap();
        let ranked = films
            .iter()
            .filter(|film| film.category == "Ranked Arena gameplay")
            .collect::<Vec<_>>();
        assert!(ranked.len() >= 2);
        assert!(
            ranked
                .iter()
                .any(|film| film.description.contains(", movement, and respawns"))
        );
        assert!(ranked.iter().any(
            |film| film.match_id == "4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73"
                && film.analysis_profile == "oddball"
        ));
    }
}

fn directory_for(output: &Path, film: &ProbeFilm) -> PathBuf {
    output.join(&film.group).join(&film.slug)
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}
