# Current v41 validation evidence and outstanding gates

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Historical details and superseded checkpoints remain in `PORT_STATUS.md`.
The architecture in `NEXT_PHASE.md` is deferred until current parity is complete.

## First decoded parent selection

The pending parent-selection candidate below is now reproduced and corrected.
Rust previously stopped at the first successful named parent attempt, even if
calibration skipped its value. It now selects the first decoded parent value,
matching native occupancyFromRecord. A recorded free parent still takes
precedence over a later attached parent, and both-skipped remains absent.

All 16 native-backed cases and their JSON reconstruction checks pass (0.01s).
The host suite passes 629 tests, zero failed, 49 ignored (122.29s); Clippy and
WASM execution pass. Four captured march films pass (83.94s), and six complete
documents match (112.53s). Native fixture bytes, 485 source hashes, formatting/diff
and script syntax pass. See parent-result-validation.json for final evidence.
Full parity remains incomplete.

## Last decoded dead-state retention

A live-calibration native oracle reproduced a missing harvested death: a later
zero-width dead-state skip caused Rust to discard an earlier decoded value.
Native retains the last non-nil result. EntityRecord and KeyframeRecord now expose
captured_dead_state using ordered attempt fields, and MarchRecordFacts uses that
projection before checking Mort. A later decoded Mort=false still replaces an
earlier death; a skip does not. Thirty-two native cases cover both record entry
points, positive/negative outcomes and JSON reconstruction. Host suite passes
628 tests, zero failed, 49 ignored (143.65s); Clippy and WASM pass. All 134,657
anchors pass (71.29s), four captured march films pass (85.05s), and six complete
documents match (110.50s). Native bytes, source hashes and formatting/diff checks
pass. Final evidence is in dead-result-validation.json.

Next source-audit candidate: vehicle_occupancy_from_record selects the first
successful parent attempt, whereas native occupancyFromRecord continues until
ParentOf returns a payload. A zero-width calibrated first parent followed by a
decoded parent may expose the analogous first-non-nil selection gap. This is
source evidence, not a validated fix. The 16-case native fixture now records
the expected first-non-nil behavior; Rust reproduction remains pending. See
../fixtures/parent-result-selection-v41.md.

## Component result metadata correction

Generic attempts now retain the native returned variant. Full keyframes retain
variant/ported metadata and an ordered attempt trace including the unsupported
final component; their existing successful-component list remains available.
Zero-width calibrated replacements preserve native variant zero, distinct from
the absent sentinel. Old exports retain unknown new metadata. This corrects an
output-retention gap without changing component bit consumption.

Ten native cases compare generic and full-keyframe entry points, including
ordinary reads, zero-width calibration, unsupported readers, zero-width stubs and
a positive weapon variant distinct from its high word. The host suite passes
627 tests (103.51s), Clippy passes (22.75s), and WASM execution passes after adding
the existing private profile fixture adapter to the standalone harness. The
expanded 134,657-anchor metadata comparison passed (68.91s), including existing
payload/source/JSON checks; all six complete documents matched (104.91s). Native
fixture bytes, 485 source hashes, formatting and diff checks pass. Final results
are in component-result-validation.json. Full parity is not established by this
correction.

## Broad captured acceptance checkpoint

The release run executed 44 asserting ignored corpus tests, excluding
the diagnostic-only frontier report and four corpus tests already passed in
component-mask-wrap-validation.json. Results are tracked individually in
corpus-acceptance-audit.json. Runtime output is
captured in /private/tmp/halo-complete-corpus-acceptance.log. This run does not
establish all-data parity by itself.

The original run finished with 43 passes and one failure in 733.63s. The failure
was a stale kill-walk calibration fixture: Rust's added native_profile field had
no native expectation. The Go generator now publishes the complete profile;
every previous expectation in both captured rows is unchanged. The focused Rust
rerun passed in 5.31s, checking the full profile and calibration JSON roundtrip.
Clippy passed (10.24s), as did formatting/diff checks and all 485 pinned hashes.
No production parser code changed. All 44 selected asserting tests have passing
evidence across these runs; the original broad run is not reported as green.
See KILL_WALK_CALIBRATION_ACCEPTANCE.md for the correction and provenance.

The legacy raid assembly fixture has seven of its twelve conditionally checked
layers. It omits skull_layer, carrier_marks, objective_object_layer, score_layer
and ground_layer, which that test skips. The audit records the exact present and
absent keys. The separate complete raid document test compares its full document
fixture, with native vehicle inputs, and passed in this run; neither test
alone proves every intermediate native output is retained. Missing positive
captured VIP/Assault and complete equipment-recovery evidence remain open.

Previous full host baseline after signed cursor migration: **596 passed, zero
failed, 49 ignored** (108.76s). The formerly failing 96-case signed-continuation
gate now passes without an ignore, and 80 direct-component cursor cases check
native outcomes and host panic recovery. Clippy passes (18.19s). Actual wasm32
execution passes, including 52 nonpanicking direct-component cases and four
public-frame signed-continuation cases. The harness's historical 14,554-case
headline excludes these and other supplemental controls.

Follow-up narrow-target assertions explicitly retain the remaining header
address limitation. Fresh release comparisons pass for all four captured march
films (84.28s) and six complete documents (108.00s). Remaining signed
header/MPP/source-address and whole-frame panic integration gaps are described
in SIGNED_CURSOR_MIGRATION.md. Full v41 parity is not yet established.
Earlier counts below are historical checkpoints. The recovered-binding
regression extension changes an existing test and does not increase its count.

## Latest stateful frame recovery correction

NativeFilmReader frame reads now preserve cursor and capture slot on unwind.
The independent 80-case native oracle passes on the host: 72 panics and eight
successful controls, with callback order, prior deletion, identity retention and
reset/read recovery. It also found and corrected a negative optional-prefix
Skip(32) being treated as a source read. Clippy passes (18.00s), the full suite
passes **597 tests, zero failed, 49 ignored** (88.11s), and actual WASM execution
passes with eight new nonpanicking controls (23.53s build). See
frame-panic-cursor-validation.json. Earlier broad whole-frame-copyback warnings
are narrowed to untested contracts, including the separate NativeFilmBits frame
adapter. Signed/native headers and MPP/raw widths remain open.

## Source-only frame adapter follow-up

The separate NativeFilmBits frame adapter now preserves the reached cursor on
component panic and retains earlier binding changes. All 24 applicable native
oracle cases pass, including 16 panics and eight successful controls. Clippy
passes (15.11s); the latest full suite passes **598 tests, zero failed, 49 ignored**
(86.35s), and actual WASM runtime passes its eight source-adapter controls
(14.70s build). The prior
source-adapter copyback gap is closed for this represented configuration domain;
signed starts and raw-width/header restrictions remain open. See
source-frame-panic-validation.json for exact coverage and final validation.

## Signed header correction validated

RecordHeader and creation-origin coordinates now retain i64. Native header reads
use signed NativeFilmBits, and both generic frame APIs accept signed starts with
native panic/cursor behavior. The 411-case direct oracle and 361-case frame-entry
oracle pass; actual WASM passes 275 direct and 225 frame cases. Earlier generic
header address-refusal statements are superseded. Latest host suite: **600 passed,
zero failed, 49 ignored** (105.04s). Clippy passes (10.09s); captured four-film
march (82.31s) and six-film complete documents (104.44s) pass in release mode.
Higher-level inference/resync/march
source-address projections and MPP/raw component widths remain open. See
signed-header-validation.json for final status and exact evidence boundaries.

## Native MPP width correction validated

MPP lead/index reads now preserve the native u64 read count and signed cursor
behavior. ComponentField.width is u64 on every target. The new oracle passes 272
direct native cases (212 panics) and two complete native 2^32-bit frames with
source-prefix retention and JSON checks. Latest suite: **602 passed, zero failed,
49 ignored** (86.75s). Clippy passes (16.06s), actual WASM passes, and release
comparisons pass for four march films (79.74s) and six complete documents
(101.75s). Earlier MPP address-domain refusals
are superseded; raw position/profile adapters and higher-level recovery source
projections remain open. See mpp-domain-validation.json.

## Phase boundary for interpreting this audit

Current acceptance is complete v41 parity with the pinned reference parser:
all of its decoded outputs, ordering, source references, observable scan outcomes
and relevant diagnostics must be implemented and retained. A whole-film check
must compare the complete relevant reference result, not only selected counts or
fields. Native-exposed opaque data is part of that scope too. Green fixtures
cover only their actual inputs and branches; the remaining parity gaps below
are not waived by this clarification.

The separate canonical Film hierarchy, preservation of all source information
beyond what the reference exposes, ResolvedFilm event semantics, checkpointed
playback, browser integration and independently annotated semantic golden suite
belong to NEXT_PHASE.md. They must be proposed after parity and before that
refactor. Historical mentions below of whole-source losslessness or missing
independent semantic evidence describe limits on those claims; they must not
require implementing the deferred architecture before completing this phase.
Film JSON is still not a lossless byte archive. Byte-for-byte re-encoding has
never been established.

## Current Film profile integration (verified)

The map-aware Film constructor now passes complete native scan settings to march.
KillCalibration.native_profile is produced at initialization and updated by handle
calibration, not reconstructed from the exported legacy encoding. Replay installs
inherited settings before map world-object precision and recorded corruption.
Old calibration exports without native_profile continue to report absence.

The actual-native six-film document oracle now includes march facts and raw kill
results. Native generation passed (96.165s); all previously retained fixture fields
are unchanged. The Rust integrated comparison now checks full march configuration,
all death/occupancy facts and counters, complete kill results and native calibrated
profiles, as well as the original document. Full suite passed (549 tests, 46 ignored,94.32s), Clippy passed (25.71s),
WASM passed (13.84s), and source hashes/fmt/diff passed. The first release compilation failed because the filesystem had only 229 MiB
free. `cargo clean --profile dev -p halo_api` removed regenerable package outputs;
76 GiB is now free. The second release build succeeded. All four no-inherited-profile Film
constructor cases passed (75.51s), including complete native configuration,
facts, coverage, and JSON. All six inherited-profile cases also passed (88.89s), including full native
march configuration/facts, raw kill results/calibrated profiles, and complete
documents. Log: `/private/tmp/halo-film-native-profile-documents-retry.log`. Earlier
captured results below predate this change.

## Fresh-reader inference/resync lifecycle audit

Two additional native-backed tests passed (0.90s): 1,024 inference and 256 raw
resync passes with seeded traversal-world positions, persistent observers/worlds,
and fresh readers. Ordered callbacks, records/endpoints, complete world snapshots
and JSON roundtrips match. Native delivery totals are 1,366 inference positions
and 12 raw-resync positions; baseline positions are not emitted. This verifies
that saved world positions do not implicitly attach an accumulator to these paths.
No production parser change was needed.

Both original extended-fixture tests also passed (0.64s and 0.06s); every old
native expectation was unchanged. Clippy passed (30.07s), format/diff/generator
syntax and all 485 source hashes passed. Logs:
`/private/tmp/halo-native-reader-lifecycle-{oracle,tests,clippy}.log` and
`/private/tmp/halo-reader-lifecycle-original-{inference,resync}.log`.
The last full suite below predates only these test additions; production code is
unchanged. Details: `../fixtures/reader-lifecycle-v41.md`.

## Current verified results

- Expanded recovered-anchor comparison passed all **134,657 anchors / 451
  keyframes / 32 films**, 399.18s: 189,416 typed captured payloads, 43,631
  dead-state values, source/padding checks and complete record JSON roundtrips.
  Log: `/private/tmp/halo-keyframe-anchor-payloads-tests.log`. Native default
  context plus corruption flag; recovered anchors are not proven boundaries.
- Integrated native recovery witness passed (0.68s), retaining bounded truncation
  separately from native padding, crossing diagnostics, shared identical reads
  and old-export absence. The full Theater suite and map-aware retention regression also pass;
  the six-film complete-document comparison passed (1805.71s, debug).
  `/private/tmp/halo-native-retention-documents.log`; the timing wrapper failed
  only its post-test host-statistics query. Release comparison also passed: 75.20s, peak RSS 2,847,883,264 bytes
  (macOS child getrusage). Log: `/private/tmp/halo-native-retention-release-documents.log`.

- Regular Theater suite: **549 passed, zero failed, 46 ignored**, 94.32s.
  Log: `/private/tmp/halo-film-native-profile-suite.log`. Includes the map-aware
  recovered-record retention and complete replication JSON regression.
- Clippy lib/tests with warnings denied, all-features WASM, formatting/diff and
  all 485 pinned source hashes passed after native context march integration.
  Final Clippy: 25.71s; WASM: 13.84s, after Film profile integration.
  Logs: `/private/tmp/halo-film-native-profile-{clippy,wasm}.log`.
