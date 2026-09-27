# Complete cached document assembly in native pass order

Pinned replay/build.go is the order authority. Do not copy the control flow of
replay_document_film.rs verbatim: its combat helper assembles vehicles and recovers
vehicle shots early. The native reference attaches vehicles after coverage and
ground items, logs vehicle coverage/heading/ride attribution, then recovers shots.
This is an inspected order difference, not a claim that captured document JSON
currently differs. Runtime/log order is part of the final parity audit.

Native order (all in replay/):

1. ouvrir: initialize document, caller interval/defaults and fallback counter;
   sort input positions and establish clock; empty timeline publishes layers and returns.
2. poserLesPistes: player/identity tracks and movement state reduction.
3. poserLesEquipesEtLeRoster: teams, seats and roster using the completed registry.
4. poserTirsProjectilesEtGrenades: initial shots/orphans, loadouts, projectiles and
   grenades; retain only published player slots under each layer's own rules.
5. poserScoreEtObjectifs: named objective/score passes and their diagnostics.
6. poserEpisodesDEquipement: equipment episodes and associated kills.
7. composerLaCouverture: create the coverage envelope from preceding measurements.
8. poserGrappinEtPoses: map-gated grapple, placements, original-frame player positions,
   powerup census, spawn events, life/death and equipment-change evidence.
9. poserPrisesEtSocles: weapon changes, pickups, two-stream pads, date_pad_pickups.
10. poserArmesAuSolEtVehicules: individual ground items; cached vehicle publication;
    coverage, heading-source and ride logs; THEN cached orphan-shot recovery.
11. poserObjectifsVivants: flags, VIP, skull, bomb carries, free objectives, zones,
    bomb armings and Assault statistics. Preserve separate bomb carry/arming gates.
12. poserLibellesEtInventaire: labels, neutral deaths, inventory and grenade reads.
13. poserCapacitesEtTranslocations then poserImpulsionsEtCharges: establish palette
    first, then publish impulses/charges under their independent scanner gates.
14. clore then poserLesCalquesProduits: accumulate assembly fallbacks and finalize
    coverage/layer availability only after all mutating passes have finished.

## Available building blocks

The cache player bridge returns FilmReplayPlayers without constructing Film or
reading chunks. It still has explicit legacy String/usize domain errors; full
parity requires reconciling those rather than claiming native rejection rules.
Most channel adapters are now present; use FACTS_ASSEMBLY_NEXT.md and individual
FACTS_*_PORT.md files for domains and native comparison scope.

attach_facts_replay_vehicle_shots_to_document is the explicit second shot gate.
It takes an existing ReplayDocument, preserved cache orphans, owner mapping and
clock. It requires no synthetic source events. Missing coverage is a no-op; otherwise
it updates document shots, shot/vehicle coverage and the shots verdict in place.
Its native fixture compares final document fields across 1,024 cases, including
signed owner identities and optional vehicle coverage. Runtime logging is emitted
only when shots are appended and vehicle coverage exists. Current added assertions
compare output fields; they do not independently capture the new log call.

Cached bomb adapters are now available: replay_facts_bomb_held_events,
build_facts_replay_held_object_carry and build_facts_replay_bomb_carries. The latter
shares source recognition/bridge/presence assembly and retains raw carries for
statistics. Its shared wrapper is source-inspected; reducer/filter/publication
expectations are native-fixture checked. Native attachBombCarries logging still
belongs in the document composer. See FACTS_BOMB_INPUTS_PORT.md.

`build_facts_replay_flag_layer` now composes the two named-event passes, native
caller identity precedence and death-instant fallback, cache marker/carry reduction,
and cached return-gauge attachment. It takes cached deaths directly and preserves
raw gamertag bytes without converting them. Its free lives should come from the
cache weapon scan; the full composer still owns FlagReturnZone and runtime logs.
Do not initialize an unresolved caller identity with resolve_statborg_round_identity
on empty inputs: that result is resolved-empty, a distinct authoritative state.

VIP and skull cache assembly helpers are now available. `build_facts_replay_vip_crown`
resolves its own identity from supplied records/death instants after its mode gate.
`build_facts_replay_skull_carries` accepts caller identity, using death-instant
fallback only when unresolved. Both consume cached death identities/times directly,
without converting names. Skull presence is supplied from published tracks and
deduced indices. Native full-composer runtime log attachment remains pending.

Bomb arming and statistics document mutations are now explicit shared passes:
`attach_replay_bomb_armings_to_document` consumes sorted published detonations and
returns its fallback count/coverage/verdict; `attach_replay_bomb_stats_to_document`
consumes raw carry and identified objectives, with arming readability taken from
actual document coverage. The arming gate and family-wide carry/stats gate remain
independent. Source-matched runtime logs are emitted at their mutation boundaries;
the new fixture assertions compare documents/fallbacks, not captured runtime logs.

The explicit cached carry document pass is now available as
`attach_facts_replay_bomb_carries_to_document`. Supply the temporal identity state,
clock and deduced track indices; presence comes from the current document tracks.
Pass its returned raw carry to the statistics pass. It shares the standalone
source/cache reducer and preserves missing-coverage and missing-bridge behavior.

