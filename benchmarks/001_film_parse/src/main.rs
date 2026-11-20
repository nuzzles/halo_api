//! Compare public film decode paths on the same compressed chunk input.
//! JGtm's measured path is partial because full replay construction requires
//! external map bounds. These timings do not represent equivalent decoded work.
use std::hash::Hasher;
use std::hint::black_box;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use filmshell::{Chunk, Film as ShellFilm};
use halo_api::clients::hi::models::{RawChunkKind, RawFilmChunk};
use halo_api::theater::Film as HaloFilm;
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
const JGTM_REV: &str = "037ec4d4db6578309a9d8139ae8a11af1b3de871";
const DEFAULT_RUNS: usize = 100;

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
    libfilmshell_ms: f64,
    jgtm_partial_ms: f64,
    film_fingerprint: String,
}

#[derive(Serialize)]
struct Report {
    schema: u8,
    generated_at_unix_seconds: u64,
    halo_api_commit: String,
    libfilmshell_commit: &'static str,
    jgtm_commit: &'static str,
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

fn parse_halo(entries: &[Input]) -> Result<HaloFilm, String> {
    let chunks = entries
        .iter()
        .map(|entry| RawFilmChunk {
            kind: RawChunkKind(entry.kind),
            index: entry.index,
            start_ms: entry.start_ms,
            data: entry.data.clone(),
        })
        .collect::<Vec<_>>();
    HaloFilm::parse(chunks).map_err(|error| error.to_string())
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

fn fingerprint(film: &HaloFilm) -> Result<String, Box<dyn std::error::Error>> {
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

fn timed<T>(entries: &[Input], parse: fn(&[Input]) -> Result<T, String>) -> Result<f64, String> {
    let start = Instant::now();
    let film = parse(black_box(entries))?;
    black_box(&film);
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    drop(film);
    Ok(ms)
}

struct JgtmRunner {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl JgtmRunner {
    fn start(binary: &Path, fixture: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new(binary)
            .arg(fixture)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take().ok_or("JGtm helper stdin unavailable")?;
        let mut output = BufReader::new(
            child
                .stdout
                .take()
                .ok_or("JGtm helper stdout unavailable")?,
        );
        let mut ready = String::new();
        output.read_line(&mut ready)?;
        if ready.trim() != "ready" {
            return Err(format!("JGtm helper failed to load {}", fixture.display()).into());
        }
        Ok(Self {
            child,
            input,
            output,
        })
    }

    fn timed(&mut self) -> Result<f64, Box<dyn std::error::Error>> {
        self.input.write_all(b"run\n")?;
        self.input.flush()?;
        let mut line = String::new();
        if self.output.read_line(&mut line)? == 0 {
            return Err("JGtm helper exited during benchmark".into());
        }
        Ok(line.trim().parse()?)
    }
}

impl Drop for JgtmRunner {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn benchmark(
    fixture: &str,
    label: &str,
    archive_path: &Path,
    jgtm_binary: &Path,
    requested_runs: Option<usize>,
) -> Result<ResultRow, Box<dyn std::error::Error>> {
    let entries = load(archive_path)?;
    let bytes: usize = entries.iter().map(|entry| entry.data.len()).sum();
    let runs = requested_runs.unwrap_or(DEFAULT_RUNS);
    let parsed = parse_halo(&entries)?;
    let film_fingerprint = fingerprint(&parsed)?;
    drop(parsed);
    parse_shell(&entries)?;
    let mut jgtm = JgtmRunner::start(jgtm_binary, archive_path)?;

    let mut halo_times = Vec::with_capacity(runs);
    let mut shell_times = Vec::with_capacity(runs);
    let mut jgtm_times = Vec::with_capacity(runs);
    for run in 0..runs {
        match run % 3 {
            0 => {
                halo_times.push(timed(&entries, parse_halo)?);
                shell_times.push(timed(&entries, parse_shell)?);
                jgtm_times.push(jgtm.timed()?);
            }
            1 => {
                shell_times.push(timed(&entries, parse_shell)?);
                jgtm_times.push(jgtm.timed()?);
                halo_times.push(timed(&entries, parse_halo)?);
            }
            _ => {
                jgtm_times.push(jgtm.timed()?);
                halo_times.push(timed(&entries, parse_halo)?);
                shell_times.push(timed(&entries, parse_shell)?);
            }
        }
    }
    halo_times.sort_by(f64::total_cmp);
    shell_times.sort_by(f64::total_cmp);
    jgtm_times.sort_by(f64::total_cmp);
    let result = ResultRow {
        fixture: fixture.to_owned(),
        label: label.to_owned(),
        chunks: entries.len(),
        compressed_mib: bytes as f64 / 1_048_576.0,
        runs,
        halo_api_ms: median(&halo_times),
        libfilmshell_ms: median(&shell_times),
        jgtm_partial_ms: median(&jgtm_times),
        film_fingerprint,
    };
    println!(
        "{label}: halo_api {:.1} ms, libfilmshell {:.1} ms, JGtm partial {:.1} ms ({} runs, {} chunks)",
        result.halo_api_ms, result.libfilmshell_ms, result.jgtm_partial_ms, runs, result.chunks
    );
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut output = None;
    let mut fixture_names = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--output" {
            output = Some(PathBuf::from(args.next().ok_or("--output needs a path")?));
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
    let jgtm_binary = std::env::var_os("JGTM_BENCH_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/jgtm-film-bench")
        });
    let mut results = Vec::with_capacity(fixture_names.len());
    for fixture in fixture_names {
        let label = FIXTURES
            .iter()
            .find(|(name, _)| *name == fixture)
            .map(|(_, label)| *label)
            .ok_or_else(|| format!("unknown fixture {fixture}"))?;
        results.push(benchmark(
            &fixture,
            label,
            &root.join(format!("{fixture}.tar")),
            &jgtm_binary,
            requested_runs,
        )?);
    }
    if let Some(output) = output {
        let commit = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()?;
        let report = Report {
            schema: 2,
            generated_at_unix_seconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            halo_api_commit: String::from_utf8(commit.stdout)?.trim().to_owned(),
            libfilmshell_commit: LIBFILMSHELL_REV,
            jgtm_commit: JGTM_REV,
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            logical_cpus: std::thread::available_parallelism()?.get(),
            cpu_model: cpu_model(),
            threading: "Multi-Threaded",
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
