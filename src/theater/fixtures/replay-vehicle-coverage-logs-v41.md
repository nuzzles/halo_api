# Native vehicle coverage observations

Reference: JGtm/LevelUp feat/v75 at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_vehicle_coverage_logs_test.go.txt`,
`TestHaloRustVehicleCoverageLogs`, run in the native replay package.

513 cases compare actual `logVehicleCoverage` slog events against Rust tracing.
One nil coverage case emits nothing. Every nonnil case emits three INFO records;
conditional warnings cover positive, zero and negative threshold boundaries,
absent aim, unresolved chassis, and zero to three unknown chassis keys. Counts
vary independently to detect incorrect field mappings. The all-zero control
still emits the native three summaries. Total: 1536 INFO and 937 WARN records.

Only wall-clock log time is removed. The contiguous unknown-chassis warning
block is sorted by chassis because Go map traversal order is unspecified;
all surrounding event order and all attributes are compared exactly. Rust uses
stable BTreeMap order without dropping or aggregating warnings. These synthetic
logger cases complement the existing 1024 track/coverage and 512 combined vehicle
publication cases; they are not independently annotated physical-action films.

Native generation passed (0.678s); Rust comparison passed (0.10s). The document
calls the logger after attaching vehicle tracks/cycles/coverage, including empty
unscanned coverage. Adjacent heading-source and ride-resolution logs, vehicle
source diagnostics and broader build context remain separate audit items.