## Entry/capture contract

BuildFromFacts takes matchID/titleSlug explicitly; it restores optional FilmIdentity (decoder/build/registry metadata),
accumulates persisted scan-only fallback reports into the caller's counter, applies
FilmInputs to options and calls the pure position builder. It performs no source
reads, freshness checks or automatic Statborg/Kills application. Those extra file
sections are applied by the separate native application adapter, not this entry.

The eventual caller-options type must preserve explicit catalogs and mode settings.
Do not derive zone_scanned from catalog presence on cache replay, and do not merge
bomb carry recognition with arming recognition. Flag marks/gauge and zone/bomb
readings can use their existing cache adapters/shared native types directly.

Fresh scan capture must happen before assembly fallback accumulation. Layout comes
from the actual FilmContext layout and imposed-layout status; failure means no facts.
Clone optional identity for capture rather than aliasing mutable assembly identity.

This is completion of reference parser parity, not the deferred Film/ResolvedFilm/
playback redesign. Do not expose a partial document builder as complete BuildFromFacts.

Resolved replay DTO boundary: native Track.Points nil/null now uses None, while
an explicit empty array uses Some(empty Vec). The carry fixture no longer
normalizes null to empty and compares native track JSON directly. This closes
that specific distinction, not every document null/empty or JSON fidelity issue.

## Combined live-objective stage

`assemble_facts_replay_live_objectives` now composes the native live-objective
stage over one document: flags, VIP, skull, bomb carry, free objects, zones,
bomb armings and statistics. Cached mode-guard readings override only their
consumer inputs, while caller records, identities, catalogs and recognition gates
remain separate. Free flag evidence is computed lazily after the two named-event
passes and identity resolution, without duplicate named-event observations.
It returns ordered flag/VIP diagnostics, assembly fallback counts and raw bomb
carry evidence. Do not capture its fallbacks as scan-only facts.

This is one document stage, not BuildFromFacts. Non-bomb layer coverage logs and
the concluding equipment log remain to be ported into this stage; emitted runtime
log parity is not established. The new native whole-stage fixture calls
poserObjectifsVivants for 128 synthetic cases and compares document fields. It
uses empty free-object/zone catalogs and tests zero fallback counts; populated
geometry and positive fallback evidence remain covered only by focused fixtures.
Next: complete full cached document options/composition in the native pass order,
including scan-only fallback/FilmIdentity restoration and fresh capture.

## Initial combat stage

`assemble_facts_replay_combat` now composes native pass 4 over cached events and
prepared chronological positions: shots with published-slot filtering, loadouts,
projectiles with raw-to-published links, then grenades using those links. It returns
shot/grenade coverage, optional projectile coverage and original cached shot
orphans. It does not create or update document coverage, and does not attempt
vehicle recovery. Supply the orphans to the later vehicle-shot document pass.

The 1,024-case native fixture calls poserTirsProjectilesEtGrenades. It compares
whole typed documents, all returned measurements and cache projections of native
orphans, including signed player indices, absent coverage, deliberately preserved
existing coverage, truncated projectile tracks and grenade links. The fixture uses
explicit empty point arrays for its synthetic published player tracks; it does not
close the separate null-points DTO gap. Runtime warnings/truncation logs are
source-matched, not independently captured by these output comparisons.

## Score/objective attachment stage

`attach_replay_score_and_objectives_to_document` implements native pass 5:
measure team coverage from the current roster and preceding track counts, choose
origin/frames from the current document and interval from the assembler, replace
objective actions, then replace the score timeline. Missing score input clears a
seeded timeline and returns absent score coverage; supplied empty input retains
its distinct coverage. Actions without published tracks stay published. Existing
coverage is untouched; return values feed the later coverage envelope.

The report retains round-zero fallback counts and the native ordered diagnostics,
including team measurement, missing origin, objective loss, inner score warnings,
score coverage and round-boundary logs. Diagnostic records are returned to the
caller; connecting them to the final composer's runtime sink is still required.

The 1,536-case native generator calls `poserScoreEtObjectifs` directly with seeded
documents, caller interval distinct from the document field, absent/present origin,
empty/absent/multi-round score inputs, team contradictions/refusals, objective
roster refusals, named/unidentified events and missing published tracks. Whole
mutated documents, all coverage outputs, missing-track counts, fallback counts and
ordered log level/message/attribute arrays are compared. The stage fixture supplies
no death instants; round-identity death evidence remains tested by the underlying
score reducer fixtures. Track points are explicit empty arrays, not evidence that
the native null-points DTO gap is closed. This is not full BuildFromFacts.

Next native stage: equipment episodes/movement and equipment kill attribution,
then coverage creation. Preserve cached stance byte strings while reconciling the
legacy document String boundary; do not silently replace invalid UTF-8 or claim
native domain rejection. Continue with complete caller options, native pass order,
FilmIdentity/scan-only fallback restoration and fresh capture before fallbacks.

## Equipment/movement stage

