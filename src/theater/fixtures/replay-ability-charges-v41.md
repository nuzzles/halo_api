# v41 ability publication and logging oracle

Reference: JGtm/LevelUp feat/v75 at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_replay_ability_charges_test.go.txt`,
`TestHaloRustReplayAbilityCharges`, executed in the reference replay package.

The 1024 deterministic cases preserve every previous input/output field and add
actual slog records from `logAbilityCoverage` and the native
`assemblage.poserImpulsionsEtCharges` stage. The stage receives the same ranks,
lives, palette, families, tracks and clock as the publication oracle. Charge
scan availability cycles independently of impulse scan availability. Native
wall-clock log time is removed; level, message, attributes and order are retained.
There are 2474 INFO records and 598 WARN records. Both absent-component booleans
and scans that never ran are covered; missing scans use input reading counts.

Rust compares complete publication outputs and captured tracing events. The
original 1024 cases were checked field-by-field against the extended fixture
before replacing the zlib payload. Native generation passed (1.284s); focused
Rust comparison passed (1.57s). Document integration emits identity coverage
before equipment/translocation logs, then impulse/charge observations afterward.
The separate palette classification log and global document log-order parity
remain outstanding. This fixture is reference agreement, not independent
annotation of physical actions or completion of full parser parity.


Palette follow-up: the same 1024 cases now include the actual INFO record from
`assemblage.poserCapacitesEtTranslocations`, using identical ranks, inventory,
tracks, palettes and projection clock. There are 1014 unclassified cases and ten
classified cases (a: six, b: three, c: one). All previous fields, including the
3072 earlier log records, were checked unchanged. Native passed in 1.110s;
Rust complete-output/log comparison passed in 1.51s. The Rust document emits
the palette record after translocations and before impulse/charge observations.
This closes the palette-log gap mentioned above; global observation parity is
still a separate acceptance gate.
