# 001 — Film parse

This harness times halo_api `Film::parse_with_options` and, in multi-threaded
runs, libfilmshell `Film::from_chunks` on the same compressed film chunks.
`--benchmark replication` measures `Film::parse_replication_with_options`;
`--benchmark summary` measures `Film::parse_summary_with_options`. Both include
registry parsing and omit the unselected chunk type before timing. Partial
benchmarks measure only halo_api, not libfilmshell's full-film parser.
It includes seven complete tar fixtures in
[`fixtures/`](fixtures/), stored with Git LFS. Install Git LFS and run
`git lfs pull` after cloning.

From the repository root, run the Rust harness:

```sh
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --threading multi --output benchmarks/website/results.json
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --threading single --output benchmarks/website/results-single.json
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --benchmark replication --threading multi --output benchmarks/website/results-replication.json
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --benchmark replication --threading single --output benchmarks/website/results-replication-single.json
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --benchmark summary --threading multi --output benchmarks/website/results-summary.json
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --benchmark summary --threading single --output benchmarks/website/results-summary-single.json
```

Each command runs 25 trials for each of the seven fixtures. Pass fixture names
after the options to measure a subset, for example `ranked_slayer ranked_ctf`.
Set `FILM_BENCH_RUNS=3` for a quick local check. The output JSON drives the
static site; commit it only when you intend to publish a new measured snapshot.

The harness reads all files before timing and gives each decoder fresh owned
copies of the compressed chunks (copying is timed). It warms each decoder, rotates run order,
and reports medians of parse-to-return time. It also records a serialized
halo_api film fingerprint outside the timed section. Download, disk I/O, and
runtime seeks are excluded. Partial results must match the corresponding
registry and section from a full parse, checked by fingerprint before timing.
Compare fingerprints only within the same benchmark; the three output types
serialize differently. Chunk counts and compressed sizes cover selected inputs,
including the registry. Summary parsing currently runs on the calling thread
in either threading mode. Reports record the benchmark and flag local source
changes alongside the HEAD revision; run against unchanged sources for a
commit-reproducible snapshot. The Rust entrypoints construct different output
models; this is a comparison of their public film parse paths. libfilmshell is
pinned in `Cargo.toml`. The single-threaded dataset measures only halo_api;
libfilmshell has no single-threaded entry point in the pinned revision.

The fixture set covers Aquarius Idle, Bazaar Idle, Octagon first-to-50, Ranked
Slayer on Live Fire, Ranked Oddball, Ranked CTF on Aquarius, and the hour-long
Clan Raid. Each tar contains a manifest and zlib-compressed chunk bytes, as
`RawFilmChunk` expects. The manifests contain only chunk kind, index, start
time, and local filename. Archive extraction happens before timing.
