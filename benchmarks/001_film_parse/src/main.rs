//! Compare public film decode paths on the same compressed chunk input.
use std::hash::Hasher;
use std::hint::black_box;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use filmshell::{Chunk, Film as ShellFilm};
use halo_api::clients::hi::models::{RawChunkKind, RawFilmChunk};
use halo_api::theater::Film as HaloFilm;
use halo_api::theater::film::{FilmParseOptions, Threading};
use halo_api::theater::{FilmReplication, FilmSummary};
use serde::{Deserialize, Serialize};

const FIXTURES: [(&str, &str); 7] = [
    ("aquarius_idle", "Aquarius Idle"),
    ("bazaar_idle", "Bazaar Idle"),
    ("octagon", "Octagon"),
    ("ranked_slayer", "Ranked Slayer"),
    ("ranked_oddball", "Ranked Oddball"),
    ("ranked_ctf", "Ranked CTF"),
    ("clan_raid", "Clan Raid"),
];
const LIBFILMSHELL_REV: &str = "031e2179f0758314e2a97fb96438c7b1e82ce1fb";
const DEFAULT_RUNS: usize = 25;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Benchmark {
    Full,
    Replication,
    Summary,
}

impl Benchmark {
    fn includes(self, kind: i32) -> bool {
        match self {
            Self::Full => true,
            Self::Replication => kind != RawChunkKind::SUMMARY.0,
            Self::Summary => kind != RawChunkKind::REPLICATION.0,
        }
    }

    fn compares_shell(self, threading: Threading) -> bool {
        self == Self::Full && threading == Threading::Multi
    }
}

#[derive(Deserialize)]
struct Manifest {
    chunks: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    kind: i32,
    index: i32,
    start_ms: i64,
    file: String,
}

struct Input {
    kind: i32,
    index: i32,
    start_ms: i64,
    data: Vec<u8>,
}

#[derive(Serialize)]
struct ResultRow {
    fixture: String,
    label: String,
    chunks: usize,
    compressed_mib: f64,
    runs: usize,
    halo_api_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    libfilmshell_ms: Option<f64>,
    film_fingerprint: String,
}

#[derive(Serialize)]
struct Report {
    schema: u8,
    benchmark: Benchmark,
    generated_at_unix_seconds: u64,
    halo_api_commit: String,
    halo_api_dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    libfilmshell_commit: Option<&'static str>,
    os: &'static str,
    arch: &'static str,
    logical_cpus: usize,
    cpu_model: String,
    threading: &'static str,
    results: Vec<ResultRow>,
}

fn cpu_model() -> String {
    #[cfg(target_os = "macos")]
    if let Ok(output) = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if output.status.success() && !name.is_empty() {
            return name;
        }
    }
    #[cfg(target_os = "macos")]
    if let Ok(output) = std::process::Command::new("system_profiler")
        .arg("SPHardwareDataType")
        .output()
        && output.status.success()
        && let Some(chip) = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.trim().strip_prefix("Chip:"))
    {
        return chip.trim().to_owned();
    }
    format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn median(sorted: &[f64]) -> f64 {
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

fn load(archive_path: &Path) -> Result<Vec<Input>, Box<dyn std::error::Error>> {
    let mut archive = tar::Archive::new(std::fs::File::open(archive_path)?);
    let mut files = std::collections::HashMap::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let name = entry.path()?.to_string_lossy().into_owned();
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        if files.insert(name, data).is_some() {
            return Err("duplicate archive member".into());
        }
    }
    let manifest: Manifest = serde_json::from_slice(
        &files
            .remove("manifest.json")
            .ok_or("archive has no manifest.json")?,
    )?;
    let mut inputs = Vec::with_capacity(manifest.chunks.len());
    for entry in manifest.chunks {
        let data = files
            .remove(&entry.file)
            .ok_or_else(|| format!("archive is missing {}", entry.file))?;
        if data.len() < 2
            || data[0] & 0x0f != 8
            || data[0] >> 4 > 7
            || !u16::from_be_bytes([data[0], data[1]]).is_multiple_of(31)
        {
            return Err(format!("{} is not a zlib chunk", entry.file).into());
        }
        inputs.push(Input {
            kind: entry.kind,
            index: entry.index,
            start_ms: entry.start_ms,
            data,
        });
    }
    Ok(inputs)
}

