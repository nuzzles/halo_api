# Positive movement constructor parity (v41)

Reference: LevelUp feat/v75 at `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `reference/halo_rust_movement_positive_test.go.txt`, registered in
`reference/generate_oracles.py`. Expected values come from native
`ScanMovementStates`, not from Rust.

The 32 synthetic cases produce 64 nonempty crouch readings across two packet
chunks, slots 513 and 514, alternating crouch values and varying progress and
wire timestamps. The registry supplies `unit-crouch-component` at archetype 35.
The existing movement-source fixture has no positive readings; this fixture
adds positive constructor coverage without replacing its failure cases.

`film_movement_retention_tests::explicit_film_constructor_retains_native_movement`
replaces the oracle's synthetic chunk zero with a valid v41 bootstrap carrying
the equivalent registry, then calls the public explicit-encoding constructor.
It compares ordered movement readings and all native scan counters and verifies
Film JSON roundtrip. This verifies constructor retention, not independent
physical-action semantics or lossless archive fidelity.

Both map and explicit constructors retain failed scan reports and error text.
The replay adapter suppresses failed reports to match native
`replay/film_scan_mouvement.go`, which resets readings and stats on error.
`retained_failed_movement_is_not_published` verifies that distinction survives
JSON roundtrip; the missing-archetype constructor test covers both constructors.
