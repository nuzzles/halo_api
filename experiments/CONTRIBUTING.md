# Contributing to Theater Lab

Supported parsing and captured regression fixtures belong in `../src/theater/`.
Follow the root `CONTRIBUTING.md` for upstream changes. Keep filesystem access,
download tools, visualizations and new exploratory work in this directory.
Commit source, research notes and `films.csv`; keep downloaded/generated data in
the gitignored `films/` directory. Build outputs and local recovery archives are
also ignored; do not force-add them.

Use `cargo test --manifest-path ../Cargo.toml --lib theater` for the fast decoder
loop. Avoid rescanning the corpus or compiling historical probes for each edit.
The current scripts and optional corpus/browser checks are in [README.md](README.md).

The active catalog has 32 films. Retired forced-end recordings and old tools are
recoverable from the local `archive/cleanup-2026-09-10/` when available; this
archive is not included in Git. Add entries explicitly to films.csv
when restoring a film to the active lab. A catalog row requires its category,
group/slug, canonical match ID, description and analysis profile.

Preserve research notes and captured fixtures even when retiring full recordings.
Record match/chunk/payload/bit offsets and independent evidence for new claims.
Do not turn unknown values into initial defaults, equate firing events with
bullets, or infer grenade paths from a simulated arc. The nine modules under
`examples/legacy/` exist solely to retain exact byte-inspector annotations until
that presentation can use equivalent upstream scalar ranges.