- Complete integrated replay documents: **all six native goldens matched**,
  83.51s. Includes Bazaar, empty Aquarius, Bandit, Oddball, Cadet Blue and Cadet
  Brick; decoded kill results and all exported document fields are compared.
  Log: `/private/tmp/halo-release-decoded-documents.log`.
- Player/combat/vehicle assembly: **all six films passed**, 157.70s.
  Log: `/private/tmp/halo-release-player-assembly.log`. Nonempty player tracks:
  Bazaar 1/71 points, Bandit 128/34,163, Oddball 256/65,705, Cadet Blue and Brick
  each 1/8; Aquarius has no timeline. Both document and assembly binaries predate
  static-width compaction; ordinary corpus width behavior is unchanged.
- Direct position accumulation: 45,056 native sequence/lifecycle reads.
  Separate/shared frame accumulation: 1,024 native cases, 5,073 records and
  20,616 callbacks, including complete world snapshots.
- Bomb start-zero fallback: 288 native boundary cases, 24 positive and 264 zero.
  Failed-scan publication gate preserves Film data/errors; its populated
  partial-result/error regression is injected, not a captured false-arming case.
- Large-width cases: live and static overrides use checked cursor arithmetic
  and retain actual source bits with compact padding. Both paths match the
  18-case native oracle, with explicit refusal of negative overflow results.
  Mobility metadata has 128 policy cases plus 208 large-width cases (204
  nonnegative matches and four explicit negative-overflow refusals).

The subsequent `native_live_chain_budget_exhaustion` test also passed (3.32s):
eight native cases at the normal 200,000-trial budget, three BudgetExhausted and
five NoConfirmation. Complete live outcome counters, failed-header retention,
frame diagnostics and JSON were compared. No production code changed in this
addition; the regular suite count above includes this added test.

## Additional completed captured checks

Release-mode tests using the static-width implementation all passed:

- **134,657 native recovered anchors across 451 keyframes / 32 films**, 6.34s:
  `/private/tmp/halo-current-keyframe-corpus.log`. Compares ordered slot,
  generation, archetype and bit offset; this is anchor recovery, not every
  component body in those keyframes.
- **Complete embedded CTF document**, catalog flag geometry and decoded kills,
  4.02s: `/private/tmp/halo-current-ctf-document.log`.
- **Four-film raw inventory/gameplay comparison**, 6.38s:
  `/private/tmp/halo-current-inventory-corpus.log`. Includes 8,636 inventory
  records, 39 charge reads, four impulses, 26 grapple reads, 4,401 camo readings
  and 33 ability ranks, with packet ordinals and scan counters.

The earlier debug process handles disappeared after interruption without terminal
results. Their partial output is superseded by the completed release runs above.
Aquarius exposed a test-harness assumption: the native zero-frame document runs
only the retained-source stage. Expected stages now use the native golden frame
count; complete document field equality remains unchanged.

Additional captured position/equipment validation passed both tests in 29.00s
using the release binary with the same production implementation (only the later
live-budget test was absent). Log:
`/private/tmp/halo-current-position-equipment-corpus.log`. The equipment-state
comparison includes 29,852 samples across manual/automatic configurations. The
position comparison includes quantized/world values, companion fields, masks,
packet ordinals, placements, creation records and equipment-change outputs.
All four native equipment-change fixtures have zero recovered emissions; this
run does not close positive recovery source coverage.

## Latest equipment recovery source change

Recovery now follows native first-match numbered chunk lookup and native packet
prefix/End behavior rather than strict global packet indexing. The 24-case actual
native fixture includes 12 positive recoveries and 12 controls, with reordered,
missing and duplicate chunks, independent chunk-type metadata, malformed tails
and prefixes, and non-recovery packet ordinals. Accepted emissions are also
checked through final equipment-change assembly and JSON retention. All five
focused recovery tests passed (3.01s); clippy and WASM passed. The regular suite
passed (542 tests, 70.86s); affected captured
position/document comparisons also passed (two tests, 72.61s) in
`/private/tmp/halo-recovery-source-captured.log`.

## Latest inventory source change

Keyframe inventory now uses native packet/chunk selection and returns retained
counts/default-cap fallback alongside source errors. Film retains the report and
`keyframe_inventory_error`; replay input excludes errored streams. All three
focused inventory tests passed (1.39s), including the 120-case native source
oracle and a Film/JSON missing-prefix regression. The regular suite passed
(544 tests, 83.59s), as did clippy, WASM, formatting/diff and generator syntax.
Captured position/document checks passed (two tests, 72.31s), including all six
integrated document goldens, in `/private/tmp/halo-inventory-source-captured.log`.
The earlier captured checks above predate this latest change.

## Captured source and scanner retention rerun

Six additional ignored tests passed in 6.10s:
`/private/tmp/halo-current-source-scanners-fixed.log`.

- Source loading: mixed raw/zlib and clear Bazaar chunks retain equal bytes,
  packet ranges and resulting Film JSON. This does not prove re-encoding.
- Loaded statborg: 16,113 native records across all 32 local films.
- Managed properties: 16,569 native readings across six films, with counters,
  source packet ordinals, component boundaries and serialization assertions.
- Radial scanner: 13,805 native readings across the five available fixture rows,
  with counters and source checks. Aquarius has no radial fixture row here.
- Unit equipment: 204 native list emissions across six films, plus the positive
  Cadet Blue Film export and JSON retention.

The initial run passed five tests and exposed a stale equipment-export assertion:
it compared a legacy scan against the constructor's context-aware scan, which
also retains a preceding velocity observation. The test now compares complete
retention against the matching context path, checks both paths' equipment records
against the unchanged native golden and keeps stats and Film JSON assertions.
No production decoder behavior changed. Clippy lib/tests with warnings denied,
formatting/diff checks and all 485 pinned reference hashes passed.

## Duplicate component-source lookup

Six scanner implementations now use the first metadata entry for numbered chunk
bytes: ability channels, charges, ability states, unit equipment, inventory and
held-weapon changes. Packet-ordinal lookup uses the same selection. Previously a
later duplicate could substitute payload bytes after native anchors were found;
the new regression reproduced a Truncated error from an empty later duplicate.

Two native 512-case fixtures compare actual scanner outputs for empty, truncated
and conflicting later duplicates, including missing/reordered requested numbers.
All 1,024 native results equal the original no-duplicate goldens. Rust compares
values, counters and packet ordinals. Positive counts and limitations are in
`../fixtures/duplicate-component-sources-v41.md`; these cached-context tests are
not proof of every full-constructor source policy. Both new tests pass (0.14s).

The regular suite passes (546 tests, 77.30s); clippy, all-features WASM,
formatting/diff, generator syntax and all 485 pinned hashes pass. Captured
position/document rerun passed (two tests, 75.45s), including all six integrated
document goldens, in `/private/tmp/halo-duplicate-sources-captured.log`. Prior
captured runs above predate this source lookup change.

## Follow-up strict lookup audit

The four remaining direct chunk-number maps in `biped_scan.rs`,
`anticipated_bindings.rs`, `translocator.rs` and `event_heads.rs` run behind
`packets::index`, which explicitly rejects duplicate numbers before payload
access. They do not share the reproduced component-scanner substitution bug.
Native anticipated-binding and translocator APIs already have separate source
traversal. This audit does not establish complete constructor framing parity;
it prevents replacing intentional strict APIs indiscriminately.

The existing hour-long raid assembly and complete-document comparisons are
both passed (530.59s) in `/private/tmp/halo-current-raid-parity.log`. Eight existing captured combat/scanner tests passed (62.65s) in
`/private/tmp/halo-current-combat-scanners.log`: movement values and counters on
four films; native march/death/occupancy scans on four films; calibrated kill
record walks on Bandit and Oddball; raw weapon shots/hits/base, tracks, prefix
tracks, aggregates and weapon patterns on six films. This is reference-scanner
agreement, not independently annotated physical-action validation. These runs predate native recovery retention.

## Captured keyframe and objective scanner rerun

Six existing ignored tests passed (6.73s),
`/private/tmp/halo-current-keyframe-scanners.log`:

- Six-film I0 layout results and 346 keyframe position probes.
- 64 native keyframe-queue fixture rows across four films, including seeded
  variants; this is not every component in every corpus keyframe.
- Four-film loadout source comparisons.
- Objective extraction against all 32 native corpus rows; positive VIP/bomb
  recordings are still absent from this corpus.
- Two captured research-directory cases with detected/fallback precision, using
  only chunks numbered through seven as prescribed by the existing fixture.

The object-death configuration retention gap is now closed for the Film constructor
and native context paths. NativeFrameMetadata retains all FrameConfig scalars and
the full native profile after the scan, with detached width maps. The constructor
uses the actual retained KillCalibration.native_profile and native profile-then-map
composition. The 256-case native fixture, four no-inheritance Film cases and six
inherited-profile cases compare every native Config field in addition to facts
and coverage. Old exports/encoding-only callers retain absent metadata. This does
not establish arbitrary observer mutation or unsupported cursor-policy contracts.

## Captured identity, kills and raid assembly

Seven existing ignored identity/kill tests passed (8.50s),
`/private/tmp/halo-current-identity-kills.log`: player tables, 3,667 native
highlights and 1,395 deaths, complete kill-source results and supporting evidence
for Bandit (122 kills) and Oddball (247 kills), and complete Bazaar documents with
missing identity, unknown-format variants and zero-axis fallback. These include
native refusals/fallback behavior; they do not imply every variant is decoded.

The hour-long raid assembly test passed in the ongoing sequential run
`/private/tmp/halo-current-raid-parity.log`: 269 player tracks, 208,865 position
points, 86 vehicle tracks and 149 rides match the native assembly golden.
The complete raid document also matched; both tests finished successfully in
530.59s. This run precedes the packet-preamble metadata addition; it validates
the unchanged decoding/assembly behavior, not that newly retained field.

## March packet-preamble retention

FilmMarchFacts.packet_preamble_bits now retains the actual two-bit setting used
by non-event packet traversal. The traversal and report use the same constant.
No-delta scans and older serialized exports retain None. Existing native loaded
march data verifies 201 calibrated configurations and 55 early returns, along
with complete JSON roundtrip and old-export absence. Both focused march tests
pass (8.61s); clippy (17.29s), WASM (25.27s) and formatting/diff pass. Full
suite passed (546 tests,89.22s) and captured four-film march passed (58.65s),
including the new preamble metadata assertion and complete result roundtrips.
Logs: `/private/tmp/halo-march-preamble-{suite,captured}.log`.

That checkpoint closed only the preamble field; the subsequent full-profile
integration and native configuration comparisons above close the remaining
configuration retention gap.
The completed raid run used the preceding binary; its decoding behavior is the
same, but it does not validate this newly retained metadata field.

## Sequential keyframe-table corpus

Actual native WalkKeyframeRecords was run on all 451 captured keyframes from
32 films with the default context and each film's corruption flag. It returns
902 records and 5,412 component traces; all tables stop at the same unsupported
component in record two, and none of those traces has a captured Payload value.
This is not complete component-body coverage for the 134,657 recovered anchors.

The new ignored corpus test and three matching existing tests pass (23.70s),
`/private/tmp/halo-keyframe-tables-tests.log`. The shared comparison checks native
ordered identities/record bounds/component starts/stop, source packet ranges and
ordinals, and each retained raw field against original source bits. Generator
syntax, formatting/diff and clippy pass. No production code changed
in this corpus extension. Details: `../fixtures/keyframe-table-corpus-v41.md`.

The newest test binary adds one ignored test (44 total); the last broad run above
predates that test-only addition. It does include the preamble-retention code.

## Full-state reads at recovered anchors

The new actual-native corpus oracle invokes WalkKeyframeFullState at every
WalkKeyframeWorld anchor: 134,657 reads across 451 keyframes / 32 films.
Native generation passed (14.326s): 95,532 complete, 39,125 desynchronized,
468 padded endpoints and 43,631 dead-state values. The full JSONL traces also
retain six kinds of captured payload, including later vitality/parent/timer
values absent from the sequential two-record prefix. Details and counts:
`../fixtures/keyframe-anchor-bodies-v41.md`.

The Rust streaming comparison passed all 134,657 reads (54.43s), with exact
endpoints, desynchronization indices, identities and ordered ported component
indices/names/starts. Clippy passed (8.67s); generator syntax and formatting/diff
checks passed. Log: `/private/tmp/halo-keyframe-anchor-bodies-tests.log`.
Two initial failures were
incomplete test context: the historical encoding fixture lacked explicit world
axes and left simulation completion at the legacy default. The comparison now
uses NativeScanProfile's default component encoding with simulation completion
explicitly false, matching ContexteParDefaut. No production behavior was changed
for those setup corrections. The subsequent extension below adds typed payload and source-padding assertions.

