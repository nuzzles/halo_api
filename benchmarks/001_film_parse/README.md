# 001 — Film parse

This harness times halo_api `Film::parse`, libfilmshell `Film::from_chunks`, and
JGtm's Go decoder on the same compressed film chunks. It includes seven complete tar fixtures in
[`fixtures/`](fixtures/), stored with Git LFS. Install Git LFS and run
`git lfs pull` after cloning.

Install Go 1.26.11 or newer. From the repository root, build the pinned Go helper
before running the Rust harness:

```sh
mkdir -p target
(cd benchmarks/001_film_parse/jgtm && go build -o ../../../target/jgtm-film-bench .)
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --output benchmarks/website/results.json
```

The command runs 100 trials for each of the seven fixtures. Pass fixture names
after `--output` to measure a subset, for example `ranked_slayer ranked_ctf`.
Set `FILM_BENCH_RUNS=3` for a quick local check. The output JSON drives the
static site; commit it only when you intend to publish a new measured snapshot.

The harness reads all files before timing and gives each decoder fresh owned
copies of the compressed chunks. It warms each decoder, rotates run order,
and reports medians of parse-to-return time. It also records a serialized
halo_api film fingerprint outside the timed section. Download, disk I/O, and
runtime seeks are excluded. The Go helper stays alive between trials and reports
its own in-process duration, so IPC and process startup are excluded.

**The Go result is a partial decode.** It calls `decfilm.Load` (inflate and packet
framing), `ParseRegistryChunk`, and `ParseHighlightEvents`. Its public full replay
builder requires an external map-bounds catalogue, which these film fixtures do
not contain. The Rust entrypoints construct different output models too. The
chart compares these specified workflows, not identical decoded work or a full
replay throughput ranking. libfilmshell is pinned in `Cargo.toml`; JGtm is pinned
to v1.4.1 in `jgtm/go.mod` (commit `037ec4d4db6578309a9d8139ae8a11af1b3de871`).

The fixture set covers Aquarius Idle, Bazaar Idle, Octagon first-to-50, Ranked
Slayer on Live Fire, Ranked Oddball, Ranked CTF on Aquarius, and the hour-long
Clan Raid. Each tar contains a manifest and zlib-compressed chunk bytes, as
`RawFilmChunk` expects. The manifests contain only chunk kind, index, start
time, and local filename. Archive extraction happens before timing.
