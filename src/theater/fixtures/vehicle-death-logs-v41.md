# Vehicle death-read diagnostics

Pinned reference: JGtm/LevelUp feat/v75,
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_vehicle_death_logs_test.go.txt`,
`TestHaloRustVehicleDeathLogs`, executed in the native replay package.

512 cases capture actual `logVehicleDeathReads` slog output: 256 INFO, 256 WARN.
Each case includes mixed archetype 35/40/42 deaths and independent per-archetype
record, clean-record, declared-mask and desynchronized-mask maps. Missing ti=40
keys and positive losses on other archetypes test that only vehicle losses select
WARN. Empty/non-ASCII match IDs, absent calibration, retained/default calibration,
and independent event/localized packet counts are included. Native wall-clock
time is removed; all other fields and level are compared exactly.

Rust consumes existing `FilmMarchFacts`, filters deaths by archetype 40 and
projects the retained calibration/default flag. Source integration runs only
after the vehicle scan succeeds and the shared march result exists, immediately
after native vehicle scanning in the map-aware Film constructor. No second
march is performed and death/occupancy data are unchanged. The source-stage gate
was inspected against `decodeFilmVehicleScan`; this isolated oracle proves the
logger mapping, not all vehicle failure/context paths or captured-film coverage.

Native generation passed (0.739s); Rust comparison passed (0.13s). Failure-stage
warnings and the remaining vehicle source summary/context gates stay open.