## Keyframe typed-payload extension

KeyframeRecord.captured_payload projects native typed values from retained fields
and observations. The expanded 134,657-anchor corpus test passed in 399.18s in
`/private/tmp/halo-keyframe-anchor-payloads-tests.log`, comparing 189,416 captured
payloads,43,631 dead-state values, all consumed source/padded bits and each record's
JSON roundtrip. Parent's absent-observer EndBit difference is checked separately
and documented in `../fixtures/keyframe-anchor-bodies-v41.md`.

Clippy passed (13.36s) and all-features WASM passed (19.79s). The broad suite passed (546 tests,45 ignored,70.57s) in
`/private/tmp/halo-keyframe-payload-suite.log`. Earlier trace-only
success alone did not cover these expanded assertions; the 399.18s run does.

## Native context march configuration path

NativeFilmContext.scan_march_facts now walks the cached registry and numbered
source prefix using its complete native frame configuration. It retains a frozen
NativeFrameMetadata in FilmMarchFacts.native_config after calibration and harvest.
Encoding-only APIs and older exports explicitly have no such metadata. The later constructor integration above now uses this full native path.

The existing 256-case native loaded-march oracle passes (17.54s), comparing all
facts, coverage, calibration scores and every native configuration field, plus
JSON roundtrips. Native no-delta cases preserve absent configuration. Snapshot
map-alias isolation passed. All four captured source-context films passed in
17.64s (release), including every native Config field, calibration counters,
complete death/occupancy results, coverage and JSON roundtrips. Logs:
`/private/tmp/halo-native-march-{snapshot-tests,context-corpus}.log`. The subsequent no-delta/version regression passed (0.02s): no-delta v41
scans skip invalid unused profile widths and retain absent configuration, while
v75 is explicitly rejected before that early return. The 548-test full suite
predates only this version guard and added regression. Native trial walks run during calibration and restore
their world rather than omitting traversal. No architecture refactor is started.

## Remaining acceptance gates

- New native recovery retention shares identical bounded/native records, but
  distinct native padded records require additional storage. Captured padding
  and regular integration tests pass. The release six-film comparison passed in 75.20s at peak RSS
  2,847,883,264 bytes (macOS child getrusage). There is no equivalent pre-change
  memory measurement, so this is not a measured memory regression. The earlier
  `/usr/bin/time -l` wrapper was refused a host-statistics query after successful
  tests; child getrusage measured the completed release run without that query.

- Whole-Film source/field retention and callback orchestration beyond the tested
  document and scanner paths; complete corpus/reference acceptance. A matching
  replay document is not proof that every lower-level field is retained.
- Further inference/recovery lifecycle coverage beyond the now-verified fresh-reader
  seeded-world paths; remaining observer mutation and lazy-source failure contracts.
- Negative-cursor continuation; remaining rare
  inference/repair paths and callback mutation around budget exhaustion.
- Positive captured v41 VIP/bomb and complete equipment-recovery coverage. The
  32 local manifests do not establish positive VIP/bomb recordings; sampled
  upstream minifilms cannot establish complete-film parity.
- Source bytes are owned by FilmSource. Film JSON is not a lossless archive.
  Reference-generated fixtures do not satisfy the deferred independently
  annotated semantic golden-film suite or browser/playback requirements.

## Ignored-test index

These are excluded from the regular suite. Separate completed executions are
listed above; this index is not a claim that those tests have never run. Some are
diagnostic probes rather than acceptance assertions; inspect their bodies before
treating a passing run as evidence for a requirement.

```text
test theater::corpus_tests::biped_position_scan_matches_production_films ... ignored, requires four downloaded films; compares actual reference positions and companion fields
test theater::corpus_tests::inventory_scan_matches_production_films ... ignored, requires four downloaded films; focused native inventory fields and exported ordinals
test theater::corpus_tests::local_film_player_assembly ... ignored, requires six downloaded films; compares native player, combat and vehicle assembly from film bytes
test theater::corpus_tests::local_film_raid_assembly ... ignored, requires the hour-long raid and its generated native oracle
test theater::corpus_tests::local_keyframe_frontiers ... ignored, diagnostic frontier report over downloaded v41 keyframes; not a parity assertion
test theater::corpus_tests::local_managed_property_scan ... ignored, requires six downloaded films; compares native managed-property reads and counters
test theater::corpus_tests::local_native_highlights_and_deaths ... ignored, requires downloaded v41 film footer chunks
test theater::corpus_tests::local_radial_scan ... ignored, requires six downloaded films; compares all raw native radial readings and scan counters
test theater::corpus_tests::local_source_roundtrip ... ignored, requires the downloaded Bazaar v41 film
test theater::corpus_tests::local_v41_corpus_preserves_existing_observations ... ignored, requires downloaded experiments/films and pre-existing decoded-film.json files
test theater::corpus_tests::movement_scan_matches_production_films ... ignored, requires four captured films; production movement transition and counter parity
test theater::equipment_creations::tests::local_vehicle_creation_corpus ... ignored, requires five captured v41 films, including the hour-long raid
test theater::equipment_state::tests::local_equipment_state_corpus ... ignored, requires four captured v41 films
test theater::i0_layout::tests::local_i0_layout_corpus ... ignored, requires six downloaded v41 films
test theater::keyframe_diagnostics::tests::local_keyframe_queue_corpus ... ignored, requires four local v41 captured films
test theater::keyframe_loadouts::source_tests::local_loadout_sources_match_native_corpus ... ignored, requires four local v41 films
test theater::keyframe_position_probe::tests::local_keyframe_position_corpus ... ignored, requires six downloaded v41 films
test theater::kill_decode::tests::local_kill_source_decode ... ignored, requires downloaded v41 captures; complete native kill-source decoder
test theater::kill_film_evidence::tests::local_kill_film_evidence ... ignored, requires the downloaded v41 corpus
test theater::kill_walk::tests::local_kill_record_walk ... ignored, requires downloaded v41 captures; calibration and full record walks
test theater::march_scan::tests::local_native_march_corpus ... ignored, requires downloaded v41 films
test theater::objective_extract::tests::local_objective_corpus ... ignored, requires the local decompressed film corpus
test theater::player_table::tests::native_player_table_corpus ... ignored, requires the local v41 film corpus
test theater::recovery::tests::corpus_anchor_recovery_matches_reference ... ignored, requires downloaded 32-film corpus; compares all 451 captured keyframes
test theater::replay_document_film::tests::complete_ctf_decoded_kill_document ... ignored, complete embedded v41 CTF film with decoded kills; takes several minutes
test theater::replay_document_film::tests::complete_ctf_document ... ignored, complete embedded v41 CTF film; takes several minutes
test theater::replay_document_film::tests::complete_ctf_geometry_document ... ignored, complete embedded v41 CTF film with catalog flag geometry
test theater::replay_document_film::tests::local_complete_film_documents ... ignored, requires six downloaded films; compares complete native documents
test theater::replay_document_film::tests::local_complete_raid_document ... ignored, requires the hour-long raid and full-document oracle; generate with --include-raid
test theater::replay_document_film::tests::local_decoded_kill_documents ... ignored, requires six downloaded films; integrated kill decoder and positive Oddball mode
test theater::replay_document_film::tests::local_missing_identity_document ... ignored, requires captured Bazaar film; native missing-identification document
test theater::replay_document_film::tests::local_unknown_format_documents ... ignored, requires captured Bazaar film; native unknown-format documents
test theater::replay_document_film::tests::local_zero_axis_document ... ignored, requires captured Bazaar film; native zero-axis fallback document
test theater::statborg_source::tests::local_loaded_statborg_corpus ... ignored, requires the 32 local v41 films
test theater::unit_equipment::tests::local_unit_equipment_corpus ... ignored, requires six downloaded v41 films
test theater::unit_equipment::tests::local_unit_equipment_film_export ... ignored, requires captured Cadet Blue v41 film; verifies positive Film export
test theater::weapon_hit_scan::tests::local_weapon_hit_aggregate ... ignored, requires six downloaded v41 films
test theater::weapon_hit_scan::tests::local_weapon_hit_corpus ... ignored, requires six downloaded v41 films
test theater::weapon_hit_scan::tests::local_weapon_hit_prefix_tracks ... ignored, requires six downloaded v41 films
test theater::weapon_hit_scan::tests::local_weapon_hit_tracks ... ignored, requires six downloaded v41 films
test theater::weapon_patterns::tests::local_weapon_patterns_corpus ... ignored, requires six downloaded v41 films
test theater::world_object_research::directory_tests::local_research_directory_detected_and_fallback_precision ... ignored, requires local v41 film corpus
```

## Positive objective recording availability

Inspected the pinned reference replay/testdata provenance files. Its bundled
v41 mini-film fb1a1a72 is CTF; the other named mini-films identify versions 33,
37, 38, 39 or 40. Native VIPV0Qualification requires ATT_FILM plus VIP_FILM,
and AssautBombArmsGate requires ASSAUT_CACHE; the positive VIP/bomb recordings
are external cache inputs, not provided by those tests. Requested local paths
or match IDs for v41 VIP/Assault from the user while continuing other parity work.
This is an input gap for those captured gates, not a blocker for all parser work.

## Reference source integrity

`python3 src/theater/reference/verify_reference.py <LevelUp film directory>`
checks every source file against the manifest's pinned SHA-256. The current
checkout passed all 485 files. A temporary altered-source check produced exactly
one mismatch and a nonzero exit. This verifies inventoried source integrity,
not completeness of the inventory or correctness of the port.

The four revision/ files are reference build tooling for Go source fingerprints,
godoc history, gate messages and golden regeneration. They do not decode film
recordings and are marked reference-only. Native revision identities exposed by
parser output are a separate requirement and remain in scope. Other pending or
partial file entries have not been promoted by this source-integrity check.

## Partial cache metadata selection follow-up

A native LoadFilm oracle exposed eager Rust conversion of unused manifest chunk
types. Directory loading now selects the first matching manifest row per existing
file before converting its type. Missing-file rows, shadowed duplicates and the
unknown-number row no longer reject an otherwise loadable partial source. All
source files and raw manifest bytes remain retained by their existing owners.
Details: `../fixtures/cache-partial-metadata-v41.md`.

Native generation passed (0.322s); all six disk-cache tests passed (0.03s).
Clippy passed (14.25s), all-features WASM passed (8.82s), formatting/diff checks
passed, generator AST parsed, and all 485 reference source hashes matched.
Selected i64 chunk types outside FilmSourceMetadata's i32 range still return an
explicit error and remain an acceptance gap; the regression does not label
that fourth oracle case as equivalent native loading.

The full Theater suite passed: 552 passed, 0 failed, 46 ignored (75.99s).
Log: `/private/tmp/halo-cache-partial-suite.log`.

## Full-width native source metadata

Supersedes the selected-type loading limit documented immediately above.
FilmSourceMetadata.chunk_type now uses i64, matching the pinned native machine
integer. FilmCacheSource loads selected oversized values without narrowing;
normal native directory alignment still ignores missing rows, later duplicates
and the unknown-number manifest row. The temporary selection-before-conversion
helper is no longer needed and has been removed.

The native partial-cache oracle now has six cases, adding selected i64::MIN and
2^32 alongside i64::MAX. All six compare complete metadata and chunk bytes;
metadata JSON roundtrips preserve exact values. The cache test also verifies
normal FilmChunkData conversion and explicit FilmSourceError::ChunkType for
values outside that external model's i32 range. This is an adapter boundary,
not a native source-loading limit. The existing client API model is unchanged.

Native generation passed (0.359s); all six Rust cache tests passed (0.03s).
All 485 pinned source hashes matched and the generator AST parsed successfully.

Full Theater suite: 552 passed, 0 failed, 46 ignored (74.45s), logged in
`/private/tmp/halo-source-types-suite.log`. Clippy passed (14.28s) after
removing a newly redundant test-only integer conversion; WASM passed (7.30s),
and formatting/diff checks passed. The full suite predates only that equivalent
test-expression cleanup; clippy checked the final test sources.

## Live-reader wide primitive reads

The signed-cursor audit exposed a separate primitive mismatch:
NativeFilmReader.read_bits rejected widths above 64, although the pinned
source.Bits.ReadBits consumes the whole width and retains the final 64 bits.
The live reader now does the same with at most one 64-bit read and exact tail
padding. Checked usize endpoint overflow still returns None without advancing;
this does not claim signed-overflow or negative-position continuation parity.

