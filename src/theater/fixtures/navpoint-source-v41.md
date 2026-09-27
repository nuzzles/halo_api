# Navpoint source and Film constructor oracle

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_navpoint_source_test.go.txt`.

The 64 sources reuse the first 96 primitive navpoint cases, excluding corruption
checks because the synthetic bootstrap has no build identity and resolves to
unchecked traversal. These are synthetic wire cases, not captured bomb films.
Each source contains three delta packets and a keyframe, with independent wire
and manifest clocks. The scanner discovers its own slot band from the keyframe.

The native harness calls `ScanNavpointRadial` and records its complete output.
It also wraps the native walk's observer and verifies that the instrumented walk
produces exactly the source scanner's output before exporting its ordered
callbacks. Both native and Rust parse equivalent bootstrap bytes. Injecting an
untrimmed component-name list would incorrectly preserve a trailing empty name;
the first fixture attempt caught this mismatch and was corrected before acceptance.

Expected output: 2,313 accepted readings and 2,374 callbacks, including 491
keyframe callbacks and 1,883 delta callbacks. The callback total deliberately
includes publications outside the final accepted readings.

The Rust constructor test runs all 64 cases through both explicit-encoding and
map-aware constructors. It compares complete accepted scans and ordered callback
fields/values, validates each retained packet ordinal, source metadata, record
start and projected timestamp, then roundtrips the complete Film through JSON.
This checks export retention; it does not claim source-byte archival or native
byte-for-byte re-encoding. Positive delta/keyframe emissions and broken walks
are required by explicit aggregate assertions.

Regeneration: decompress `navpoint-radial-scan-v41.json.zlib` into
`/private/tmp/halo-navpoint-primitive.json`, copy the retained generator into the
pinned grammar package as a `_test.go` file, run `TestHaloRustNavpointSource`, and
zlib-compress `/private/tmp/halo-navpoint-source.json` at level 9. The existing
primitive generator supplies the wire inputs; Rust does not generate expectations.
