# Catalog float32 rounding oracle

Pinned reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_map_rounding_test.go.txt`, registered in
`generate_oracles.py`. Calls the actual native catalog file loader and lookup.

1,601 decimal tokens exercise all finite exponent bands, selected subnormal and
normal boundaries, exact adjacent-float midpoints and perturbations on both sides,
positive/negative values, signed zero, underflow and the maximum finite boundary.
Midpoints and perturbations are generated with exact rational arithmetic. Epsilon
is the float32 spacing divided by 2^64; 220 decimal places preserve these binary
rational values exactly. Expectations are native acceptance and raw float32 bits.

Native outcomes: 1,598 accepted, three rejected, five negative-zero results.
Rust compares all acceptance decisions and every accepted bit pattern through
parse_film_map_catalog, rather than passing the token through serde_json::Value.
A separate Python float64-to-float32 control differs on 531 accepted tokens,
demonstrating that ordinary double-precision parsing would miss these boundaries.
The current serde float_roundtrip f32 path passes without a production change.

This is broad adversarial rounding evidence, not exhaustive enumeration of all
possible decimal strings or a claim about invalid platform filesystem operations.
Regeneration writes /private/tmp/halo-map-rounding.json, then compresses it with
zlib level 9 after native test success.