The existing source-bits-v41 native fixture covers 29,376 nonnegative reads,
including 7,344 widths above 64 and 11,995 reads ending beyond the source. Its
Rust test now compares live-reader values, endpoints and padding as well as the
existing standalone bit-reader contracts. Before the production change the
new assertion failed at p=0,width=65 (None versus native zero), captured in
`/private/tmp/halo-live-wide-before.log`. No fixture was generated or changed.

Expanded primitive oracle passed (0.46s). Full Theater suite: 552 passed,
0 failed, 46 ignored (73.58s). Clippy passed (12.50s), WASM passed (7.08s),
formatting/diff checks passed and all 485 pinned source hashes matched.
Logs: `/private/tmp/halo-live-wide-{after,suite,clippy,wasm}.log`.

## Signed source-reader continuation

The source-bits-v41 native oracle now adds 1,512 chronological sequences and
13,608 operations. All existing data/cases in its 24 rows were verified unchanged.
Sequences cover negative starts, zero reads, panic cursor preservation, signed
skip recovery, and read widths on either side of the native 64-bit fast path
near i64::MAX. The actual pinned source.Bits produces every expected result.

The new Rust regression failed before the fix: a read that should wrap its
cursor instead hit the former checked-add assertion. NativeFilmBits now wraps
short-read endpoints like native ReadBits. For wide reads, it preserves the
loop's consumed prefix and panics at i64::MIN if a later bit would index before
the source. It avoids iterating synthetic padding while preserving continuation
state after a caught panic. The API still accepts widths through u32.

This closes the tested standalone source-reader overflow/continuation contract.
It does not close negative endpoints in component width overrides or the live
NativeFilmReader's unsigned record positions; those remain integration gaps.
The architecture refactor remains deferred.

Validation: native generator passed (0.642s); both source-bit tests passed
(0.55s). Full Theater suite: 553 passed, 0 failed, 46 ignored (75.04s).
Clippy passed (13.87s), WASM passed (7.05s), formatting/diff checks passed,
generator AST parsed and all 485 reference source hashes matched. Logs:
`/private/tmp/halo-signed-cursor-{before,after,suite,clippy,wasm}.log`.

## Retained signed component targets

NativeWidthAdjustment.native_end_bit now exposes the native wrapping signed
endpoint from the already-retained starting bit and width. The checked end_bit
field describes the bounded decoder's usable target; None is not evidence that
the native endpoint was absent. Negative native targets are explicitly cursor
outcomes, not source ranges. The accessor works for existing serialized
adjustments without adding redundant data to Film exports.

Signed-width, large live/static-width and mobility-extra oracle tests now compare
this endpoint directly to native output, including refused negative targets.
This separates field-retention evidence from continuation parity: the bounded
record decoder still refuses those targets, and that integration remains open.

Validation: all five focused tests passed, covering 396 oracle cases across
signed widths, large live/static widths and mobility metadata. Negative targets
are compared to actual native endpoints rather than only asserting refusal.
Signed/large record JSON roundtrips retain the inputs needed by the accessor.
Clippy passed (18.05s), formatting/diff checks passed and all 485 pinned source
hashes matched. Logs: `/private/tmp/halo-width-endpoint-{signed,large,policy,clippy,wasm}.log`.
The broad suite was not rerun for this diagnostic accessor and assertion-only
change; the preceding signed-cursor suite remains 553 passed,46 ignored.
All-features WASM check passed (9.95s).

## Truncated control-view production path

The expanded native view oracle exposed two control-view boundary mismatches:
Rust consumed part of the guarded 13-bit analog group, and bounded action reads
stopped before native zero-tail decoding. Both are corrected. Action padding is
scoped to that block and DecodedFrameView.padded_bits retains synthetic bits;
the resulting view remains Truncated. No unsupported body grammar was invented.

All 2,048 existing oracle rows were verified unchanged apart from added prefix
arrays. All 5,060 byte-prefix cases pass (0.09s), including 2,583 control prefixes
and 11 padded native endpoints. The test compares completion, ordered kinds,
exact boundaries, contiguous fields, every source/padded field bit and JSON
roundtrips. The pre-fix regression failed at a guarded analog pair (Rust 28,
native 22). Native generation passed (0.480s), and all 485 source hashes matched.
Details: `../fixtures/views-prefix-v41.md`.

Full Theater suite passed: 553 tests,0 failures,46 ignored (73.48s).
Clippy passed (14.88s), WASM passed (7.84s), formatting/diff checks passed.
Logs: `/private/tmp/halo-view-prefix-{before,after,suite,clippy,wasm}.log`.

## Production control-tail integration

The same view fixture now adds 2,583 actual DecodeFrameViewsCurseur outcomes.
All earlier standalone and prefix fields were verified unchanged. Native
configuration/preamble composition places empty message/entity views before
control prefixes, and seeds a view-2 entity with a position. Rust compares final
cursor, completed-view count, absence of fabricated entity records, current
world view, every native slot field and complete ProductionFrame JSON roundtrip.
Frame padding includes earlier entity-header padding; the first test run exposed
an incorrect equality assertion between total and control-only padding. Correcting
that assertion required no production change. Native endpoints and world state
match after the previous control-view fix.

Native generation passed (0.375s). All three view tests passed (0.20s), covering
the new frame cases, 5,060 standalone prefixes, 2,048 original views and the
selector regression. Formatting/diff checks and all 485 pinned source hashes
passed. This test-only extension does not rerun the full suite or WASM; the
preceding production change passed 553 tests and WASM. The new test raises the
ordinary suite count to 554. Details: `../fixtures/views-prefix-v41.md`.

Final clippy passed. Logs: `/private/tmp/halo-production-control-prefix-tests.log`
(initial assertion failure), `halo-production-control-prefix-retry.log` and
`halo-production-control-prefix-clippy.log` in the same directory.

## Film control-view export integration

The native fixture now supplies 1,559 standard-preamble frames constructed from
nonempty control prefixes, using no-op control records to preserve alignment.
All 11 padded control outcomes are retained. Actual native frame/control calls
provide expected endpoints, view counts, ordered kinds and completion. Every
prior fixture field was checked unchanged before adding these expectations.

The new test batches the frames into a packet stream with the captured v41
bootstrap and calls Film.try_from_chunks_with_encoding. It verifies packet
source identity, offsets, lengths and timestamps; message/entity termination;
absence of fabricated entity records; native frame endpoints, view counts and
control results; exact unparsed tails; explicit padding; and full Film JSON
roundtrip. This is synthetic integration evidence, not a captured gameplay
or independent semantic golden suite. No production code changed.

Native generation passed (0.424s). The first end-to-end run passed (0.64s), and
all three existing view tests passed (0.20s). The final test also explicitly
asserts native completed-view counts and empty entity-view output. This adds
one ordinary test (555 total); it does not rerun the previous full suite or WASM
for this test-only change. See `../fixtures/views-prefix-v41.md`.

Final end-to-end test passed (0.64s), final clippy passed (8.62s), formatting/diff
checks passed and all 485 reference source hashes matched. Logs:
`/private/tmp/halo-film-control-prefix-{final-tests,view-tests,final-clippy}.log`.

## Malformed datum-body retention

Film.try_from_chunks and replication-stream assembly previously propagated an
invalid type-1 table body as a whole-construction error. They now preserve the
failed body, source packet and error text while continuing through independently
framed packets. Film.datum_failures retains ordered failures even with coverage
retention disabled; ReplicationPayload.InvalidDatums retains the outcome in the
packet hierarchy. The standalone decoder still refuses malformed bodies, and
invalid packet envelopes still fail indexing. No table values are invented.

Actual native LireBlocDeDatums accepts two of 129 tested lengths (62 and 104)
and rejects 127. The public Film integration test passes (0.75s): two tables,
127 exact failed payloads/source references, 130 surrounding frames and full
JSON roundtrip. Native generation passed (0.406s). Error classification and slot
counts are compared; exact native French error strings are not claimed.
See `../fixtures/datum-lengths-v41.md`.

Full suite passed:556 tests,0 failures,46 ignored (73.82s). Clippy passed
(44.70s including build-lock wait), WASM passed (27.87s including lock wait),
formatting/diff checks passed, generator AST parsed, and all485 pinned source
hashes matched. Logs: `/private/tmp/halo-datum-failure-{tests,suite,clippy,wasm}.log`.

## Typed native datum refusal reasons

DatumTableError now retains the three native size-refusal forms: NoEntries
with byte length, AboveCapacity with derived slot count, and Misaligned with
total bits, derived slots and signed remainder. DecodeError.Datums exposes the
typed cause. Film failures and InvalidDatums packet payloads retain it as
optional native_error; older exports leave that field absent. The existing
message string now uses the exact pinned native wording.

The length oracle was extended from 129 to 139 cases by adding offsets around
8,191- and 8,192-slot table sizes. Every old length/error/slot expectation was
verified unchanged. Tests compare native acceptance, slot counts and every
error string, cover all three variants, and roundtrip typed errors. The small
interleaved Film test checks retained native reasons as well as raw failures.

Native generation passed (0.486s); all five datum-related tests passed (1.88s),
including the captured full table and keyframe datum comparisons. This
supersedes the earlier caveat about not comparing native error text. Details:
`../fixtures/datum-lengths-v41.md`.

Full suite passed:557 tests,0 failures,46 ignored (81.55s). Clippy passed
(46.37s including lock wait), WASM passed (36.98s including lock wait),
formatting/diff checks passed, generator AST parsed and all485 source hashes
matched. Logs: `/private/tmp/halo-datum-reasons-{tests,suite,clippy,wasm}.log`.

## Rejected datums preserve recovered bindings

The existing captured-keyframe regression now inserts NoEntries, Misaligned
and AboveCapacity bodies separately before a later update. Each run retains
all 123 recovery anchors, exactly the baseline binding table and decoded later
frame, rejected payload/source/native reason, and adjusted later packet offset.
Complete replication JSON roundtrips pass. The targeted test passed (0.65s),
Clippy passed (8.99s), formatting/diff checks passed and all 485 source hashes
matched. No production code changed, so the 557-test baseline above remains
the latest broad run. Logs: `/private/tmp/halo-datum-binding-retention-tests.log`
and `/private/tmp/halo-datum-binding-retention-clippy.log`.

This closes a Rust continuation regression, not native damaged-stream parity
or the remaining whole-Film acceptance gates. See
`../fixtures/datum-lengths-v41.md` for the fixture and evidence boundary.

## Live signed variable-width reader

NativeFilmReader now exposes read_signed_variable, using the existing padded
cursor codec. The actual native Lecteur.ReadSignedVarWidth oracle provides
3,784 sequences with 11,352 signed reads and subsequent primitive reads. All
selectors/alignments and byte prefixes are covered, with 2,694 negative results
and 8,624 padded endpoints. Rust compares every value and endpoint before the
next read. No old oracle fields were regenerated or changed.

Native generation passed (0.428s), targeted comparison passed (0.05s), full
Theater suite passed (558 tests, zero failures, 46 ignored, 78.19s), Clippy passed
(11.38s), all-features WASM passed (15.65s including lock wait), formatting/diff
and generator syntax checks passed, and all 485 pinned hashes matched. Logs:
`/private/tmp/halo-signed-variable-reader-{oracle,tests,suite,clippy,wasm}.log`.

This closes the live codec omission. Negative signed cursor continuation and
address-overflow parity remain outside this unsigned reader's contract. Whole
Film/corpus and missing positive objective gates above remain open. The queued
architecture is unchanged. See `../fixtures/signed-variable-reader-v41.md`.

## Live independent profile/observer replacement

NativeFilmReader now exposes profile, replace_profile and replace_observer.
The independent setters return the previous profile/shared receiver and preserve
cursor, capture slot and optional accumulator. The existing 1,064-case native
sequence fixture now checks the live reader alongside the per-attempt decoder:
6,384 original reads, 6,348 ordered callbacks, and 72 component names. Native
detachment adds 1,064 reads (19 rejected) with no receiver deliveries. Every
earlier fixture field was compared unchanged. Position tests alternate complete
context replacement with the independent setters and compare native callbacks
and accumulated positions after 32,768 reads across 2,048 sequences.

Native generation passed (0.389s). The original six-step live comparison passed
(17.57s), all three position tests passed (0.73s), and the final test including
detachment passed after resuming the interrupted run (18.16s). Handles from the
interrupted runs were absent and logs lacked final outcomes for replacement,
Clippy and WASM; only those checks were restarted. Resumed Clippy passed
(22.49s), WASM passed (26.78s including lock wait), and the final full suite passed
(558 tests, zero failures, 46 ignored, 72.85s). Formatting/diff and all 485 pinned
source hashes pass. Logs: `/private/tmp/halo-live-reader-replacement-` followed
by `resumed-tests.log`, `position-tests.log`, `resumed-clippy.log`,
`resumed-wasm.log` or `suite.log`.

