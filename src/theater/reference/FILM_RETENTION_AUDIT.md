# Current Film field-retention audit

Scope: existing v41 parser and portable Film, before the deferred architecture.
Reference commit: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
This is a field-level audit, not a claim of whole-source losslessness.

| Decode result | Film mapping | Evidence and limits |
| --- | --- | --- |
| Native weapon-shot/damage reads, pairing and distance evidence | Film.weapon_hits and weapon_hits_error | Constructors now retain direct results automatically; map constructors refresh with resolved sampling precision. Errors preserve other Film data; failed explicit refresh preserves previous results. See the automatic weapon-hit retention checkpoint below. |
| Registry archetype Index, Components, Levels | Film.registry.archetypes | Native parse-result fixture (116 cases) compares order, index, all names and levels; typed JSON roundtrip. |
| Header major/format words | Film.registry.major_version / format_version | Compare both physical LE words with FilmRegistryRead.header and registry fields; Film constructor requires a complete header. Short native reads remain available through parse_registry_chunk. |
| Registry Truncated | Film.registry.truncated | Native parse-result and 256 edge cases distinguish structural termination from aligned/unaligned truncation. |
| Registry TruncatedBytes | Film.registry_diagnostics.truncated_bytes | Retained with source chunk number and length. Existing Film pipeline regression covers typed export and old exports lacking diagnostics. |
| Registry cached fingerprint and namedSlots | FilmRegistry.fingerprint() and sum of component counts; also identity fields when identity exists | Native edge fixtures verify parsed values. Rust recomputes fingerprint while native caches it: arbitrary post-parse mutation/default-construction accessor parity is not established. |
| Session population entries and RosterReport | Film.roster_updates[].roster | Constructor stores the complete decode_roster_update result with source packet. Leaf oracle compares every serialized entry/report field and stopping position, then roundtrips. Constructor selects this path only for its supported build identities. |
| Footer events, counts and unsupported types | Film.summary_events and optional summary_diagnostics | Previous loss of packet counts/types fixed. Pipeline test covers empty/mismatched counts, unsupported types, compact export and missing fields in old JSON. |
| Packet header type, bytes 2/3, size, timestamp and range | Film.packets | Preserved by the indexed packet type. Compact mode explicitly omits this vector. This row does not establish source-byte retention. |
| Source player-table decoder refusal and unknown-build metric increment | Film.player_table_diagnostics | Typed native error and per-call increment derived from the already decoded table; missing identity remains explicit. Source oracle compares exact native error text and actual expvar deltas across 20 cases. Film constructor/JSON checks cover success, unknown build and no identity; old exports retain None. |

## Open questions

- FilmSource owns loaded decompressed bytes, but the current Film export does not
  embed them. Checked-region complements do not themselves retain unknown bytes.
  Do not describe Film JSON alone as a lossless source archive. The proposed
  native hierarchy/source ownership belongs to NEXT_PHASE.md and is not started.
- Mobility callbacks and named component callbacks have separate ordered vectors.
  Individual DecodedComponent values now expose ordered_publications: the native
  mobility callback precedes its optional movement-state publication; other
  direct components contain no mobility callback. A 1536-read native experiment
  checks 2640 publications, including 256 mixed-family reads, suppression, failed
  reads and restored profiles/observers. This does not recover chronology from
  merged scans by inference alone. New FilmReadDiagnostics.mobility_offsets now
  explicitly preserves aggregate interleaving through merges, suppression and
  detached resync routing. Four native repair/harvest fixtures compare 89056
  measured offsets. The movement resync fixture additionally compares 92 scan
  and 107 full-frame callback offsets across 512 cases, including caller-visible
  routing and JSON roundtrips. Old exports without offsets report unavailable
  ordering. Malformed incoming offsets must remain unavailable when shifted,
  including values that would otherwise overflow usize.
  Other publication sinks still need separate integration audits.
- This table covers the named rows only. Other decoder results and their
  constructor/portable propagation still require audit.

## Production-frame optional record prefixes

The production-frame API now supports the reference inference loop's optional
32-bit word before each entity header. ProductionFrame.record_prefixes retains
words in read order, with payload-relative bit offset and width, including words
before End and rejected headers. The frame's padded_bits still identifies how
far decoding read beyond the physical payload. Old/default exports omit the
empty vector and deserialize it as empty; the previous production API rejected
this option. This is distinct from generic-loop NEW/DEL guard handling, which
this native production loop does not perform.

Evidence: movement-hook-production-v41 extends 512 unchanged original cases with
512 prefixed cases (2786 records, including 420 deletes). The native oracle
compares ordered component callbacks, record ends, view completion, final cursor
and final entity bindings. Rust additionally checks retained prefix words against
source bits, prefix/header boundaries and a complete ProductionFrame roundtrip.
This proves this option in the standalone production API; the existing high-level
movement scanner and strict event locator still reject extra_fields and are not
covered by this expansion. No broader Film/source losslessness claim follows.

## Production admission observations

ProductionFrame.admission_diagnostics retains the native unbound/wrong-view
rejection counts and anticipated-binding histogram per frame. It is optional:
None means this information was not retained in an older export, rather than
asserting zero. A decoded frame always carries Some, including measured zero.
The rejection reason remains attached to its terminal header independently.

MovementStateStream.admission_diagnostics retains only nonzero frame reports,
in scan order, with source packet and native packet ordinal. Some(empty) means
frame scanning was entered without counted admissions; None means unavailable
in an older export or setup returned before frame scanning. The Film constructor
retains the stream through its existing movement_states field. Native movement
scanning uses a private observer and does not return these counters; retaining
these measured frame observations in Rust adds introspection without claiming
that they are directly recorded gameplay actions.

The native production-admission fixture now captures actual Observation counters
in 16 cases: 6 unbound, 2 wrong-view and 2 anticipated bindings. Original fields
are unchanged. A source-level test replays the native unbound case in two packets
and verifies reports at ordinals 1/3 and timestamps 123/456, complete Film
construction and JSON roundtrip. Missing-field and setup-failure checks preserve
unavailable status; existing native source cases cover measured-empty scans.

## Kill-calibration position observer context

KillCalibration.position_observer_context now retains the native returned
profile's observer dequantization values: float32 minimum/maximum and delta
quantum bits, world-object region, axis widths and region-index width. It does
not install an observer or change bit consumption. The native range is the
reference profile range, not the played map's world bounds. Older JSON without
this field retains None rather than being labeled as observed context.