`assemble_facts_replay_equipment` implements native pass 6. It derives death-closed
track indices from the current document and temporal identity registry, constructs
camo/overshield episodes and movement stances, then credits resolved kills and
assists through the occupant at the kill frame. It replaces the two published
layers while preserving any existing coverage envelope. The report returns death
boundaries, movement coverage, kill-read status, non-binary camo count and ordered
native diagnostic records for the eventual composer logging sink.

Movement kind bytes now survive into `ReplayDocument.content.stances` and
`ReplayCoverage.stances.by_kind` through `ReplayByteString`. The source-based
constructor converts its UTF-8 strings losslessly to bytes. JSON still follows
native invalid-byte replacement, so raw identities and their ordering must be
inspected before JSON serialization; distinct invalid byte keys can produce the
same JSON key. This is a native-domain parity repair, not the deferred Film/
ResolvedFilm/playback architecture.

`build_replay_equipment_coverage` is the native late measurement over the current
published episodes, track windows and death boundaries. Unknown families are
ignored; unmatched episodes of a supported family remain counted with a fallback
life key (slot, -1). The standalone episode builder shares this calculation, and
the full composer must invoke it when creating coverage, after kill attribution.

The 1,024-case native fixture calls poserEpisodesDEquipement and compares whole
document JSON, raw ordered movement identities, raw coverage keys, signed movement
counters, death-closed indices, kill-read gates and native ordered diagnostics.
It includes native invalid UTF-8 kinds and independently checks late equipment
coverage with unknown families and orphan supported episodes. Track point arrays
are explicitly empty; the separate null-points DTO gap remains open. This is a
synthetic whole-stage oracle, not captured full-film composition evidence.

Next: create the native coverage envelope and continue stages 8 onward. Full
BuildFromFacts, caller option precedence, raw string/signed counter/null audits,
runtime logging sink, metadata/fallback restoration and fresh capture remain open.

## Coverage-envelope stage

`assemble_replay_document_coverage` implements native pass 7, replacing the old
coverage envelope with measured shot/grenade/objective, registry bridge and score
coverage before attaching projectile, track, team, stance and seat measurements.
It always publishes decoder provenance, even when FilmIdentity is absent; major
version and death-path coverage retain their independent optional presence.
Remaining identity counts are applied after the bridge verdict, and equipment
coverage is recomputed from the current document and death-closed track indices.
The equipment kill-read flag does not gate the presence of death-path coverage.

Kickoff detection uses the assembler interval and current published tracks. With
an origin it replaces the kickoff timestamp and its coverage; without an origin
it leaves a seeded document timestamp untouched and publishes no kickoff measure
in the newly created coverage. This intentionally differs from the standalone
`ReplayDocument::detect_kickoff` convenience method, which clears old timestamps.
The coverage stage returns ordered native bridge-alignment and kickoff-refusal
diagnostics for the eventual composer sink. `ReplayCoverage::new` retains its
existing runtime warning behavior; the stage uses the shared pure base constructor
to avoid emitting those same warnings twice through its future sink.

The 1,024-case native fixture calls composerLaCouverture directly. It compares
whole typed documents and native diagnostic order across optional inputs, seeded
coverage/T0 replacement, bridge collisions/calibration, positive and refused T0,
decoder metadata, equipment counts and independent death-path/read gates. Maximum
signed origins, intervals and frame values exercise native wrapping arithmetic in
the shared kickoff helper; frame differences, interval products, burst offsets and
final origin addition now preserve the native integer domain instead of panicking
in debug builds. Source hash verification remains pinned to 485 production files.

This is a synthetic whole-stage comparison, not full BuildFromFacts or a captured
film composition audit. Other legacy counter/string/null domains remain open.
Next pass: poserGrappinEtPoses in replay/build_calques.go, using the existing cached
grapple and placement evidence adapters, followed by pickups/pads/dating, ground
items/vehicles/recovery, remaining labels/inventory and abilities/impulses/charges.

## Grapple/placement stage

`assemble_facts_replay_grapple_and_placements` implements native pass 8 after the
coverage envelope exists. With map bounds it replaces grapple lines and coverage;
without bounds it preserves both seeded fields and only diagnoses available reads.
Equipment placements and their coverage are always replaced. Their frame limit
comes from the current document; origin and step come from the assembler. Caller
census, undecimated chronological positions, spawn/change evidence, registry lives
and catalog inputs remain explicit. No source read or identity inference is added.

The coverage envelope is a required precondition, as in the native pass. The report
returns manifest fallback counts and ordered native grapple/placement diagnostics
for the future composer sink. Those fallbacks belong to assembly, not fresh facts
capture. Unrelated coverage fields remain unchanged.

A 1,024-case fixture calls poserGrappinEtPoses directly and compares whole typed
documents, fallback counts and ordered diagnostic fields. It includes seeded
fields, absent map bounds, zero clock step, negative frame counts, full-width
placement life generations/times, undecimated owner/heading evidence, death/taken/
spawn origin evidence and census endings. The whole-stage family catalog contains
valid UTF-8 names; the broader raw-string and unsigned-counter audit remains open.
The synthetic tracks use explicit point arrays and do not close null-points DTO
fidelity or captured full-film composition. Runtime sink wiring remains pending.

