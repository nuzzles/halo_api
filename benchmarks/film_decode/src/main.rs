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
use halo_api::theater::film::ReplicationChange;
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

struct FieldStats {
    total: usize,
    inline: usize,
    wide: usize,
    name_bytes: usize,
    top: Vec<(String, usize)>,
    top_components: Vec<(String, usize)>,
    byte_sizes: [usize; 10],
    component_field_counts: [usize; 17],
}

fn field_stats(entries: &[Input]) -> Result<FieldStats, String> {
    let film = parse_halo(entries)?;
    let mut total = 0;
    let mut inline = 0;
    let mut wide = 0;
    let mut name_bytes = 0;
    let mut by_name = std::collections::HashMap::<&str, usize>::new();
    let mut by_component_name = std::collections::HashMap::<&str, usize>::new();
    let mut byte_sizes = [0; 10];
    let mut component_field_counts = [0; 17];
    for observation in &film.replication_stream.observations {
        let (fields, default_fields, components) = match &observation.change {
            ReplicationChange::EntityCreated {
                fields,
                default_fields,
                components,
                ..
            }
            | ReplicationChange::Keyframe {
                fields,
                default_fields,
                components,
                ..
            } => (fields.as_slice(), default_fields.as_slice(), components.as_slice()),
            ReplicationChange::EntityUpdated {
                fields, components, ..
            } => (fields.as_slice(), &[][..], components.as_slice()),
            _ => continue,
        };
        for component in components {
            component_field_counts[component.fields.len().min(16)] += 1;
            *by_component_name.entry(component.name.as_str()).or_default() += 1;
        }
        for field in fields
            .iter()
            .chain(default_fields)
            .chain(components.iter().flat_map(|component| &component.fields))
        {
            total += 1;
            name_bytes += field.name.len();
            *by_name.entry(field.name.as_ref()).or_default() += 1;
            if let halo_api::theater::film::ReplicationValue::Bits(bits) = &field.value {
                byte_sizes[bits.bytes.len().min(9)] += 1;
                if bits.bytes.len() <= 8 {
                    inline += 1;
                } else {
                    wide += 1;
                }
            }
        }
    }
    let mut names = by_name.into_iter().collect::<Vec<_>>();
    names.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));
    let top = names
        .into_iter()
        .take(20)
        .map(|(name, count)| (name.to_owned(), count))
        .collect();
    let mut component_names = by_component_name.into_iter().collect::<Vec<_>>();
    component_names.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));
    let top_components = component_names
        .into_iter()
        .take(20)
        .map(|(name, count)| (name.to_owned(), count))
        .collect();
    Ok(FieldStats {
        total,
        inline,
        wide,
        name_bytes,
        top,
        top_components,
        byte_sizes,
        component_field_counts,
    })
}
fn shell(entries: &[Input]) -> Result<(usize, usize), String> {
    let film = parse_shell(entries)?;
    Ok((film.players().len(), film.highlights().len()))
}

fn parse_shell(entries: &[Input]) -> Result<ShellFilm, String> {
    let chunks = entries
        .iter()
        .map(|c| Chunk::from_manifest(c.index as u32, c.kind as u32, c.data.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    ShellFilm::from_chunks(chunks).map_err(|e| e.to_string())
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

fn bench_returned<T>(
    name: &str,
    entries: &[Input],
    runs: usize,
    parse: fn(&[Input]) -> Result<T, String>,
    observe: fn(&T) -> (usize, usize),
) {
    let warm = parse(entries).unwrap();
    println!("{name}: output {:?}", observe(&warm));
    drop(warm);
    let mut times = Vec::with_capacity(runs);
    for _ in 0..runs {
        times.push(time_returned(entries, parse, observe));
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

fn time_returned<T>(
    entries: &[Input],
    parse: fn(&[Input]) -> Result<T, String>,
    observe: fn(&T) -> (usize, usize),
) -> f64 {
    let started = Instant::now();
    let film = parse(black_box(entries)).unwrap();
    black_box(observe(&film));
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    drop(film);
    millis
}

fn bench_pair(entries: &[Input], runs: usize) {
    let halo_counts = |film: &HaloFilm| {
        (
            film.replication_stream.observations.len(),
            film.summary.highlights.len(),
        )
    };
    let shell_counts = |film: &ShellFilm| (film.players().len(), film.highlights().len());
    let halo_warm = parse_halo(entries).unwrap();
    let shell_warm = parse_shell(entries).unwrap();
    println!("halo_api_parse: output {:?}", halo_counts(&halo_warm));
    println!("libfilmshell_parse: output {:?}", shell_counts(&shell_warm));
    drop((halo_warm, shell_warm));
    let mut halo_times = Vec::with_capacity(runs);
    let mut shell_times = Vec::with_capacity(runs);
    for run in 0..runs {
        if run % 2 == 0 {
            halo_times.push(time_returned(entries, parse_halo, halo_counts));
            shell_times.push(time_returned(entries, parse_shell, shell_counts));
        } else {
            shell_times.push(time_returned(entries, parse_shell, shell_counts));
            halo_times.push(time_returned(entries, parse_halo, halo_counts));
        }
    }
    halo_times.sort_by(f64::total_cmp);
    shell_times.sort_by(f64::total_cmp);
    println!(
        "halo_api_parse: median {:.1}ms min {:.1}ms max {:.1}ms runs {}",
        halo_times[runs / 2], halo_times[0], halo_times[runs - 1], runs
    );
    println!(
        "libfilmshell_parse: median {:.1}ms min {:.1}ms max {:.1}ms runs {}",
        shell_times[runs / 2], shell_times[0], shell_times[runs - 1], runs
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
        if std::env::var_os("FILM_BENCH_FIELD_STATS").is_some() {
            let FieldStats {
                total: fields,
                inline,
                wide,
                name_bytes,
                top,
                top_components,
                byte_sizes,
                component_field_counts,
            } = field_stats(&entries).unwrap();
            println!(
                "halo_api fields {fields}, inline bit values {inline}, wide bit values {wide}, name bytes {name_bytes}"
            );
            println!("top field names {top:?}");
            println!("top component names {top_components:?}");
            println!("bit value byte sizes {byte_sizes:?} (last bucket 9+)");
            println!("component field counts {component_field_counts:?} (last bucket 16+)");
        }
        if std::env::var_os("FILM_BENCH_STATS_ONLY").is_some() {
            continue;
        }
        if std::env::var_os("FILM_BENCH_PARSE_ONLY").is_some() {
            if std::env::var_os("FILM_BENCH_INTERLEAVED").is_some()
                && std::env::var_os("FILM_BENCH_ONLY_HALO").is_none()
            {
                bench_pair(&entries, runs);
                continue;
            }
            bench_returned("halo_api_parse", &entries, runs, parse_halo, |film| {
                (
                    film.replication_stream.observations.len(),
                    film.summary.highlights.len(),
                )
            });
            if std::env::var_os("FILM_BENCH_ONLY_HALO").is_none() {
                bench_returned("libfilmshell_parse", &entries, runs, parse_shell, |film| {
                    (film.players().len(), film.highlights().len())
                });
            }
        } else {
            bench("halo_api", &entries, runs, halo);
            if std::env::var_os("FILM_BENCH_ONLY_HALO").is_none() {
                bench("halo_inflate", &entries, runs, inflate);
                bench("libfilmshell", &entries, runs, shell);
            }
        }
    }
}
