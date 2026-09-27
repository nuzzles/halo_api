# Equipment-change publication and logging oracle

Reference: LevelUp feat/v75 at 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_replay_equipment_publication_test.go.txt,
installed in internal/games/halo_infinite/film/replay as a Go test. Run
TestHaloRustReplayEquipmentPublication. The fixture is the generated
/private/tmp/halo-replay-equipment-publication.json compressed with zlib level 9.

All 1024 cases call buildEquipmentChanges, filter through the actual native
keepEquipmentChangesOfPublishedTracks, and capture logEquipmentChangeCoverage
using slog's JSON handler. Only wall-clock time is removed. Level, message and
all twelve attributes are compared with an actual Rust tracing event. The Rust
capture is thread-local and requires exactly one event. Existing input, stats,
clock, slot and publication fixture fields were verified unchanged.

Controls include empty input, zero step, spawned records, records before origin,
filtered tracks, recovered records, counter gaps and nonzero scanner statistics.
Coverage remains measured before track filtering; it is not reconstructed from
the smaller published list. Standalone builders remain free of logging effects;
document assembly emits this coverage immediately before translocation coverage,
matching the native relative order of these two layers. Global ordering against
other not-yet-ported log records is not established by this fixture.

These are synthetic reference comparisons, not independently observed actions.
Negative arbitrary Go enum values are outside the typed Rust enum contract.