The actual native context was added to 216 source/corruption cases and both
captured kill-walk calibration fixtures. All previous fixture fields are
unchanged. The source test compares every context field and checks JSON
roundtrip/missing-field behavior. The complete kill-result comparator now also
checks the retained range/quantum against native ProfilCalibre in addition to
its prior explicit-field checks. The captured rerun passed on Bandit (122 kills)
and Oddball (247 kills), 185.99s; see PORT_STATUS.md.
Remaining implicit frame policies and the traversal descriptor's Region still
need an integrated contract audit before claiming full profile parity.

## Kill-calibration fixed reader policy

KillCalibration.reader_policy retains Traversal.Region and the native chain
inference, strict-generation, per-view-table and view-class flags. These are
fixed metadata of the returned v41 kill profile, not new mutable parser options.
Older exports retain None. The legacy-export regression checks both missing
policy metadata and missing position-observer context explicitly.

The complete kill-result comparator reconstructs every native ProfilCalibre
field from retained Rust values and checks whole-object equality. It normalizes
only float32 JSON representation and null/empty override maps. Both captured
kill-walk fixtures include policies read from the actual native profile; all
previous fixture fields remain unchanged. Validation is tracked in PORT_STATUS.md.
This does not establish arbitrary reader mutation/restoration API parity.

## Candidate position packet ordinals

BipedPositionCandidate.packet_index now retains the native per-chunk ordinal
among all packet types. Both candidate-producing scans populate it before
position filtering, so rejected candidates retain source attribution too.
The loaded SourceQuantizedPosition path already retained this field. The
candidate stream is retained as Film.biped_positions; old serialized candidates
and synthetic caller-created observations without framing retain None.

The 256-case native record-mask fixture now includes all 4084 accepted native
position records and their packet ordinals, without changing any previous data.
The candidate scan checks these against its retained source, slot and quanta.
A preceding keyframe regression exercises both candidate-producing entry points:
ordinals shift by one while records remain unchanged; byte offsets cannot be
mistaken for packet indices. Stream serialization and absent-field compatibility
are checked. Existing differences in source-adapter admission remain separate.

## Highlight layout metadata

Film.native_highlights.profile retains the native header-derived major/read
distinction, layout name and gamertag offset. The Film constructor reads the
actual decompressed bootstrap header. Explicit-version scans leave profile
unavailable rather than claiming their configuration was read from source.
Old portable streams without the optional profile deserialize as unavailable.
The pipeline regression covers actual v41 header provenance, explicit-version
absence, portable roundtrip and old-export absence. This does not establish
whole-source fidelity or extend supported major versions.

## Biped scanner intermediate attempts

The existing Film biped channels, inventory, charge, unit-equipment, held-weapon
and ability-state streams retain component_attempts automatically through their
existing typed fields and portable serialization. The trace includes all attempted
components, not only successful published targets or rejected reads. Source ranges,
raw component fields/references, dispatch and bounds status, and callback
diagnostics remain attached to each attempt. Ordered source-tagged attempts
are independently checked against the pinned native context fixtures.

These are overlapping scanner traces, not a newly canonical record hierarchy.
Old exports default missing traces to empty, which is not proof that the film
contained no intermediate reads. Existing rejected_components is retained as a
compatibility subset. Native position-anchor selection and each scanner's stop
boundary still limit which records these traces cover. The deferred architecture
and whole-source fidelity contract remain separate.

## Equipment-object component attempts

Film.equipment_state.component_attempts now retains successful and failed
ti=37 component reads with packet source, native packet ordinal, candidate record
start, slot/generation, component index, status, bounds and decoded fields/callbacks.
Both current constructors retain the complete stream, including setup diagnostics.
Accepted samples and counters keep the native completion gate. The 512-case native
scan fixture emits 5339 equipment callbacks versus 4878 populated fields in
accepted samples. Samples collapse repeated field writes and omit failed walks;
the trace keeps every callback, including six from directly rejected component
reads. Portable stream roundtrip and old-export absence are
checked. Empty legacy traces do not prove that no attempts occurred.

## Managed-objective attempt and Film retention

ObjectiveScan.attempts preserves ordered delta-component and full-keyframe
attempts, including data suppressed by native publication gates. Delta attempts
retain dispatch status, separate source bounds and DecodedComponent; keyframe
attempts retain the entire optional KeyframeRecord with raw fields, component
ranges, stop reason and callback diagnostics. Each has FilmPacket source, native
packet ordinal, candidate record start and slot. These remain speculative scan
evidence rather than validated objective facts or a canonical record hierarchy.

The 256-case native scan oracle now repeats the actual objectiveWalk with its
ObjectiveHook wrapped to record emissions. The wrapped and public scan outputs
are checked identical in Go. All prior fixture data was preserved exactly. Rust
compares the complete ordered 43302 callbacks (27750 delta, 15552 keyframe),
including callbacks removed from accepted outputs, and all existing accepted
readings/counters. Source packets/ordinals, bounds and complete JSON roundtrip
are checked. This is not independent verification of every retained raw field.

Both profile-aware Film constructors now retain this previously standalone scan
as Film.objective_scan and retain setup errors as objective_scan_error. Failures
keep the partial scan and do not stop other channels. Old Film exports deserialize
without either field. The explicit-encoding pipeline regression verifies a missing
slot-band result and export compatibility; positive standalone output is covered
by the native oracle. The positive constructor regression now runs both APIs
against source-valid native cases 1, 2 and 4, including broken keyframes, and
case 19 for a missing archetype after slot discovery. It compares every accepted
reading/counter and every callback in Film, checks source packet/ordinal links,
and roundtrips the complete Film. Captured-film trace integration remains separate.
The source bootstrap cannot represent the hole in synthetic case 7's injected
registry; that case remains covered by standalone context tests, not by an
invalid claim that the bootstrap encodes arbitrary in-memory registries.

## Managed-property attempt provenance

Film.managed_properties now includes the standalone scanner's complete attempts
trace through existing construction and serialization. Every component attempt
retains fields, references, callbacks, slot/record start and separate dispatch
status/bounds. Source scans attach actual FilmPacket and per-chunk packet ordinal
before packet-type filtering. The public payload-only API retains None provenance
rather than inventing a chunk or packet index from its timestamp.

The 1024-case native context fixture compares all 54483 property callbacks and
all pre-existing accepted readings/counters. The captured six-film comparison
checks source packet lookup, timestamp/bounds and complete result serialization,
alongside 16569 expected accepted readings. Result status is in PORT_STATUS.md.
This is an overlapping scanner trace; it does not close canonical hierarchy or
whole-source losslessness requirements.

## Navpoint attempts and explicit encoding