This closes these live API operations and their tested preservation contracts.
Remaining callback-time mutation, lazy conversion, signed cursor and whole-Film
acceptance gates are unchanged. See READER_CONTRACT.md and
`../fixtures/reader-sequence-v41.md`.

## Source player-table diagnostics in Film exports

The source adapter's diagnostic variant retains typed native decoder refusals
and unknown-build metric increments alongside the projected table. Film now
retains the same diagnostics from its already decoded player table, including
explicit absent identity and measured success. Older JSON leaves the optional
field unavailable. The original table-only adapter keeps its return shape.

The native source oracle adds actual log observations and levelup expvar deltas
to its 19 existing cases, verifying every old field unchanged, plus a twentieth
double-compressed input. A first metric probe used the wrong expvar namespace;
corrected native generation passed (0.594s). An early Rust run still had the old
fixture and failed on its missing observations field; after the checked fixture
update, the complete source/Film comparison passed (4.67s). It checks exact
native error strings, increments, successful/unknown-build/no-identity Film
construction, and both current and old-export JSON behavior.

Full suite passed: 558 tests, zero failures, 46 ignored (79.14s). Clippy passed
(39.27s), WASM passed (60s including build-lock wait), formatting/diff and all
485 source hashes passed. Logs: `/private/tmp/halo-player-table-source-observations-`
with suffixes `metrics-oracle.log`, `updated-tests.log`, `suite.log`, `clippy.log`
and `wasm.log`. The diagnostic is publication data; native logging and global
counter accumulation are not performed. Whole-parser acceptance remains open.

## Source player-table publication and interleaved seats

ReplayFilmPlayerTable.log now emits native success/refusal records, plus the
separate interleaved-vacancy warning after measured success counts. Film calls
it with match identity and its retained diagnostics. Source-only callers can
publish explicitly. Metric increments remain retained per-call data; no global
counter is modified. Native nil error becomes an omitted tracing field, normalized
to null only in the test; retained diagnostic absence is checked independently.

The source oracle now has 21 cases, preserving every prior field. Its additional
case splices a generated slot body from the existing native player-table fixture
after the captured identity endpoint. Native decoding reads seven occupied seats,
marks interleaved vacancies, refuses direct identity mapping and emits INFO then
WARN. The test checks all source-table messages, levels, fields, match IDs and
order, plus four Film constructor/JSON cases. This is a synthetic composition,
not a new captured gameplay film. Registry warning delivery remains separate.

Native generation passed (0.700s), targeted test passed (5.16s), full suite passed
(558 tests, zero failures, 46 ignored, 72.07s), Clippy passed (15.15s), WASM passed
(22.13s including lock wait), formatting/diff and all 485 source hashes passed.
Generator syntax passed; the harness is now explicitly registered after its
player-table input fixture with relocatable paths. Logs:
`/private/tmp/halo-player-table-publication-{oracle,tests,suite,clippy,wasm}.log`.
Whole-parser/corpus acceptance and the queued architecture remain unchanged.

## Native fixture regeneration registration

The generator now invokes fourteen retained source/publication harnesses that
were previously absent from its execution path. All fourteen were executed
through the new runner with writes disabled: complete JSON results, including
array order, match their retained fixtures. No expectations were replaced.
NavpointSource now declares its primitive source dependency instead of reading
an unexplained host temporary file. MovementTrace remains a diagnostic-only probe.
See ORACLE_REPRODUCIBILITY.md and source-publication-regeneration.json for the
scope, outcomes and hashes. Syntax/diff/source-hash checks pass. No Rust code or
included fixture changed; this does not require repeating the parser baseline.

## Native zero delta-width fallback

The shared NativeScanProfile component adapter now applies native deltaAxisW:
zero override selects the traversal descriptor's corresponding axis; positive
values override all three axes. The original profile remains unchanged.
The pre-fix native-backed regression reproduced a Rust unsigned-subtraction panic.

All 1,440 native cases now match: 288 zero overrides, 708 padded endpoints,
980 ordered position callbacks, accumulator state and subsequent read alignment.
Source prefixes include 60 absolute callbacks as branch controls. Native generation
passed (0.525s), and the regression passed (0.06s). Expectations come from the
actual native dispatcher; these are synthetic cases, not semantic action goldens.
See ../fixtures/delta-width-fallback-v41.md for the matrix and scope limits.

Full Theater suite: 559 passed, zero failed, 46 ignored (69.75s). Clippy passed
(17.99s including build-lock wait); all-features WASM passed (4.99s).
Formatting, generator syntax, diff checks and all 485 pinned source hashes passed.
Logs: /private/tmp/halo-delta-width-fallback-{oracle,before,tests,suite,clippy,wasm}.log.
This closes this fallback mismatch, not the remaining signed-cursor, lazy-width,
source/corpus acceptance or independently annotated semantic validation gates.

## Native zero/wide scalar and position boundaries

Native padded component reads now consume widths above 64 and return the native
low 64 bits. Additional discarded-prefix fields retain every actual source bit
outside that numeric result, bounded by source size rather than padded width.
The primary field spans the complete native read and overlaps its prefix fields;
see ../fixtures/absolute-width-boundaries-v41.md for the public field contract.
Bounded Cursor reads retain their previous refusal and cursor behavior.

Immediate and deferred position capture now reproduce Go's uint64 shift rules:
zero traversal widths are valid, shifts at or above 64 yield zero, and absolute
results can be NaN/infinity. Native float bits are retained in diagnostics.
The expanded delta oracle first reproduced native success versus Rust refusal.
All original fields in the previous 1,440 cases remained unchanged.

The native corpus for these branches now has 3,528 delta cases plus 720 absolute
cases. Both reader/capture paths compare native status, field values/boundaries,
ordered callbacks, complete slot position bits and subsequent read alignment.
Independent bit-by-bit source-prefix checks and DecodedComponent JSON roundtrips
pass. Native generation passed (0.372s delta, 0.340s absolute); four targeted
reader tests passed (0.31s), including the existing context and signed-variable
regressions. The width tests include 3,681 wide and 1,065 zero-width axis fields.

Full Theater suite: 560 passed, zero failed, 46 ignored (69.92s). Clippy passed
(30.22s); all-features WASM passed (16.59s), both including build-lock waits.
Formatting, generator syntax, diff checks and all 485 pinned source hashes pass.
Logs: /private/tmp/halo-width-boundaries-{tests,delayed-tests,suite,clippy,wasm}.log;
separate native logs use halo-{delta,absolute}-width-boundaries-oracle.log.
WASM is a compile check; exact nonfinite bit comparisons ran on the native host.
These synthetic grammar fixtures do not replace captured-film acceptance or
independent semantic annotations. Signed cursor continuation and lazy conversion
of unused native widths remain open; the architectural refactor stays deferred.

## Complete scan-input retention before publication

The six-film actual-native document oracle now retains all 44 exported FilmInputs
fields before document publication. Structured MPP-width maps are represented as
sorted key/value arrays because the native aggregate cannot be JSON-marshaled
directly. All prior document fields were verified unchanged. Native generation
passed (67.728s); the initial direct-JSON attempt failed on the structured map
and was superseded by this explicit field-preserving projection.

Full inputs are stored in six separate fixtures referenced by the existing
full-document fixture. The raw native output is 445 MB. The tests stream past
unchecked inputs instead of loading all intermediate positions into Value.
Generator write_full_document_oracles writes the six input files and document
references together; this helper was executed against the native output.

The existing complete-document test now compares 20 raw input fields on every
film, including all ordered creation, fire, loadout, movement, grenade, ability,
camo and pickup records, player tables/team results, and selected scan counters.
It additionally checks the unchanged complete replay document. All six passed
in 82.34s; the separate complete-document schema retention test passed too.
Clippy passed (18.58s), formatting/diff/generator syntax passed, and all 485 pinned
source hashes passed. No production decoding code changed in this checkpoint;
the 560-test full-suite and WASM baseline above remains applicable.

Evidence: ../fixtures/full-document-inputs-v41.md and
film-input-retention-coverage.json list exact checked fields, the 24 remaining
fields and fixture hashes. Logs: /private/tmp/halo-full-inputs-{oracle,tests,
clippy,schema-tests}.log. The first release build was deliberately interrupted
after the large oracle size was discovered; the final release build and both
test commands completed successfully. These input comparisons do not establish
independent gameplay semantics or complete source/parser acceptance.

## Native Film clock retention and expanded scan inputs

Film.native_clock_origin now retains Read(FilmPacket), MissingChunk or
MissingPacket; absence in older exports remains unavailable. It is independent
of the legacy minimum-nonzero origin. Replay identity assembly uses the retained
result, preserving the original source fallback only for older exports.
The compact constructor regression passed (1.36s), including source ranges,
recorded zero, both failures, complete JSON and old-field absence. All 1,024
native identity-evidence cases pass through the retained-clock path too, including
174 native clock failures. See ../fixtures/film-clock-origin-retention-v41.md.

The pre-publication Film comparison now checks 29 of the 44 retained native
inputs. Added: inventory and delta inventory, ammo-refusal verdict, equipment
changes and complete scan/assembly counters, spawn events/counters, zoom events
and the native clock. All six captured films and their complete documents match
(82.17s). All native input fixture hashes are unchanged. Fifteen inputs remain
explicitly outside this integrated check; the exact list is in
film-input-retention-coverage.json.

The initial expanded run stopped on three Bandit inventory gauges represented as
Rust JSON 0.0 versus native JSON 0. Gauge is now normalized through its declared
float64 type, as Fire Aim already is through float32; no tolerance or field/record
filter was introduced. Mismatches now report the first precise path, avoiding
large full-array logs. The successful rerun supersedes that representation-only
failure.

Full Theater suite: 561 passed, zero failed, 46 ignored (93.53s). Clippy and
all-features WASM passed (65s and 22.97s including build-lock waits). Final Clippy
after the test's Gauge normalization passed in 9.01s. Format/diff and all 485
reference source hashes passed. Logs: /private/tmp/halo-film-clock-retention-
{tests,suite,clippy,wasm}.log and /private/tmp/halo-full-inputs-expanded-
{tests,retry,clippy}.log. No architectural refactor has begun.

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

### Fallible source-provider contract

The public facade audit found that Rust lacked native source.Load's fallible
provider input. FilmChunkProvider and FilmSource::load_from now query the count
once, read/inflate/index each chunk in order, stop on the first read error and
retain its position and original typed cause. Existing slice loading delegates
through this path. Loaded bytes and metadata are owned.

The pinned native source-provider fixture has 256 cases comparing read order,
first-error behavior, errors.Is/original-cause identity, loaded bytes, metadata
and ordered packet payloads/timestamps. It includes negative and zero counts,
empty/malformed/zlib/raw chunks and differently sized metadata. All three focused
source tests pass (0.29s). Full Theater: **573 passed, zero failed, 46 ignored**
(87.23s). Clippy passes (41.90s), all-features wasm32 compilation passes (17.67s),
and formatting, diff, generator syntax and all 485 pinned source hashes pass.
This WASM check compiles the API; it is not a provider runtime oracle on wasm32.

PUBLIC_DECODER_AUDIT.md maps source/profile exports to implementations/evidence
and records the remaining boundaries: i32::MAX count limit, unsigned source
positions, owned errors/data, eager directory file reads, and no compressed-byte
archive. Captured-film tests were not rerun for this change. Broader scalar,
cursor, callback, retention and corpus gates remain open; full parity is not
complete. The three-layer architecture remains deferred in NEXT_PHASE.md.

Evidence: halo_rust_source_provider_test.go.txt;
../fixtures/source-provider-v41.json.zlib; logs
/private/tmp/halo-provider-{tests,suite,clippy,wasm}.log.

### Lazy directory source and first-error propagation

FilmDirectorySource now discovers and numerically sorts paths without reading
contents, implements FilmChunkProvider with owned reads, and retains file paths
and original I/O errors. FilmSource::load_directory aligns metadata and delegates
to load_from, so each file is inflated/indexed before the next file is read.
This supersedes the eager-directory limitation in the previous checkpoint.

The independent pinned native lifecycle oracle covers 64 cases: 17 successful
loads, 24 missing-file errors and 23 directory-read errors. It compares 288 direct
position reads, repeated reads after overwrites, 190 loader reads, stable source
counts after additions, bounds errors, and first-error stopping. Rust also checks
owned previous-read bytes and the error chain down to std::io::Error.

Four focused source tests pass (0.28s). Full Theater: **574 passed, zero failed,
46 ignored** (77.64s). Clippy passes (40.01s), all-features wasm32 compilation passes
(6.69s), formatting/diff/generator syntax and all 485 source hashes pass. Captured
film tests were not rerun. Directory filesystem APIs remain host-only.

