# Native external geometry loader oracle

Pin: LevelUp feat/v75 at 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: ../reference/halo_rust_geometry_loader_test.go.txt, registered in
`generate_oracles.py` as `TestHaloRustGeometryLoader`.

The generator calls production `parseF32`, `readCSV`, and `LoadGeometry`. It does
not compute expectations using Rust. Inputs include deterministic generated
numbers, exhaustive four-byte CSV tails over five symbols, and curated malformed
and valid files.

- 2,009 float conversions: decimal and hex, underscores, signs, whitespace,
  malformed syntax, overflow, subnormal rounding, exact halfway cases, and
  nonfinite values. Expectations are IEEE f32 bits, including signed zero.
- 658 CSV inputs: 334 accepted and 324 rejected. Successful rows and column keys
  are byte-encoded to preserve invalid UTF-8. Failure expectations retain native
  category, header/record stage, start line, physical line and byte column.
- 4,046 complete loads: 4,037 successes and nine errors. Covers missing catalogs
  and maps, empty files, duplicate columns/types, short/long rows, malformed IDs,
  unmeasured/nonpositive sizes, ordering, unknown-type counts, late parse errors,
  directory reads, rounding and nonfinite results. Every accepted object has its
  type ID and all six f32 field bits compared. Sixteen results fail native JSON
  encoding; the Rust result wrapper must also reject serialization.

In-memory tests exclude two missing-catalog cases and two directory-read cases,
which require filesystem behavior; all 4,046 run through the host filesystem
adapter. The actual WASM suite runs the public in-memory adapter.

Rust I/O errors retain the selected file and `std::io::Error` rather than native
French wrapping text or its header label for a directory read. CSV errors retain
structured native positions and categories; their Display wording differs.
Native nil object slices map to an empty Rust vector. These are explicit API
representation choices, not additional film data. These assets are external to
the recording; this fixture does not validate recorded actions or map extraction.