Film.navpoint_radial now retains complete delta/keyframe scan attempts with
callback diagnostics and source references, alongside its existing accepted
readings/counters. Keyframe rollback does not erase the trace. The explicit-
encoding Film constructor now runs the same scanner and preserves its nonfatal
setup error, matching the existing map constructor's retention behavior.

The pinned 1024-case oracle now compares all 23127 observer emissions from the
actual native walk, while retaining every previous expected reading and counter.
The trace distinguishes source wire timestamps from projected match time; raw
payload invocations do not invent packet provenance. Six captured-film checks
compare accepted data, verify retained packet/ordinal references and roundtrip
the complete scanner output. Final outcomes are in PORT_STATUS.md. This does not
prove complete source coverage of clockless packets, which the native scan skips,
or byte-for-byte re-encoding.

## Captured navpoint Film integration regression

The six-film player-assembly regression now checks navpoint retention before
its player-output early return. Where the pinned native corpus supplies navpoint
expectations, both the map constructor and explicit-encoding constructor must
match all accepted readings and counters. The two positive gameplay films have
13805 expected readings; empty controls retain their native empty results.

Both complete Films are serialized and reloaded. Their entire navpoint traces
must match each other, and every attempt must resolve to its actual source packet
and per-chunk ordinal. Native output equality remains separate from Rust-to-Rust
trace consistency: the native fixtures do not independently annotate every raw
field in these captured traces. The 1024-case native observer oracle remains
independent callback evidence. Test outcomes are recorded in PORT_STATUS.md.

Captured navpoint constructor validation completed: local_film_player_assembly
passed all six films in 2850.04s. Both constructors' accepted data and complete
traces, native source packet references and Film roundtrips passed alongside the
existing native assembly checks. This closes that integration gate; it does not
turn Rust-to-Rust trace agreement into independent native field annotation.

## Weapon-damage loaded source provenance

WeaponDamageRead now retains optional source FilmPacket and packet_index from
loaded scans. The ordinal is counted before filtering packet types. Source packet
ranges locate the complete payload; end_bit and padding_bits remain the existing
read endpoint and synthetic-tail provenance. Payload-only decode_weapon_damage
leaves both source fields None, as do older exports lacking the fields.
FilmWeaponHits.damages and Film.weapon_hits carry the enriched read automatically.
No raw damage values, attribution base, hit pairing or distance policy changed.

The native explicit-range regression still compares its original 36 cases and
four positive damage results. For each loaded read it also looks up the actual
packet by ordinal, re-decodes the referenced payload, compares every read field
and roundtrips the enriched JSON. Four focused weapon-hit tests passed (2.70s).
Six-film captured scan comparison passed in 50.87s: all 105 positive damage
reads pass source packet/ordinal lookup, referenced-payload re-decode, full field
comparison and enriched JSON roundtrip. Removing both new fields reconstructs
legacy reads with unknown source and every other field preserved. Four empty
controls still match native results. Shot/damage results and attribution bases
are unchanged. This does not establish whole-source losslessness.

## Loaded weapon-shot provenance

WeaponShotRead retains source packet, all-packet ordinal and the native WeaponShot.
scan_weapon_shot_reads and scan_weapon_shot_reads_through expose this data;
existing scan_weapon_shots APIs project the same reads into their original return
shape. FilmWeaponHits.shot_reads retains the complete trace, with None for older
exports and Some(empty) for a successful empty scan. Film.weapon_hits carries it
through portable export alongside the compatibility shots vector. No second scan
is required to populate both forms. Payload-only decode_weapon_shot is unchanged.

The explicit-range oracle includes ten emitted shots without readable identity
pairs, so source retention cannot silently filter the total-shot denominator.
Source regression checks actual packet ordinal lookup, re-decodes referenced
bytes and compares serialized reads. Captured coverage contains 4395 native shots
with identity pairs across six films; this is separate from unpaired coverage.

The six-film shot/damage source regression passed (52.95s). A positive
Film::retain_weapon_hits test also passes (0.37s): full Film JSON preserves the
new shot trace, and old JSON with shot_reads removed leaves None while retaining
all other weapon-hit fields. The fixture uses nonzero packet timestamps, as
required by Film construction. This does not claim complete raw-source archival.

## March scanner policy retention

FilmMarchFacts.walk_policy preserves the actual eight-view limit and the
explicit generation/simulation flags alongside calibration.encoding. Both flags
previously affected the scanner but were absent from its result. Film retains
the result through native_march_facts. None distinguishes old exports and
no-delta scans from known false settings. The native loaded-march fixture checks
both flags against Config.Profil.Grammaire in 201 nonempty cases, with 55
no-delta controls, and checks whole-result serialization and legacy omission.
All existing deaths, occupancy and counters continue to match. This closes two
specific lost profile values, not arbitrary native profile/context equivalence.

## Captured march-policy validation result

The retained march policy and full march-result JSON roundtrip now pass the
four-film native corpus test (1534.02s, session 64835 terminal success). The test
compares complete deaths and occupancy readings, all existing counters and
calibration assertions alongside the new policy fields. Final Oddball results
are 182 deaths and 144 occupancy readings; see CURRENT_VALIDATION_GAPS.md for
the other films and binary scope. This closes the captured verification for the
specific retained policy addition, not arbitrary native profile/context parity.

## Complete native profiles through Film march scanning

KillCalibration.native_profile now retains the complete profile produced by native
starting-profile composition and handle calibration. The legacy projection and
policy/capture metadata remain available. Older exports retain None. New profiles
preserve native nil-versus-allocated width maps rather than normalizing both to an
empty override map in the exported representation.

The map-aware Film constructor carries that profile through native profile-first,
map-second composition into march scanning. FilmMarchFacts.native_config retains
all native FrameConfig scalars and the full profile after calibration/harvest,
detaching mutable width maps. No-delta scans retain absent configuration, matching
the native early return. This is parser configuration, not a recorded wire event.

The six-film native oracle now emits actual ScanMarchFacts plus full kill results.
Every previously retained oracle field is unchanged. All six integrated cases pass
(88.89s), including full march configuration/facts, raw kill results/calibrated
profiles and complete documents. Four no-inheritance Film constructor cases also
pass (75.51s). Full suite: 549 passed, 46 ignored; Clippy and WASM pass. This closes
configuration retention on these paths, not the remaining callback/lifecycle and
whole-source acceptance gates in CURRENT_VALIDATION_GAPS.md.

## Captured pre-publication scan inputs