Next native pass: poserPrisesEtSocles, including weapon changes, native pickups,
the pickup-origin judge, two-stream pad publication and date_pad_pickups. Preserve
its native dependency on already-published equipment placements.

## Pickup/pad stage

`assemble_facts_replay_pickups_and_pads` implements native pass 9: weapon changes,
origin/identity-resolved native pickups, weapon and power-up pads, then occupation
dating. The origin judge consumes the original position order and current
published equipment placements; pad assembly consumes a separate chronological
position slice. Equal-distance origin candidates must not be silently reordered.
The two cached scan gates and their signed source counters remain independent.

The stage requires the previously created coverage envelope and preserves its
unrelated fields. It returns the unchanged weapon objects for ground-item assembly
and native diagnostic records for the future runtime sink. The native pass emits
no fallback hits in the comparison; none of the consumed reducers uses the
fallback counter. Native pad coverage retains its own original traversal counts;
the separate pad-dating coverage describes the later native-event join.

The 1,024-case fixture calls poserPrisesEtSocles directly with seeded documents,
original-versus-sorted position order, recycled temporal identities and slot
bridges, map and dropped-placement origin evidence, independently gated scans,
wide signed counters, full-width clocks and unknown raw weapon-change kinds.
Native event inputs are also constructed around reference-produced pad windows to
exercise unique and ambiguous joins. Expectations come from the pinned native
whole-stage call, not Rust. It compares whole typed documents, retained weapon
objects, ordered diagnostics and the empty native fallback report. Original pad
bounds remain published when events cannot uniquely date them.

This is synthetic whole-stage evidence. General family/state/point-kind strings
still use the legacy UTF-8 API, and the broader raw-string/counter/null audit and
captured full-film composition remain open. Runtime logging sink wiring is pending.
Next: native pass 10, ground items then vehicles and their logs, followed by the
second vehicle-shot recovery gate using preserved initial combat orphans.

## Ground/vehicle stage

`assemble_facts_replay_ground_and_vehicles` implements native pass 10. It consumes
weapon objects retained by the pickup/pad pass and the undecimated chronological
position slice, replaces ground items and their coverage, then publishes cached
vehicle tracks/cycles and their coverage. It emits ground, vehicle death,
vehicle coverage, heading-source and ride-attribution diagnostics in native order.
Only after publication does it attempt the second shot gate with the initial
combat pass's original cached orphans. Recovery updates the existing shot coverage
and verdict. The stage requires coverage and takes frame count from the document.
A zero interval clears old vehicle publication and cannot recover shots.

The stage returns assembly fallback counts for the eventual composer. It does not
restore scan-only fallbacks, read source data or construct a full BuildFromFacts.
Unlike stages that currently return diagnostic records, this stage emits through
the existing runtime tracing sink; full composer integration must preserve that
ordering and avoid duplicate emission.

The native generator calls `poserArmesAuSolEtVehicules` for 1,024 cases using the
cached vehicle-publication generator's scan construction and the independently
native-generated `facts-ground-items-v41.json.zlib` input corpus. To regenerate,
inflate that corpus to `/private/tmp/halo-facts-ground-items.json`, copy the new Go
generator into the pinned replay package and run
`TestHaloRustFactsGroundVehicleDocument`. The whole-stage expectations come from
the pinned Go parser, including ordered runtime logs and fallback counts.

The fixture contains 8,860 ground items, 848 ground pickup links, 3,315 vehicles,
five respawn cycles and 1,446 recovered shots. It also exercises 256 zero-interval
cases, 147 unscanned vehicle cases, 84 unpublished recovered shots, 898 ambiguous
shot attempts and 9,220 out-of-episode attempts. Existing ground, vehicle and cycle
fields are seeded to check replacement; unrelated verdict data is preserved.
All 7,389 native runtime logs are compared. This exposed the missing vehicle-death
attribution log: the shared death assignment reducer now emits it after assignment,
using WARN for unmatched deaths and preserving the native early-return silence.

This remains synthetic stage evidence. Track points are explicit arrays; the
separate null-points DTO gap is not closed. General catalog and ground-object
strings retain the legacy UTF-8 API, and this generator's imported ground changes
have passed through native JSON. It is not evidence of raw invalid-byte retention
at that boundary. Full captured-film composition, legacy domain audits and runtime
sink integration across all other stages remain open.

Next: finish remaining live-objective logs, then native pass 12
`poserLibellesEtInventaire`. Weapon labels replace the table; kill effects replace
only for a nonempty caller table. Neutral deaths use the purified identity bridge.
Inventory coverage is guarded by input presence, while grenade coverage is guarded
by built reads; a skipped coverage attachment preserves seeded coverage. Death
attribution follows inventory publication and coverage, before grenade publication.
Do not substitute the standalone source-based inventory wrapper for that order.

## Label/inventory stage

