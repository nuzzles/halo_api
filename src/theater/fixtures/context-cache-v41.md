# Native source-context caches

Pinned reference: JGtm/LevelUp `feat/v75`, commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The fixture is zlib-compressed JSON from
`reference/halo_rust_context_cache_test.go.txt`. The native harness uses the 144
source layouts generated independently by `TestHaloRustI0Loaded`, loads those
bytes through native `source.Load`, and calls actual FilmContext accessors.
It verifies lazy construction, repeated registry pointer/error identity,
mutation visibility, parse-time fingerprint/count stability, and repeated layout
value/error identity. It records native chunk prefixes, biped membership, registry
fields before/after mutation, forced layouts, layout values and error presence.

There are 36 successful layout results: 29 forced (deliberately invalid, still
accepted) and seven detected. Other layouts retain native refusal behavior.
Registry cases cycle through missing, valid, short and still-compressed input.
The input source cannot be externally mutated while borrowed by the Rust context;
mutable access to its cached registry is explicit. Registry fingerprint/count
accessors preserve the original parse-time identity even after mutation.

Rust additionally checks that source buffers and packet tables are borrowed,
repeated result references have stable identity, construction has not populated
caches, imposed-layout copies cannot mutate the context, and a missing source
retains missing-registry/no-chunks outcomes. Individual layout detector errors and
measurements have their separate `i0-loaded-v41` oracle; this fixture checks native
context composition and caching, not a new layout algorithm.

This fixture covers NativeFilmContext's source, registry, biped-band and layout
caches. Lazy resolved profiles are covered separately by profile-source-v41.
The context does not replace Film assembly or start the deferred replay
architecture. Complete map-constructor composition/warnings, scan-profile
setters, shared observers and reader/frame creation remain required parity work.

Regeneration is registered immediately after `TestHaloRustI0Loaded` in
`reference/generate_oracles.py`. Standalone generation requires its
`/private/tmp/halo-i0-loaded.json` output first, then runs
`TestHaloRustContextCache` in the pinned grammar package, producing
`/private/tmp/halo-context-cache.json`.

The subsequent scan-profile-sequence-v41 fixture covers eager map construction,
scan-setting replacement/restoration and native shared calibration maps. Shared
observers, reader/frame creation and Film-pass integration are still open.