Twenty complete native FilmInputs fields now have an integrated Film comparison
across six captured films, before replay publication can filter their data.
All 44 native public fields are retained in separate per-film oracle files; the
remaining 24 are explicitly unverified by this new comparison. This supplements
the standalone source fixtures and complete replay-document equality.
See ../fixtures/full-document-inputs-v41.md and film-input-retention-coverage.json
for the field list, native map projection, selected-field streaming and hashes.
All six input/document comparisons passed in 82.34s with no production changes.

## Native clock-origin source result

Film.native_clock_origin now retains the actual first native packet of chunk 1,
including timestamp, header fields and source range. It does not reuse the legacy
minimum-nonzero origin across packets. Read(0), MissingChunk, MissingPacket and
unavailable in an older export remain distinct. The replay-player builder uses
this retained result; only older exports fall back to reading the supplied source.

The compact-export regression covers all four outcomes, a native origin later
than the legacy minimum, complete Film JSON roundtrip and old-field absence.
The existing 1,024 native identity-evidence cases also compare the retained-clock
path, including 174 native failures. Full Theater suite: 561 passed, 46 ignored;
Clippy and WASM pass. See ../fixtures/film-clock-origin-retention-v41.md.

### Captured position input checkpoint

The integrated native-input comparison now covers 30/44 fields, including all
ordered accepted Positions and every exported companion/vitality/mask field.
The six captured films and complete replay documents pass (97.78s); Clippy,
formatting, diff checks, and 485 pinned reference hashes pass. No oracle changed.
See `film-input-retention-coverage.json` for the fourteen remaining inputs and
`../fixtures/full-document-inputs-v41.md` for the exact projection contract.
PlayerIndices still requires separate retention work: replay identity evidence
currently rebuilds its scan from source. All other previously documented parity
gates remain open; the next-phase architecture remains deferred.

### Native identity input retention checkpoint

Film now retains the death feed, death roster, raw replication player-index table,
and named read failures. Collision rejection applies to a copy, preserving raw
conflicts. Replay reuses matching retained inputs, rescans for changed external
rosters, and preserves the old-export source fallback. This supersedes the prior
note that player-index evidence is always rebuilt during replay assembly.

Integrated comparison now covers 32/44 native inputs, adding Deaths and
PlayerIndices. All six input/document comparisons passed (95.25s); 561 Theater
tests passed, 46 ignored (95.27s). The 1,024-case native evidence test exercises
retained inputs without source, serialization, and changed-roster fallback,
including 92 collision cases, 282 death-read failures and 26 index-read failures.
Clippy and all-features WASM pass. See film-input-retention-coverage.json and
../fixtures/film-identity-retention-v41.md. Twelve integrated inputs and the
previously listed broader parity gates remain; architecture changes stay deferred.

Next audit: Translocations. Replay currently consumes Film.translocations while
Film.native_translocations retains the native padded scan. Check that distinction
against the native input and publication contracts before claiming parity there.

### Native translocation publication checkpoint

Resolved the previous adapter audit: replay now publishes retained native
translocations, including native padded-header admission and native source/order
rules, using a shared borrowed publication loop. Film retains padding provenance;
this compatibility output is not proof that padded actor bits were recorded.
An explicit native scan marker distinguishes empty scans from legacy-only exports.

All six complete-document checks pass (98.24s), bringing integrated field checks
to 33/44. **Translocations are empty in all six captured films.** Positive native
synthetic decode/source/publication tests pass, but captured positive validation
is still missing. This limitation is explicit in the coverage manifest; it is not
a declaration of complete parity. Full Theater: 561 passed, 46 ignored (90.07s).
Clippy, all-features WASM, formatting, diff checks and 485 reference hashes pass.
See ../fixtures/film-translocation-publication-v41.md. Eleven integrated inputs,
all broader parity gates, and the deferred architecture phase remain outstanding.

### Captured world-input comparison checkpoint

Integrated comparison now covers 37/44 inputs, adding Projectiles, Placements,
PlacementStats, and Pads. Positive captured coverage includes 509 projectile
tracks, 548 placements, 2,580 pad creations, and 2,794 pad motion tracks. All
ordered fields, points, scan counters, calibration, and census entries match.
All six complete input/document comparisons passed (100.26s); Clippy, formatting,
diff checks, and 485 pinned source hashes passed. This checkpoint changes test
projections and documentation only, with no production or native oracle changes.

Seven remaining integrated inputs: Vehicles, FlagMarks, FlagGauge,
FlagGaugeScanned, ZoneReads, ZoneScanned, and BombReads. The manifest records the
exact scope. Complete parser parity is still unproven; all previously documented
broader gates and the deferred architectural phase remain outstanding.

### Vehicle input comparison checkpoint

The six-film input/document comparison now covers 38/44 inputs, adding the full
Vehicles structure (101.37s). Positive appearance controls contribute 12 vehicle
creations, 716 positions and two occupant-aim reads. All exported vehicle fields,
including complete death-scan settings/counters, are projected. Clippy passed.

A separate pinned-native raid capture passed (274.512s), retaining 150 creations,
89,945 positions, 71,488 aims, 135 events, 12 deaths and 291 occupancy reads. The
new raid-vehicle-inputs-v41 fixture is linked from the existing full-raid document;
all previous document expectations remain unchanged. The Rust raid input and full
document comparison is currently running, not yet verified. Its log is
/private/tmp/halo-raid-vehicle-inputs-test.log. See the fixture provenance document
and film-input-retention-coverage.json for precise status and hashes.

Remaining integrated inputs: FlagMarks, FlagGauge, FlagGaugeScanned, ZoneReads,
ZoneScanned, BombReads. Broader parity gates and the architecture deferral remain.

### Raid vehicle validation completed

The previously running full-raid vehicle-input and complete-document comparison
passed in 278.98s. This verifies all captured vehicle input fields, including the
12 deaths and 291 occupancy reads, against the pinned native fixture. The prior
pending status is superseded. Broader parity gates remain open.

### All 44 FilmInputs fields compared; full parity remains incomplete

All six captured input/document comparisons passed with all 44 exported fields
(101.76s). Objective fields under default caller options are empty/disabled.
Independent native controls enable the resolved consumers on those same bytes,
checking 16,569 managed-property reads per consumer and 13,805 radial reads, plus
disabled outputs. This does not assign CTF/Assault modes or semantic actions to
those recordings. Captured positive carrier marks and translocations remain gaps.
The positive full-raid vehicle-input/document comparison passed (278.98s).

Fixed a consumer failure-contract omission: zone and flag-gauge replay now reject
reads from a managed scan marked failed while Film retains its partial data/error.
Full Theater passed 561 tests, 46 ignored (107.28s); Clippy, all-features WASM,
formatting, diff checks and 485 source hashes pass. See the machine-readable
coverage manifest and ../fixtures/objective-inputs-v41.md for exact limits.