`assemble_facts_replay_labels_and_inventory` implements native pass 12 over caller
labels and cached keyframe/delta inventory. It replaces weapon labels from current
loadouts/shots/pads, copies a nonempty kill-effect table, filters neutral deaths
through current tracks and the purified identity bridge, then publishes inventory.
Inventory coverage attaches only for supplied input and existing document coverage.
Missing input is `None`; a supplied empty slice is `Some(&[])`. Neither a missing
coverage envelope nor a skipped attachment creates or clears coverage.

After inventory publication and its coverage log, the stage marks unknown empty
reads using temporal identity and offset death times, then logs the empty-read
measurement. Grenade reads are independently built from both channels and filtered
against current tracks. Their coverage attaches only when reads were built, even
if all were later filtered. Empty built input preserves seeded grenade coverage.
The function returns the count of death-corroborated empty readings. It emits its
native diagnostics directly through tracing, rather than returning duplicate log
records. The eventual composer supplies the already-defaulted nonzero interval.

The 1,024-case native generator calls `poserLibellesEtInventaire` directly and
compares the entire document and 3,416 ordered runtime logs. It includes 12,976
published inventory reads, 11,322 grenade reads, 766 neutral deaths, and 411 empty
inventory reads marked dead. Inputs cover nil versus supplied empty inventory,
missing/seeded coverage, empty/nonempty kill-effect tables, selection-only deltas,
present empty grenade lists, recycled or ambiguous identity, signed full-width
rank/counter values, and maximum unsigned timestamps. The imported catalog is
valid UTF-8; raw catalog string fidelity remains part of the broader domain audit.
Track point arrays are explicit, so this does not close native null-point fidelity.
This stage fixture is not full BuildFromFacts or an independently annotated film.

Next document stages: `poserCapacitesEtTranslocations`, then
`poserImpulsionsEtCharges`. Keep raw rank readings for life joins and classify the
palette from published noise-filtered reads before naming actions. Grenade labels
are gated by inventory OR grenade throws OR grenade reads. Action scan gates are
independent; an unscanned input warns without replacing seeded coverage. Then
finish closing coverage/layer publication, remaining live-objective logs and the
full cached composer, preserving entry metadata and scan-only fallback semantics.

## Ability palette and action stages

`assemble_facts_replay_abilities_and_translocations` implements native pass 13:
grenade labels are replaced only when inventory, throws or grenade reads refer to
them; ability reads are noise-filtered before the published-slot filter; equipment
changes and translocations replace their layers and coverage. The selected palette
and its labels come from the published ability reads. The function returns that
palette for `assemble_facts_replay_ability_actions`, which implements the native
impulse-before-charge order using raw rank readings and registry lives.

Both action layers are always replaced. Each scan gate independently decides
whether to replace its coverage or warn and preserve old coverage. Unscanned does
not suppress the builder's output if callers nevertheless supplied readable data;
this follows the pinned native document stage. Both stages require the previously
created coverage envelope. The palette stage takes the assembler's already
validated nonzero interval; the action stage retains the reducers' zero-step gate.
They emit runtime tracing observations at native publication boundaries.

The native generator invokes both `poserCapacitesEtTranslocations` and
`poserImpulsionsEtCharges`, recording the intermediate document and palette as
well as the final document and combined ordered logs. Controlled cases supply a
single known palette, one life and matching rank/action times to exercise positive
publication; the remaining cases cover absent and ambiguous palette selection,
noise filtering, independently unscanned channels and signed full-width counters.
Equipment change kinds include invalid UTF-8 and are transferred as hex; palette
labels and family names remain valid UTF-8, so the general string audit stays open.
The generator explicitly supplies track point arrays, not native null points.
This remains synthetic native parity evidence, not observed golden-film actions.

After these stages, finish closing coverage and produced-layer publication,
remaining live-objective logs, then connect the full cached composer and its caller
options. Restore optional FilmIdentity and persisted scan-only fallback counts at
entry; fresh capture must precede assembly fallbacks. The deferred Film/ResolvedFilm/
playback redesign still begins only after current v41 parity is complete.

The combined ability fixture contains 1,024 cases, 264 classified palettes,
7,503 published ability reads, 2,078 equipment changes, 1,285 translocations,
256 impulses and 256 charge readings. It compares 6,144 runtime observations and
covers 342 unscanned impulse inputs, 256 unscanned charge inputs and 799 cases
with invalid-byte equipment-change kinds. Both intermediate and final document
states are asserted; positive publication totals prevent a rejection-only fixture
from silently replacing this evidence.

## Final coverage and layer publication

`finalize_replay_document` implements `clore` followed by
`poserLesCalquesProduits`. It logs the current document's shot counts and verdicts,
not stale measurements from before vehicle-shot recovery. Grenade counts come
from the assembler's earlier grenade measurement, matching the native split.
The fallback counter's sorted positive report then replaces seeded fallback
coverage, and produced-layer revisions are published last. Empty-position entry
still skips finalization and publishes only its produced layers.

`attach_replay_fallback_coverage` exposes the native independent attachment gate:
missing coverage is a no-op, while a missing counter clears a seeded report.
`ReplayFallbackHit.hits` now uses i64 on host and WASM. The convenience fallback
aggregator uses signed wrapping accumulation and removes nonpositive totals,
matching the native counter; it no longer narrows scan fallback counts through
usize when publishing the source-based document. This is a native-domain repair,
not the deferred canonical Film architecture.