Literal directory paths are supported; Go filepath.Glob expressions, enumeration
failure semantics and platform-specific error wording remain unreconciled. These
checks do not close record/field retention or whole-corpus parity. NEXT_PHASE.md
remains deferred; the three-layer refactor has not started.

Evidence: ../fixtures/source-directory-lifecycle-v41.md and its compressed JSON;
halo_rust_source_directory_lifecycle_test.go.txt; PUBLIC_DECODER_AUDIT.md;
logs /private/tmp/halo-directory-lifecycle-{tests,suite,clippy,wasm}.log.

### Automatic Film weapon-hit retention

A concrete propagation gap is closed: Film constructors previously initialized
weapon_hits to None and required a separate retain_weapon_hits call. Base and
explicit-encoding construction now preserve direct shot/damage reads and native
pairing output automatically. Map-aware construction refreshes with resolved
sampling precision to retain tracks and distance statistics. weapon_hits_error
retains independent failure; a failed refresh keeps earlier direct results,
and successful refresh clears its error. Other Film channels remain available.

Four focused tests pass (2.66s), covering positive hits without distance, missing
native chunk-one prefix with other indexed packets retained, explicit failed
refresh/retry, JSON and old exports. The map-constructor regression now verifies
automatic retention. All six captured base-constructor cases match the pinned
native oracle: 4,395 shots, 105 damage observations, landing bases and no-distance
statistics, with packet-source checks and whole-Film JSON roundtrips (20.41s).
The release build took 3m29s. No fixture expectations were generated by Rust.

Full Theater: **574 passed, zero failed, 47 ignored** (95.08s). The new captured
test was explicitly run as above; other captured suites were not rerun. Clippy
passes (12.24s), all-features wasm32 compilation passes (7.66s), formatting/diff and
all 485 pinned hashes pass. Map-aware construction currently rescans hit inputs
when it adds distance evidence, increasing work and export size. This is current
parser retention work, not the deferred architecture.

See FILM_RETENTION_AUDIT.md and ../fixtures/weapon-hit-film-retention-v41.md.
Logs: /private/tmp/halo-hit-retention-{tests,suite,captured,clippy,wasm}.log.
Whole-source fidelity, remaining native contracts and broader corpus acceptance
still prevent a full-parity claim. NEXT_PHASE.md remains deferred.

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

### Record-ID follow-up: confirmed native contract

The pinned frame_records.go readRecordID reads low bits only for a strictly
positive IDLowBits value. Nonpositive widths consume no low bits but still read
the two generation bits. It casts the low result to uint32, adds IDBase with
uint32 wrapping, and masks the result to 30 slot bits before OR-ing generation.
End records do not read an ID. These differ from the current checked Rust header
adapter (30-bit width/base bounds and checked addition), and contextual setup
still validates ID width eagerly. This remains an actionable native parity gap.
The next fixture should vary signed/zero/wide widths, base overflow/masking,
record kinds and bit alignment, checking endpoints, IDs and world outcomes.
Keep the bounded public header helper contract separate from native context.

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

### Objective/statistics facade audit checkpoint

The 19 objectives function targets in decfilm are now reconciled to explicit
Rust implementations and native fixtures in public-objectives-targets.json and
PUBLIC_DECODER_AUDIT.md. Source-pass ordering, score-policy ordering and complete
result comparisons were inspected. Focused revalidation passes 36 tests, zero
failures, one captured test ignored. No production/fixture change was needed.
This narrows the remaining public-target audit; it is not a full-parity claim.
The previous 579-test production baseline remains applicable; architecture is
still deferred. Other targets/types and the existing corpus/contract gates
remain open.

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

## Complete public type-alias inventory and representation map

inventory_public_types.go.txt resolves all 50 decfilm type aliases to their
native declarations and records fields (including private and embedded fields),
alias targets, JSON tags and receiver methods with source locations. The saved
public-type-declarations.json has 274 directly declared field entries after
expanding grouped names and 98 directly declared exported receiver methods.
These counts are inventory sizes, not completion percentages. Alias-resolution
chains remain explicit; transitive field types require their own audit.

public-type-rust-map.json maps all 50 names/targets to existing Rust storage and
queries, with test pointers and representation distinctions. All names/targets
match decfilm-export-inventory.json exactly. All Rust paths and named test
pointers exist; native regeneration reproduced the declarations byte-for-byte,
and all 485 pinned source hashes passed. The pre-existing tests cited for 34 of
the 35 newly mapped types pass in the preceding 583-test baseline. The health
method evidence is now supplied by the separate new oracle below.

Important distinctions are explicit: BipedCreation.HasIndex is implied by the
native acceptance gate; source packet metadata is outside the primitive record;
BipedPosition world availability is represented by world/quantized result types;
FrameConfig embeds the profile/observer through NativeReaderContext; Registry
trailing bytes and cached parse identity are separate from mutable archetypes;
map width projection and source ownership/index domains are documented. Ordinary
biped i2 roll is skipped by the native companion scanner too; its absence there
must not be mistaken for a Rust-only omission or silently interpreted.

This checkpoint maps representations and locates evidence. It does not claim
all 98 method input domains, mutable aliases, callbacks, transitive types or
remaining source contracts have been proven equivalent. No deferred structural
Film/ResolvedFilm/playback implementation was started.

### Native health-method coverage

The audit found that captured result-field checks did not independently exercise
all health methods. kill-health-methods-v41.json.zlib now contains 2,800 native
health states, crossing warning thresholds/catalog/roster flags with coverage
cases independently. All seven KillSourceHealth methods, PathStats.Ratio and
16,800 Result.LineByLinePublishable decisions are compared. It yields 1,820
ALERTE, 808 HORS DOMAINE MESURE and 172 NOMINAL states. Diagnostics and metric
pairs retain order; ratios compare by f64 bits. Counts other than DeathsReal
are nonnegative in this fixture. See ../fixtures/kill-health-methods-v41.md.

The first 560-case fixture matched method values but its independent coverage
assertion exposed correlated inputs that omitted NOMINAL. It was replaced by
the crossed coverage matrix above; no production parser correction was needed.
The focused test passed in 0.18s; native generation took 0.385s.

Public-type audit validation: full Theater 584 passed, zero failed, 49 ignored
(73.25s); Clippy passed (9.40s). Actual WASM passed its existing 14,474 native
comparisons/80 address-domain refusals and supplementary controls, plus all
2,800 new health states/16,800 publication decisions (build 8.86s). Formatting,
diff, Python syntax, deterministic AST inventory regeneration, mapping integrity
and all 485 pinned reference hashes passed. No production decoding behavior
changed, so captured-film tests were not rerun at this checkpoint. Their prior
signed-options results remain recorded separately. No tests remain running.

Evidence: public-type-audit-validation.json and /private/tmp/halo-public-types-
{native,tests,suite,wasm,clippy}.log. Missing positive v41 VIP/Assault recordings
were requested while this independent work continued. The full parity goal is
still active; type/field counts and passing subsets do not close the remaining
function, method/transitive-type, source/runtime or captured-mode gates.

### Public function and constant reconciliation

All 71 facade function names now have implementation/evidence pointers in
PUBLIC_DECODER_AUDIT.md. The independent snapshot contains all 41 constants and
three serializable variable aliases; regeneration matches exactly. A new test
compares production values and observable adapters against that snapshot.
Validation: 585 Theater tests passed, 49 ignored; clippy, formatting, diff checks
and 485 pinned hashes passed. No production behavior changed; captured and WASM
runs were not repeated. See public-constant-audit-validation.json for exact scope.
Name reconciliation is not full semantic parity. Existing source/callback/cursor
and captured-mode gates remain open; the architecture phase stays deferred.

### Stateful reader signed cursor integration

NativeFilmReader now uses NativeFilmBits for direct signed cursor movement,
remaining counts, scalar reads and variable-width integers. The new wide-read
API accepts the pinned uint64 width domain on host and WASM. Host validation
compares 13,608 sequence operations and 78 overflow/recovery cases; WASM checks
4,896 pre-panic prefix operations and 28 nonpanicking overflow cases. All 587
regular Theater tests pass (49 ignored), as do clippy, formatting and 485 pinned
reference hashes. The four-film debug march regression is still running; only
Bazaar and Aquarius have reported success so far. No complete captured result
is claimed. See reader-cursor-validation.json and ../fixtures/reader-cursor-v41.md.
Component/frame address restrictions and internal negative width-adjustment
continuation remain open, along with the existing all-data parity gates. The
three-layer architecture remains deferred.

### Stateful quantized-vector API and signature-type audit

The 26 named types exposed through facade signatures beyond the 50 aliases are
now inventoried and mapped in PUBLIC_DECODER_AUDIT.md (156 direct fields, 23
defining files checked against pinned git objects). This is not a closed
transitive graph. The audit identified native Lecteur.ReadQuantizedVec3 as a
missing stateful method; NativeFilmReader now exposes it with signed cursor
continuation and native float evaluation order. Native comparisons cover 4,096
vectors and 72 signed boundary cases; WASM covers the ordinary vectors and the
15 nonpanicking boundary cases. Full suite: 589 passed, 49 ignored; clippy,
formatting and WASM passed. The captured debug march run has passed Bazaar,
Aquarius and Bandit Evo and remains active on ranked Oddball. See
reader-quantization-validation.json and ../fixtures/reader-quantization-v41.md.
Broader parity gates and the deferred architecture phase remain open.

### Resync callback invalidation and iteration guard

The accepted-candidate failed-re-read gap is now checked in 16 native cases:
eight callback deletions retain a failed record and eight controls remain clean.
NativeSharedWidths::remove is public so external callbacks can delete a shared
calibration/stub override. Six additional native cases check 4,095/4,096/4,097
records with/without prefixes, comparing 24,574 ordered IDs and record endpoints.
All three focused resync tests, WASM and clippy pass; traversal logic is unchanged.
The previous full-suite baseline remains 589 passed/49 ignored. The captured
debug march run still awaits ranked Oddball after three successful films.
See resync-recovery-validation.json and the resync-mutation/guard fixture notes.
Final-loop cursor evidence, broader parity gates and the deferred architecture
remain open.

### Final resync cursor and unbound Delta baseline correction

A hash-guarded AST copy of native DecodeFrameResync exposes its final cursor by
adding only br.BitPos() to four returns. Original/instrumented runs publish
identical records, callbacks, maps and bindings for all 70 cases. The comparison
found that Rust's native contextual lookup stopped before an unbound Delta's
one/eight-bit baseline selector. That path now consumes the selector first;
strict-generation entry points retain their existing policy. All 24,582 returned
records, 21 acceptance calls and final cursors match. Full suite: 592 passed,
49 ignored; WASM, clippy, formatting and pinned-source checks passed. See
resync-cursor-validation.json and ../fixtures/resync-cursor-v41.md.
The captured march process remains active on ranked Oddball after three films
passed; it predates this fix and validates the earlier signed-reader checkpoint.
All broader parity gates and the architectural deferral remain in place.

### Nested type audit and provenance presentation

The syntactic transitive graph now resolves 121 public roots, 268 nodes (including
71 function-signature nodes) and 71 additional native declarations. Every edge
resolves; repeated generation is identical; 84 source hashes match pinned git
objects. See TRANSITIVE_TYPES.md. This is an audit inventory, not semantic parity.

The six ability storage contracts are reconciled in TYPE_CONTRACTS.md using
existing complete field/counter comparisons. KillSourceProvenance now supplies
native String behavior through Display; 48 actual-native cases pass on host and
WASM. The initial WASM harness import failure was corrected and the retry passed.

Full Theater suite: **1 passed, zero failed, 0 ignored** (0.01s).
Clippy passes (20.89s); formatting, diff checks, generator syntax and all 485 pinned
source hashes pass. See kill-provenance-validation.json. The current resync
correction also passed the four-film captured march comparison (81.30s, release;
2m40s build). The earlier signed-reader debug run completed too (1896.42s). This
supersedes their prior running statuses; the binaries have separately recorded
scopes. No build/test remains running at this checkpoint.

All-data parity remains incomplete: nested runtime/dynamic-payload contracts,
internal signed-cursor continuation and remaining captured-mode/source acceptance
still require work. NEXT_PHASE.md remains deferred.

### Signed intermediate cursor acceptance gap reproduced

A new 96-case native oracle covers two calibrated/stub skips followed by an
optional real component read. Native produces 78 successful records and 18
panics; Rust differs in 58 cases (40 successful native records and 18 panics).
Thirty-eight controls match. In 18 native successes, an intermediate component
starts at a negative position but the final cursor is nonnegative. Exact native
records and panic endpoints are retained; regeneration is byte-identical.