The integrated field checklist is closed, not the parser objective. Remaining
work includes signed/overflowed cursor continuation, lazy width conversion,
callback mutation/lifecycle and source-failure audits, broader whole-Film
acceptance, and missing controlled semantic evidence. Architecture remains deferred.

### Direct-reader lazy width conversion verified on wasm32

NativeFilmReader::read_component no longer rejects unused oversized profile
widths up front. Raw widths are converted at their actual field reads; gates and
unrelated fields can proceed, while the profile preserves its original values.
A reached unrepresentable width reports an error at the reached cursor position.

648 independently generated native cases pass in host/simulated 32-bit tests and
in an actual wasm32 Node runtime. The runtime also confirms that the old eager
adapter rejects the same profiles. Six focused tests pass; full Theater 563
passed, 46 ignored (100.72s); Clippy and the six complete captured input/document
checks pass (98.69s). Formatting, diff checks and 485 source hashes pass. See
../fixtures/lazy-widths-v41.md and verify_lazy_widths_wasm.py for reproduction.

This is the direct component path only. NativeScanProfile::component_encoding
and NativeFrameConfig::frame_encoding still validate eagerly. Frame-level lazy
conversion and signed/overflowed cursor continuation remain open, alongside the
previous callback/source/whole-Film gates. Architecture remains deferred.

### Live frame position widths and missed world-object read corrected

Live NativeFilmReader frame records now check raw position dimensions when their
fields are reached. The audit also found and fixed object-position-component
still consuming policy placeholders in direct reads. The original 128 context
expectations are unchanged; 128 new unused-width frame cases and 144 world-object
direct/frame cases compare against native output, including axis values and
field boundaries. Both 512-case accumulator fixtures continue to match.

Actual wasm32 execution passes all 1,424 native cases plus a reached-width error
control at bit 35. Full Theater: **565 passed, zero failed, 46 ignored** (82.91s).
Clippy, formatting, diff checks, Python syntax and all 485 pinned source hashes
pass. See ../fixtures/frame-lazy-widths-v41.md and frame-lazy-width-validation.json.
Logs: /private/tmp/halo-frame-lazy-{tests,wasm,suite,clippy}.log.

Other frame adapters (resync, inference, repair and production views) and frame
scalar settings still validate eagerly. Signed/overflowed cursor continuation,
callback/source/whole-Film audits and missing positive captured/independent
semantic evidence remain open. The six captured complete-document checks from
the previous checkpoint were not rerun for this live-reader change. Parity is
not complete; the architecture remains deferred.

### Contextual frame position widths and explicit refusals

All NativeFrameConfig contextual entry points now use raw movement widths at
field reads: bound records, chain trials, repair, inference, validated/raw resync
and production views. NativeWidthRefusal diagnostics retain field, bit position,
raw width and address-domain maximum through failed records/speculative results,
merge and serialization. This makes decoder limitations distinguishable from
missing source data. Signed scalar settings and cursor overflow remain open.

The five expanded native context fixtures contain 576 new unused-width cases.
Every original native expectation is unchanged, and all new cases produce the
same native records/callbacks/world state as their originals. Focused host checks:
17 passed, one corpus test ignored. Actual wasm32: 2,832 native cases passed plus
reached-width/refusal-retention controls. Full Theater: **570 passed, zero failed,
46 ignored** (91.20s). Clippy, fmt/diff, Python syntax and 485 source hashes pass.
The captured six-film comparison passed (103.08s): all 44 native inputs,
objective-consumer controls and complete replay documents match.

See ../fixtures/context-lazy-widths-v41.md, context-lazy-width-validation.json
and verify_frame_lazy_widths_wasm.py. Logs are
/private/tmp/halo-context-lazy-{native,tests,wasm,suite,clippy,captured}.log.
These checks do not close the remaining whole-Film/source/cursor gates or begin
the deferred architecture.

### Native New-record nonpositive tail policy

Native TraverseEntity skips its terminal tail only after successful components
and only when BitsDeQueueRecordNew > 0. NativeFrameConfig now applies that rule
instead of rejecting negative settings during unsigned conversion; the original
signed value remains in the native profile. A 384-case native oracle covers
negative extremes, zero, positive tails, clean/failed New, Delta/Delete/End, and
all starting bit offsets. Endpoints, fields, callbacks and world occupancy match.

All four focused New-record tests pass (0.39s), including existing generic,
inference and recovery fixtures. The expanded actual wasm32 runner passes 3,216
native cases plus prior reached-width controls. Clippy, fmt/diff, Python syntax
and all 485 reference hashes pass. The full 570-test and six-film checks in the
previous checkpoint predate this small policy correction; they were not rerun.

See ../fixtures/new-tail-policy-v41.md and new-tail-policy-validation.json.
Logs: /private/tmp/halo-new-tail-{tests,wasm,clippy}.log. Unused oversized positive
tails, other scalar settings and signed/overflowed continuation remain open.
This is parser parity work; the deferred architecture has not begun.

### Large and unused New-record tails

Contextual New-record tails now convert at successful New completion, using the
raw grammar setting. Unused positive widths no longer reject other records.
Consumed large tails retain every actual source bit and describe their synthetic
padding once, tagged NativeWidthPurpose::NewRecordTail. Address and signed-target
limits retain explicit diagnostics instead of expanding huge padded ranges.

224 new native cases pass on the host; all 384 earlier tail expectations are
unchanged. Five focused New-record tests pass. Actual wasm32 passes 3,392 native
comparisons and 48 explicit address-domain refusals, plus a resulting-cursor
control that preserves native target 2^32 at bit 21. Full Theater: **572 passed,
zero failed, 46 ignored** (92.41s). Clippy, fmt/diff, Python syntax and all 485
reference hashes pass. The previous captured six-film input/document comparison
predates these terminal-tail changes and was not rerun.

See ../fixtures/new-tail-large-v41.md and new-tail-large-validation.json.
Logs: /private/tmp/halo-new-tail-large-{tests,wasm,suite,clippy}.log.
Default-state/MPP/ID scalar handling, negative/wrapped cursor continuation and
broader whole-Film gates remain open. Architecture remains deferred.

### Public facade inventory and measured equipment fallback counter

The decoder facade was inspected with Go ASTs: 166 exports, including 71 single
returned calls forwarding arguments unchanged and in order. Fifty types and all
45 constant/variable exports are qualified aliases. The historical 163-symbol
comment is stale. This establishes delegation, not Rust parity of the targets;
the complete inventory and remaining audit boundary are in PUBLIC_DECODER_AUDIT.md.