The finalization generator exercises 1,024 complete documents with seeded reports
and layers, independently differing current/stale shot counts and document/local
grenade counts, missing verdict keys, missing coverage/counters, signed full-width
fallback increments and overflow. It records whole final documents and runtime
logs; the existing exhaustive produced-layer fixture covers the full guard table.
Unknown-name trigger observations belong to the counter's input boundary and are
not duplicated during final publication. General raw-string and other legacy
counter/null audits remain open; finalization alone is not BuildFromFacts.

## Cache entry provenance and native byte domains

`restore_facts_replay_provenance` is the explicit provenance step before cached
assembly: accumulate persisted scan-only fallback increments into the caller's
counter and restore `file.identity` as the optional decoder identity projection.
The full entry must create a counter when the caller did not supply one. This
helper does not read old header coverage as current decoder provenance, apply
Statborg/Kills sections, read source bytes, or claim to be complete BuildFromFacts.
The full facts file retains all native identity fields beyond the decoder subset.

`ReplayDecoderIdentity` now retains raw build bytes and signed i64 format/registry
counts. `ReplayRegistryCoverage` publishes i64 counts on host and WASM. Known-build
lookup uses exact known ASCII keys; invalid build bytes cannot match those keys,
but remain present in the identity. Registry classification still uses the raw
build's match status even when the build is not published.

`FallbackTrigger.name`, `ReplayFallbackHit.name`, and `fallback_report_text` now
use `ReplayByteString`. Counter lookup, aggregation, report ordering and text
rendering preserve bytewise identity. JSON/log display applies native replacement
only at that boundary, retaining distinct invalid names even when their rendered
strings collide. Counts remain signed/wrapping as in the preceding repair. Unknown
positive increments now emit the native ERROR log before accumulation, while their
raw diagnostic records remain available through `unknown_triggers`.

The 1,024-case native fixture uses BuildFromFacts' empty-position branch to inspect
entry counter side effects without assembly increments, compares native decoder
coverage separately, and compares fallback publication and raw report text. It
contains 6,109 entry diagnostics, 3,332 published fallback entries/logs, 1,907 counts
above u32 and 147 absent identities. Direct in-memory native inputs include invalid
build/fallback bytes; they are transferred as hex, independently of JSON display.
Four known zero-valued bounds fields normalize only `0` versus `0.0` spelling in
the comparison; integer counts, strings, unknown byte identities, ordering and
presence are not relaxed. This does not establish full nonempty BuildFromFacts,
fresh capture timing, remaining caller precedence or the broader domain audit.

## Live-objective publication diagnostics

The cache stage now emits the native flag, skull, free-object, zone and concluding
active-equipment publication logs through tracing. Flag overlap and uncorrelated
return-gauge warnings, zone pairing/owner warnings, and nil-coverage silence are
implemented in `facts_live_objective_logs.rs`. VIP attachment has no separate
coverage log in the pinned source. The concluding equipment observation remains
guarded for the standalone Rust stage's missing-coverage case; native full
assembly guarantees an equipment coverage envelope before this stage.

The existing 128-case native whole-stage generator now retains its complete runtime
log stream. The Rust comparison checks all publication and bomb observations in
order, with five explicit exclusions: the three returned Statborg diagnostic
messages, the unresolved flag-opening diagnostic, and the no-zone-catalog reducer
warning. Those exclusions are remaining work, not accepted full runtime parity.
The existing whole-stage inputs do not exercise flag closed-overlap warnings or
nonzero zone pairing/owner warnings; those branches remain source-inspected only.
Do not claim the live-objective sink integration is complete until the exclusions
are removed and their native ordering is checked. Full BuildFromFacts composition,
fresh capture and the broader string/counter/null audit remain pending.

The completed sink integration removes those five exclusions. Retained flag diagnostics
must emit before the flag reducer (which can itself diagnose unresolved opening
slots); VIP diagnostics emit before its attachment. Returned diagnostic records
remain inspection data and must not be emitted again by a composer. The no-zone
catalog warning also applies to a nonpositive frame count or zero clock step,
provided the zone scan gate is enabled. These are the native reducer's gates.

`replay_diagnostic_sink.rs` provides the internal structured tracing bridge for
other returned stage diagnostics. It interns only parser-defined ordered field
schemas and severity, not messages or values; tracing requires static callsite
lifetimes. Each dispatch checks the current subscriber outside the schema lock.
Primitive fields retain their types; compound values use JSON display at the
tracing boundary while the retained diagnostic keeps its structured JSON value.


Full live-objective runtime comparison now passes on host and WASM: all 2,872
observations from 128 native stage calls, with no exclusions. Messages, levels,
fields and event order are checked alongside whole documents and fallback counts.
This supersedes the earlier five-message exclusion limitation. The fixture has
one statborg slot, so it does not establish an ordering for native map iteration
across several slots. The compound-field capture compares JSON display, and the
floating `cv` field compares the visitor's Debug representation of the native
numeric value. The deferred architecture is unchanged.

