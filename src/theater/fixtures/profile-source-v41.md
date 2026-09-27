# Loaded v41 profile resolution

Reference: JGtm/LevelUp `feat/v75`, pinned at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`profile-source-v41.json.zlib` comes from the actual native ResolveProfile and
FilmContext.Profile calls in `reference/halo_rust_profile_source_test.go.txt`.
It contains 64 cases reconstructed from the retained captured bootstrap, with
explicit format/build edits and truncation offsets. Cases include missing source,
headers shorter than four/eight bytes, truncated identification, known formats
without MPP widths, format 27, unknown formats, both supported builds, unknown
builds, optional map metadata and independent copies of identity type tables.

Rust compares every identity field and boundary, highlight metadata, keyframe
framing, all movement values (including exact float32 bits), MPP and slot widths,
map values, and typed format/build issues. It preserves unread header keys and
missing identity as Option values instead of recorded zero/false assertions.
Missing maps are not native profile errors. Native resolved MPP zeros are exposed
as None; these are distinct from the default MPP widths of a scan configuration.

The lazy NativeFilmContext.profile accessor returns an owned deep copy. Mutating
its cached registry before profile access does not change profile keys: native
ResolveProfile reads original source bytes. Caller mutation of the returned
profile cannot change the context's cached profile. Complete profile JSON exports
roundtrip. This source resolver rejects non-v41 major versions rather than adding
older identification grammar.

This covers the resolved profile and lazy no-map accessor. The full native map
constructor's grammar switches, eager warning order, scan-profile replacement,
shared observers and reader/frame construction remain separate parity work.
The deferred resolved replay model is unrelated to this native parser profile.

Regeneration is registered in `reference/generate_oracles.py`. For manual use,
copy the retained harness to the pinned native grammar package and run
`TestHaloRustProfileSource`; its output is `/private/tmp/halo-profile-source.json`.
The harness reads the retained `bootstrap-v41.zlib` fixture directly.

The subsequent scan-profile-sequence-v41 fixture covers eager map construction,
scan-setting replacement/restoration and native shared calibration maps. Shared
observers, reader/frame creation and Film-pass integration are still open.
