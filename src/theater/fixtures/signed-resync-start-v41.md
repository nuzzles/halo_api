# Signed raw-resync scan starts

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: reference/halo_rust_signed_resync_start_test.go.txt.

108 native cases combine empty, one-byte and 16-byte payloads, nine signed
starts, optional prefixes, and accepting/rejecting callbacks. There are 38 native
panics and 70 successful calls. A prefixed scan starting at -32 accepts the target
at bit zero but returns the original signed landing -32.

Both contextual and standalone Rust raw scanners are compared for panic outcome,
landing, acceptance calls and unchanged world. The contextual callback also checks
position/has-position arguments. WASM runs the 70 nonpanicking cases for each API.
The sequential caller's different treatment of negative landings is covered in
signed-resync-frame-v41.md. Native starts at i64::MIN requiring impractical scans
are not included in this finite oracle.
