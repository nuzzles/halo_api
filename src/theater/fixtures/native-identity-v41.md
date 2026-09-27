# Native identity-section retention

Reference commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Reference layout: `internal/grammar/film_identity.go:16-29`, with bounded build
anchor selection shared with `decode_film_identity`.

The native reader retains the per-type words, three 32-byte string fields, build
and changelist words, corruption flag, two 256-byte name fields, four words
(including the timestamp), three 4096-byte blocks and two 16-byte blocks. Source
coordinates are bootstrap-relative bits. Scalars retain wire bit order; the
legacy identity separately applies endian conversions. Fixed byte fields retain
all bytes, including string terminators and tails. Block extents are known but
remain opaque; their contents are in the retained bootstrap source.

Tests use the existing independently generated `bootstrap-v41-oracle.json` and
captured `bootstrap-v41.zlib`. The expected ordered widths come from the reference
writer map, not from reader output. Direct bit extraction independently checks
scalar and byte values. The identity projection and final body bit match the
reference. Controlled changes to a string tail and an uninterpreted word verify
that information survives even when the reduced identity is unchanged. Truncation
checks retain preceding fields and the first unavailable requested range. Missing
anchors produce no guessed fields. The native-entry corpus additionally checks
integration and identity endpoints across all 32 films.

These checks establish retention of the documented identity layout, not a grammar
for its opaque blocks. Build-anchor selection remains the reference's bounded
search; it is not upgraded into a canonical boundary proof. Partial reads never
become a complete legacy identity. The original error remains available.

Reproduce:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib native_identity_ -- --nocapture
CARGO_INCREMENTAL=0 cargo test --release --lib bootstrap::tests -- --nocapture
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_ -- --include-ignored --nocapture
```
