# Reader context contract audit

Reference: LevelUp `feat/v75` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`, `internal/grammar/lecteur.go`.
This is an audit of the current v41 port, not the deferred architecture proposal.

Current-state note: the sections below retain historical checkpoints. The live
`NativeFilmReader` now has context replacement, shared observers, frame factories
and traversal/accumulation methods; see `READER_CONTRACT.md` and the corresponding
native context fixtures. Statements below that those APIs do not yet exist are
superseded, not additional open implementation items. This does not close all
caller integration, signed-cursor or failure-path contracts.

The signed variable-width codec is now also exposed on the live reader as
`read_signed_variable`. Its native oracle checks 3,784 sequential inputs and
11,352 signed reads, all four selectors, byte alignments, negative results and
zero-padded prefixes, followed by an ordinary primitive read. The targeted test
passes. See `../fixtures/signed-variable-reader-v41.md` for coverage and limits.

## Native contracts and current representation

| Native operation | Contract | Rust mapping / remaining work |
| --- | --- | --- |
| `LecteurSur` | Source bits at zero, invariant profile, zero capture, nil observer | `NativeFilmBits` preserves source-reader semantics; grammar paths use `Cursor` and explicit encoding/capture arguments. These are separate APIs, not a claim of an interchangeable native reader object. |
| Promoted `source.Bits` methods | Reads, skips, seeks and remaining bits retain native padding and signed offsets | `source_bits.rs`; bounded grammar `Cursor` has a deliberately different offset/error contract. |
| `ReadSignedVarWidth` | Two-bit selector; 8/16-bit sign extension; 32-bit conversion; 64-bit low-word truncation | `Cursor::signed_variable` and `NativeFilmReader::read_signed_variable`; the live sequential oracle covers values, endpoints and padded continuation. |
| `PoserProfil` / `Profil` | Replace/return prior profile; inspect current profile; preserve bits, capture and observer | NativeFilmReader::profile / replace_profile copy or replace the live profile while preserving cursor, observer and capture. Native sequence and accumulated-position comparisons cover restoration. |
| `PoserObservation` | Replace/return prior observer pointer, including nil; preserve bits, profile and capture | NativeFilmReader::replace_observer returns the previous shared receiver, including None. The native sequence oracle compares swaps, restoration and detached delivery. |
| `PoserContexte` / `poserCadre` | Replace profile and observer together; preserve bit cursor and capture | Explicit `FrameEncoding` and capture/diagnostic arguments cover individual Rust paths. Integrated caller mapping remains open. |
| `cadre` | Return the current keyframe subprofile | `FrameEncoding.keyframe_layout` and simulation-completion policy, with keyframe native fixtures. |
| `poserMouvement` | Replace movement subprofile only | No production callers found. Rust passes `PositionEncoding` and `PositionCaptureEncoding`; arbitrary reader-local replacement remains unmapped. |

Capture is not part of `ContexteDeLecture`: native context contains only profile
and observer. Capture retains component start, slot, accumulator, accumulator slot
and fallback status. Rust separates these into component source offsets,
`NativePositionCapture`, and internal reader fields. Position accumulation has a
native oracle, but that alone does not establish every context-switch contract.

`FilmContext.NouveauLecteur` installs the film's context, including its observer.
`FilmContext.CadreDeBalayage` supplies its profile in a default frame config and
does not inherit the observer. Preserve this distinction when auditing wrappers.

## Equipment and ground-weapon creation

`equipCreationWalk.contexte()` returns `{Profil: w.prof, Obs: w.obs}`.
`readCreation` calls `PoserContexte(w.contexte())`, then
`PoserObservation(w.obs)`. Both install the same pointer. The second call is not
an override with a different callback sink and needs no Rust replacement mechanism.

Each candidate clears the accumulator and constructs a fresh reader. Its capture
starts at zero; no capture state is inherited from the previous candidate.
`installCreationHooks` installs only equipment-creation and default-state MPP
callbacks. Rust projects the corresponding retained default fields into
`EquipmentCreation`. Default-field callback fixtures and complete creation-record
fixtures test these two boundaries separately.

Ground ammo traversal creates another fresh reader under the same context.
`consumeObjectMultiplayerProperties` reads a frame TLV component; it does not
publish the default-state `MppHook`. Traversing component 9 therefore must not
overwrite creation MPP fields. The ground-creation profile oracle now includes
component 9 before ammo, alongside corruption, simulation and width policies.
It compares complete records and counters, not just ammo quantities.

## Direct-dispatch versus traversal context

The live `navpointRadialWalk.walk` (`navpoint_radial_scan.go`) and
`managedPropertyWalk.walk` (`zone_state_scan.go`) each create a fresh reader per
component, install their full context, and call `consumeByName`. The biped helper
`walkComponentsAt` (`ability_rank.go`) has the same fresh-reader pattern. These
calls do **not** apply traversal calibrated/stub overrides or corruption guards.
Those are a different contract from `traverseComponentLoopFrom` and the
creation/keyframe traversal wrappers. Merely receiving a full profile does not
mean a caller uses every setting in it.

Direct dispatch does inspect `Grammaire.SimStateComplet`: `dispatch_biped.go`
consumes the simulation-state body and then returns this flag as its ported status.
Consequently even an absent-body bit must stop a record when the flag is false.
The native walk retains earlier callback values, drops later ones, increments the
broken count and cannot give the earlier values a chaining witness.

Rust's `consume_component_at` already implements this contract, including the
standalone false default. `objective_scan.rs` already used that entry point.
The navpoint and managed-property delta walks incorrectly used the older
position-only native component API, losing the completion gate. Both now use
`consume_component_at`; they do not acquire traversal override semantics.

Independent native fixtures expanded from 512 to 1,024 cases for each scanner;
the original 512 rows were compared and preserved exactly. The additional cases
use supplied registries with a simulation component between two observed fields,
256 enabled and 256 disabled contexts, bit alignments and truncated tails. These
are context-contract tests, not evidence of this component order in a captured
v41 navpoint/managed-property registry. Each disabled group has 1,024 records,
1,024 earlier readings, zero completed/chained records. Each enabled group has
2,011 readings, 987 completed records and 768 chained records. Before the fix,
both Rust tests failed at row 512: seven readings and three chaining witnesses
versus native four readings and no chaining. Full scan objects are compared,
including counters and blocked-component indices where exposed.

The independent direct-probe fixture additionally covers callbacks and endpoints;
the scanner fixtures validate the caller's acceptance/publication boundary.
The biped scans now also expose complete-context entry points:
`scan_biped_channels_with_context`, `scan_inventory_deltas_with_context`,
`scan_ability_charges_with_context`, `scan_biped_ability_states_with_context`, and
`scan_unit_equipment_with_context`. They share a fresh context-aware component
reader, reject reads beyond the payload, and preserve the existing partial-body
ability publication rule. Legacy position-only APIs retain their prior behavior.
Map-aware Film assembly uses the resolved profile and its explicit completion gate.
The full-context path also retains movement callbacks in its decoded target
components; these callbacks are supplied by the native observer, not derived actions.

`biped-context-v41.json.zlib` / `halo_rust_biped_context_test.go.txt` compare 512
native contexts: 2,156 successful component visits with exact endpoints and 2,724
ordered callbacks. Contexts vary simulation support, ability-anchor bodies, full
precision, levels, bit alignment, and truncation. Zero and random payloads cover
negative stops and positive traversal; anchors rejected by position filtering
remain eligible for these gameplay scans. The shared native walk supplies the
visited sequence, and independent fresh native reads supply endpoints/callbacks.
The public scanner tests check their acceptance gates against that native sequence;
existing scanner fixtures remain the independent evidence for complete output
values. This fixture does not claim callback retention for rejected intermediate
reads or arbitrary observer replacement APIs. Native movement observation is
explicitly installed to match the context dispatcher; omission initially produced
a fixture mismatch at the active-ability callback and was corrected in Go.

Held-weapon and counter-gated equipment-recovery now also expose
`scan_held_weapon_changes_with_context` and `scan_equipment_changes_with_context`.
Map-aware Film assembly supplies the same resolved context to strict ability,
held-weapon and recovery reads. Sparse recovery starts immediately after its mask;
dense recovery validates its i0 region before entering the same component walker.
Legacy public position-only APIs keep their original behavior.

The independent `biped-remaining-context-v41.json.zlib` oracle covers 512 supplied
contexts, including 128 cases in each enabled/disabled by sparse/dense combination.
Native component visits and native held-weapon classification produce the expected
245 held-weapon emissions; `walkEquipRecoveryAt` produces 232 accepted recoveries.
Rust compares every emitted weapon field/counter, recovery counter/rank, and the
public recovery scan's candidate acceptance within a real packet and counter gap.
Wrong-region and truncated records remain rejected. This does not establish the
prevalence of custom component ordering or recovery in captured v41 films.

Fixture ordering matters: native map-width installation sets SimStateComplet=true.
The fixture must override the gate after that call to exercise disabled contexts;
the initial positive-only fixture was corrected and explicit matrix assertions
now prevent that coverage loss. Rust's existing world_precision.rs already matches
this map-installation behavior; no production map-profile change was needed.
Arbitrary reader replacement contracts below remain open.

## Remaining gate

The combined Rust camo/ability scan now preserves channel independence when the
registry omits camo. Its `camo_error` and zero camo counters coexist with ability
emissions and counters, matching the two native public scans in 256 synthetic
packet contexts. The supplied registry/layout/slot setup makes this a channel
contract test; it does not close native setup ordering. The oracle now contains
1,024 cases, preserving its original 256, with requested missing chunks before
or after available data and missing-only scans. The combined channel API now
matches `walkDeltaBipedRecords`: an absent chunk contributes no record counters
or reads. Invalid ranges in externally supplied anchors return `Truncated` rather
than panic (these cannot originate from the native packet-discovery path).
Ability charges, ability state (impulses/grapple), and unit-equipment now apply
the same missing-source rule. A separate 512-case public-scanner oracle validates
all native fields and counters with explicit native profile, registry, layout and
slot inputs. Inventory and held-weapon wrappers now follow the same rule, with
a separate 512-case public-scanner fixture covering all inventory/weapon fields
and counters. Shared setup failures remain to audit, including their ordering
before registry/component absence and whether Film assembly retains other inputs
after a scanner error. Guard changes must account for existing `?` propagation.

Keep `lecteur.go` partial. Finish the production context/capture caller mapping
and decide how to expose the unused native replacement contracts within the
current parity scope. Do not introduce the queued Film/ResolvedFilm/playback
refactor to resolve this audit. Document/export propagation and positive complete
captured CTF, VIP and bomb validation remain separate acceptance gates.

## Equipment-object state scanner

Film assembly retains held-weapon and inventory scan errors independently,
matching the nonfatal branches in native replay `film_scan.go`. Complete Film
construction and serialization are verified for either or both registry-role
failures, with an explicit biped keyframe ensuring the scanners are reached.
The legacy fixed-index schema check now gates only the signature passes
(appearance, spawn/lifetime binding, motion/combat/clock/projectile observations).
An incompatible layout produces a limitation rather than aborting native scans.
Bootstrap registry decoding remains required. Native scanners resolve their own
roles and retain their own errors; no fixed-layout reader runs on the mismatched
schema. A separate captured-record pipeline regression verifies suppression of
otherwise valid legacy signatures while retaining packet and summary outputs.
This does not establish a complete raw-source fidelity contract or eliminate
other constructor error propagation paths.

`equipmentWalk.walk` now maps to `equipment_state.rs`, using a fresh
`consume_component_at` reader for every component through the last wanted field.
The full encoding and simulation policy are passed unchanged; traversal stubs,
calibration skips and corruption guards do not apply to these direct reads.
Only completed samples and their read/gated counts are published. The 512-case
oracle covers enabled/disabled simulation and failed walks. This scanner does
not claim arbitrary observer replacement or preservation of rejected callbacks
as accepted state. Diagnostics-preserving APIs retain native partial statistics alongside errors;
Both map-aware and explicit-encoding Film constructors use those entry points.
The explicit-encoding path previously discarded the partial result on error; a
full-constructor regression now records one ti=37 keyframe slot with the registry
ending before ti=37. It asserts Slots=1, zero records/samples, the missing-archetype
error, continued replication and complete Film serialization, following native
`equipment_state.go::ScanEquipmentState` (slot count precedes archetype lookup).
The initial test envelope had only eight terminator bytes and correctly failed
bootstrap admission as truncated; a full terminating registry block fixes the
test envelope without changing bootstrap parsing.

### Constructor failure reachability audit

Fire and grenade scans currently fail only when `native_chunk_prefix` finds no
readable prefix. `decode.rs` already handles that exact error without aborting;
the remaining error arm is not an independently reachable scanner failure under
these implementations. Equipment changes fail on absent biped archetype or
strict packet indexing; the map constructor has already resolved that archetype
for combined channels and indexed the same immutable chunks. Its present `?`
does not introduce an independent role-missing failure after those steps.
These are source-level reachability findings, not proof of all setup-order
parity: missing-biped handling in combined channels, ability states and charges,
and native-vs-strict packet admission still need a separate audit.


The missing-biped constructor case reaches `scan_movement_states` before
combined channels. Native replay `film_scan_mouvement.go` explicitly resets
reads/stats and continues on this failure. Film now retains a separate
`movement_states_error` and leaves the stream absent, which existing stance
assembly maps to the native empty reads/default stats. Native
`bomb_armings.go::decodeFilmBombReads` similarly catches navpoint scan failures;
Film retains `navpoint_radial_error`, and existing mode-gated bomb assembly uses
empty readings when the stream is absent or `navpoint_radial_error` is present.
The partial stream remains in Film. General native scan availability is
still distinct from publishing a mode-gated replay layer.

Combined biped channels, equipment changes, ability states and charges now have
independent error fields too. Equipment changes report their unavailable channel
input when combined scanning fails; this does not fabricate a successful empty
scan. The full-constructor regression uses a registry ending before ti=12 and
separate chunks with navpoint and biped keyframes. It checks the discovered biped
band, all missing-archetype errors, retained replication/march/position/precision
outputs, and complete Film serialization. The first test put both keyframes in
one chunk, so first-keyframe band discovery skipped the biped; separating chunks
corrected the fixture rather than changing the native band policy. Healthy and
inventory/weapon-only failure controls assert no new unrelated errors.

This closes the constructor failure-retention gates above for these reachable
missing-archetype paths. It does not prove standalone scanner setup/error order
or preservation of all native counters on direct scanner errors. Those remain
separate from replay's deliberate reset-on-error behavior and the remaining
source-admission and reader replacement audits.


### Raw movement errors versus replay reset

`scan_movement_states_with_diagnostics` retains raw counters alongside errors;
its scanner initializes MapWidths before source/archetype checks, as native
`movement_states.go::ScanMovementStates` does. Existing Result-based and observed
APIs remain wrappers over the same scan. Film retains raw failure counters in
`movement_states_error_stats` while keeping `movement_states` absent on failure.
This lets `build_film_replay_stances` continue using native replay's reset-to-zero
inputs without discarding the raw scanner diagnostic. No synthetic action is
published from failure diagnostics.

Direct missing-source and missing-biped tests assert exact counters and stream
serialization; the full map-constructor missing-archetype test checks nonzero
resolved widths survive the Film roundtrip. This closes movement counter loss
for those errors, not general native source selection/error ordering: the Rust
scanner still uses strict packet admission and explicit encoding validation.
Native context-derived default precision and source-prefix traversal remain
separate audit requirements.

The next concrete movement source mismatch is confirmed in live source:
`ScanMovementStates` obtains `fc.ChunkNumbers()`, skips unavailable `ChunkAt`
entries, and uses native packet lists. Rust's current scan builds a BTreeMap
from every supplied chunk and calls strict `packets::index(chunks)`, including
chunks outside the native prefix. Compare packet order, ignored tails and
anticipated binding inputs with a pinned native oracle before changing this;
retain the existing raw failure-stat API and replay reset distinction.


### Movement source selection parity checkpoint

Movement scanning now follows `native_chunk_prefix` and `native_chunk_packets`
instead of sorting all input chunks and using strict packet indexing. Its
anticipated bindings use the new `build_native_anticipated_bindings`, matching
`ConstruireTableAnticipee`: only selected chunks contribute, framing is tolerant,
and an absent prefix returns an empty table. The existing strict binding-table
API is retained for callers whose contracts have not yet been audited. Core Film
admission still uses strict indexing; this change covers the native movement API.

Registered pinned-Go harness `halo_rust_movement_source_test.go.txt` generates
`movement-source-v41.json.zlib`: 128 cases across contiguous/gapped/reordered/
duplicate/negative metadata, short tails, oversized packet lengths and extra
valid delta packets. Rust matches every movement counter, 200 scanned packets,
160 declarations (including identities, archetypes and chunk dates), and all 32
no-prefix errors. These synthetic source cases publish no movement actions;
positive movement data remains a separate captured-film comparison. The harness
injects an explicit native registry/profile but uses real native source metadata,
packet framing, movement scanning and anticipated-table construction.

This supersedes the earlier source-selection mismatch note, not the remaining
context-derived default precision and setup-error ordering audits. Other strict
binding-table callers must be checked against their native call paths before
switching their behavior.

A current call-site search found no remaining internal production callers of
`build_anticipated_bindings`; only its strict captured-table test remains.
Retaining that public API therefore preserves its documented arbitrary-chunk
behavior without leaving another known internal traversal on the wrong source
policy. External consumers, if any, are not covered by this repository search.


### Movement setup matrix

The source oracle now has 384 cases, retaining the first 128 inputs/outputs
unchanged (apart from new case labels and normalized Go map iteration order).
Two additional groups remove the biped archetype or all movement component
names. All groups run the pinned native public scanner with its default profile.
Expected categories distinguish 96 no-source failures, 96 missing-biped failures,
96 successful absent-channel scans, and 96 ordinary successful scans. Rust
compares the error category, every counter, records and anticipated declarations.
This verifies source failure precedes missing archetype and absent-channel
success, including MapWidths retention in all cases. It does not claim parity
for invalid caller-supplied FrameEncoding, which is an explicit Rust input
contract rather than a native FilmContext state represented by these cases.

The separate captured movement comparison has now passed (159.86s), checking all
movement records and counters in the four-film oracle against local recordings.
No production Rust changes were needed for the expanded setup matrix; the
previous source-selection repair already follows the tested native ordering.


### Navpoint partial failure output

Native `ScanNavpointRadial` returns its allocated scan and the slot/keyframe
census even when ti=12 lookup subsequently fails. Rust now exposes
`scan_navpoint_radial_with_diagnostics`; the existing Result API wraps it.
Map-aware Film retains that partial scan plus `navpoint_radial_error`. Setup
failures have no readings, so bomb replay does not gain inferred facts from
retaining the census. The full-constructor missing-archetype regression asserts
one observed slot, one band slot, one keyframe entry, no readings and Film
serialization, replacing the older assertion that the entire scan was absent.

Registered native harness `halo_rust_navpoint_setup_test.go.txt` and fixture
`navpoint-setup-v41.json.zlib` cover 256 source/registry cases: 64 source errors,
96 missing-archetype errors (160 total retained keyframe entries), and 96 normal
scans. Rust compares error category and every native output field. These cases
cover source prefixes, reordered/duplicate metadata and malformed packet tails;
they do not supply a positive bomb recording. Native v41 support boundaries and
explicit invalid precision inputs remain separate from these valid contexts.


### Managed-property partial setup results

Native `zone_state_scan.go::ScanManagedProperties` sets Slots before archetype
lookup and returns the partial scan on error. Rust now mirrors this through
`scan_managed_properties_with_diagnostics`; the original Result API is a wrapper.
Both map-aware and explicit-encoding Film constructors retain the partial scan
and `managed_properties_error`. Zone and flag consumers only use the empty
reading list on this failure, consistent with native `ti13Partage.lire`.

Registered `managed-setup-v41.json.zlib` compares every output field and error
category in 256 pinned-Go cases: 64 source failures, 96 archetype failures with
160 aggregate retained slots, and 96 successful scans. Its explicit native
registry/profile and source metadata cover gaps, duplicates/reordering and
malformed tails. The complete Film test supplies a ti=13 keyframe but no matching
archetype and verifies Slots=1, no reads, both constructors and serialization.
Positive managed field readings remain covered by the existing component/packet
and captured-corpus fixtures; this new oracle tests setup behavior only.


### Reader replacement/restoration sequence evidence

[READER_CONTRACT.md](READER_CONTRACT.md) records the current ownership mapping
and a new native sequence oracle. In 256 cases (1536 reads), native readers swap
profiles/observer receivers, temporarily neutralize capture, restore the previous
values, restore callbacks, and install a whole context. Rust's explicit per-read
contexts match endpoints, rejection status and every ordered callback/action;
restored contexts reproduce earlier results and suppression preserves raw reads.
The 34 rejected reads retain 34 callbacks, verified through serialization.

This covers four component families and selected body/movement profile fields.
It does not introduce or claim a stateful Rust equivalent of the native mutable
reader API, arbitrary aliasing, capture-state preservation through context swaps,
or rejected intermediate callbacks in every integrated scanner. Those broader
contracts remain open. The focused comparison and Clippy passed; production
parser code was unchanged, so the already-passing full suite was not repeated.


### Rejected biped channel attempts

The native walkComponentsAt invokes consumeByName before checking ported status
and payload bounds. Observer callbacks therefore survive a failed component even
though the visit callback is not called. Rust's shared helper previously discarded
that complete failed attempt. It now has an internal observation path that sees
raw dispatch status and bounds separately, before the existing acceptance gate.
BipedChannels.rejected_components retains the failed component index, status,
in_bounds, packet/record/slot source, raw fields, references and diagnostics.
Failed components do not become successful camo/ability publications.

The existing biped-context fixture gained native walk callbacks and all attempts;
its previous inputs, successful visits, values and outputs were compared byte-for-
byte as parsed JSON and remain unchanged. Across 512 contexts, Rust matches 2408
attempts, 2156 successful visits and 2771 ordered callbacks. There are 252 failed
attempts with 47 callbacks in the complete traversal. The camo/ability scanner
stops at index 48, so its retained subset is 235 failed attempts with 29 callbacks.
Those source identities and complete channel serialization are checked as well.
The focused test, full suite (464 passed, 42 ignored), Clippy and WASM checks
passed. A fresh focused captured gameplay comparison remains running.

This closes rejected-attempt retention for BipedChannels. Other consumers of the
success-only traversal helper (inventory, energy and equipment, among others)
still require their own retention audit. Successfully visited intermediate
components remain distinct from a scanner's published target values; no claim of
complete Film structural fidelity follows from this one output extension.


### Inventory, charge and unit-equipment rejection retention

The shared retaining walk now supplies rejected_components on InventoryDeltaStream,
AbilityChargeStream and UnitEquipmentStream as well as BipedChannels. Source,
record/slot, optional native packet ordinal, raw dispatch status, bounds, fields,
references and diagnostics survive a failed intermediate or target component.
Accepted visits still use the same native status/bounds gate and existing
publication logic. The source attribution/retention code is shared, avoiding a
separate rejection implementation per scanner.

The same 512 native contexts compare each scanner against the native attempt
sequence truncated at its own last target. Inventory (i22 in this fixture):
224 rejected attempts/4 callbacks. Unit equipment (i26): 228/22. Charges (i56):
236/30. Every source identity, status, endpoint and ordered callback matches;
complete output serialization is checked. Focused test, Clippy and WASM passed.
Full suite passed (464 tests, 42 ignored); the fresh captured comparison
remains running at this checkpoint.

The prior biped-only rejection change passed its four-film captured comparison
(94.68s): every existing published inventory/channel field and counter still
matched. That older run is not validation of these newer scanner sidecars.
Held-weapon changes still use the success-only record helper. Ability-state and
equipment-recovery walks have custom partial-body/offset-only paths and need
separate auditing; do not imply all failed attempts are now retained everywhere.


### Held-weapon/ability rejected reads and native publication gates

HeldWeaponChangeStream and BipedAbilityStates now retain rejected component
attempts with raw status, separate source-bounds result, source/record/slot,
optional native packet ordinal, fields, references and callback diagnostics.
The weapon oracle adds 267 rejected attempts and 11 callbacks over 512 contexts;
all pre-existing fixture fields are unchanged.

The expanded ability scan exposed two publication differences. A disabled
anchor body can dispatch successfully without coordinates; publishing on dispatch
success caused a panic. Grapple publication now uses the native typed hook's
BodyOK and recognized inner value (1 or 2), preserving its position quanta.
Conversely a hook can publish before the walker rejects an out-of-bounds read.
Ability scans now use actual hook presence independently of walker acceptance;
these reads retain complete=false and a rejected-component sidecar. A failed
read without the hook does not fabricate a published state.

The pinned native grappleScanner and abilityImpulseScanner now run directly
in all 512 biped-context cases. Rust compares complete scanner statistics and
ordered publications: 275 grapple reads, 24 tag-3/broken bodies (11 with the
anchor body disabled), 551 impulse reads and 71 impulse publications. Existing
4096 ability-body cases provide positive anchor coverage. The native fixture
retains its original inputs and all earlier attempt/callback expectations.
The focused regression passes; full-suite/captured validation is tracked in
PORT_STATUS.md. This does not establish complete film fidelity.

Equipment recovery remains a separate audit: native walkEquipRecoveryAt uses
hooks before checking whether i48 was reached, but only publishes a recovery
candidate when both reached and last.got hold. Rust currently exposes accepted
counter/rank candidates and drops speculative intermediate diagnostics. Future
retention must identify speculative offsets and must not turn rejected candidates
into recorded equipment facts. Integrated caller/capture and facade audits remain
open; the three-layer redesign is deferred.


### Speculative equipment-recovery component retention

The offset-only recovery walk now uses the shared observed component walker.
EquipmentChangeStream.recovery_attempts preserves every component-consuming
probe, including successful intermediate reads and rejected targets. Each
probe retains source, native packet ordinal when known, slot, candidate header
bit offset and candidate flag; each component retains index, raw status,
independent bounds, full decoded component and callback diagnostics.
Candidate means only the native component walk produced counter/rank; later
window/counter validation can still reject it. Probes are explicitly speculative,
not newly established native records. Header-only rejections produce no component
attempt and remain represented by the preserved source recording.

Expanded the pinned remaining-context harness with hooks from the actual
walkEquipRecoveryAt call and direct native component attempts. All pre-existing
fixture inputs/outputs remain unchanged. The independent direct attempt hook
sequence is also checked equal to the actual recovery walk's callbacks for all
512 inputs. There are 734 attempts, 258 rejected attempts and 244 callbacks;
12 callbacks occur in rejected walks. Tests compare ordered component indices,
start/end bits, raw status, bounds, callbacks, candidate outcome, retained source
attribution and full stream serialization. The existing Film.equipment_changes
export carries these diagnostics automatically. The first focused callback/source
regression and Clippy passed; final endpoint/full/captured checks are recorded in
PORT_STATUS.md. Positive complete captured recovery remains unestablished.

## Generic native reader adapter (current audit)

`decode_native_frame_records` now advances caller-owned `NativeFilmBits`, applying
positive preambles only at position zero (512 native reader cases). It uses the
same explicit-boundary native loop with tail provenance and wildcard/strict
binding policies. Negative reader positions remain explicitly unsupported.

The native generic loop now honors the encoding's simulation completion flag;
None means native standalone false. A 64-case native simulation fixture found
the previous forced-true behavior and verifies both settings. The bounded
convenience loop's historical true policy remains separate.

All 30 named native hooks have retained Rust representations listed in
OBSERVER_STATUS.md. That per-hook evidence does not close arbitrary observer
pointer replacement/restoration or cross-family callback interleaving contracts:
mobility actions are currently a separate ordered vector. RecordMaskHook is a
biped scanner hook, not a callback of generic DecodeFrameRecords. Do not require
it to fire on every generic record. The native private position accumulator has
no public installer or production caller; its optional component-level semantics
are separately covered by 1024 native cases.

## Loaded position scanner context audit

`ScanBipedRecords` installs `ContexteDeLecture`, but the reachable position and
companion paths do not consult its profile. Header matching and quanta use the
explicit layout. Ordinary velocity/forward/angular/aim readers use fixed widths.
Dynamic forward uses its explicit component parameter and fixed bit forms;
dynamic angular delegates to fixed 19/8-bit movement decoding. Body and shield
call their fixed vitality decoders directly, not observer-publishing dispatch.
The only callback site in this call graph is `RecordMaskHook` after saturation
rejection and companion reads. Profile/observer setter parity elsewhere cannot
be inferred from this fact.

The pinned source fixture now repeats all 1280 loaded-source quantized calls with
two installed profiles: the zero profile and the default profile with full
precision and simulation completion enabled. Reflection instruments all 30 native
observer function fields. Each run compares complete returned native records,
errors, and ordered masks against the prior baseline and rejects any non-mask
callback. These 2560 native context experiments retain 19906 mask publications.
Rust verifies the experiment witnesses and every returned direction, vitality,
orientation and aim companion, alongside the existing position/source assertions.

This closes the suspected missing profile effect for the loaded position scanner.
No Rust context parameter is needed by this path. Arbitrary reader observer
replacement/restoration and context installation in other caller families remain
separate API contracts; this evidence does not close those general gaps.

## Kill-calibration corruption source lookup

Native calibrate calls GrammaireSousFilm, whose ResolveProfile finds chunk number
zero through metadata. This lookup is independent of the kill timeline registry,
which is read from the first source buffer. Rust starting_calibration previously
reused the first buffer and timeline registry for both contracts. It now selects
the first metadata entry numbered zero and parses that chunk's registry before
reading its identification flag. Missing chunk zero or missing identification
retains the native invariant with corruption_control_read=false.

The pinned corruption-source oracle covers 216 cases combining reordered and
missing chunk zero, duplicate zero entries, negative metadata numbers, missing
identification and both recorded flag values. It reports 126 declared flags,
including 63 enabled. The old Rust path failed case 27 by reporting a declaration
when metadata had no chunk zero. All 216 cases pass after the lookup correction.
The template, compressed fixture and generator registration are retained.

## Complete attempted-component traces for biped channel consumers

The six existing outputs BipedChannels, InventoryDeltaStream, AbilityChargeStream,
UnitEquipmentStream, HeldWeaponChangeStream and BipedAbilityStates now carry
component_attempts in scanner order. Each attempt retains index, dispatch status,
independent bounds status, source/packet/record/slot identity and the full decoded
component with fields, references and ordered callback diagnostics. Successful
intermediate reads are retained even when they are not the scanner's published
target. Existing rejected_components remains the rejected subset for compatibility.
BipedComponentAttempt names the common record; BipedChannelRejection is its
compatibility alias. Existing target publications and acceptance gates are unchanged.

The independent native attempt fixtures already contained the evidence: across
512 biped contexts the scanner-specific traces contain 730 inventory attempts,
1018 unit-equipment attempts, 1580 camo/ability attempts, 1857 charge attempts and
2408 ability-state attempts. Respectively 506/790/1345/1621/2156 are accepted reads.
A second 512-context fixture has 768 held-weapon attempts, 501 accepted. These
are per-scanner traces and can overlap; consumers must not merge them as distinct
recorded events. No inference or architectural layer is introduced. Old exports
without this trace deserialize to empty and cannot prove absence of native reads.
Validation results are recorded in PORT_STATUS.md after checks finish.

## Equipment-object attempted reads

EquipmentStateStream.component_attempts retains every consume_component_at
result in the ti=37 delta scanner, before its status/bounds gate. Each entry
contains the actual FilmPacket, per-chunk packet ordinal, candidate record bit,
slot/generation, component index, raw status, bounds and full decoded component.
This keeps successful intermediate fields and callbacks from failed reads without
publishing a partial EquipmentStateSample. Both existing Film constructors retain
the stream through Film.equipment_state, including its portable representation.

The 512-case equipment-state native fixture now records the actual scan observer's
ordered equipment emissions, with slot/generation and field/value/presence.
All previous inputs, accepted samples and native counters were verified unchanged
before replacing the fixture. There are 5339 emissions, including emissions before
walk failure; test assertions compare all of them against the retained attempt
trace. This is callback and retention evidence, not independent proof of every
component field or a complete canonical hierarchy. Component codec fixtures
remain the evidence for leaf fields; positive captured checks remain separate.

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

## Managed-property component attempts

ManagedPropertyScan.attempts now retains every direct component attempt before
the status/bounds gate, including successful intermediate fields and failed
reads. Each contains timestamp, record start/slot, component index, separate
status/bounds and the full decoded component. Payload-only scan_delta calls
retain unavailable packet provenance; the source scanner supplies the actual
FilmPacket and ordinal counted before delta filtering. Existing accepted reads,
chaining and setup-error policies are unchanged. Film.managed_properties carries
the complete trace through its existing constructors and serialization.

The 1024-case native fixture wraps the actual managedPropertyWalk observer and
records 54483 callbacks in order, including both simulation completion policies.
All prior fixture inputs/outputs were verified unchanged before replacing it.
Rust compares the full callback sequence and unchanged accepted readings/counters,
and roundtrips the complete result. The captured corpus comparison additionally
checks packet/ordinal identity and bounds for source-tagged attempts. These
remain scanner traces rather than validated gameplay events or canonical records.
Validation outcomes are recorded in PORT_STATUS.md.

Positive source-to-constructor retention is additionally verified by
`managed-source-v41.json.zlib`: both configured Film constructors compare 54
actual native source scans, 5,391 accepted readings and 5,404 callbacks. These
include rejected-attempt callbacks, source ordinals counted before packet
filtering, exact wire timestamps and complete Film JSON roundtrips. The native
scanner discovers the observed band from retained keyframe bytes; it does not
reuse the primitive fixture's supplied band. Internal registry holes are excluded
because they cannot be represented in bootstrap bytes, and remain standalone
context cases. See `../fixtures/managed-source-v41.md` for construction/provenance.

## Navpoint delta and keyframe attempts

NavpointRadialScan.attempts retains every consumed delta component and complete
optional keyframe result before publication/rollback gates. Attempts retain
candidate record start/slot, decoded fields and callbacks, status/bounds or
keyframe stop reason, native projected match time, and available source packet
metadata/ordinal. Match time does not overwrite the source wire timestamp.
Clockless packets remain skipped as in the reference; their absence from the
trace is not proof that they contain no data. Payload-only diagnostic invocations
retain unavailable packet provenance.

The 1024-case native fixture now wraps the actual navpoint observer, capturing
23127 callbacks while preserving every prior input, reading and counter exactly.
Rust compares the complete ordered callback sequence and accepted output, plus
complete result serialization. Captured corpus checks additionally validate
packet/ordinal provenance. Tests and status are recorded in PORT_STATUS.md.

The explicit-encoding Film constructor now invokes the navpoint scan and retains
its result/error, matching the map constructor's data availability. The native
empty-band policy is a successful empty scan; it differs from the objective and
equipment scanners' empty-band failures. A new constructor test initially
assumed an error and was corrected to assert the native successful-empty result.
The scanner behavior itself was unchanged. Positive constructor trace validation
is now covered separately by `navpoint-source-v41.json.zlib`: 64 synthetic
sources run through both configured constructors, comparing the complete native
source scan and 2,374 ordered callbacks (491 keyframe, 1,883 delta). The native
source scanner discovers the band; it does not reuse the primitive fixture's
explicit band. Both languages parse the same synthetic bootstrap structure.
The tests verify source packet ordinals, record starts, projected times and
complete Film JSON roundtrips. This closes positive constructor retention
coverage, not the separate captured bomb-film gate. See the fixture provenance
in `../fixtures/navpoint-source-v41.md`.

## FilmContext constructor and ownership audit

The pinned FilmContext constructors expose a context with lazy source caches,
a by-value scan profile and a shared observer. They are not aliases for Film
assembly. NewFilmContext starts with invariant scan settings and resolves the
film profile lazily; NewFilmContextForMap resolves that profile eagerly and
selects forced layout, then valid catalog layout, then detection. A forced
layout is copied without Valid validation, including invalid widths/gates.
resolve_imposed_i0_layout now exposes that exact selection rule for already
computed catalog layouts. It returns an owned copy, not a decoder-validity claim.

The 256-case native constructor oracle records actual constructor outcomes,
simulation-completion gates and ownership witnesses. Presence of an imposed
layout enables native simulation completion even for an invalid forced layout.
Repeated Observation calls return the same pointer; NouveauLecteur inherits it;
CadreDeBalayage uses the profile with a nil observer. ImposedLayout returns a
copy. The constructor helper verifies layout selection, not arbitrary observer
aliasing or the complete stateful native context surface.

| Native context operation | Current Rust representation / limit |
| --- | --- |
| Film, ChunkNumbers, ChunkAt | NativeFilmContext borrows FilmSource and lazily caches its chunk prefix; source bytes and packet ranges remain borrowed. |
| Registry and named archetype access | NativeFilmContext caches the registry result/error and exposes explicit registry_mut access. Parse-time fingerprint/count snapshots survive mutation. Named archetype error adapters remain separate. |
| BipedSlots | NativeFilmContext caches source_biped_slot_band across data chunks, separately from the detector's six-chunk band. |
| I0Layout / ImposedLayout | NativeFilmContext caches detector outcomes, including failed candidates and diagnostics, and copies imposed layouts. NativeFilmContext.for_map composes eager profile, imposed layout and simulation grammar state. |
| Profile / ProfileErr | NativeFilmContext.profile lazily caches loaded-source resolution and returns a deep copy with ordered typed issues. NativeResolvedFilmProfile retains identity, keys, highlight, keyframe, movement, map, slots and MPP metadata. Eager map construction and scan-profile state are now implemented; shared observer/reader composition remains open. |
| ProfilDeBalayage / setters | NativeScanProfile and NativeFilmContext now implement replacement/restoration, MPP/raw/layout precision setters, scalar-copy/shared-map semantics and the separately cached film corruption override. Conversion into existing reader/frame APIs remains open. |
| ContexteDeLecture / NouveauLecteur / CadreDeBalayage | Explicit per-reader/frame parameters and retained outputs. Selected sequence oracles exist; full shared observer/context API remains open. |

All three facade declarations are now inspected and mapped as partial. This is
not closure of the context implementation or the larger v41 parity gate.

## Managed-player team context correction

`player_teams.go::lireEquipeDuRecord` installs the full context twice: once for
the full-state walk and once for an independent managed-player default-state
boundary check. Both header and size-word widths come from that profile. Rust
previously hardcoded default start 140 and 32/64-bit trailing guards; its
position-only walk also lost calibrated/stub component widths.

The new read/collect/source `_with_context` APIs pass FrameEncoding through
the native padded keyframe walk, derive both boundaries from KeyframeLayout,
and read four actual source bits only after their boundaries agree. They use
the first successfully traversed component span, not the presence of a decoded
team field: native calibrated skips can omit that field while still accepting
the raw team bits. Truncation at the team bits remains a refusal. Legacy APIs
use default widths and share this implementation. Both configured Film
constructors replace the initial default-profile team scan with the effective
context scan after applying the Film corruption policy.

The native team oracle preserves its original 1,024 cases and adds 512 profiles:
headers 64/72/96/108/128/160, size words 0/1/5/16/24/31/32, both corruption
policies, calibrated team skips 0/9 and stub width 2. These add 2,882 positive
and 1,732 refused direct reads. Full packet counters and published assignments
are independently obtained from native scanPaquetEquipes/publierEquipes.
Constructor regression inputs come from native cases 1025, 1029 and 1031;
source bootstrap/packets encode the same inputs, and expected results remain
the native fixture. This closes the team width/context gap, not arbitrary
shared observer APIs or retention of callbacks from the entire team walk.

## Team-walk data retention

FilmPlayerTeams.attempts now retains full native keyframe attempts before team
boundary/domain/conflict admission. Each attempt includes start bit, slot,
optional KeyframeRecord, and independent raw team reading. Source scans attach
FilmPacket and ordinal counted before filtering; payload-only calls mark those
references unavailable. Both configured constructors and the base Film path
retain these attempts through Film.player_teams and serialization. No second
walk is introduced within the scanner: publication uses the retained result.

The team oracle now has 2,048 cases with unchanged previous output fields,
15,079 attempt start/slot/end/reading comparisons and 2,914 ordered callbacks.
217 rejected team reads retain native callbacks. Complete keyframe fields and
stop reasons roundtrip with the attempts, while existing independent keyframe
fixtures cover those leaf values. The accepted-team corpus check ignores only
the additive trace when comparing its older native value fixture. This closes
team-walk output retention; general shared observer API equivalence remains open.

## Aim-only and companion reader context audit

`offline_aim_only.go::ScanBipedAimRecords` installs a context on one reader, but
its only reader-based component calls are readBodyVitalityComponent and
readShieldVitalityComponent. Their vitality decoders use fixed bit widths and
constant dequantization ranges, with no profile reads or observer callbacks.
Velocity/orientation/aim helpers consume explicit payload offsets directly.
Therefore the absence of a profile argument on Rust's aim-only entry point is
not itself a missing v41 context dependency. Loaded aim selection already has
its independent oracle. Vehicle source composition now has 32 positive aim
cases and 32 out-of-band controls in its 120-case native source fixture.

The corresponding offline_biped companion vitality reads have the same inert
context usage. That scanner additionally publishes RecordMaskHook explicitly;
its separate native mask-hook oracle remains the evidence for those callbacks.
This audit does not claim arbitrary context equivalence for grammar dispatch
or replace the other source/caller acceptance gates.

## Incomplete profile constructor warning

V41FilmProfile::warn_incomplete now publishes journaliserProfilIncomplet's
WARN for unknown recorded format/build keys, including missing identity. It
preserves native format-then-build error order and the newline separator, and
includes format/build attributes. A source without a registry is silent; map
issues alone are silent. Map-aware Film construction invokes it immediately
after resolving the profile. Base decoding and pure profile resolution do not.

The native 128-case oracle calls profile.Resoudre and the actual logging function
against loaded registry-present/data-only sources, comparing 44 WARN records
and 84 silent controls. This closes that constructor warning payload/gate gap;
complete constructor-wide ordering and lazy/shared context APIs remain open.

## Native source-context cache port

NativeFilmContext now owns per-context lazy source caches while borrowing the
loaded FilmSource. Registry success/failure references have stable identity;
registry_mut changes the cached result rather than a clone. Its separate
registry_identity accessor preserves native parse-time fingerprint/named-slot
snapshots after mutation. FilmRegistry::fingerprint remains the standalone
recomputation helper and is explicitly distinct from that cached identity.
I0 results retain refusal and measurements, including implausible candidates.
Forced values bypass detector validation; absent sources preserve no-chunk and
missing-registry outcomes. Rust borrowing prevents external source mutation
while a context holds it; no unsafe shared mutation is introduced.

The 144-case native context-cache oracle reuses independently generated loaded
i0 sources, then calls actual FilmContext methods for lazy/cache/ownership
behavior. It includes 29 invalid forced successes, seven detected successes,
missing/short/compressed registries, repeated errors and registry mutation.
This adds the source-cache API, not the full FilmContext: eager map profiles,
profile replacement/restoration, shared observers, reader/frame creation and
integration of those facilities with existing Film passes remain open. The
resolved replay architecture remains deferred.

## Loaded-source resolved profile and lazy context accessor

NativeResolvedFilmProfile now exposes all native resolved-profile groups:
recorded identity and key presence, highlight layout, keyframe frame widths,
movement invariants, map metadata, build-dependent slot widths, format-dependent
MPP widths and ordered typed format/build errors. Unread keys/identity are
Option values. The resolved MPP absence is not replaced by scan defaults; map
absence is not a native profile error. This source resolver is restricted to
v41 identity parsing and explicitly rejects other recorded major versions.

NativeFilmContext.profile resolves lazily from original FilmSource bytes,
independently of cached-registry mutations, and returns an owned deep copy.
The 64-case native source-profile oracle compares every identity field and
boundary, full metadata, exact movement float32 bits, error membership and
joined error text/order, while validating native identity-copy semantics.
Rust also checks cached-registry independence, detached profile edits and
full profile JSON roundtrip. It does not yet implement NewFilmContextForMap's
eager profile/grammar/warning composition, mutable scan profiles, shared
observers or reader/frame creation.

## Stateful scan profiles and eager map construction

NativeScanProfile now retains every movement, keyframe, MPP and grammar field.
NativeSharedWidths represents allocated Go maps: clones alias entries while
profile scalar fields remain by-value copies. Nil maps differ from allocated
empty maps. Snapshots serialize values; they do not promise to reconstitute alias
graphs. Precision descriptor widths use u64 so arbitrary native metadata survives
WASM export before reader-specific validation.

NativeFilmContext.for_map resolves the source profile eagerly, selects forced
or valid catalog layout, switches simulation completion on layout presence,
and emits the native incomplete-profile warning once. It leaves the movement
world descriptor untouched, as native construction does. Context profile
replacement returns the previous effective settings. The separately cached
film corruption flag overrides replacements; missing identity preserves the
initial inherited fallback. Raw descriptor replacement does not toggle grammar;
layout installation refuses zero axes, preserves the index width when gate<=4,
and otherwise updates axes/index/region and simulation completion.

The 96 native sequence cases compare nine state snapshots each, including
map-alias mutation, prior MPP values, negative widths, 2^40 axis metadata,
corruption declarations/fallback and restoration. A 128-case warning fixture
additionally checks eager/lazy cache state and one-time warning output.
Shared observers, reader/frame construction and integration with current Film
passes remain unimplemented context contracts. This is native parser parity,
not the queued Film/ResolvedFilm/playback refactor.