The equipment-origin oracle now records the actual native fallback.Compteur
instead of using coverage.ByCause as its only counter proxy. All 1,024 original
case fields remain unchanged. Rust matches 739 positive cases, 285 zero cases
and 2,581 named fallback hits. Focused test and Clippy pass; fmt/diff and all 485
reference hashes pass. The six captured document fixtures have no positive hits
for this particular fallback, so positive complete-document forwarding remains
explicitly open. No production code changed in this checkpoint.

See fallback-callsite-audit.json and logs
/private/tmp/halo-equipment-fallback-{native,tests,clippy}.log.
The full parity goal and deferred architecture boundary remain unchanged.

### Automatic weapon-hit retention verified

The audit found that every Film constructor initialized weapon_hits to None,
although the native shot/damage/aggregate parsers were implemented and compared.
Callers had to invoke retain_weapon_hits separately. The base constructor now
retains direct scans and an independent weapon_hits_error; map-aware construction
refreshes the scan with resolved sampling precision, retaining tracks and distance
buckets as well. A failed refresh preserves previous direct results and records
its error. Successful refresh clears that error. This corrects current Film
propagation without starting the deferred native/resolved/playback refactor.

Four focused tests pass (2.66s), including a native chunk-one prefix failure in
an otherwise indexed recording, failed refresh/retry, missing distances and
old-export/JSON checks. The map-constructor test now checks automatic retention
without calling the refresh method. A new six-film constructor comparison uses
the existing pinned native fixture for 4,395 shots and 105 damage observations,
including no-distance aggregates and empty-film controls. All six films passed
(20.41s, after a 3m29s release build). Source references and whole-Film JSON
roundtrips also passed. Full Theater: 574 passed, zero failed, 47 ignored (95.08s);
Clippy passes (12.24s), all-features WASM compilation passes (7.66s), and formatting,
diff and all 485 pinned hashes pass. The new ignored corpus test was run explicitly;
the other captured suites were not rerun. See
../fixtures/weapon-hit-film-retention-v41.md and logs
/private/tmp/halo-hit-retention-{tests,suite,captured,clippy,wasm}.log.

This adds work and export size to constructors. Map-aware construction currently
rescans the direct inputs when it adds distance evidence. Computed pairing and
distances remain distinct from the retained native read fields. Whole-Film
losslessness and broader parity are not implied by this propagation change.

### Captured map-constructor weapon-hit retention verified

The new six-film map-constructor comparison passes against the pinned native
weapon-hit oracle (83.41s; release build 2m46s). It first confirms identical
sampling bounds, then compares all 4,395 shots, 105 damage observations, landing
bases, distance bases, distance errors, aggregate distance histograms and 590,758
track samples with timestamps/positions. Packet provenance and the retained
FilmWeaponHits JSON roundtrip also pass. The base-constructor comparison was
rerun after sharing test code and passes all six cases (21.07s).

These tests use the native scanner's bootstrap/gameplay prefix; they are not a
new complete replay-document comparison. The map test roundtrips FilmWeaponHits;
whole-Film JSON remains covered by the base corpus and synthetic map regression.
Clippy passes (9.34s), formatting/diff and all 485 reference hashes pass. No
production code changed; the full suite and WASM check from the preceding
production checkpoint were not repeated. One additional ignored test was added.

PUBLIC_DECODER_AUDIT.md now maps the weapon-hit and keyframe-position exports to
Rust implementations and evidence, including explicit chunk-range semantics and
the native position probe's heuristic nature. The phase-boundary note near the
top of CURRENT_VALIDATION_GAPS.md distinguishes all native output parity from the
separately deferred source-hierarchy, resolved-event and playback architecture.
This clarification waives no native output or pending parity gate.

See ../fixtures/weapon-hit-film-retention-v41.md and logs
/private/tmp/halo-hit-map-retention-{captured,base,clippy}.log.

### Lazy New default-state widths and nonnegative rewinds

NativeFrameConfig no longer eagerly converts NewDefaultStateBits when building a
contextual frame encoding. The raw signed option is retained in NewRecordEncoding
and consumed only by the selected New fallback branch. Biped/archetype defaults,
invalid New archetypes and Delta/Delete/End leave it unused. Negative skips with
nonnegative targets now continue at that target and retain a NewRecordDefault
adjustment; large padded skips keep every source bit and compact only padding.
The explicit legacy adapter remains checked. Terminal New tails retain their
separate strictly-positive policy, covered by the existing regressions.

624 independent native cases pass: 480 unused-width cases, 48 rewinds, 48 large
consumed widths and 48 ordinary skips. Ordered records, masks, cursor endpoints,
complete world state, retained source fields and result JSON are checked. Seven
focused New tests pass (2.27s). The actual wasm32 runner passes 3,984 native
comparisons and 80 explicit address-domain refusals (4,064 cases), plus its prior
standalone controls. The new default fixture contributes 592 native comparisons
and 32 reached-width refusals there. Raw widths remain available after refusal.

Full Theater: **575 passed, zero failed, 48 ignored** (71.85s). Clippy passes
(12.15s) after an equivalent test-only range predicate cleanup. Formatting/diff,
Python syntax and all 485 pinned hashes pass. Captured films were not rerun for
this context-option change; the previous map weapon-hit corpus predates it.
The WASM runner reports its existing harmless unused copied-test warning.

Remaining gaps include MPP/ID scalar admission and negative/overflowed cursor
continuation. This fixture compares the raw option at configuration boundaries
and cursor effects, not the native Trace.DefaultBits field on each returned New
record; per-record trace metadata still needs reconciliation. Broader native
output parity remains incomplete. The architectural phase is still deferred.

See ../fixtures/new-default-policy-v41.md; new-default-policy-validation.json;
logs /private/tmp/halo-new-default-{tests,wasm,suite,clippy}.log.

### Native per-New default trace metadata retained

EntityRecord.default_state_bits now retains native Trace.DefaultBits at New
trace entry, including invalid archetypes and dedicated/biped readers that do
not consume the configured skip. It is explicitly decoder context, not recorded
payload length. Non-New/unreached traces and older exports retain None. The
checked legacy width is represented only when it fits the native signed domain.
This closes the per-record metadata gap identified in the preceding checkpoint.

