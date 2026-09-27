# Native march slot restoration

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Four cases cross calibrated/stub widths -160 and zero. A Delete removes slot 0
before a delta on slot 1. With -160, the subsequent negative header read panics:
slot 0 stays absent and slot 1 remains present. Normal completion restores both
slots. Host compares all four; aborting WASM executes the two normal returns.
Native source: reference/halo_rust_march_rollback_test.go.txt.
See reference/signed-wrappers-validation.json for checkpoint status.
