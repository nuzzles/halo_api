# Native delta-axis width fallback and boundaries

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_delta_width_fallback_test.go.txt`,
`TestHaloRustDeltaWidthFallback`, executed in native `internal/grammar`.
Registered in `generate_oracles.py`; expectations are actual native outputs.

The original 1,440 cases exposed a Rust panic when delta override zero was used
as a zero-bit width instead of selecting each traversal axis independently.
All existing fields in those cases were verified unchanged on extension.

The current 3,528 cases cross seven nonuniform traversal descriptors, seven
positive/zero overrides, eight bit alignments, three accumulator modes (absent,
unseeded, seeded), and three source prefixes. Width boundaries now include zero,
63, 64, 65, 127 and 129. Native primitive reads additionally expose each delta
field's offset, width and raw low-64-bit value. There are 345 zero-width delta
fields, 3,105 fields wider than 64, 2,079 padded endpoints and 2,401 callbacks.
Truncated headers change some branches to absolute position, retained as controls.

Both the live reader and deferred capture API compare exact native callback bits,
ordering, slot/source offset, accumulator state and subsequent read alignment.
The raw profile retains zero. Unseeded accumulation covers non-emission; positive
overrides verify precedence over differing traversal widths. DecodedComponent
JSON roundtrips and independent source-prefix retention checks also pass.

The extension first reproduced native success versus Rust refusal at width 65.
Padded component reads now consume the full width while preserving native low
bits and additionally retaining discarded source prefixes. Bounded reads remain
unchanged. See [absolute-width-boundaries-v41.md](absolute-width-boundaries-v41.md)
for the field-overlap contract, native shift behavior and complementary absolute
NaN/infinity coverage.

These are synthetic grammar cases, not independent player-action annotations.
Signed negative/overflowed native cursors and lazy conversion of unused profile
metadata on narrower targets remain separate parity gaps.