The existing native fixture supplies 384 returned New records, including 120
negative values, 40 zeros and 24 i64::MAX values. All are compared directly; full
view JSON roundtrips and old-export removal preserve the availability distinction.
No native expectations were regenerated. Seven focused New tests pass (2.18s).
The actual wasm32 runner also passes 3,984 native comparisons plus 80 domain
refusals, including 352 returned New metadata values and the unchanged reached-
width refusal cases. Existing generic/inference/recovery and tail checks pass.

Full Theater: **575 passed, zero failed, 48 ignored** (70.60s). Clippy passes
(26.60s, including build-lock wait); formatting/diff and all 485 reference hashes
pass. The actual WASM build/run completed in its existing runner (12.45s build).
Captured films and separate all-features WASM compilation were not rerun for
this additive record field. MPP/ID admission, negative/overflowed cursor targets
and broader native-output reconciliation remain open; architecture stays deferred.

See ../fixtures/new-default-policy-v41.md and new-default-policy-validation.json.
Logs: /private/tmp/halo-new-default-metadata-{tests,wasm,suite,clippy}.log.

### Lazy native MPP widths verified

Native contextual readers now defer original signed MPP lead/index widths to
the reached read instead of applying the legacy 1..=32 gate at setup. Zero and
wide reads match native numeric truncation and preserve discarded source bits.
Unused invalid settings do not reject frames. Explicit platform/signed-cursor
refusals retain original widths, field locations and preceding observations.
The bounded public default-state helper retains its checked contract.

All 1,632 native cases pass (288 consumed and 1,344 unused widths), comparing
1,152 ordered callbacks, records/masks/endpoints, world state, source fields and
JSON roundtrips. Additional negative consumed-width controls pass without trying
those impractical native reads. Actual wasm32 execution passes 5,616 native
comparisons plus 80 existing address refusals and additional MPP refusal controls.

Full Theater: **576 passed, zero failed, 48 ignored** (70.59s). Clippy passes
(13.91s); formatting/diff, Python syntax and all 485 pinned reference hashes pass.
WASM build took 11.09s; its existing unused copied-test warning is unchanged.
Captured films and separate all-features WASM compilation were not rerun.
ID admission, negative/overflowed cursor continuation and broader native output
reconciliation remain open. NEXT_PHASE.md remains deferred.

See ../fixtures/mpp-width-policy-v41.md and mpp-width-policy-validation.json.
Logs: /private/tmp/halo-mpp-width-{tests,wasm,suite,clippy}.log.

### Native contextual record-ID policy verified

Native contextual setup now carries signed IDLowBits in
FrameEncoding.native_id_low_bits and defers its admission until a non-End
header. Nonpositive widths consume no low field. Positive reads truncate to
uint32, add IDBase with wrapping, then mask to 30 slot bits before combining the
separate generation. Ordered frame/inference/production/resync/chain header
pre-reads share this policy when that raw native setting is present.

8,640 native-generated header/frame cases pass: five prefix forms, eight
alignments, three values, five bases and 16 signed/zero/wide widths, with huge
positive settings consumed only by End (which ignores them). The fixture also
compares complete world slots, returned records/masks/endpoints and full-view
JSON roundtrips. Four additional Rust-domain controls cover ID/generation
refusals with/without the optional prefix. Generic reader cursors and structured
inference/resync/production diagnostics retain the reached failure location.
ProductionFrame.header_diagnostics preserves errors before a complete record;
older exports deserialize with empty diagnostics.

Focused tests: **2 passed** (0.71s). Full Theater: **578 passed, zero failed,
48 ignored** (76.44s). Actual wasm32: **14,256 native comparisons + 80 existing
address refusals**, plus MPP/ID standalone controls (21.95s build). Clippy passes
(20.27s). Formatting/diff, Python syntax and all 485 pinned hashes pass. Captured
films and separate all-features WASM compilation were not rerun.

This closes raw native contextual ID handling for the exercised frame paths;
it does not establish parity for every adapter. Native march still uses the
checked frame adapter for locator setup and calibration export. Native APIs
supplied a legacy FrameEncoding without native_id_low_bits still use its checked
ID policy, including checked base arithmetic. These paths need reconciliation;
do not change the bounded public decode_record_header contract inadvertently.
Signed/overflowed cursor continuation and broader native-output reconciliation
also remain open. NEXT_PHASE.md remains deferred.

See ../fixtures/record-id-policy-v41.md and record-id-policy-validation.json.
Logs: /private/tmp/halo-id-{tests,wasm,suite,clippy}.log.

### Legacy native IDs and march adapter reconciliation

Native generic/inference traversal now applies wrapping ID arithmetic even when
a legacy FrameEncoding has no raw native_id_low_bits field. 1,440 compatible
cases from the unchanged native ID fixture compare generic and inference headers,
frame endpoints and completion. The bounded public header helper still rejects
out-of-range sums, verified by explicit controls.

Native march locator setup now uses the contextual frame reader. A new 90-case
oracle calls native calibration, localization, record traversal and harvest for
End-only payloads with/without event markers. It varies unused ID/base, MPP,
New-default and world-index settings through negative/zero/wide values. All
facts, coverage maps, packet/calibration counters and retained configuration are
checked; these cases must emit no deaths or occupancy. This is unused-setting
coverage, not positive action or captured unusual-width evidence.

A completed native scan no longer fails because calibration's checked convenience
encoding cannot represent its settings. MarchCalibration.encoding is now optional;
encoding_error reports conversion failure, while native_config retains complete
calibrated settings. No substitute settings are exported. Existing encoding-object
JSON remains compatible and ordinary successful output shape is unchanged. Rust
callers must handle the optional field. Encoding-only calibration always supplies
Some of its actual encoding. Projection availability is checked across 30 host
and 26 wasm32 cases, with explicit absence/error in the remainder.

Focused ID tests: 2 passed (0.73s); march test: 90 cases passed (0.12s). Full
Theater: **579 passed, zero failed, 48 ignored** (85.73s). Actual wasm32 passes
14,346 native fixture comparisons plus 80 existing domain refusals; the 1,440
legacy-ID extensions and standalone MPP/ID controls run additionally. WASM build
18.85s; Clippy 17.09s. Formatting/diff, Python syntax and all 485 pinned hashes pass.

The four-film captured native-context march comparison is running in release
mode; do not treat it as passed until its terminal result is recorded. Other
captured suites and separate all-features WASM compilation were not rerun.
Negative/overflowed cursor continuation, broader native-output reconciliation
and remaining corpus acceptance still prevent full-parity completion. The
NEXT_PHASE.md architecture remains deferred.

See ../fixtures/march-lazy-policy-v41.md and march-lazy-policy-validation.json.
Logs: /private/tmp/halo-id-legacy-tests.log and
/private/tmp/halo-march-lazy-{tests,wasm,suite,clippy,captured}.log.

