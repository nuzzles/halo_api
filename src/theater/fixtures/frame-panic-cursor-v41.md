# Stateful frame panic and recovery

Reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_frame_panic_cursor_test.go.txt` invokes actual
DecodeFrameRecords with a stateful Lecteur and World. Expectations are generated
by the pinned native implementation, not the Rust implementation.

The 80 cases cross calibrated/stub skip modes, optional preceding deletion,
optional per-record prefixes, observer callback panic and five skip endpoints.
Endpoints include -1, -8, i64::MIN, i64::MAX-3 and an ordinary successful control.
There are 72 native panics and eight successful frames. Repeated generation is
byte-identical. The oracle retains final cursor, capture slot, ordered EMP
publications, world slots, completion status, and reset/read recovery.

The host Rust test compares all cases. The WASM harness includes only the eight
nonpanicking controls because wasm32 uses panic=abort. Rust compares preserved
identities/archetypes and deletion outcomes, not every world field or arbitrary
observer/capture mutations.

This oracle exposed two bugs: the caller retained its original cursor and slot
on unwind, and a negative optional record prefix was read instead of skipped.
Live mirrors preserve component cursor and capture slot through unwind; negative
native prefixes now retain a skip adjustment instead of accessing source data.

Remaining boundaries include signed/unaddressable header entry, raw-width limits,
and static-width configuration limits. This fixture does not establish those
contracts or complete v41 parser parity.

The source-only decode_native_frame_records adapter additionally compares all
24 applicable native cases: no observer panic and nonnegative static skip widths.
These include 16 panics and eight successful controls. It now mirrors its cursor
through unwind and preserves earlier binding mutations. WASM compares its eight
nonpanicking controls. See source-frame-panic-validation.json for check status.
