# Native observer publication audit

Scope: LevelUp `feat/v75`, pinned commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81`, v41 Rust port.
Source: `internal/grammar/observateur.go`. This is a publication checklist,
not a declaration of full parser parity. `PORT_STATUS.md` records test runs;
the source manifest retains broader file-level gaps.

Reader replacement contracts and the equipment-creation caller correction are
mapped in [READER_CONTEXT_PORT.md](READER_CONTEXT_PORT.md). The explicit
equipment `PoserObservation` call reinstalls the same observer pointer already
installed by its context; it is not evidence of a missing distinct override.

“Verified stream” means native callbacks were recorded directly and compared with
Rust's ordered publications, including speculative reads. Context switches not
represented by Rust's supported v41 profile remain part of the separate profile
audit. A raw decoded field by itself is not proof of callback publication parity.

| Native hook | Current Rust evidence | Remaining observer work |
| --- | --- | --- |
| AbilityEnergyHook | Verified stream, `AbilityEnergy` | None for tested v41 contexts |
| CamoStateHook | Verified stream, `CamoState` | None for tested v41 contexts |
| MobilityActionHook | Verified stream, `mobility_actions` | None for tested v41 contexts |
| SpartanAbilityHook | Verified stream, `SpartanAbility` | None for tested v41 contexts |
| AbilityNonPredictedHook | Verified stream, `AbilityNonPredicted`, including disabled anchor bodies and level tails | Other inherited/recovery combinations remain under audit |
| GrenadeSetHook | Verified stream, `GrenadeSet` | None for tested v41 contexts |
| AbilitySetHook | Verified stream, `AbilitySet` | None for tested v41 contexts |
| GameEngineHook | Verified stream, `GameEngine` | None for tested v41 contexts |
| ManagedObjectHook | Verified stream, `ManagedObject` | None for tested v41 contexts |
| NavpointHook | Verified stream, `Navpoint` | None for tested v41 contexts |
| ObjectiveHook | Verified stream, `Objective` | None for tested v41 contexts |
| ManagedPropertyHook | Verified stream, `ManagedProperty` | None for tested v41 contexts |
| HeldWeaponHook | Verified stream, `HeldWeapon` | None for tested v41 contexts |
| ObjectParentStateHook | Verified stream, `ObjectParent` | None for tested v41 contexts |
| PlayerStateHook | Verified stream, `PlayerState` | None for tested v41 contexts |
| ProbeHook | Verified stream, `Probe` | None for tested v41 contexts |
| EtatMouvementHook | Verified seven typed snapshots; direct, generic record, production, inference, harvest, raw resync, chain, repair and keyframe streams | None for tested v41 contexts |
| MppHook | Verified stream, `Mpp`, direct defaults and New-record recovery | None for tested v41 contexts |
| EmpTimerHook | Verified stream, `EmpTimer` | None for tested v41 contexts |
| EquipmentCreationHook | Verified stream, `EquipmentCreation`, defaults 37 and 42 | None for tested v41 contexts |
| EquipmentStateHook | Verified stream, `EquipmentState` | None for tested v41 contexts |
| RecordMaskHook | Verified ordered `BipedPositionStream.record_masks`, including CaptureDirs and filter ordering | None for tested scan contexts |
| PosCaptureHook | Ordered direct, sequence, frame, keyframe, inference, harvest, resync and march-walk publications; resolved map-profile callbacks verified | None for tested v41 contexts |
| UnitRefHook | Verified ordered `UnitReference` publications, with raw fields preserved separately | None for tested v41 contexts |
| GrenadeCountsHook | Verified stream, `GrenadeCounts` | None for tested v41 contexts |
| UnitEquipmentHook | Verified stream, `UnitEquipment` | None for tested v41 contexts |
| WeaponAmmoHook | Verified stream, `WeaponAmmo` | None for tested v41 contexts |
| WeaponRoundsHook | Verified stream, `WeaponRounds` | None for tested v41 contexts |
| DesiredWeaponSetHook | Verified stream, `DesiredWeaponSet` | None for tested v41 contexts |
| GroundWeaponAmmoHook | Verified stream, `GroundWeaponAmmo` | None for tested v41 contexts |

The 28 typed stream variants live in `FilmComponentObservation`; mobility has its
own ordered vector. The shared native test observer installs 26 callbacks; movement
uses an explicit reader-slot observer in its dedicated fixtures.
The `component-hook`, `ability-hook`, `object-hook`, `managed-hook`, `engine-hook`,
`player-hook`, `probe-hook`, and `equipment-hook` fixture families cover
direct reads, keyframes, chain inference, harvest, repair and raw resynchronization.
The `mobility-*` family independently covers mobility capture suppression.
The `movement-hook-*` family verifies all seven movement snapshots, direct
suppression, default production-frame routing, inference, harvest and raw resync.
Chain and repair fixtures verify suppression. Dedicated recovery-slot fixtures
exercise New records after the native reader is recreated; validation is recorded
in PORT_STATUS.md. Keyframe and fresh native component reads emit at slot zero.
The generic record loop assigns each New record its own slot, unlike production
and inference inheritance; its separate fixture compares all callbacks and endpoints.
Other inherited profile settings remain in the broader context audit.
The `default-hook-*` family covers direct default readers, keyframes, New-chain
inference, repair, harvest and resync, with actual default guards enabled.

Raw resynchronization shares callback closures with its caller even when the
absolute-index histogram is newly allocated in a temporary observer. Hook streams
therefore belong in `caller_visible`, not `detached_scans`. The independent resync
oracles exercise both histogram initialization states.

The `record-mask-hook` fixture compares 256 native scans, all 2,647 mask
publications, their payload bytes and after-i0 bit offsets, and final filtered
positions. It toggles direction capture and saturation rejection, includes dynamic
orientation, and verifies that isolation/speed filtering does not remove earlier
publications. Rust scan options retain the existing companion-capture default;
callers can explicitly disable `capture_dirs` to match native CaptureDirs=false.

The `unit-reference-hook-*` family adds all three encodings (variable-width,
optional word32, unconditional word32) to ordered reads/keyframes, chain inference,
harvesting, raw resync and repair. Unit refs remain live when native suppresses
position/movement only; repair, single-step inference and validated resync suppress
reference publications. The raw reference vectors remain available in either mode.
NativeComponentCapture exposes independent direct-read reference/movement switches.

Position publications retain exact float bits in `FilmComponentObservation::Position`.
`NativePositionCapture` supplies explicit map/quantum/slot/optional accumulator and
an emission switch. Direct and sequence oracles cover all five kinds and callback
order relative to references. Shared non-i0 vector readers never publish positions.
FrameEncoding carries optional exact map/quantum context from the v41 profile.
Frame, keyframe and recovery wrappers now route it with the native slot policy.
Raw target scans replace the caller's position hook; accepted rereads publish.
Chain, repair and validated-resync fixtures prove suppression, including positive
unsuppressed controls. Accepted march walks publish at each record's own slot.
The observed march locator matches native trial callbacks, including non-signature
slots and repeated strict generation-check reads, while suppressing movement.
The integrated caller/profile audit remains open; the observer is not yet counted
complete.

Native context ownership audit: CadreDeBalayage carries the profile with a nil
observer even if the FilmContext has hooks; NouveauLecteur inherits the observer.
A 512-case native guard confirms this distinction. Default high-level march's
use of an offset-only locator is therefore not missing native callback data.
Custom ability/mobility body switches now have direct, keyframe, generic and
inference native evidence, including disabled-body publications and positive skip
widths. Other inherited switches and recovery combinations remain under audit.

Configured keyframes now inherit the simulation-completion gate, including the
zero-axis Film fallback. A 256-case native fixture checks stop index, callbacks,
width-override bypass and table binding publication. Captured Bazaar's complete
zero-axis document still matches. This closes that specific propagation gap;
the remaining integrated caller/profile audit is still open.


Keyframe chain observation routing now has independent 512-case native evidence:
all ordered callbacks match, including 101 failed-body cases. Attempt records and
aggregate diagnostics survive Desync; successful no-archetype entries emit no
body callbacks. This does not close the remaining integrated observer audit.

Native sequential keyframe tables now share the padded record reader with chains.
A separate 512-case oracle matches ordered callbacks and retains failed-record
observations in 172 cases; six positive truncated-body reads verify padding.
The bounded replication-table API remains separate. Integrated audit remains open.


## Position caller/profile closure (2026-09-24)

The position observer now has verified routing for the supported v41 contexts:
all 30 hook rows have publication evidence. This does not close arbitrary custom
profile combinations, other source-level APIs, or full captured-film parity.

The non-test source audit found one position-hook installer,
`frame_harvest.go::scanForTargetDelta`: it copies the caller observer, replaces
only the position callback with a first-sample collector, and shares other
closures. Existing raw-resync fixtures compare that behavior. The only hook
neutralizers are `neutraliserCaptures` (position/reference/movement) and
`neutraliserCapturePosition` (position/movement); their inference, chain, harvest
and repair callers are covered by the dedicated ordered-callback fixtures.
`CadreDeBalayage` still has nil observer; `NouveauLecteur` inherits it. There is
no production installer of the optional position accumulator.

The remaining resolved-profile check exposed and fixed a real factory mismatch:
`V41FilmProfile::frame_encoding` used the map bounds for observer dequantization.
Native `Movement.Range` remains `QuantRangeCEBiped` when the map descriptor is
installed. The factory now preserves that range and sets the descriptor's region
index width to the native effective minimum of one bit. Explicit caller-supplied
capture ranges and dedicated map-position scanners retain their separate roles.

The profile oracle now compares exact capture settings for all 237 map/build
combinations. A new `position-hook-resolved` oracle builds 512 native production
frames across all 79 catalog maps and compares all 863 ordered position callbacks,
other callbacks, record ends, final cursors and view counts. Rust constructs its
settings through its own profile factory, rather than loading the oracle's
encoding. The first diagnostic compared against the immutable native profile;
the final fixture applies the native map-descriptor installation before comparing
the effective scan context. The range mismatch persists under that installation.

The historical open-position notes above describe earlier checkpoints. Broader
inherited/custom-profile and complete captured objective gates remain open.

### Objective-budget publications (2026-09-24)

`StatborgBudgetDiagnostics` now captures native rejection/exhaustion/summary WARN
records through the shared budget increment path. A direct slog oracle compares
638 ordered publications across 64 cases, including repeated summaries and the
shared eight-detail rejection cap. Legacy budget value contracts remain intact.
The opt-in low-level API is verified; integration into named-event, slot-identity
and cross-check passes remains open and named_bounds.go remains partial-v41.

### Objective-budget public-pass integration (2026-09-24)

The three native-equivalent passes now expose `_with_diagnostics` APIs with
fresh per-pass budgets and the native origins/summary placement. Existing APIs
share the same implementation with observation disabled. `countsOf` traverses
sorted slots in both forms; named/cross-check passes retain sorted counter order.
The 64-case native pass oracle compares 108,418 events and 639 ordered warnings,
including exact attributes, identities and cross-check values. A separate real
one-million-event exhaustion fixture checks a digest of every ordered named-event
field, identity/cross-check results and all six warnings. Eleven focused statborg
tests pass. This closes the prior budget-specific pass-integration gap.

The named_series.go audit found a separate missing warning in ChronologicalTotal
for backward-time drops. That source remains partial; the new APIs specifically
expose budget diagnostics and do not yet capture chronology warnings.

### Chronology warnings and inherited callers (2026-09-24)

Added shared `StatborgDiagnostic` (the previous budget record name remains a type
alias). ChronologicalTotal now offers the native WARN publication with exact
slot, dropped/retained counts and first backward timestamp. Observation flows
through cumulative slots, descriptor totals, named events, identity and cross
checks. Replay round-series construction also exposes it before clock clipping.
Legacy APIs use the same implementations with observation disabled.

Direct/cumulative oracle: 128 cases, 67 direct and 213 cumulative warnings.
Pass oracle: 3,814 ordered warnings, including chronology/budget interleaving.
Replay oracle: 1,024 cases, 976 warnings; old score output values were independently
checked unchanged before refreshing the fixture. Native multi-slot cumulation
walks an unordered Go map; Rust documents ascending slot order. Tests normalize
only that unordered collection and retain exact per-warning attribute ordering.
This closes the chronology gap identified in the budget integration checkpoint.
Higher-level Film/replay document diagnostics remain subject to their own audit.

### Loaded statborg source publications (2026-09-24)

The source scanner now exposes the exact missing-manifest INFO and whole-frame
record-limit WARN, with ordered attributes and the native metadata index.128 native
source cases compare680 records and55 INFO records; the actual record-limit case
compares33,078 retained records and its WARN. Source/legacy scanners share the same
implementation. All16,113 records in32 captured v41 films match the pinned oracle.
Other objective source scanners' missing-manifest publications still need auditing;
film.go's earlier blanket ported status was corrected accordingly.

### Loaded objective pass publication order (2026-09-24)

Loaded named events, cross-checks and identity preserve source diagnostics before
budget/chronology diagnostics, including unsupported-mode source scanning. Native
128-source and positive-control fixtures compare values and messages; the real
source-limit case compares five ordered WARN records per wrapper. Reports retain
source truncation explicitly. Caller audit correction: StatRecordsCtx is the sole
production chunksDatables caller, so its missing-manifest INFO is now fully covered
at that boundary. Footer/capture loaded-source selection remains a separate gap.

### Film statborg source diagnostic retention (2026-09-24)

The legacy chunk scanner now exposes `scan_film_statborg_with_diagnostics`, using
exactly the same StatborgScan as its non-observing API and the loaded source API.
`Film::try_from_chunks` stores its ordered output in `statborg_diagnostics`, with
the supplied match ID (empty when absent). It scans once. JSON retains nonempty
diagnostics and defaults absent fields for older exports. This field specifically
represents the source pass; it does not claim downstream event/score diagnostics.
The native128-source fixture checks representable legacy metadata paths. The
real33,078-record cap fixture also runs through Film construction and serialization,
including an old-export compatibility check and a non-truncated negative control.

### Player-score warning path audit (2026-09-24)

Flat score publication loads personal score, kills, deaths and assists in that
order; each total can publish chronology warnings before frame clipping.
Multi-round publication builds per-XUID round segments and publishes cumulative
warnings in sorted XUID order, then personal/kills/deaths/assists order. Empty or
out-of-window curves can therefore still produce a warning. The native caller
computes flat identity before dispatching to the multi-round path as well.

The separate `build_replay_player_scores*_with_diagnostics` APIs retain these
warnings. Full top-level replay ordering remains open: native buildScoreTimeline
loads team score, team frags and player frags, then flat identity once before
team/player publication and optional hold ticks. Rust's existing top-level
helpers repeat/reorder some of those computations; concatenating their warning
vectors would not establish native top-level parity.

The player-pass oracle compares1,536 cases, all1,536 direct and1,988 automatic
chronology warnings, with exact attributes and order. Original1,024 output cases
were checked unchanged before fixture refresh. The512 new cases use one slot and
reversed-round timelines, avoiding normalization of native unordered slot maps.
The handler is shared with the existing score-series harness and both are copied
by the generator before replay tests run. Top-level score ordering remains open.

### Complete score computation order (2026-09-24)

The top-level score builder now prepares team score, team kills, player kills and
flat identity in native order. Prepared totals and identity are reused for team,
player and optional hill-hold publication. The player path accepts the prepared
identity, so multi-round selection does not recompute the flat pass. Hill totals
are read only after the native team-identity/mode gates allow that stage.
`build_replay_score_with_diagnostics` and `build_film_replay_score_with_diagnostics`
retain these pass diagnostics separately from Film.statborg_diagnostics; no
source scan or native replay-schema change is introduced.

The complete-score native fixture now includes1,536 cases and2,003 chronology
warnings across482 positive cases, all in exact native order. The original1,024
value cases were checked unchanged before refreshing the fixture. The512 added
cases use one team slot and one player slot with opposing or reversed round times,
so no normalization of native unordered multi-slot warning groups is needed.
Budget message details remain covered by the independent identity/budget fixtures;
this complete-score fixture's positive warnings are chronology messages.
Film/document-wide report propagation beyond the score wrapper remains open.

### Identity completion diagnostics and provenance (2026-09-24)

`StatborgRoundIdentity::completed_by_lines_with_diagnostics` observes the existing
identity pass only after native nonempty-lines and single-round gates. The Film
player builder retains its output as `FilmReplayPlayers.statborg_diagnostics`,
with serde defaults for older reports. Source and score warnings remain separate.

The new512-case native oracle compares1,647 warnings:657 chronology,880 detailed
budget rejections and110 budget summaries. It also found and verifies a value
fix: after a nonempty triplet result, native rebuilds origins only for returned
links, while Rust previously retained unrelated slots/rounds.329 provenance
rebuilds match. Early returns preserve inputs verbatim; incumbent links, duplicate
XUID refusal, empty origins and immutable inputs are covered. Native private nil
origin maps are normalized only to the Rust empty-map representation (Origin()
returns the same empty string); values and warning ordering are otherwise exact.

Registry construction consumes round boundaries but does not call an additional
statborg counter/identity pass. Document reporting still cannot be called complete:
`replay_objective_actions`, `replay_flags_film` and `replay_vip` call unobserved
statborg_named_events. Their mode/early-return boundaries and warning propagation
must be audited before composing a complete statborg document report.

### Objective, flag and VIP named-event diagnostics (2026-09-24)

Added diagnostic-returning objective identification/publication, flag publication
(with default or explicit equipment catalog), and VIP publication APIs. Their
vectors are specifically named-event pass diagnostics; source, identity and score
reports remain separate. General objectives extract named events before roster
and death-evidence refusal. The outer replaybuild application also logs match-row
counts/errors; these wrappers do not claim those application-context messages.

Native attachFlagCarries first calls flagFilmSignalsOf (which extracts events),
then extracts events again before checking Scanned/recognition. Rust now retains
both passes in that order. The first event vector is dropped after counting the
signals, before allocating the second. This is the attachment boundary, not the
separate earlier decodeFilmCarrierMarks source scan. VIP returns before extraction
when unrecognized; recognized VIP extracts events before resolving death identity.

The128-case native oracle compares the flag prelude's signals/events and1,548
ordered warnings, plus actual attachVipCrown output/coverage and320 warnings.
All1,863 resulting VIP periods match. Inputs include reversed rounds, oversized
increments, disabled recognition, and invalid/empty clocks. No warning normalization
was needed (one statborg slot). The existing downstream flag and generic-objective
fixtures remain separate evidence for those publication algorithms. Full statborg
report composition at document assembly remains open.

## Document statborg report scope

`build_film_replay_document_with_statborg_report` returns the document result and
an independent serializable `FilmReplayStatborgReport`, including on errors.
Ordered stages retain Film source diagnostics, player identity-completion
warnings, one general objective named-event pass, optional score diagnostics,
flag attachment's two named-event passes, and gated VIP attachment diagnostics.
Stages refer to wrapper execution, including wrappers returning no warnings due
to internal mode gates; they are not an assertion that every inner pass ran.
No player timeline means only the retained-source stage is present. Source
observations are copied from Film rather than rescanned. The legacy document API
uses the same assembly path and discards the report; its JSON schema is unchanged.
This is not a report of all parser observations or native application logging.
In particular it excludes carrier-mark source extraction and `identifiedEvents`
application logs requiring match-row/error context absent from these inputs.

## Single-component cross-family order

`DecodedComponent::ordered_publications` exposes borrowed typed publications
without changing its serialized fields. MobilityActionHook is emitted first in
`consumeBipedMobilityAction`; the optional EtatMouvementHook follows it. The
remaining mobility-body reads publish no additional hooks. Other direct component
readers publish only the already ordered component_observations vector. The API
rejects multiple mobility reads and mobility flags attached to another component
name; its contract is an individual component, not an aggregated scan.

The reader-sequence native harness wraps existing installed callbacks to capture
actual call order. All prior fields remain identical. Across 1536 reads it records
2640 publications (384 mobility and 2256 named-component publications); 256 reads
contain both families. It covers observer replacement/restoration, capture
suppression and callbacks retained before rejected reads. Rust compares typed
payloads and ordering against this stream, and the existing serialization test
ensures order remains recoverable after a component roundtrip. This closes the
single-component ordering question only. Merged speculative/repair/scan diagnostic
vectors still need their own ordering preservation.

## Merged publication order

FilmReadDiagnostics now retains optional mobility_offsets: each value counts the
named-component publications preceding the corresponding mobility callback.
The normal mobility emitter records its offset; merges shift incoming offsets
by the existing component count. Position/reference suppression remaps offsets,
and detached resync routing moves both streams with their order evidence.
FilmReadDiagnostics::ordered_publications borrows the combined typed stream.
Older exports with mobility callbacks but no offsets return None, never an
invented concatenation. Invalid lengths, bounds and nonmonotonic offsets are
also rejected; merging unavailable order keeps it unavailable.

Four native harness extensions measure actual offsets in the installed callbacks:
512 mobility repair cases (74359 offsets), 512 movement-hook repair cases (11539),
512 mobility harvest cases (2726), and 512 movement-hook harvest cases (432).
All prior fixture fields are unchanged. Their existing full-diagnostic equality
assertions now compare order too. The reader-sequence test additionally merges
1536 native reads, compares the complete stream of 2640 typed publications and
roundtrips it, with negative cases for old/invalid metadata. Focused suppression
and detached-resync tests exercise remapping and separation explicitly.

This adds parser observation metadata to the existing export. It does not add
resolved events or begin the queued playback architecture. Verification applies
to the normal emitter, merge/suppression operations and these native caller paths;
other publication sinks still require their own integration audits.

## Production admission counter propagation

ProductionFrame now retains per-frame RejetsHorsDatum, RejetsDeVue and
LiaisonsParAnticipation as optional ProductionAdmissionDiagnostics. Counters are
recorded at world.admit_delta, before any rejected header exits the view or an
anticipated binding proceeds to its body. They are not reconstructed from the
world's cumulative histogram. Older frames with no retained counters remain
explicitly unavailable.

The 16-case native production-admission oracle compares all three counters,
while retaining previous record/cursor/anticipation assertions. Source scanning
keeps nonzero reports with packet attribution in MovementStateStream, and the
real Film constructor/export regression verifies propagation. These counters
are parser admission observations, not new resolved gameplay events.

## Complete declaration audit (2026-09-24)

The pinned Observation struct declares 30 callbacks, all mapped in the table
above, and the following 12 counter/map fields. This reconciles the old manifest
note about unmapped hooks; it does not close context or caller-level parity.

| Native field | Rust FilmReadDiagnostics field |
| --- | --- |
| CompWidths | component_widths |
| ChaineReparees | repaired_records |
| ChaineImmediat | chain_outcomes[Immediate] |
| ChaineProfond | chain_outcomes[Deep] |
| ChaineAmbigu | chain_outcomes[Ambiguous] |
| ChaineAucun | chain_outcomes[NoConfirmation] |
| ChaineBudget | chain_outcomes[BudgetExhausted] |
| ResyncValides | validated_resyncs |
| RejetsHorsDatum | rejected_unbound |
| RejetsDeVue | rejected_other_view |
| LiaisonsParAnticipation | anticipated_bindings |
| IndexAbsolus | absolute_indices |

`take_absolute_indices` snapshots and clears with `mem::take`. RawResyncDiagnostics
retains initialized shared versus newly allocated detached histograms separately;
callback closures remain caller-visible. Native capture neutralization suppresses
position, reference and movement selectively, while mobility stays active.
Direct FilmContext readers inherit the observer; frame configs do not (512 native
guard cases). Existing replacement/restoration evidence covers 1,064 sequences,
6,384 reads and 6,348 callbacks, including callbacks before failed reads.
These facts concern existing APIs, not the deferred Film/ResolvedFilm design.

## Identity registry publication

IdentityRegistryOutput::log emits the native registry INFO, creation INFO and
three conditional creation refusals, then four registry warnings in order.
It consumes final section coverage, preserving post-scoreboard residue without
subtracting pre-scoreboard bot counts. Document assembly calls it after player
publication/origin resolution, before combat layers. The existing 1,024-case
registry oracle now captures 2,048 INFO and 3,655 WARN directly from native
logRegistry; all prior fixture fields are unchanged. Every warning condition
has positive and negative cases. See identity-registry-logs-v41.md for provenance.
Full document-wide logging and shared context APIs remain separate acceptance
gates. No deferred architectural changes are introduced.


## Live direct-reader delivery

NativeFilmReader now invokes shared observer hooks at component publication sites,
retaining diagnostic candidates independently of installed hooks. Absolute-index
counts update at their native read location. The 256-case live-observer fixture
compares mid-component enable/disable and histogram take/reset from callbacks;
see [fixture evidence](../fixtures/live-observer-v41.md). Record/frame observer
installation, other cumulative scan counters, world accumulation, and whole-Film
integration remain separate acceptance work. Thread-safe hook storage does not
claim native cross-thread execution equivalence.
