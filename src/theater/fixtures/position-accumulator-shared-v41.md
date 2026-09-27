# Shared frame position accumulator oracle

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `reference/halo_rust_position_accumulator_shared_test.go.txt`.
Native generation passed (4.799s); registered in `generate_oracles.py`.

The 512 cases use the same native World for traversal and accumulation. Slots
0/50/51 are bound and seeded before reading; even cases attach the accumulator,
odd cases leave accumulation disabled. The fixture contains 2,560 records and
10,475 callbacks. Expected worlds come directly from the native parser.

Rust's `read_frame_records_accumulating` uses one mutable world for both roles.
The standard method covers disabled accumulation. Tests compare ordered callbacks,
record IDs/end bits, final cursor, all world slot fields, float32 position bits
and complete decoded-view JSON roundtrip. This verifies generic frames; it does
not establish all inference/recovery accumulator scopes or every lifecycle edge.
