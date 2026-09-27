# Captured raid vehicle inputs

Pinned native reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
`halo_rust_full_document_test.go.txt::TestHaloRustRaidVehicleInputs` captures the
actual `scanFilmInputs(...).Vehicles` result for raids/01-hour-long-raid. Corpo
provides the recorded Forge module's quantization bounds, as in the existing raid
oracle. The dedicated path skips document assembly and exports only Vehicles.
The native run passed in 274.512s. No Rust implementation generates expectations.

The fixture retains 150 creations, 89,945 positions, 71,488 occupant aim reads,
135 boarding/exit events, 12 vehicle deaths, and 291 occupancy reads. It also
retains census, complete creation counters, and all fourteen death-statistics
fields including the complete native configuration/profile and record maps.
Structured-key maps use the full-input harness's documented JSON-key ordering.

`write_raid_vehicle_oracle` compresses the native output separately and adds a
vehicle_inputs_fixture reference to the existing full-raid document. Existing
raid document expectations were verified unchanged. Generate with --include-raid.
The ignored local_complete_raid_document test reads that reference at runtime and
compares vehicle inputs before assembling and comparing the full replay document.
The comparison includes every exported field and record order, normalizing only
nil/empty collections and declared float32 JSON representations. Absent Rust scan
configuration projects the native zero-value struct, not configured defaults.

The ordinary six-film matrix additionally covers 12 vehicle creations, 716
positions, and two occupant-aim reads from the appearance controls; it passed in
101.37s. The full raid Rust vehicle-input and complete-document comparison passed in
278.98s. Native
reference agreement remains separate from independent semantic golden validation.
