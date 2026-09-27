# Native map-entry projections

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`,
`internal/profile/map_bounds.go`. Generator:
`../reference/halo_rust_map_entry_test.go.txt`.

256 synthetic entries call the native EffectiveRegionIndexBits,
PrecisionAbsolue, Layout, Layout.Valid and Range operations directly. The
expected ranges are exported as float32 bit patterns, including negative zero.
Entries include absent/zero region widths, explicit widths through 127,
nonzero regions, valid 13/14/15 layouts and invalid axis widths through 127.
These are value-level contracts, not evidence that arbitrary widths can be
consumed by a decoder or that the synthetic geometry occurred in a recording.

Rust compares all outputs and verifies that the existing from_map and position
encoding APIs agree with the new projection helpers. Explicit assertions require
both valid and invalid layouts and historical region-width defaults.

Run TestHaloRustMapEntry from the pinned grammar package; it imports and calls
the actual profile package. Running the profile package's entire test dependency
graph on this ARM host pulls in incompatible x86 ooz sources. The grammar harness
avoids that unrelated test dependency without changing native production code.
Compress /private/tmp/halo-map-entry.json with zlib level 9 after native success.

This covers entry projections only. Catalog loading, schema checking and non-null catalog lookup were already
implemented in map_catalog.rs and covered by its 328-case native oracle. Optional
nil-catalog lookup is additionally covered by 3,280 calls in that oracle.
Platform error wording and broader filesystem failures remain separately audited.
