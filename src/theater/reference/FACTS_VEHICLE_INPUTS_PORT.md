# Cached vehicle lifetimes, tracks, rider aim, and written occupancy

build_facts_replay_vehicle_lives and assign_facts_replay_vehicle_deaths share
census-window and death-assignment reducers with recording inputs. They retain
full slot/generation keys, original census-array order, native lower-bound search,
reuse windows, earliest written death, timestamp-zero sentinel behavior and all
unmatched/tail-desync tallies. Cache assembly consumes the supplied death slice
without re-filtering archetypes or interpreting dead-state payload; fresh scan
filtering belongs to its earlier native boundary.

replay_facts_vehicle_positions_by_slot filters only HasWorld, then stably sorts
each slot by timestamp while retaining complete cache rows. Heading queries use
native configuration-mode chassis orientation first, then checked velocity above
the native horizontal-speed threshold. Presence flags, defaults and full u32 codes
are retained. Velocity magnitude saturates natively rather than narrowing before
decoding. native_film_chassis_heading is the shared consumed-field calculation;
it does not manufacture source provenance or a delta flag.

build_facts_replay_vehicle_samples shares source publication: inclusive life
windows, carried heading, native decimal rounding, frame decimation, and last-seen
time updates even when a reading does not publish a new frame. The caller performs
world filtering and slot grouping first, as the native pipeline does.

The new native fixtures cover 1,024 census/death cases (including irregular census
order, u32 generation and timestamp/frame extremes) and 1,024 heading/grouping/sample
cases with full u32 direction/roll/magnitude values and unsupported mode bytes.
The native lifetime sort is only (slot, first timestamp), so ties across generations
are map-order dependent in Go; these deterministic oracle cases use distinct first
timestamps per same-slot generation. Do not claim a unique native order for tied
cases. See facts-vehicle-inputs-validation.json for completed checks.

replay_facts_vehicle_spawns_by_life shares earliest-timestamp/first-on-tie
selection and retains the complete selected cache creation. Drawable selection
shares the native lower-bound search and last-supplied-life behavior for duplicate
keys. build_facts_replay_vehicle_track consumes spawn time, world position and
presence-gated low-u32 chassis identity through a private value view, preserving
zero-timestamp absence, bounds, family suppression of rides, and decimal rounding.
The new 1,024-case track oracle compares complete native cache creation buffers,
drawability, and published tracks. Additional expected fields in its JSON are not
implicitly covered by that test.

replay_facts_vehicle_aim_by_slot retains complete cached rows with stable sorting.
build_facts_replay_vehicle_ride_aim shares source sampling through raw consumed
fields: inclusive windows, native binary search on supplied ordering, first reading
per frame, full-u32 yaw/pitch calculation, and publication rounding. No interpolation
or synthetic source coordinates are introduced. Its oracle covers 1,024 groups and
12,288 sampling probes, including irregular arrays and extreme timestamps/clocks.

build_facts_replay_film_vehicle_rides shares the written-occupancy reducer through
FactsReplayVehicleRideContext. ReplayVehicleRideContext has default generic types
preserving the source API; its borrowed context remains Copy without requiring
record types to be Copy. Occupancy keeps the original cache/native type. The first
matching slot/lifetime wins regardless of parent generation; the next occupancy
reading, next world position, and life end compete with native tie precedence.
Written rides retain film provenance, full-u32 seats, temporal XUID lookup, aim,
and the separate occupant/window contradiction indexes. Its native fixture has
1,024 cases, 1,715 published rides and 20,480 contradiction probes (684 true,
19,796 false), with full-width generation/seat/aim and signed frame-count extremes.
See facts-vehicle-inputs-validation.json for actual completed checks.

## Combined rides and cached vehicle publication

build_facts_replay_vehicle_rides shares event/gap resolution with the recording
entry point. Its private VehicleRideEvent contains only consumed values, preserving
signed i64 kinds without constructing source VehicleEvent coordinates. Unsupported
kinds whose low byte equals 8 or 22 remain unsupported. Complete cached event merges
retain every cached field and give exits precedence at equal timestamps. Public
cache episode helpers accept pure ReplayPlayerPosition projections; these carry no
source coordinates. The shared reducer preserves nearest-event selection,
geometry fallback, open-episode reappearance limits, written-occupancy contradiction,
resolved-episode suppression of gaps, first matching seat, and final stable ordering.
Event/geometry output retains native proximity provenance, never film provenance.

The dedicated 1,024-case native oracle compares full event merges, per-occupant and
combined episode output, coverage probes, seat assignment, final rides and tallies.
It exercises 44 direct event resolutions, 75 nearest-life event resolutions,
188 geometry resolutions, 568 gap fallbacks, 100 contradiction rejections and
1,030 seat assignments. Expected logging fields present in the fixture are not
implicitly tested; source logging regressions remain separate evidence.

build_facts_replay_vehicle_publication now composes all cached vehicle inputs,
sharing finish_vehicle_publication with the recording path. Scanned and nonzero
step gates precede assembly, while aim_reads is still reported for unscanned input.
The default-frame fallback comes from cached death stats. Supplied deaths are not
re-filtered. Tracks are sorted, relays merged, coverage tallied, and cycles computed
in native order. The full cache oracle encodes and decodes VehicleScan natively
before building expectations and compares Rust decoding plus complete publication.
Its 1,024 cases include 3,315 published tracks, 156 relay merges, 5 accepted cycles,
76 cycle gaps, 3,208 published rides and 829 unknown-family publications.
See facts-vehicle-inputs-validation.json for completed validation status.

replay_facts_vehicle_heading_sources now counts every supplied cache reading before
world filtering or decimation, sharing the source provenance reducer and log method.
Its 1,024 native comparisons use heading_log fields from the dedicated track fixture;
source logging regressions verify the shared emitted fields and message.

Still pending: vehicle shot attachment at the complete
document boundary, objectives, full BuildFromFacts composition, scan capture and
fallback restoration, source/field/runtime audit, captured-film comparison, and
legacy raw-string/counter reconciliation. The Film/ResolvedFilm/playback architecture
remains deferred.
