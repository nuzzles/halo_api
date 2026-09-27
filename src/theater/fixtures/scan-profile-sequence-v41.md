# Native scan-profile sequences and map contexts

Pinned reference: JGtm/LevelUp `feat/v75`, commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The retained `reference/halo_rust_scan_profile_sequence_test.go.txt` calls actual
native FilmContext constructors and setters in 96 sequences. Sources use the
captured v41 bootstrap with absent, false or true corruption declarations;
map entries include absent, valid and zero-axis metadata, plus deliberately
invalid forced layouts. The fixture retains these inputs and nine snapshots per
sequence: initial, replacement input, previous, installed, shared-map mutation,
MPP replacement, raw precision replacement, layout installation and restoration.
All snapshots are serialized immediately to avoid later map alias mutations
changing earlier expectations.

Rust compares every movement/keyframe/MPP/grammar field. It verifies the native
split between scalar copies and shared calibration maps, negative raw MPP widths,
corruption-control override after replacement, fallback declaration status,
zero-axis refusal, preservation of index width for gates <=4, region changes,
and scalar restoration. Raw precision widths include 2^40; the native metadata
uses u64 in Rust to avoid truncation on WASM. Reader width validation is separate.
Nil maps and allocated empty maps remain distinct. Serialized snapshots preserve
values; deserialization does not reconstruct cross-object alias identity.

The map constructor resolves profiles eagerly and enables simulation completion
when a forced or valid catalog layout is imposed. It does not install world
precision; that is a separate native operation. A separate 128-case constructor
test compares actual Rust tracing output against `profile-warning-v41.json`,
checks eager versus lazy cache initialization, and confirms that later profile
reads/replacements do not repeat the constructor warning. Lazy construction is
silent. This covers the warning payload/gate, not unrelated decoder logging.

Regeneration is registered in `reference/generate_oracles.py`. Manual native
execution runs `TestHaloRustScanProfileSequence` and writes
`/private/tmp/halo-scan-profile-sequence.json`. It reads the retained bootstrap
fixture directly. Shared native observers, reader/frame construction, and
integration of this context with existing Film passes remain separate work.