### Captured native march comparison completed

The four captured v41 films (Bazaar idle, Aquarius, Bandit Evo, ranked Oddball)
pass the native-context march comparison after the legacy-ID and march adapter
changes. The test compares complete deaths/occupancy, all four coverage maps,
packet/localization counts, native configuration/profile, calibration counters
and FilmMarchFacts JSON roundtrip against the pinned oracle. Execution: 19.83s;
release build: 2m56s. This supersedes the preceding running status. Other captured
pipelines were not rerun; it is not a complete replay-document rerun.

Current baseline remains 579 passed, zero failed, 48 ignored in the regular
Theater suite, with actual WASM and Clippy passing. No compiler/test processes
remain active at this checkpoint. See /private/tmp/halo-march-lazy-captured.log.

### Kill-source input error precedence corrected

The public-target audit found that full kill-source decoding attempted its feed
before checking source/replication presence. The v41 full decoder now follows
native order: no chunks, then no type-zero replication packets, then no usable
kill feed. DecodeError::KillSource retains NoChunk, NoReplicationPacket and
NoKillFeed with exact native sentinel text. The explicit unsupported-version
boundary remains separate. The initial guard adds a packet-presence scan to
successful calls without copying source bytes.

The actual native exported Decode now supplies decode_error in the existing
128-case loaded-source oracle. All earlier fields were verified unchanged.
10 no-chunk, 24 no-replication and 94 no-feed cases pass through the Rust full
entry point; both typed categories and text are checked. This verifies failure
precedence, not cancellation/logging or every successful decoding path.

Focused comparison passes (0.20s); full Theater: **580 passed, zero failed,
48 ignored** (98.82s). Clippy passes (50.08s). Actual WASM passes 14,474 native
fixture comparisons plus 80 existing domain refusals and prior supplementary
controls (47.27s build). Formatting/diff, Python syntax and all 485 source hashes
pass. The two-film complete kill-source result comparison is running in release
mode; its result must be recorded before treating it as passed. Other captured
pipelines were not rerun.

PUBLIC_DECODER_AUDIT.md also maps all seven weaponscan/weaponv3 function exports,
distinguishes marker-based and packet-based clocks, and identifies the kill
catalog/profile/default-option targets. The remaining public target/type and
native-contract audit is still open; NEXT_PHASE.md remains deferred.

See ../fixtures/kill-decode-input-errors-v41.md and
kill-decode-input-validation.json. Logs: /private/tmp/halo-kill-input-
{native,tests,wasm,suite,clippy,captured}.log.

### Captured full kill-source results verified

The two-film full-result comparison passes after input-error precedence was
restored: Bandit Evo (122 kills) and ranked Oddball (247 kills), including the
existing complete result/configuration and publication-gate assertions.
Execution: 10.71s; release build: 3m 06s. This supersedes the preceding running
status. Other captured pipelines were not rerun. The regular suite remains
580 passed, zero failed, 48 ignored; actual WASM and Clippy pass. No build/test
process remains active at this checkpoint. Broader parity remains incomplete.

Log: /private/tmp/halo-kill-input-captured.log.

## Kill timeline registry admission

Native `killsource.newTimeline` accepts successful truncated registry reads;
Rust previously rejected every `FilmRegistry.truncated` result and all short
headers. `KillTimeline::from_chunks` now uses `parse_registry_chunk`, preserves
ordered partial archetypes and the native trailing-byte count, and maps native
compressed-input failure to `KillSourceFilmError::Registry` with its typed cause
and native wrapper text. An empty chunk list reports `NoChunk`.

The 20-case actual-native constructor fixture covers short/header-only inputs,
partial and exact block boundaries, named precision levels, structural endings,
and two compressed-header refusals. It is registered in `generate_oracles.py`
and the public WASM harness. Existing timeline comparisons also pass. See
`../fixtures/kill-registry-v41.md`. This does not close option-domain,
cancellation/warning, complete public-target, or remaining corpus gates.

Registry checkpoint validation: 581 Theater tests passed, zero failed, 48 ignored
(75.47s); focused timeline tests 2 passed (0.21s); Clippy passed (10.43s).
Actual WASM passed the existing 14,474 native comparisons and 80 address-domain
refusals, plus the new 20 registry cases and existing supplementary controls
(build 13.14s). The captured full kill decoder matched all 369 kills across
Bandit Evo and ranked Oddball, including full existing output/configuration and
publication assertions (debug 175.38s). Formatting, diff, Python syntax and all
485 pinned hashes passed. Machine-readable evidence: `kill-registry-validation.json`.
Logs: `/private/tmp/halo-kill-registry-{native,tests,suite,wasm,clippy,captured}.log`.
This checkpoint leaves full parity active; it does not begin NEXT_PHASE.md.

## Signed kill-source options

`KillDecodeOptions` now retains signed i64 multiplicity/restart/view counts.
Native `Options.normalize` replaces all nonpositive counts with defaults; Rust
previously represented only the unsigned subset. The public `normalized` method
matches native while preserving caller-owned raw settings and booleans.
Execution checks conversion to usize and reports typed `DecodeError::KillOption`
with field/value on a narrower target, rather than wrapping or saturating.
See `../fixtures/kill-options-v41.md` for API migration and domain distinctions.

The 512-case 64-bit native oracle compares all option fields and default values,
including negatives, zero, booleans, i64 extremes and 32-bit boundaries. The WASM
harness runs the same public normalization and an actual decoder admission
control for an unrepresentable positive view count. Captured nonpositive-option
results are checked against independently generated native default results.
This closes the previously missing signed normalization domain; it does not
claim to execute enormous loop counts or close source-warning/cancellation and
remaining public-target/corpus gates. NEXT_PHASE.md remains deferred.

Signed-options checkpoint validation: 583 Theater tests passed, zero failed,
49 ignored (72.72s); Clippy passed (26.17s). Actual WASM passed the existing
14,474 native comparisons/80 address-domain refusals plus the new 512 option
normalization cases and actual public execution-domain control (build 26.30s).
With nonpositive counts, full captured decoder results matched all 369 native
default-option kills across Bandit Evo and ranked Oddball, including existing
full result/configuration/publication assertions (debug 175.96s). Formatting,
diff, Python syntax and all 485 pinned hashes passed. The additional ignored
test is the captured nonpositive-options control. No tests remain running.
See kill-options-validation.json and /private/tmp/halo-kill-options-
{native,tests,suite,wasm,clippy,captured}.log. Full parity remains active.
