# Stateful native generic record-loop context

Pinned native commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_context_frame_test.go.txt`, registered in
`generate_oracles.py`. The native `DecodeFrameRecords` implementation produces
all expected records, callbacks, final reader positions and world slots.

128 cases contain creation, delta, deletion, recreation and a final matching or
mismatching generation. Variations include extra prefixes/guards, corruption
checks, strict generation mode, preamble widths, 19 nonzero initial reader
positions, and truncated payloads. The oracle has 584 records, 888 callbacks,
110 completed loops, and 25 reads beyond the source tail.

The input reader has an observer that must be replaced by the frame context.
EMP callbacks mutate the shared calibration map before the next component reads
its width. The rounds component must use that new width and emit no callback;
crouch callbacks must retain their recorded values and record slots. World
snapshots retain an unrelated slot's existing position and view while clean
creations/deletions change the target slot, including its generation and view.

Rust compares every record ID/header boundary, success/desynchronization,
non-deletion trace end and mask, ordered component attempts/start bits, callback
order/values, final cursor/capture slot, live calibration width, world slots and
record serialization. The reference supplies no deletion Trace.EndBit; final
cursor and subsequent header boundaries validate its consumption. Rust retains
End explicitly while the native result list omits it. Position values in world
snapshots are compared as f32, avoiding JSON integer/float spelling differences.

This covers the generic record loop. View-class dispatch, inference, resync,
record-mask scanning, optional position accumulation, unsupported metadata widths
and constructor-wide Film integration remain separate work. The native loop does
not automatically attach its World as a position accumulator.
