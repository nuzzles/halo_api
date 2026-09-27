# Inventory publication, death attribution and empty-read logging

Pinned LevelUp feat/v75: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_replay_inventory_publication_test.go.txt,
installed in the native replay package. Run TestHaloRustReplayInventoryPublication;
the fixture is /private/tmp/halo-replay-inventory-publication.json, zlib level 9.

All prior fields in 1024 cases were verified unchanged before extending the
fixture. Actual native logInventoryEmptyCoverage output contributes 887 INFO
records and 137 silent controls. Rust compares actual tracing events including
level, message, reads, empties, newly death-marked and unexplained counts, plus
all existing inventory/death-attribution outputs. Only wall-clock time is removed.

The logger runs after published-track filtering and death marking. Any nonempty
empty-reason string contributes to the empty count; no log is emitted if that
count is zero. This is a diagnostic about retained inventory observations, not
an assertion that every empty inventory is caused by death. Other source/setup
failures and independent gameplay validation retain separate acceptance gates.
