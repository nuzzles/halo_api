# Vehicle source-stage observation fixture

Reference: JGtm/LevelUp feat/v75,
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_vehicle_source_logs_test.go.txt`,
`TestHaloRustVehicleSourceLogs`. Runs actual `decodeFilmVehicleScan` with the
Bazaar catalog entry, pinned captured v41 registry, and synthetic keyframe-only
chunks. The registry is byte-identical to the already-retained
`film-player-table-source-input-v41.zlib` (1973120 decompressed bytes); no second
copy is stored. Generated packets are MSB-first with native packet headers.

48 cases: 30 absent-band early returns; 18 successful vehicle scans, of which
eight lack a biped band and ten have one. All vehicle-source log records are
retained in order (66 INFO, eight WARN), removing only wall-clock time. These
include absent-band keyframe denominators, missing-biped aim warnings, death
observations and final scan summaries. Source success flags also agree. The
Rust test verifies aim-failure retention and healthy controls explicitly.

Native generation passed in 0.352s; Rust comparison passed in 0.07s. This fixture
uses no delta packets: it checks nonzero census/slot denominators and zero
creation/position/event/aim/death counts. Nonzero delta-derived summary counters,
creation/position/other optional failure diagnostics and full shared-context
behavior remain separate validation work. The already-existing lower-level
vehicle and march fixtures are complementary evidence, not replaced here.

The map-aware Film constructor enables source observations with its match ID.
Standalone lower-level scan entry points retain issues but do not emit replay
source logs. Missing biped-band aim is now retained as unavailable, rather than
silently represented as a successful empty reading; it remains additive and does
not suppress vehicle scanning. Summary logging follows the shared death pass.


## Delta-bearing extension

The suite now has 80 cases. The first 48 cases and all their previous fields
remain unchanged. Thirty-two added cases wrap known native creation payloads
(from `halo_rust_vehicle_creations_test.go.txt`) and vehicle-position packet
streams (from `halo_rust_vehicle_scan_test.go.txt`) in the same source loader,
using their matching map entries and MPP widths 9/5. The generator depends on
those harnesses' `/private/tmp/halo-vehicle-creations.json` and
`/private/tmp/halo-vehicle-scan.json` outputs. Final inputs/expectations are
retained in the compressed fixture, so Rust tests require neither temporary file.

The added cases compare 89 complete creation records, all creation counters,
122 world-position projections, 64 complete event records, aim outputs, and all
ordered source logs. Full position companions and raw ranges remain covered by
the separate vehicle scanner oracle; this source test compares position identity,
timestamp and coordinates. Across all 80 cases there are 50 successful scans,
129 INFO and nine WARN records, including 103 observed creation anchors.

The source test receives native shared-march facts as independent inputs to the
death/summary composition. It does not retest march traversal. The generator
reinstalls the same MPP widths, reads all-entity march facts, and asserts their
full statistics equal the vehicle scan's march statistics. This corrects an
initial test-input mistake that supplied vehicle-only deaths; a generated case
has one nonvehicle death and proves the all-entity diagnostic must count it.
Existing captured march tests independently cover the traversal.

Native generation passed (0.776s) and the expanded Rust comparison passed (0.20s).
Nonzero source aim/vehicle-death/occupancy summary counts are still absent from
this fixture. Other source failure diagnostics and shared-context API equivalence
remain open. This extension changes tests/evidence only, not production parsing.


## Profile and creation-failure extension

There are now 88 cases; all previous fields of the original 80 cases are unchanged.
Every case varies native inherited MPP widths and supplied calibration. Recorded
format-27 widths win, and native before/after profile snapshots agree. Rust now
runs the same width-resolution inputs before scanning. Eight new cases truncate
the retained registry to 33 through 40 blocks while retaining vehicle keyframes;
all emit the native missing-vehicle-archetype WARN and suppress source success.
Rust additionally asserts its retained issue and partial slot count.

Totals are 129 INFO and 17 WARN. Native generation passed (0.884s); the Rust
comparison passed (0.42s). See `../reference/VEHICLE_SOURCE_CONTEXT_AUDIT.md` for
the precise local restoration contract and the still-open failure/context gates.

## Positive source aim extension

The fixture now has 120 cases, preserving every field of the previous 88.
Thirty-two added cases retain matching-map vehicle creation/position inputs
and append two independently encoded sparse aim packets: known biped slot 600
and out-of-band slot 650. Yaw and pitch vary across the cases. Actual native
decodeFilmVehicleScan must publish exactly one slot-600 aim in each case; its
source packet ordinal, timestamp and raw quantized fields become the oracle.
The out-of-band packet must never publish an aim.

Rust compares all outputs and ordered diagnostics, including the positive
viseesSansPosition summary count. New cases also compare 90 creations, 125
position projections and 64 vehicle events. Overall diagnostics now total
192 INFO and 18 WARN. Native profile restoration is still asserted on every
case. The source test continues to inject native shared-march facts for summary
composition; it does not substitute for the separate march traversal tests.

Native generation passed (0.991s); focused Rust passed (0.64s). No production
code changed. This closes the previously missing positive source-aim evidence;
positive vehicle-death/occupancy summary counts and other source failure/context
contracts remain separate open gates.

## Positive death and occupancy source extension

The fixture now has 152 cases with every prior field of the first 120 unchanged.
Thirty-two new sources add a timed keyframe binding vehicle slot 512/gen 2 and
biped slot 600/gen 1, then explicit dead-state and parent-state deltas at the
pinned registry's component indices 11 and 10. Each source has a Mort=true
vehicle record followed by Mort=false, and attached then detached biped records.
The native harness asserts exactly one vehicle death at its input timestamp, no
death at the false control timestamp, and both occupancy states with the exact
parent slot/generation and timestamps. These are source wire controls, not
independent annotations of recorded real-world gameplay.

Actual native decodeFilmVehicleScan/ScanMarchFacts supplies all expectations.
The added sources contain 32 vehicle deaths, one other biped death from earlier
composed inputs, and 64 occupancy readings. Rust now runs its own complete
chronological march on these 32 sources, comparing every death/occupancy field,
coverage maps, event/located counts and retained calibration flag before using
those Rust results for source diagnostics. Existing cases retain the earlier
composition-only isolation. Across all cases there are 255 INFO and 19 WARN.

Native generation passed (1.211s), focused Rust source+march comparison passed
(1.06s). No production parser changed. Positive death/occupancy source summaries
are now covered; this does not close other source error/observer contracts or
the captured complete-film objective/recovery gates.
