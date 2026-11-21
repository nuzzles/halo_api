# 001 — Film parse

This harness times halo_api `Film::parse_with_options` and, in multi-threaded
runs, libfilmshell `Film::from_chunks` on the same compressed film chunks.
It includes seven complete tar fixtures in
[`fixtures/`](fixtures/), stored with Git LFS. Install Git LFS and run
`git lfs pull` after cloning.

From the repository root, run the Rust harness:

```sh
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --threading multi --output benchmarks/website/results.json
cargo run --locked --release --manifest-path benchmarks/001_film_parse/Cargo.toml -- \
  --threading single --output benchmarks/website/results-single.json
```

Each command runs 25 trials for each of the seven fixtures. Pass fixture names
after the options to measure a subset, for example `ranked_slayer ranked_ctf`.
Set `FILM_BENCH_RUNS=3` for a quick local check. Add `--halo-only` to skip
libfilmshell when comparing halo_api revisions. Reports include every halo_api trial in `halo_api_samples_ms`, in execution order, as well
as the median. The output JSON drives the
static site; commit it only when you intend to publish a new measured snapshot.

The harness reads all files before timing and gives each decoder fresh owned
copies of the compressed chunks. It warms each decoder, rotates run order,
and reports medians of parse-to-return time. It also records a serialized
halo_api film fingerprint outside the timed section. Download, disk I/O, and
runtime seeks are excluded. The Rust entrypoints construct different output
models; this is a comparison of their public film parse paths. libfilmshell is
pinned in `Cargo.toml`. The single-threaded dataset measures only halo_api;
libfilmshell has no single-threaded entry point in the pinned revision.

The fixture set covers Aquarius Idle, Bazaar Idle, Octagon first-to-50, Ranked
Slayer on Live Fire, Ranked Oddball, Ranked CTF on Aquarius, and the hour-long
Clan Raid. Each tar contains a manifest and zlib-compressed chunk bytes, as
`RawFilmChunk` expects. The manifests contain only chunk kind, index, start
time, and local filename. Archive extraction happens before timing.

A focused [before/after comparison of common-name pools and incremental projection](results/name-pools-and-projection/README.md)
includes 25-trial measurements, raw samples, and fresh-process peak memory results.
