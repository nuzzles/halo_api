# Kill timeline registry admission

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_kill_registry_test.go.txt`, registered in
`generate_oracles.py` as `TestHaloRustKillRegistry` in `internal/facts/killsource`.

The generator calls actual native `newTimeline` with a loaded source. Its 20
cases cover empty/short data, the eight-byte header, partial and whole registry
blocks, a named component with precision level 3, structural endings, malformed
zlib-looking input and a non-zlib control. The fixture stores the bytes after
source loading, so timeline admission is compared at the same boundary.

Rust compares success/error, full ordered archetypes/components/levels,
truncation and trailing-byte count. It also checks the typed compressed error
and the empty chunk-list error. Metadata index 19 verifies that source position,
not manifest index zero, selects the registry. Native nil component arrays are
normalized to empty arrays for typed Rust comparison. No keyframe records are
supplied; existing timeline and captured kill fixtures cover those paths.

The adapter now accepts truncated registries, as native does. Its
`registry_truncated_bytes` is `Some` for chunk construction and `None` for a
caller-supplied registry, whose source length is unavailable. The full decoder
still validates replication and kill-feed availability before this constructor.
This is current parser parity work; the deferred Film architecture is unchanged.