Next: implement the complete cached caller Options and document composer, using
all stage helpers in native order. Earlier stages returning diagnostic records can
use `replay_diagnostic_sink`; emit each at its native boundary, without duplicating
stages (including live objectives) that already emit. Keep scan-only fallbacks and
fresh capture separate from assembly increments. The complete legacy-domain and
captured-film audits are still required before declaring parity.

## Cached document composer

`build_facts_replay_document` now connects all fourteen native assembly stages over
a decoded `NativeFilmFactsFile`. Caller-only settings live in
`FactsReplayDocumentOptions`; channel data and nested flag/zone/bomb readings come
from the cache, so restoring them cannot overwrite caller catalogs or mode flags.
The entry restores optional decoder identity and scan-only fallback increments,
initializes caller geometry/structure, preserves the empty-position return, builds
players, and runs the remaining stages in the pinned `build.go` order. Vehicles
precede orphan-shot recovery; palette selection precedes impulse/charge naming;
fallback coverage and produced-layer revisions publish last.

The cached player route accepts the caller's Statborg identity explicitly. It does
not invoke the convenience API's identity completion. That legacy convenience API
keeps its existing behavior. Cached inventory is supplied even when empty: native
`filmfacts_decode.go` initializes a non-nil empty slice. The cache DTO does not
represent an in-memory pre-encoding nil inventory, and this distinction remains
part of the broader native-domain audit.

The existing `facts-players` fixture contains complete native documents; the new
composer comparison uses the whole typed document rather than only its former
player projection. `halo_rust_facts_document_test.go.txt` extends those same source
inputs with fire, loadout, grenade, inventory, ability and weapon-change channels,
caller identity variants, score presence, independent mode/scan gates, explicit
label tables, and a caller fallback counter. The file's extra Statborg section is
populated independently of caller score presence and is deliberately not applied.
The native generator stores its complete logs for the next runtime-order audit.

Current limits: typed document comparisons do not close unknown JSON-field/null
DTO fidelity; player projections still reject unsupported raw strings and usize
counters; other legacy domains require migration. Caller objective counters now
retain signed i64; see signed-layer-validation.json.
Full-composer runtime log parity has not yet been asserted. In particular, initial
identity/player diagnostics and fallback trigger timing require audit, and earlier
stages returning diagnostics now emit through the sink after each stage. Fresh
scan capture, richer world/vehicle combinations, captured-film comparison and the
complete export/field/error audit remain open. This is parser parity work, not the
deferred canonical Film / resolved replay / playback architecture.

## Composer runtime observations

The cached player route now optionally emits runtime observations at the native
boundaries: composed index-table coverage before owner construction, track refusal
and gap coverage before bounds measurement, rejected bounds after measurement, and
remaining-name coverage before team/roster publication. The existing standalone
player/registry functions keep their behavior through unobserved wrappers.

The populated 128-case composer fixture now compares all 4,274 native observations
in order, with no message exclusions, alongside its complete typed documents.
Primitive fields are compared directly. Compound/null values use JSON display in
the tracing capture, and the floating `cv` field uses its numeric Debug spelling.
The fixture includes six empty timelines, four track-refusal messages, and five
remaining-name messages. It does not exercise table collision/refusal warnings,
rejected bounds, replication-gap messages, or a remaining unnamed-track error;
those new diagnostic branches are currently source-inspected, not oracle-proven.

Still required: audit runtime branches inside owner/scoreboard/roster-elimination
and temporal-exclusion resolution, and compare richer identity/world/vehicle
compositions. This fixture's successful log comparison does not establish every
runtime path or fallback trigger timing. Remaining byte-string/counter/null
migration, fresh scan capture and captured-film validation are unchanged.

## Signed shared layer coverage and caller objective counts

All seven `ReplayLayerCoverage` counters and the caller objective unnamed/refused
counts now retain native signed i64 on host and WASM. Balance checks, objective
availability and late vehicle-shot transfers use native wrapping arithmetic.
Negative refusal counts preserve the native `refused > 0` gate. Actual collection
lengths convert at source publication boundaries.

The pinned native oracle compares coverage balance/verdicts, objective actions,
whole typed cached documents and every ordered runtime message for 1,024 rows
(originally 80 distinct cases; now 620 after adding index counters), including
negative, above-u32 and i64 boundary values.
Host and selected WASM checks pass, as do seven shared combat/vehicle/score/coverage
regressions and Clippy. See signed-layer-validation.json for evidence and limits.
This closes the caller objective counter gap only; player/registry/vehicle and
other legacy counter domains, raw strings, null fidelity, capture and full parity
remain open. No deferred architecture work has begun.

Player-index readings/disagreements now also retain signed i64 through owners,
bridge health and published identity links. The expanded signed-layer fixture
covers all 121 selected counter pairs in complete cached document composition.
Bootstrap-seat and team-scan counters are now signed too; see the following update.