fn halo_chunks(entries: &[Input]) -> Vec<RawFilmChunk> {
    entries
        .iter()
        .map(|entry| RawFilmChunk {
            kind: RawChunkKind(entry.kind),
            index: entry.index,
            start_ms: entry.start_ms,
            data: entry.data.clone(),
        })
        .collect()
}

fn parse_halo(entries: &[Input], threading: Threading) -> Result<HaloFilm, String> {
    HaloFilm::parse_with_options(halo_chunks(entries), FilmParseOptions { threading })
        .map_err(|error| error.to_string())
}

fn parse_replication(entries: &[Input], threading: Threading) -> Result<FilmReplication, String> {
    HaloFilm::parse_replication_with_options(halo_chunks(entries), FilmParseOptions { threading })
        .map_err(|error| error.to_string())
}

fn parse_summary(entries: &[Input], threading: Threading) -> Result<FilmSummary, String> {
    HaloFilm::parse_summary_with_options(halo_chunks(entries), FilmParseOptions { threading })
        .map_err(|error| error.to_string())
}

fn parse_shell(entries: &[Input]) -> Result<ShellFilm, String> {
    let chunks = entries
        .iter()
        .map(|entry| {
            Chunk::from_manifest(entry.index as u32, entry.kind as u32, entry.data.clone())
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    ShellFilm::from_chunks(chunks).map_err(|error| error.to_string())
}

fn fingerprint(film: &impl Serialize) -> Result<String, Box<dyn std::error::Error>> {
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
    let mut writer = HashWriter(std::collections::hash_map::DefaultHasher::new());
    serde_json::to_writer(&mut writer, film)?;
    Ok(format!("{:016x}", writer.0.finish()))
}

fn timed<T>(
    entries: &[Input],
    parse: impl FnOnce(&[Input]) -> Result<T, String>,
) -> Result<f64, String> {
    let start = Instant::now();
    let film = parse(black_box(entries))?;
    black_box(&film);
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    drop(film);
    Ok(ms)
}

fn benchmark<T: Serialize>(
    fixture: &str,
    label: &str,
    archive_path: &Path,
    requested_runs: Option<usize>,
    threading: Threading,
    mode: Benchmark,
    parse: fn(&[Input], Threading) -> Result<T, String>,
) -> Result<ResultRow, Box<dyn std::error::Error>> {
    let mut entries = load(archive_path)?;
    // Compare selected output with the full parser outside the measured region.
    let expected_fingerprint = if mode == Benchmark::Full {
        None
    } else {
        let full = parse_halo(&entries, threading)?;
        Some(match mode {
            Benchmark::Replication => fingerprint(&FilmReplication {
                registry: full.registry,
                replication_stream: full.replication_stream,
            })?,
            Benchmark::Summary => fingerprint(&FilmSummary {
                registry: full.registry,
                summary: full.summary,
            })?,
            Benchmark::Full => unreachable!(),
        })
    };
    entries.retain(|entry| mode.includes(entry.kind));
    let bytes: usize = entries.iter().map(|entry| entry.data.len()).sum();
    let runs = requested_runs.unwrap_or(DEFAULT_RUNS);
    let parsed = parse(&entries, threading)?;
    let film_fingerprint = fingerprint(&parsed)?;
    drop(parsed);
    if expected_fingerprint.is_some_and(|expected| expected != film_fingerprint) {
        return Err(format!("{fixture}: partial output differs from full film").into());
    }
    if mode.compares_shell(threading) {
        parse_shell(&entries)?;
    }

    let mut halo_times = Vec::with_capacity(runs);
    let mut shell_times = mode
        .compares_shell(threading)
        .then(|| Vec::with_capacity(runs));
    for run in 0..runs {
        if let Some(shell_times) = &mut shell_times {
            if run % 2 == 0 {
                halo_times.push(timed(&entries, |entries| parse(entries, threading))?);
                shell_times.push(timed(&entries, parse_shell)?);
            } else {
                shell_times.push(timed(&entries, parse_shell)?);
                halo_times.push(timed(&entries, |entries| parse(entries, threading))?);
            }
        } else {
            halo_times.push(timed(&entries, |entries| parse(entries, threading))?);
        }
    }
    halo_times.sort_by(f64::total_cmp);
    if let Some(shell_times) = &mut shell_times {
        shell_times.sort_by(f64::total_cmp);
    }
    let result = ResultRow {
        fixture: fixture.to_owned(),
        label: label.to_owned(),
        chunks: entries.len(),
        compressed_mib: bytes as f64 / 1_048_576.0,
        runs,
        halo_api_ms: median(&halo_times),
        libfilmshell_ms: shell_times.as_deref().map(median),
        film_fingerprint,
    };
    if let Some(shell_ms) = result.libfilmshell_ms {
        println!(
            "{label}: halo_api {:.1} ms, libfilmshell {:.1} ms ({} runs, {} chunks)",
            result.halo_api_ms, shell_ms, runs, result.chunks
        );
    } else {
        println!(
            "{label}: halo_api {:.1} ms ({} runs, {} chunks)",
            result.halo_api_ms, runs, result.chunks
        );
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut output = None;
    let mut threading = Threading::Multi;
    let mut mode = Benchmark::Full;
    let mut fixture_names = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--output" {
            output = Some(PathBuf::from(args.next().ok_or("--output needs a path")?));
        } else if arg == "--threading" {
            threading = match args
                .next()
                .ok_or("--threading needs single or multi")?
                .as_str()
            {
                "single" => Threading::Single,
                "multi" => Threading::Multi,
                _ => return Err("--threading must be single or multi".into()),
            };
        } else if arg == "--benchmark" {
            mode = match args
                .next()
                .ok_or("--benchmark needs full, replication or summary")?
                .as_str()
            {
                "full" => Benchmark::Full,
                "replication" => Benchmark::Replication,
                "summary" => Benchmark::Summary,
                _ => return Err("--benchmark must be full, replication or summary".into()),
            };
        } else {
            fixture_names.push(arg);
        }
    }
    if fixture_names.is_empty() {
        fixture_names = FIXTURES
            .iter()
            .map(|(name, _)| (*name).to_owned())
            .collect();
    }
    let requested_runs = std::env::var("FILM_BENCH_RUNS")
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .filter(|&runs| runs > 0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let mut results = Vec::with_capacity(fixture_names.len());
    for fixture in fixture_names {
        let label = FIXTURES
            .iter()
            .find(|(name, _)| *name == fixture)
            .map(|(_, label)| *label)
            .ok_or_else(|| format!("unknown fixture {fixture}"))?;
        let archive = root.join(format!("{fixture}.tar"));
        results.push(match mode {
            Benchmark::Full => benchmark(
                &fixture,
                label,
                &archive,
                requested_runs,
                threading,
                mode,
                parse_halo,
            )?,
            Benchmark::Replication => benchmark(
                &fixture,
                label,
                &archive,
                requested_runs,
                threading,
                mode,
                parse_replication,
            )?,
            Benchmark::Summary => benchmark(
                &fixture,
                label,
                &archive,
                requested_runs,
                threading,
                mode,
                parse_summary,
            )?,
        });
    }
    if let Some(output) = output {
        let commit = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .output()?;
        let report = Report {
            schema: 1,
            benchmark: mode,
            generated_at_unix_seconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            halo_api_commit: String::from_utf8(commit.stdout)?.trim().to_owned(),
            halo_api_dirty: !std::process::Command::new("git")
                .args([
                    "diff",
                    "--quiet",
                    "HEAD",
                    "--",
                    "src",
                    "Cargo.toml",
                    "Cargo.lock",
                    "benchmarks/001_film_parse/src",
                    "benchmarks/001_film_parse/Cargo.toml",
                    "benchmarks/001_film_parse/Cargo.lock",
                ])
                .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
                .status()?
                .success(),
            libfilmshell_commit: mode.compares_shell(threading).then_some(LIBFILMSHELL_REV),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            logical_cpus: std::thread::available_parallelism()?.get(),
            cpu_model: cpu_model(),
            threading: match threading {
                Threading::Single => "Single-Threaded",
                Threading::Multi => "Multi-Threaded",
            },
            results,
        };
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
        println!("Wrote {}", output.display());
    }
    Ok(())
}
