# Generic frame position accumulator oracle

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `reference/halo_rust_position_accumulator_generic_test.go.txt`.
Native generation passed (0.534s); registered in `generate_oracles.py`.

This extends the existing native generic-position harness with a separate
accumulator world, bound/seeded at slots 0, 50 and 51. Even cases attach that
world to the reader before DecodeFrameRecords; odd cases leave it unattached.
The native context installation preserves the attached capture state. Each case
retains the frame traversal world and the separate accumulator world, native
ordered callbacks, record IDs/end bits and final cursor. The 512 cases contain
2,513 returned records and 10,141 callbacks across all callback categories.

`native_frame_accumulator_tests` consumes this fixture through the public reader,
comparing all callbacks, record IDs/end bits, final cursor and both complete worlds
with float32 position bits. The complete decoded view also roundtrips through JSON.
`read_frame_records` retains the separately attached accumulator after the call.
See the shared accumulator fixture for one-world ownership; that mode uses a
separate explicit API and does not require aliased mutable references.