`native_signed_component_continuation` is an explicitly ignored, failing
acceptance gate, not a passing parity test. Its explicit run fails with all 58
mismatches listed. See signed-continuation-validation.json and
SIGNED_CURSOR_MIGRATION.md. The implementation must preserve signed native
component/record coordinates, not only change skip arithmetic. No production
decoding changed; the existing signed-width comparison still passes. The last
full regular-suite baseline remains 593 passed/49 ignored, before this added
ignored test; the full suite and WASM were not rerun for this test-only audit.
The new gate adds one known ignored test. All 485 pinned source hashes, format,
diff and generator syntax checks pass. Full parity remains incomplete and the
Film/ResolvedFilm/playback architecture remains deferred.

## Signed component cursor migration supersedes earlier refusal checkpoints

The signed component continuation gate now matches all 96 cases. Direct reader
component calls additionally match 80 native cases, including 28 panic endpoints
and recovery sequences. Native trace coordinates migrated to i64; source indexing
and derived padding accounting remain separate. The earlier negative-skip refusal
and unsigned component-coordinate statements describe superseded checkpoints.
Header coordinates remain address-sized, MPP/raw-width restrictions remain, and
whole-frame panic copyback has not been established. See SIGNED_CURSOR_MIGRATION.md
and signed-continuation-validation.json for the exact current boundary.

## Native position-width correction

World/traversal indices and axes, delta overrides, ability anchors, and predicted
handle-tail reads now retain native u64 widths on host and wasm32. Explicit
caller maximums remain lazy; source-prefix retention is bounded by real input.
The independent native matrix contains 18 direct reads and 18 generic frames at
widths 65 and 2^32, checking cursors, recovery reads, record order, callbacks and
exact float bits. Source-retention checks also pass.

Full host suite: **603 passed, zero failed, 49 ignored** (103.58s). Clippy passes
(34.33s), formatting passes, and actual WASM execution passes (12.20s build).
485 pinned reference source hashes match. Captured release comparisons are
tracked in position-domain-validation.json. Higher-level signed inference,
resync and recovery, remaining runtime type contracts, and all-data corpus
acceptance remain open. The architecture refactor remains deferred.

Position-width checkpoint final captured validation: all four march films pass
(81.48s release), and all six complete film documents match (104.30s release).
See position-domain-validation.json. Full v41 parity remains incomplete.

## Signed inference entry correction

Inference now preserves i64 starts, including beyond-source no-op calls and
negative starts repaired by the optional Skip(32). Header reads use the native
signed reader; bound rereads preserve signed coordinates. The independent
90-case oracle passes on host (21 native panics), and 69 nonpanicking cases pass
in actual WASM. Full suite: **604 passed, zero failed, 49 ignored** (108.66s).
Clippy passes (16.28s); WASM build takes 20.10s; 485 pinned hashes match.
Captured release validation is tracked in signed-inference-entry-validation.json.

Signed chain trials remain a demonstrated mismatch: a new native probe accepts
40 negative endpoints that the Rust trial adapter rejects. See
SIGNED_CURSOR_MIGRATION.md for the reproducible next gate. Full v41 parity is
incomplete, and the architectural change remains deferred.

Signed-inference-entry checkpoint final captured validation: four march films
pass (82.10s release), and six complete film documents match (105.57s release).
See signed-inference-entry-validation.json. The signed-trial oracle remains an
open correction; full v41 parity is not established.

## Signed inference trial and confirmation correction

Native trial endpoints, chain candidates, confirmation recursion, and public
inference evidence now preserve i64 coordinates. Prefix and remaining-bit
arithmetic use native wrapping semantics. Contextual infer_unbound/infer_chain
APIs expose results and diagnostics. Successful endpoint -32 is retained through
four complete native inference frames, including record order, soft bindings and
serialization. The independent fixture contains 72 trials, 16 single-step panic
cases, and 32 chain panic cases; all host comparisons pass. Actual WASM passes
56 single-step calls, 40 chain calls, and the four complete frames.

Full suite: **605 passed, zero failures, 49 ignored** (102.08s). Clippy passes
(24.81s), WASM passes (21.31s build), and 485 pinned source hashes match.
Captured validation is tracked in signed-inference-trial-validation.json.
This supersedes the open negative-inference mismatch above. Raw resync/harvest
and broader repair-domain checks remain separate, as do runtime type contracts
and all-data corpus acceptance. Architecture remains deferred.

Signed-inference-trial checkpoint final captured validation: all four march films
pass (82.27s release), and all six complete film documents match (105.64s release).
See signed-inference-trial-validation.json. Raw resync and broader acceptance
remain open; full v41 parity is incomplete.

## Signed raw-resync correction

Both raw scanner APIs accept checked i64 starts and return signed landings.
Sequential raw-resync calls preserve signed continuation and retain the native
negative-landing rejection after callbacks. The independent native scan matrix
has 108 cases (38 panics), checked against both contextual and standalone APIs;
eight complete native frames verify rewind, positive recovery, callback delivery,
record/cursor retention, unchanged bindings, and JSON.

Full host suite: **607 passed, zero failed, 49 ignored** (90.26s). Clippy passes
(15.66s), and actual WASM passes 70 nonpanicking cases for each scanner plus all
eight frames (17.83s build). 485 pinned reference hashes match. Captured release
validation is tracked in signed-resync-validation.json. This supersedes the raw
resync-start mismatch above. Signed harvesting, view orchestration, broader repair
contracts and full all-data acceptance remain open; architecture stays deferred.

Final raw-resync metadata adjustment: both resync_bits lists are i64, avoiding
pointer-size limits on bit positions. The final host suite passes 607 tests,
zero failures, 49 ignored (101.18s), Clippy passes (13.31s), and actual WASM passes
(13.67s build), including metadata-only round trips at bit 2^32 for both result
types. Those controls do not claim an executed multi-gigabit native scan. Final
captured reruns are tracked in signed-resync-validation.json.

Final signed raw-resync captured validation, including the i64 resync_bits change:
four march films pass (81.56s release), and six complete film documents match
(104.86s release). See signed-resync-validation.json. Harvesting, view orchestration,
and broader all-data acceptance remain open; full v41 parity is incomplete.

## Signed and contextual target harvesting

NativeFrameConfig::confirm_harvest_successor and ::harvest_targets now preserve
signed successor positions and contextual width/observer behavior. The standalone
harvest path uses the same signed scan and confirmation logic. Prefix arithmetic
and remaining-bit checks wrap in the native signed domain; a successful successor
may end before bit zero. Trials and confirmation suppress position callbacks;
accepted rereads restore them. Callback mutations to shared widths affect the
reread, which is retained even when partial, and determine the next scan position.

Independent native fixtures cover 224 successor cases (60 panics, two negative
confirmations) and 32 contextual harvest cases. All focused host comparisons pass;
actual WASM passes 164 nonpanicking successors plus all contextual cases. Clippy,
format, diff checks and all 485 pinned source hashes pass. Full host/captured-film
validation is tracked in signed-harvest-validation.json; consult its status before
claiming this checkpoint finalized. Signed view orchestration, broader repair and
runtime contracts, and all-data corpus acceptance remain open. NEXT_PHASE.md stays
deferred.

Next signed-view gate: signed-views-v41.json.zlib records 792 direct native
DecodeFrameViewsCurseur calls (94 panics, 698 normal returns, 76 negative final
cursors). It varies class/generic mode, signed starts, negative/default preambles,
view count, prefixes and source size; records, cursor, completed views and current
world view are retained. Native generation passes; Rust comparison is pending.
Audit InferenceViewsOptions, NativeFrameConfig::decode_production_views,
production-frame header/component continuations, and message/control source
checks together. In particular, native placeDisponible checks signed wrapping
position+n <= frameLen; converting a negative cursor to usize before that check
cannot preserve this contract. This is a parity gate, not the deferred refactor.

Final signed/contextual harvest validation: 609 host tests passed, zero failures,
49 ignored (105.17s); Clippy passed (10.21s); actual WASM passed (10.27s build).
Four captured march films match (81.45s release), and six complete film documents
match (104.81s release). Both harvest fixtures regenerate byte-identically;
format/diff checks and 485 pinned hashes pass. See signed-harvest-validation.json.
This completes the harvest checkpoint, not full v41 parity. Signed view
orchestration and broader contracts/corpus acceptance remain open; NEXT_PHASE.md
remains deferred.

## Signed view orchestration

InferenceViewsOptions now retains native i64 view counts, lead skips and packet
preambles. Standalone and contextual inference orchestration retain signed cursors
between views; NativeFrameConfig::decode_inference_views exposes the configured
class/generic traversal. Production entry points accept checked signed starts and
preserve signed prefix/header/component continuation, including negative ends.

A prefix is skipped by native traversal. Its retained raw field therefore uses
available source bits without introducing an extra native read/panic. A completed
delta can end at -32, skip its next prefix to zero, reach End and continue into
control or subsequent generic views. Message/control guards use the native
wrapping position+width check; signed public reader variants retain the full int64
domain and native panic behavior. In particular an i64::MAX start can read a padded
terminator and finish at i64::MIN. This does not imply source bytes exist there.

Native fixtures cover 792 orchestration entries (94 panics), 120 direct readers
(48 panics), and 16 complete component-to-view continuations with calibrated/stub
widths including -83 and 2^32. Host comparisons, Clippy and actual WASM pass;
three oracle regenerations are byte-identical and 485 pinned hashes match.
Full host/captured validation is tracked in signed-views-validation.json.
March/keyframe research traversal projections and broader runtime/corpus
acceptance remain open. Full v41 parity is incomplete; NEXT_PHASE.md is deferred.

Next march gate: signed-march-entry-v41.json.zlib has 84 native marchRecordsOf
entries, including 14 panics and 16 negative starts that return normally. The
wrapper's signed Remaining >= 8 check and DecodeFrameRecords preamble behavior
must be preserved. Rust walk_march_with_policy still takes an unsigned start,
uses saturating unsigned remaining arithmetic and projects record endpoints to
usize. Native fixture generation passes; Rust comparison/correction is pending.
Also audit keyframe queue measurements and native keyframe table continuation;
bounded replication-tail projections already have explicit in-source guards and
should not be migrated merely because a text search finds native_address there.

The queue-diagnostic gate now has an independent native fixture:
queue-unused-widths-v41.json.zlib contains 210 direct WalkPacketRecords calls,
168 full measurements and 42 wrapped-header panics. Unused negative/large ID
widths survive End-only traversal; negative preambles remain in Variant without
being applied. Rust KeyframeQueueVariant is i32 and walk_keyframe_queue_records
rejects ID widths before reading the marker. Native execution passes; Rust
comparison/correction is pending. Do not infer consumed-ID behavior from this
End-only fixture.

Final signed-view validation: 612 host tests pass, zero failures, 49 ignored
(109.25s); Clippy passes (17.94s); actual WASM passes (22.02s build), including
324 class-mode production entries in addition to both inference APIs and the
direct/continuation cases. Four captured march films pass (83.15s release), and
six complete documents match (105.94s release). Format/diff checks pass. See
signed-views-validation.json. This closes the signed-view checkpoint; the march,
queue diagnostics, remaining source/runtime contracts and full corpus acceptance
are still open. The architecture proposal/refactor remains deferred.

## Signed march and queue wrappers

March entry points now accept checked i64 starts and preserve signed remaining
arithmetic, headers, baseline reads and record continuation. The packet preamble
is applied at view entry, not whenever a record rewinds to zero.
NativeFrameConfig::march_records uses the shared contextual generic reader for
eight views, snapshots the supplied world slots, and restores them on normal
return. Native panics retain preceding mutations, as marchRecordsOf does.

KeyframeQueueVariant uses i64 ID widths and preambles. Queue measurements defer
ID width use until the native header consumes it, preserve negative preambles in
the reported variant, and apply a preamble only when positive. Signed endpoints
remain intact through headers, baseline reads and bodies.

Independent fixtures cover 84 march entries (14 panics), 210 End-only queue
measurements (42 panics), ten complete march continuations and four rollback
cases (two panics). The continuation covers end -32, end 0 followed by Delete,
and end beyond 2^32. The rollback fixture confirms that a preceding Delete is
restored normally but retained after a later native panic. All focused host
comparisons and Clippy pass. Full checks are tracked in
signed-wrappers-validation.json.

The legacy source-only ComponentWidthOverrides map is still unsigned and
pointer-sized; only its represented domain is compared for continuation. The
contextual march API covers every signed fixture row. Unused-ID tests establish
lazy admission, not arbitrary consumed-ID behavior. Native keyframe boundaries,
other nested/runtime contracts and full corpus acceptance remain open. The
architecture phase remains deferred.

