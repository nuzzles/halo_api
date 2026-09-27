# Vehicle heading-source observation parity

Pinned reference: JGtm/LevelUp feat/v75,
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_replay_vehicle_tracks_test.go.txt`,
`TestHaloRustReplayVehicleTracks`. The existing
`replay-vehicle-tracks-v41.json.zlib` now also contains actual slog records from
`logVehicleHeadingSource(all)` for all 1024 cases. Every old field was recursively
compared with the new fixture and preserved unchanged.

The source cloud has 24776 samples: 4611 film headings, 8717 velocity fallbacks,
11448 without heading; 6000 velocity fallbacks also have roll from an unpublished
mode. Rust compares every log level, message and attribute, after removing only
native wall-clock time. Empty clouds still emit the native INFO with zero counts.
Existing sample/track/coverage/family/heading oracles run in the same test.

Native heading and ride fixture generation passed together in 1.143s. Rust
heading/track comparison passed in 1.39s. Document assembly logs the full accepted
vehicle cloud before decimation, after vehicle coverage and before ride
resolution. This is reference-parser observation parity, not an independent
physical-action annotation or the deferred playback architecture.
