# Native New-record terminal-tail policy

Pinned source: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81,
internal/grammar/traverse.go, TraverseEntity. Its terminal skip is guarded by
both successful component traversal and BitsDeQueueRecordNew > 0.

The 384-case fixture is produced by TestHaloRustNewTailPolicy in the saved
halo_rust_new_tail_policy_test.go.txt harness (registered in generate_oracles.py).
It covers eight tail settings (i64::MIN, -65, -1, 0, 1, 5, 63, 130), six record
paths and eight starting bit offsets. Paths are empty New, New with a fixed
component, New failing on an unsupported component, Delta, Delete and End.

Rust NativeFrameConfig now applies the native positive-only tail rule before its
checked encoding conversion. Raw signed metadata is unchanged on the reader.
Tests compare native endpoints, completion, returned record IDs/endpoints,
callbacks, world occupancy and exact admitted tail length. Positive tails retain
ordinary fields; failed New records and all other record kinds consume no tail.

This fixes the negative-tail policy, not every scalar metadata conversion.
Unused oversized positive tails, signed default-state skips, MPP dimensions,
ID settings and negative/overflowed cursor continuation remain separate audit
items. The architecture remains deferred.

Run `cargo test --lib new_record` for focused New-record checks. The existing
verify_frame_lazy_widths_wasm.py runner also includes these 384 cases (3,216
native cases total), alongside prior contextual/accumulator and width-refusal
controls. These are reference-parser comparisons, not semantic action goldens.

Validation passed: four focused New-record tests, 3,216 native cases in actual
wasm32, Clippy, formatting/diff, Python syntax and 485 pinned source hashes.
