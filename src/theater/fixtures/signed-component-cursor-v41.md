# Stateful signed component cursor

Reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_signed_component_cursor_test.go.txt`, registered
in generate_oracles.py. Native consumeByName supplies the expectations.

80 cases cross empty/nonempty buffers, four component names and ten signed
starts including i64 extrema. Names cover unsupported/no-read, EMP timer,
low-frequency data and change-scene readers. There are 28 native panics and 52
nonpanicking outcomes. The oracle retains ported status, final cursor including
panic endpoints, and a reset-to-zero/read-eight recovery result and endpoint.

native_signed_component_reader_cursor compares all 80 cases on the host and
roundtrips successful component results through JSON. WASM panic=abort excludes
the 28 panic cases; its harness includes the 52 nonpanicking cases. These checks
cover direct component calls, not whole-frame post-panic state recovery, arbitrary
payload/callback equality, or successful parsing at every signed header start.
