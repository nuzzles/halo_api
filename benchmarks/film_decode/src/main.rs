//! Run with `cargo run --release --manifest-path benchmarks/film_decode/Cargo.toml -- <film-dir>...`.
//! Each film directory needs `manifest.json` or `film.json` and its chunk files.
//! Input files are read once, outside timing. Already-inflated local files are
//! re-compressed once so both decoders receive the same downloaded-style bytes.
//! Each timed decode clones those bytes into owned input chunks. The models
//! differ: halo_api retains ECS observations; libfilmshell derives player and
//! world tracks, so these timings compare public whole-film entrypoints, not
//! equivalent output work.
use std::hint::black_box;
use std::hash::Hasher;
use std::path::Path;
use std::time::Instant;

use filmshell::{Chunk, Film as ShellFilm};
use halo_api::clients::hi::models::{RawChunkKind, RawFilmChunk};
use halo_api::theater::Film as HaloFilm;
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
struct Manifest {
    chunks: Vec<Entry>,
}
#[derive(Deserialize)]
struct Entry {
    #[serde(alias = "chunk_type")]
    kind: i32,
    index: i32,
    #[serde(alias = "start_time_offset_ms")]
    start_ms: i64,
    file: String,
}
struct Input {
    kind: i32,
    index: i32,
    start_ms: i64,
    data: Vec<u8>,
}

fn load(dir: &Path) -> Vec<Input> {
    let manifest = if dir.join("film.json").exists() {
        "film.json"
    } else {
        "manifest.json"
    };
    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(dir.join(manifest)).unwrap()).unwrap();
    manifest
        .chunks
        .into_iter()
        .map(|c| Input {
            kind: c.kind,
            index: c.index,
            start_ms: c.start_ms,
            data: {
                let data = std::fs::read(dir.join(c.file)).unwrap();
                let is_zlib = data.len() >= 2
                    && data[0] & 0x0f == 8
                    && data[0] >> 4 <= 7
                    && u16::from_be_bytes([data[0], data[1]]).is_multiple_of(31);
                if is_zlib {
                    data
                } else {
                    let mut encoder =
                        flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
                    encoder.write_all(&data).unwrap();
                    encoder.finish().unwrap()
                }
            },
        })
        .collect()
}

fn inflate(entries: &[Input]) -> Result<(usize, usize), String> {
    let mut total = 0;
    for c in entries {
        let inflated = RawFilmChunk {
            kind: RawChunkKind(c.kind),
            index: c.index,
            start_ms: c.start_ms,
            data: c.data.clone(),
        }
        .inflate()
        .map_err(|e| e.to_string())?;
        total += inflated.data.len();
    }
    Ok((total, 0))
}

fn parse_halo(entries: &[Input]) -> Result<HaloFilm, String> {
    let chunks = entries
        .iter()
        .map(|c| RawFilmChunk {
            kind: RawChunkKind(c.kind),
            index: c.index,
            start_ms: c.start_ms,
            data: c.data.clone(),
        })
        .collect::<Vec<_>>();
    HaloFilm::parse(chunks).map_err(|e| e.to_string())
}

fn halo(entries: &[Input]) -> Result<(usize, usize), String> {
    let film = parse_halo(entries)?;
    Ok((
        film.replication_stream.observations.len(),
        film.summary.highlights.len(),
    ))
}

fn fingerprint(entries: &[Input]) -> Result<u64, String> {
    struct HashWriter(std::collections::hash_map::DefaultHasher);
    impl Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.write(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let film = parse_halo(entries)?;
    let mut writer = HashWriter(std::collections::hash_map::DefaultHasher::new());
    serde_json::to_writer(&mut writer, &film).map_err(|e| e.to_string())?;
    Ok(writer.0.finish())
}
fn shell(entries: &[Input]) -> Result<(usize, usize), String> {
    let chunks = entries
        .iter()
        .map(|c| Chunk::from_manifest(c.index as u32, c.kind as u32, c.data.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let film = ShellFilm::from_chunks(chunks).map_err(|e| e.to_string())?;
    Ok((film.players().len(), film.highlights().len()))
}

type Decode = fn(&[Input]) -> Result<(usize, usize), String>;

fn bench(name: &str, entries: &[Input], runs: usize, f: Decode) {
    match f(entries) {
        Err(e) => {
            println!("{name}: ERROR {e}");
            return;
        }
        Ok(v) => println!("{name}: output {v:?}"),
    }
    let mut times = Vec::new();
    for _ in 0..runs {
        let started = Instant::now();
        let out = f(black_box(entries)).unwrap();
        black_box(out);
        times.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(f64::total_cmp);
    println!(
        "{name}: median {:.1}ms min {:.1}ms max {:.1}ms runs {}",
        times[times.len() / 2],
        times[0],
        times[times.len() - 1],
        runs
    );
}

fn main() {
    for arg in std::env::args().skip(1) {
        let dir = Path::new(&arg);
        let entries = load(dir);
        let bytes: usize = entries.iter().map(|c| c.data.len()).sum();
        let runs = std::env::var("FILM_BENCH_RUNS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|&runs| runs > 0)
            .unwrap_or(if bytes > 5_000_000 { 5 } else { 15 });
        println!(
            "{}: {} chunks, {:.2} MiB compressed",
            dir.display(),
            entries.len(),
            bytes as f64 / 1048576.0
        );
        if std::env::var_os("FILM_BENCH_FINGERPRINT").is_some() {
            println!("halo_api film fingerprint {:016x}", fingerprint(&entries).unwrap());
        }
        bench("halo_api", &entries, runs, halo);
        bench("halo_inflate", &entries, runs, inflate);
        bench("libfilmshell", &entries, runs, shell);
    }
}
