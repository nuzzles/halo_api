# Native padded zoom observations

Pinned LevelUp: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_zoom_padding_test.go.txt`, registered with the
oracle generator. Calls actual decodeZoomHead and ScanZoomEvents; a source
provenance wrapper must project exactly to the latter before exporting records.

2,304 direct inputs cover every second-byte value and lengths zero through
eight. Native admits 448 heads, 240 of which traverse a synthetic zero tail.
Some cases clear only the config bit: the direct decoder still accepts those,
whereas the source scanner's family byte excludes them. The mixed source has
308 admitted events, with repeated timestamps and filtered packet kinds.

Rust compares slot/level values, admission and reader endpoints on all direct
inputs. Source comparisons additionally check exact sorted order, wire timestamp,
payload ranges and packet ordinals. Both direct and source results roundtrip.
The fixture requires positive incomplete-reference and unrecorded-level cases.

NativeZoomRead retains raw guarded references, source length and padded extent;
unit_reference_recorded and level_recorded distinguish source-backed values from
native zero-fill outcomes. Film.native_zoom_events exposes those outcomes across
all three constructors. A two-byte zoom head survives there and in Film JSON,
while bounded zoom_events remains empty. Native padding does not establish an
observed scope exit or a fully recorded actor.

The generic head reader was factored to accept a cursor so the new native zoom
path and existing bounded heads share field decoding. The bounded API's behavior
is unchanged. This fixture is reference-parser evidence, not an independent
annotation of physical player actions or the deferred playback architecture.
