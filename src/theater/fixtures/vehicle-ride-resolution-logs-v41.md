# Vehicle ride-resolution observation parity

Pinned reference: JGtm/LevelUp feat/v75,
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_vehicle_film_rides_test.go.txt`,
`TestHaloRustVehicleFilmRides`. The existing `vehicle-film-rides-v41.json.zlib`
adds actual logs from `logVehicleRideResolution(cs)` to its 512 combined cases.
Every previous fixture field was recursively compared and remains unchanged.

345 cases emit INFO, 75 also emit WARN, and 167 emit nothing. Silence requires
both zero event episodes and zero gap fallbacks. The warning requires positive
event episodes and zero event-named episodes. Counts and messages are compared
exactly; only native wall-clock time is removed. The same Rust test retains all
previous explicit/combined ride, aim, seat, complete publication and fallback
assertions. The document logs this tally after coverage and heading provenance.
These fixture expectations come from the native parser, not the Rust logger.

Rust complete ride/publication and log comparison passed in 4.39s.
