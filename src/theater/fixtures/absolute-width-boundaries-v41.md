# Native position width boundaries

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_delta_width_fallback_test.go.txt`,
`TestHaloRustAbsoluteWidthBoundaries`, executed in native `internal/grammar`.
Registered in `generate_oracles.py`; expectations are actual native outputs.

The 720 cases cross five world-axis descriptors, eight bit alignments, three
accumulator modes, zero/all-one source patterns, and three byte prefixes. Widths
include zero, 63, 64, 65, 127 and 129. The fixture contains 720 zero-width fields,
576 fields wider than 64, 318 padded endpoints and 720 callbacks. The callbacks
contain 720 nonfinite vector components, retained as exact IEEE-754 bits.

Tests compare the actual native dispatcher with NativeFilmReader and the deferred
capture_component_position API: status, field offsets/widths/low-64-bit values,
ordered position callbacks, world positions, and subsequent read alignment.
Complete DecodedComponent JSON roundtrips retain these values. NaN and infinity
expectations are compared by bits, never JSON floating-point numbers.

Go uint64 shifts yield zero for counts of 64 or more. A zero-width delta's unsigned
width-minus-one also yields a shift with zero result. Both immediate and deferred
Rust capture use that behavior. Wide native scalar reads consume their full width
and return the last 64 bits. Bounded readers still refuse widths above 64 without
advancing. Unrepresentable address arithmetic remains an explicit refusal.

Rust additionally retains every actual source bit discarded by the native numeric
accumulator in `<field>.discarded[N]` fields, split into at most 64 bits each.
The primary field records its full source span and native low-64-bit result, so
it overlaps these prefix fields. Consumers must use offsets, not sum field widths,
when measuring coverage. Prefix retention is bounded by actual source length;
synthetic zero padding is not expanded. Tests independently check each retained
prefix against input bits, including zero-valued source bits and complete coverage.

These are synthetic native grammar cases. They do not prove signed negative-cursor
continuation, lazy conversion of unused widths on WASM, independently annotated
player actions, or full captured-film acceptance.