Bootstrap-seat/team-scan migration removes their cache projection errors. Native
signed occupied/vacant values and all eleven team-scan counts survive projection;
identity seats and team records/rejected/divergence coverage preserve them, with
wrapping sums. The expanded signed-layer native fixture retains 29,647 ordered
logs. Options.MinPoints still uses a checked usize conversion in the composer;
that caller-domain limitation is distinct from the now-exact cache counter inputs.

Next concrete caller-domain repair: native Options.MinPoints is signed int and
uses its positive value unchanged, defaulting nonpositive values to one. The
track reducer compares len(points) directly with that threshold. Rust currently
narrows it to usize in facts_document.rs; migrate the option/track threshold and
published threshold together, and test >u32 thresholds on actual WASM. This is a
caller threshold, not a collection index; no allocation of that size is needed.

## Signed caller track threshold

Options.MinPoints no longer narrows through usize. FilmReplayPlayerOptions,
ReplayPlayersInput, publish_replay_tracks and ReplayTrackCoverage retain i64;
the composer defaults nonpositive values to one and the reducer compares the
actual point count as i64. A large threshold does not allocate a large buffer.
FactsProjectionError now only describes the remaining raw-string restriction.

The signed-layer native oracle now varies thirteen caller thresholds, including
9/10/11 around its ten-point tracks and positive values above u32::MAX. Native
outputs publish both tracks at ten, reject both at eleven, and preserve a large
threshold exactly in refusal coverage. Nonpositive values publish threshold one.
The fixture has 1,024 distinct rows and 29,962 ordered logs. Platform validation
is recorded in signed-layer-validation.json; this remains synthetic evidence.

Continue with raw byte-string identity/metadata domains, null distinctions,
fresh scan capture, runtime branches, captured films and the complete parity audit.
The Film / ResolvedFilm / playback architecture remains deferred.

## Raw cached player metadata

Bootstrap build/refusal and team component names now preserve arbitrary bytes
through ReplayByteString, including refusal coverage. JSON/log conversion follows
native replacement while the original bytes remain available. The expanded
signed-layer oracle compares raw projections and all 30,858 ordered native logs
alongside typed documents. See its validation report for platform results.

The remaining cache bridge string errors are Deaths.Gamertag and
FilmTable.Seats.Gamertag. Migrate the full name path together: table name maps,
identity names, track/roster publication and name-to-XUID lookup must preserve
bytewise identity and ordering. Do not normalize names early to fit String keys.

Human death/seat gamertags now retain bytes throughout name maps, identity/roster
publication and kill identity lookup. Raw published names are oracle-compared;
a separate 169-case application lookup oracle exercises colliding JSON spellings.
The cached player bridge no longer rejects raw names. Caller bot identity strings,
other legacy strings/null DTOs, fresh scan capture, runtime branches and captured
whole-film validation remain open. Preserve native byte ordering in those follow-ups.

Next name-domain boundary: IdentityBot.name and IdentitySuccession.bot_name feed
identity_bot_names_by_seat, ReplayTrackIdentity.bot and ReplayTrack.bot. Preserve
bytewise deduplication for shared bot seats and global roster deduplication; avoid
normalizing bot names through format/display in presence or usage keys. Existing
bot and succession fixtures provide valid-string regressions; add native raw-name
cases before claiming that caller domain complete.

Bot name/succession fields now preserve bytes through direct/remaining naming,
rosters, track identity and seat/usage grouping. The 81-case native oracle compares
raw names and presence keys, including JSON-colliding invalid bytes. See the
signed-layer validation report for host/WASM results and scope limits. Further
parity work includes null/empty DTO distinctions, fresh scan capture, other legacy
native domains, runtime branches and independently validated captured films.

Next concrete DTO fidelity repair: ReplayTrack.points is still Vec, although native
Track.Points can be nil and serializes null. replay_bomb_document_tests.rs explicitly
rewrites native null to empty today. Preserve absent/null point collections in the
DTO while treating nil as empty only when iterating in bounds, grapple, kickoff,
zone and objective geometry reducers. Remove the fixture normalization and compare
native document JSON after migration; source-produced nonempty tracks stay arrays.

## Nullable track point collections

ReplayTrack.points now stores Option<Vec<ReplayPoint>>. None serializes null;
Some(empty) serializes an empty array. Source track publication explicitly produces
Some(points). The points() borrowed slice accessor implements native nil-slice
iteration for bounds, grapple, kickoff, flag geometry and zone consumers without
changing the stored option. This is reference DTO fidelity, not the deferred model
architecture or a claim of byte-for-byte film re-encoding.

The bomb-carry comparison removed its legacy null-to-empty normalization and now
compares native track JSON. A 64-case pinned native oracle compares complete track
JSON for nil/empty/one-point/two-point combinations and bounds before/after access.
See signed-layer-validation.json for actual platform results. Remaining null/data
boundaries and captured-film coverage still require audit.

The actual-context snapshot helper capture_film_scan_facts is now implemented and
native-byte-oracle checked; see scan-capture-validation.json and FACTS_ASSEMBLY_NEXT.
Full fresh scanning/conversion and a BuildFromFilmAvecFaits-equivalent entry remain
open. Preserve one shared context and capture before applying assembly fallbacks.
