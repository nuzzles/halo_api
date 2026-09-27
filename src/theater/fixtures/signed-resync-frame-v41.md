# Signed raw-resync frame continuation

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: reference/halo_rust_signed_resync_frame_test.go.txt.

Eight native cases combine calibrated/stub widths, optional prefixes, and
accepting/rejecting callbacks. Prefixed frames decode a first record that rewinds
to -32. The next record fails, scanning invokes acceptance on a negative landing,
and the native sequential caller rejects it (next < 0). Only the successful first
record is retained; the reader ends at bit 9. Unprefixed cases provide positive
recovery controls. Tests compare callbacks, ordered records, endpoints, bindings,
and JSON; all eight run on WASM.

Native instrumentation adds only a cursor return to DecodeFrameResync. Its source
is hash-checked by instrument_resync_cursor.go.txt, and this generator asserts that
records and acceptance calls match the original unmodified native entry point.
