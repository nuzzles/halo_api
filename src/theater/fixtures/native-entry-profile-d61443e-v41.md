# Native entry with explicit precision context

Reference: LevelUp `d61443ef59268ad734355db8e9974f68db5ca6d0`.
Generator: `../reference/halo_rust_native_entry_profile_d61443e_test.go.txt`.
Fixture: `native-entry-profile-d61443e-v41.json.zlib`.
SHA-256: `56e6c6a31e02445566fd904b6d659085195a392ee8e74f1992e7b2ac53f66e1b`.

The generator takes the 768 independently generated native action/control inputs,
wraps each control body in a complete native frame, and runs Go's
`DecodeFrameViewsCurseur` under its declared precision profile. It saves the ending
cursor, completed views, and control verdict with all typed action values. No Rust
output is used to create expectations. Go production files are unchanged.

Coverage includes axis widths [1,2,3], [13,13,14], [17,17,15], index widths 1..3,
full precision, quantized reads, and baseline scope. There are 105 distinct ending
cursors. Every generated frame closes under its declared profile.

Rust tests wrap these frame payloads in native packets and a minimal recorded v41
bootstrap, then call `NativeFilmData::parse_v41_with_options`. They compare the Go
endpoints, view count, control closure, actor index and typed action values, check
all retained control fields against source bits, retain original source bytes,
and round-trip options and results. A negative control parses without the supplied
profile and requires at least one different result, so ignored context cannot pass.
Separate checks cover old option JSON, inherited corruption-control precedence,
and frozen calibration maps.

`NativeFilmDataOptions::frame_profile` is external parser context, not recorded
map data. Its effective value is retained in `NativeFilmData::frame_config`.
`None` preserves default behavior. The native context's recorded/inherited
corruption setting takes precedence over profile replacement. Translocator map
context remains its existing separate option.

The shared frame configuration also feeds sequential keyframes, candidate body
reads and event continuations. These tests establish explicit profile propagation
through the native frame entry; they do not prove missing map widths for captured
films or eliminate the observed simulation-policy refusals. Unknown component
and runtime grammars remain unsupported. No resolved replay model is involved.