Next native keyframe-chain gate: signed-keyframe-chain-v41.json.zlib has 56
native boundary cases. For from=i64::MAX-63, want=i64::MAX, native wrapping
pos+64 admits the header check and returns Header; Rust's saturating addition
currently returns End. Exact-target short circuit and negative-start header
rejection are independently represented. No bodies are entered. Native generation
passes; Rust comparison/correction is pending. Native table fallback and raw
keyframe layout widths still need separate audits.

Final signed-wrapper validation: 616 host tests pass, zero failures, 49 ignored
(110.05s); Clippy passes (17.78s); actual WASM passes (22.74s build). Four captured
march films pass (82.24s release), and six complete documents match (105.24s
release). All four native fixtures regenerate byte-identically; 485 source hashes,
format and diff checks pass. See signed-wrappers-validation.json. This completes
the march/queue checkpoint, not full v41 parity. The signed keyframe-chain
boundary mismatch and broader native table/layout/runtime/corpus gates remain
open; NEXT_PHASE.md remains deferred.

## Contextual keyframe records, chains and tables

Native keyframe-chain guards now use signed wrapping arithmetic and lazy native
header reads. Table traversal preserves signed record ends and its distinct
sentinel fallback: a negative header is a Header stop in a chain, but the table
then attempts a sentinel read and can panic. Prefix/body retention does not
project endpoints to pointer-sized addresses.

NativeFrameConfig now exposes read_keyframe_record, chain_keyframes and
read_keyframe_table with the shared profile maps and observer hooks. Full-state
entry reads the archetype at record+58, then skips its header to the body; retained
header fields do not introduce reads that native never performs. Signed starts
therefore preserve prefix-repaired negative entries and native panic boundaries.
The source-only APIs retain the same traversal fixes.

Independent native fixtures cover 56 chain entries, 24 direct full-state entries
(eight panics), and 30 contextual record/chain/table calls (two panics). A live EMP
hook changes the following calibrated/stub width, including a rewind to -32 and
a skip beyond 2^32. Events and width mutations survive table fallback panics.
All focused host tests pass; full validation is tracked in
keyframe-context-validation.json. Three fixtures regenerate byte-identically;
485 pinned hashes match.

This does not close the raw keyframe layout domain: NativeScanProfile still uses
the bounded KeyframeLayout (header >=64, size word <=32). A separate native layout
fixture is needed to migrate signed header skips and lazily consumed word widths.
Other runtime/source/corpus gates remain open; NEXT_PHASE.md stays deferred.

Next raw keyframe-layout gate: keyframe-layout-domain-v41.json.zlib records 90
native chain/table calls with a no-archetype entry, nine signed header skips and
five size-word widths. Ten table cases panic on negative fallback. Size words
are unused and must not trigger eager validation. Native generation passes;
Rust comparison is pending. Migrate raw NativeScanProfile keyframe scalars to an
int64 representation and adapt the existing bounded layout only at consumers;
keep the separately bounded replication API explicit. Consumed size-word widths
need their own oracle rather than inferring them from this unused-word matrix.

Final contextual keyframe validation: 619 host tests pass, zero failures, 49 ignored
(103.31s); Clippy passes (29.25s including build-lock wait); actual WASM passes
(17.54s build). Four captured march films pass (79.55s release), and six complete
film documents match (102.44s release). Three oracle fixtures regenerate
byte-identically; format/diff checks and 485 pinned hashes pass. See
keyframe-context-validation.json. The signed entry/chain/table context checkpoint
is complete. Raw keyframe layouts, remaining runtime/source contracts and full
corpus acceptance are still open; NEXT_PHASE.md remains deferred.

## Raw signed keyframe layouts

NativeScanProfile now retains signed header and size-word widths through
NativeKeyframeLayout. Contextual consumers skip headers with native signed
arithmetic and consume words lazily, preserving low-32 signed guard semantics
and available discarded source bits. Bounded legacy APIs retain their separate
layout contract. Short-header retention does not introduce native reads.

The native oracle supplies 90 chain/table cases (ten panics), generic End-frame
controls for the same settings, and 194 consumed-word cases (32 panics, 36
positive EMP publications). This supersedes the earlier raw-layout migration
pending statement. Full validation is tracked in keyframe-layout-validation.json.
Positive huge word reads are not executed by this oracle; negative-position
panic cases and generic bit-reader evidence have distinct coverage. Other
runtime/source contracts and complete v41 acceptance remain open. NEXT_PHASE.md
remains deferred.

## Next gate: signed validated resynchronization

The public contextual validated_resync adapter and underlying validated chain
scanner still take/return usize. Native validatedResync uses signed int offsets
and TryDeltaAt applies its wrapping remaining-bit guard before the optional
32-bit skip. A new oracle generator, halo_rust_signed_validated_resync_test.go.txt,
records 120 cases: 43 panics, 72 negative results and five successful landings.
Three positive cases start at negative offsets and recover to zero after an
optional prefix. Native generation passes. Rust migration/comparison is pending;
this is a concrete remaining parity gate, separate from raw keyframe layouts.

Final raw-keyframe-layout validation: 621 host tests pass, zero failures,
49 ignored (116.08s); Clippy passes (50.00s); actual WASM runtime passes
(53.29s build). Four captured march films pass (81.78s release), and all six
complete film documents match (104.73s release). Both included keyframe fixtures
match native generator output byte-for-byte; all 485 pinned hashes, formatting
and diff checks pass. See keyframe-layout-validation.json. Full v41 parity is
still incomplete; signed validated resynchronization is the next concrete gate.

## Signed validated resynchronization

Validated resynchronization now accepts checked signed offsets and returns i64
landings in both the contextual NativeFrameConfig API and source-only chain
scanner. Native wrapping availability checks precede the optional 32-bit skip;
negative source reads retain native panic behavior. The former usize conversion
and unsigned empty-buffer scan bound are removed. Unsupported host values beyond
i64 remain outside the native int64 contract.

The expanded native oracle has 126 cases: 44 panics and six successful landings,
including a returned -32. Tests compare both APIs, signed JSON round trips,
world preservation and capture-hook restoration. Previous statements that this
migration is pending are superseded; final validation is tracked in
signed-validated-resync-validation.json. Other native runtime/source contracts
and complete all-data v41 acceptance remain open; NEXT_PHASE.md is deferred.

## Next concrete contract: native isolation-gap options

Native ScanFilmOptions.IsolationGapMS is signed. DropIsolated first disables
nonpositive values, then multiplies a positive value by 1000 in uint64 wrapping
arithmetic. Rust BipedScanOptions.isolation_gap_us is unsigned and the shared
filter treats zero as disabled. These differ when a positive native millisecond
value wraps to zero microseconds (2^61): native still filters, retaining only
zero-gap neighbors, whereas the current Rust zero setting disables filtering.
The original native option value also cannot be retained through this unsigned
unit conversion alone.

halo_rust_isolation_gap_domain_test.go.txt records 64 native cases covering
negative/zero values, positive overflow, singleton/pair samples, distinct slots,
unsorted timestamps and timestamp wrap. Native execution confirms the zero vs
wrapped-zero distinction. Rust integration/comparison is pending. The documented
chronological-input precondition still applies; unsorted controls record native
behavior, not an endorsement of unordered playback.

Final signed validated-resynchronization validation: 622 host tests pass, zero
failures, 49 ignored (106.02s); three focused tests pass (0.64s); Clippy passes
(20.06s including lock wait), and actual WASM execution passes (18.87s build).
All four captured march films pass (86.06s release) and six full documents match
(113.69s release). The native fixture is byte-identical to generator output;
485 source hashes and formatting/diff checks pass. An initial release build ran
out of disk space during stripping; regenerable crate build output was cleaned,
and a successful rebuild preceded both passing release runs. See
signed-validated-resync-validation.json. Full v41 parity remains incomplete;
the native isolation-gap option is the next concrete correction.

## Native isolation-gap preservation

BipedScanOptions now retains optional native_isolation_gap_ms as i64. When set,
it overrides the convenience microsecond value. Nonpositive native values disable
isolation; positive values convert with wrapping u64 multiplication, preserving
an active zero threshold. Both world-position and quantized-source filtering use
the resulting Option threshold. Legacy serialized options without the new field
retain their behavior, and the raw native value survives serialization.

The 64-case oracle compares world-stream filtering and raw-option conversion;
WASM runs the option/JSON cases. Existing source scans exercise the shared filter
integration. Validation is tracked in isolation-gap-validation.json; earlier
pending implementation statements are superseded, but full parity remains open.

Final native isolation-gap validation: 624 host tests pass, zero failures,
49 ignored (104.55s); three focused tests pass (1.71s); Clippy passes (1m09s
including lock wait), and actual WASM runtime passes (18.18s build), including
64 raw-option conversion/JSON cases. Four captured march films pass (82.76s
release), and all six complete film documents match (109.93s release). Native
fixture bytes, 485 source hashes, formatting and diff checks pass. See
isolation-gap-validation.json. This closes the signed millisecond and active
wrapped-zero threshold correction. Full v41 parity remains incomplete, and the
architecture in NEXT_PHASE.md remains deferred.

## Ordered payload projection across signed rewinds

A native 32-case parent oracle reproduced a payload-selection error: two
components starting at bit 172 could have different precision parameters, but
Rust selected the first observation for both. Component attempts now retain
ordered observation ranges; full keyframe and biped-probe spans additionally
retain ordered field ranges. Payload projections use those ranges instead of
source-bit overlap. Four native round-timer cases exercise partially overlapping
field ranges and distinct recorded quanta. All original source fields remain
retained; the correction changes their association with the requested component.

Range metadata is optional only for older serialized records. Missing keyframe
field ranges or parent observation provenance returns no typed payload rather
than guessing from source position; generic non-observation payloads still use
their existing ordered field ranges. This is current parser fidelity work, not
the deferred architecture. See overlapping-payload-validation.json for validation;
full v41 parity remains incomplete.

## Next gate: component iteration beyond 64 entries

Native traverseComponentLoopFrom visits every registry entry and tests mask bit
(index & 63). Rust generic record traversal, full keyframes and inference-body
trial reads still cap iteration at 64; the isolated biped probe already wraps.
The 36-case native component-mask-wrap fixture confirms, for example, 65 full
keyframe publications and generic bit-zero publications at both indices 0 and
64 in a 65-entry registry. Preserve ordered fields, callbacks and source endpoints
when removing the cap. Rust comparison/correction is pending; native inference
entry coverage must also be added rather than inferred from generic frames.

Final overlapping-payload validation: 625 host tests pass, zero failures,
49 ignored (108.71s); six focused payload tests pass (0.30s); Clippy passes
(36.37s including lock wait), and actual WASM runtime passes (28.04s build),
including all 36 overlap cases. Four captured march films pass (109.91s release),
and all six complete film documents match (156.87s release). The native probe
fixture comparison validates added Rust ranges separately from all native
fields; that final change was test-only. Native fixture bytes, 485 source hashes,
formatting and diff checks pass. See overlapping-payload-validation.json. Full
v41 parity remains incomplete; component-mask wrapping beyond index 63 is the
next confirmed correction. NEXT_PHASE.md remains deferred.

## Native component-mask wrapping

Native generic records, full keyframes and inference-body trials now visit the
entire registry component list. Presence tests use index & 63, matching native
traverseComponentLoopFrom; full keyframes consume all entries. The legacy probe
already wrapped its indices. Ordered ranges and full component indices remain
retained rather than renumbering components above 63.

The 36-case native oracle covers lengths 0, 1, 63, 64, 65, 66, 127, 128 and 129,
with full keyframes and three generic dense masks. It now also records 27 direct
body trials and recursive inference outcomes. Rust compares selected indices,
source starts, retained byte values, callback order, endpoints and inference
results. See component-mask-wrap-validation.json for validation; the earlier
pending implementation statement is superseded, not the broader parity gates.

Final component-mask-wrap validation: 626 host tests pass, zero failures,
49 ignored (113.96s); the focused 36-case matrix and 27 inference comparisons
pass (0.03s). Clippy passes (16.75s); actual WASM passes (15.59s build), including
all frame/keyframe cases and public chain inference. Four captured march films
pass (84.03s release), and six complete documents match (110.20s release).
Expanded raw validation also passes: all 134,657 recovered keyframe anchors
across 32 films (65.88s), including 189,416 typed payloads, 43,631 death states,
468 padded endpoints, ordered fields and source-bit checks; all 451 sequential
keyframe walks pass (3.50s). Native fixture bytes, 485 pinned hashes, formatting
and diff checks pass. See component-mask-wrap-validation.json. This closes the
native component-list cap correction; full v41 parity is still incomplete and
NEXT_PHASE.md remains deferred.
