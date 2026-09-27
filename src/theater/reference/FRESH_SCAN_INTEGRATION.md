> Scope superseded by the user on 2026-09-26: complete native v41 parsing only.
> Replay/identity-bridge/six-phase derivative integration is outside the active
> acceptance criteria. This document is historical work, not the remaining task
> list. See PARITY_REMAINING.md for the current native-data scope.

# Fresh v41 scan integration

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
This work targets parser parity; it does not start the deferred architecture phase.

## Entry order

`replay/build_from_film.go::BuildFromFilmAvecFaits` rejects a missing map first,
creates the shared fallback counter, warns on an unprofiled format, constructs one
map-aware context, replaces its calibrated scan profile, then installs map world
precision. Decoder identity comes from that context's resolved profile.

`scanFilmInputs` installs caller/default scan settings, catalog bounds,
CaptureDirs=true and the context's imposed layout. The six phases are positions,
carried inventory, abilities, world, movement states, and identity bridge. Capture
runs immediately after successful scanning, before options application and
position-based document assembly. Do not reuse the older Film document builder's
pass order.

## Position path

`scan_replay_initial_inputs` now composes the native first phase using one shared
context and source: profile version, translocations and sorted speed exemptions,
positions, creations, fire, then loadouts. It forces CaptureDirs and installs
ImposedLayout explicitly. Position errors stop later observations; the other
scan failures log and clear their own outputs. Typed borrowed observations expose
the five native phase boundaries. `ReplayInitialScan::apply_to_facts` fills only
the fields this phase produced, preserving unrelated accumulated facts fields.

The native combined fixture exercises positive positions, creations, fire,
loadouts and translocations as well as position refusals. It compares observation
order, INFO/WARN runtime logs and complete encoded facts files after this phase.
Those files validate this boundary only: production capture must still wait until
all remaining phases finish. See `initial-scan-validation.json` for counts and
platform results. Optional wall-clock debug timing remains outside this helper.

The first phase reads profile version, translocator events and teleport exemptions
before scanning positions. Position errors stop the build. Biped creations, shots
and keyframe loadouts follow, with their own nonfatal error/log behavior.

The source position scanners already preserve chunk/packet/bit references and
pre-filter mask observations. `facts_from_world_position_scan` and
`facts_from_quantized_position_scan` now project their accepted records into the
native facts DTO, preserving order, quanta, recorded companions and explicit
world-coordinate presence. Pass the scan's actual CaptureDirs setting; retained
Rust component indices do not imply that the native capture gate was enabled.

This projection intentionally follows the reference cache's narrower domain:
packet/range references and full vitality flags remain in the source result;
MaskBits remains in the in-memory DTO but is omitted by the cache encoder;
quanta-only Q values remain in memory but the cache only writes Q with HasWorld.
Direction mode/default are only persisted when HasRoll is true. These are native
cache limitations, not a fidelity promise for the proposed canonical Film layer.

Validation reuses `halo_rust_record_mask_hook_test.go.txt`'s independent native
source scanner fixture: 256 configurations, 19,484 accepted quantized observations
and 16,914 world observations. Tests compare every destination DTO field against
native output in order, including positive direction/vitality branches and disabled
capture. Separate round-trip checks explicitly assert known cache omissions.
These are controlled source payloads, not independently annotated gameplay films.

## Carried inventory path

`scan_replay_carried_inputs` composes native `balayerPortage` and
`balayerInventaire`: held-weapon changes classified from the preceding loadouts,
native pickups, keyframe inventory with the shared fallback counter, then delta
inventory. All eight borrowed observation boundaries and native INFO/WARN logs
remain in order. Scanner errors reset published values without deleting the raw
scan results or attempts held by `ReplayCarriedScan`. Pickup error statistics reset
separately, matching native behavior. `apply_to_facts` replaces only this phase's
fields, including InventoryDeltaAmmoRefused in the header.

The mixed-source native fixture covers successful scans and missing chunks,
registry, archetype and biped band, with present/nil fallback counters and seeded
counts. It compares publication, statistics, callback fallback snapshots, all logs
and complete encoded facts files. This is phase-boundary evidence, not a complete
fresh-entry capture: capture in production still follows all six scan phases.
See `carried-scan-validation.json` for current validation status.

## Ability phase prerequisites

`scan_context_ability_emissions` and `scan_context_camo_states` now provide
independent loaded-context scans. Ability identity targets native index 48 and
resolves the scan profile before registry admission. Camouflage resolves its first
named component, then the profile. Each scan owns an isolated observer, resets the
latest hook per announced record and requires a successful target visit before
publication. A hook from an earlier component may supply the native value when
the target itself is a different component; do not add a target-name restriction.

`ContextAbilityChannelScan` retains full hook values, no-equipment/no-channel
observations, every attempted component and source references including requested
file number. Its filtered iterators project published ranks and camo quanta into
native facts while keeping references to their source observations. Nonbinary
camouflage values remain uninterpreted. See
`context-ability-channels-validation.json` for current validation status.

`scan_context_grapple_reads` now retains non-predicted hook publications even when
the target visit fails afterward. Its projection requires tag 3, a decoded body
and native inner value 1 or 2; broken bodies remain in observations and counters.
`scan_context_ability_impulses` separately accounts for both announced channels,
including hooks emitted before a bounds/dispatch failure. It distinguishes absent
channels from an empty successful scan, retains all non-impulse tags and stably
orders impulses by timestamp, slot and predicted-first, matching native insertion
order for ties. Both use the retained `walk_context_record_to` helper, which also
serves the stricter ability/camo scans without changing their success gate.
See `context-ability-use-validation.json` for current validation status.

`scan_context_ability_charges` retains every energy hook publication, including
mask-zero readings and padded target reads. Unarmed slots remain -1 in the raw
observation and do not produce a charge reading. Published readings retain their
source observation by index and sort stably by timestamp, slot and emplacement.
The native name-priority and profile-before-registry setup match impulse scanning;
missing energy components yield successful Absent/Scanned statistics. The native
fixture covers 192 cases, 1,548 charge readings and 485 out-of-bounds attempts.
See `context-ability-charges-validation.json` for platform validation status.

`scan_context_equipment_changes` now connects strict loaded-context emissions to
bounded recovery, counter acceptance and final assembly. It retains the strict
scan, all header-matching speculative probes, pre-acceptance window candidates,
accepted recovered emissions and final changes. File numbers are explicit i64
values in EquipmentEmission/EquipmentChange, separate from FilmPacket buffer
identity. The shared legacy/new assembly uses file number and native packet
ordinal for film ordering; old serialized data still defaults to its legacy
source identity when those fields are absent. Recovery-window file bounds use
i64 too. Every probe retains its source packet, file number, bit offset, window
index, dense/sparse form, hooks and component boundaries.

See `context-equipment-scan-validation.json` for native window/candidate/final
record comparisons and platform status. The synthetic source fixture includes
numbering gaps, out-of-order admission, unnumbered buffers, missing registry,
chunk terminators, duplicate counters and birth-dependent head recovery. Its
facts projection follows the native cache's narrower fields; source references
remain in the scanner result. The composed abilities phase is described below.

`probe_context_equipment_recovery` now provides the native sparse/dense probe
with a private live observer, inherited profile and retained component attempts.
It keeps the latest hook even when the successful-target gate refuses the
candidate. This is the probe only: callers must still admit production headers
and active windows, then apply counter acceptance and final pruning. See
`context-equipment-probe-validation.json` for the native comparison and platform
status.

Recovery must use the native live hook and successful-target gate, including
when an earlier component supplies the hook; the legacy recovery helper's target
name check is not equivalent. Its dense anchor uses the context's I0Layout,
not reconstructed map defaults. Probe every bit without a post-match skip, assign
the first matching active window, retain speculative attempts separately, then
apply counter acceptance and final pruning. Recovery visits numbered chunks in
ascending window range via ChunkAt. Birth witnesses are minimum timestamps per
slot from the initial phase's raw position output.

The abilities phase publishes ten observation boundaries: ranks and statistics,
equipment changes and statistics, camouflage and statistics, grapple and
statistics, impulses, charges. Native errors clear all published records; only
equipment, impulse and charge errors also reset their statistics. Preserve this
asymmetry and the native INFO/WARN ordering when composing the phase.

## Abilities phase

`scan_replay_ability_inputs` now composes native `balayerCapacites`: ranks,
equipment recovery/changes, camouflage, grapple, impulses, then charges. It
extracts minimum raw position timestamps per slot for birth witnesses without
reordering or changing those positions. All ten borrowed observation boundaries
keep native source references beside the narrower facts projections. Raw scan
results/errors survive publication resets. Equipment, impulse and charge errors
reset their published statistics; rank/camouflage/grapple statistics survive.
The phase preserves the native INFO/WARN order and field names.

`ReplayAbilityScan::apply_to_facts` replaces only fields owned by this phase.
Its validation compares all observations, ordered logs and complete encoded facts
files at the phase boundary. This does not move production capture earlier:
world, movement and identity must still run before capture. See
`ability-scan-validation.json` for platform evidence and remaining coverage gaps.

## World phase prerequisites

`scan_source_zoom_events` now borrows loaded FilmSource buffers, preserves
requested file numbers beside buffer identities and uses native timestamp-only
sorting. It retains complete padded zoom-head diagnostics, including reference
and level recordedness. Its native facts projection omits source fields without
discarding the raw source-linked result.

`scan_context_equipment_spawn_events` now uses cached native chunk enumeration
and retains all-packet ordinals, all three guarded reference values and bit
boundaries, including the third reference whose native DTO only exports a gate.
Truncated native raw-reference reads are retained as explicit refused heads rather
than triggering native indexing panics. Native counters project separately from
the extra truncation count. Neither event scanner infers deployment or scoped
hold intervals. See `source-world-events-validation.json` for current evidence.

Next connect loaded world-object census/tracks, placement calibration/creation
and confirmation, then the calibration-width handoff to pads and vehicles. The
native placement scan temporarily installs MPP widths and restores the caller's
profile. Its published Calibration.Widths remains the measured calibration;
format-27 profile widths may independently govern creation decoding. Preserve
that distinction when wiring the following phase consumers. Spawn publication
belongs after placements, and guarded objective channels follow vehicle scans.

## Remaining wiring and scanner evidence

- `scan_source_keyframe_inventory` now borrows loaded buffers and preserves native
  map admission, packet order and fallback timing. An empty catalog returns before
  source access; a nonempty all-false catalog still scans. A zero grenade cap
  increments the shared counter before source refusal. Native facts retain ammo
  and grenade fields; source locations, gauge quanta and read methods remain in
  the scan result. `scan_context_biped_pickups` uses the context's cached band,
  retains every attempt and padding verdict, and projects published records and
  all native counters to facts. Missing band disables only off-band rejection.
  Host/WASM checks pass for 360 inventory cases (288 records, 120 refusals) and
  three pickup source scenarios assembled from 3,328 native direct payload cases.
  Clippy, formatting, diff and 485 reference hashes pass. See
  `source-inventory-validation.json`. Carried-inventory composition is described
  above; world, movement and identity phases remain open.
- `walk_context_delta_bipeds` provides the raw channel traversal boundary over
  borrowed loaded payloads and exact caller chunk order. An empty selection stays
  empty, duplicate selections repeat, and missing chunks are skipped. It applies
  no position filters and emits no observer hooks. Anchors retain the full mask,
  generation, packet ordinal, requested file number and source buffer identity.
  Invalid layouts outside the bounded reader domain return explicit errors rather
  than emulating native panics or nonprogress. Host and WASM agree on all 612 records in 256 cases; Clippy, format/diff
  and reference hashes pass. See `context-delta-walk-validation.json`.
  `scan_context_held_weapon_changes` now resolves chunks, cached band, I0Layout,
  biped registry, then scan profile before checking named weapon components. It
  uses a scan-local observer, with a fixture hook proving context subscribers do
  not receive these callbacks. It walks the entire mask after a weapon callback,
  retaining subsequent and failed attempts. Local hook state survives failed
  walks until a successful visitor clears it. The shared classification helper
  retains native initial-loadout precedence and per-slot/component history.
  HeldWeaponChange file numbers now use i64. Facts projection preserves native
  cache fields while leaving file number and low variant identity in scan output.
  See `context-weapons-validation.json` for validation status and remaining
  admission-branch coverage. The carried phase now consumes this bridge.
- `scan_context_inventory_deltas` now uses the raw context traversal and a
  scan-local observer. Grenade hooks populate the current record before the
  component bounds verdict; ammo and reserves require the successful visitor
  before assignment to a weapon slot. This distinction preserves native padded
  grenade publication without attributing failed ammo reads. Every attempt stays
  available, including the components behind whole-film ammo refusal. Publication
  and refusal logic is shared with the legacy scanner and regression-tested.
  `InventoryDeltaRead.chunk_number` keeps the requested native file identity
  separately from the source-buffer index. Facts projection carries native grenade
  and selection fields; the native cache does not encode delta ammo or locations.
  The shared phase must also copy AmmoRefused to the facts header explicitly.
  See `context-inventory-validation.json` for the native comparison and platform
  status. The carried phase now consumes this bridge and copies its verdict to
  the facts header. The composed abilities phase now preserves its own
  observation boundaries, runtime logs and nonfatal resets.
- `scan_context_biped_creations` now handles the native automatic and explicit
  sparse-band creation entries over the shared loaded context. It retains native
  admission order (context, chunks, band, profile), scan order, payload/prologue
  boundaries and rejection statistics. Automatic discovery uses the context's
  cached whole-film band. It deliberately emits no mask hooks, matching the
  reference's offset reader. Accepted records project to FactsBipedCreation while
  keeping representation/version and source ranges in the source result.
  Its separate native fixture uses normal profile resolution and a v41/27
  registry header. The older creation-source fixture intentionally seeds the
  reference corruption cache because its chunk zero contains a keyframe instead
  of a registry; retain that fixture for the original lower-level tests rather
  than treating it as a normally opened v41 context.
  Host/WASM comparisons pass for 512 scenarios and 1,220 accepted observations,
  including explicit sparse bands, automatic bands, source admission failures,
  prologue boundaries and all published rejection counters. The original creation
  scanner regression, Clippy and formatting also pass. See
  `context-creations-validation.json` for evidence and scope limits.
- `scan_source_fire_events` and `scan_source_translocator_events` now consume an
  optional loaded FilmSource without constructing a synthetic Film or copying
  source chunks into legacy envelopes. Fire output keeps source payload ranges
  alongside the complete decoded event; fire file numbers now use i64. Translocator
  output retains padded reads and partial coordinates separately from its facts
  projection. Both event types can project directly into native facts DTOs.
  The complete fresh entry must invoke them in the native sequence.
  Host and WASM oracle checks pass for 48 fire cases (35 events) and 128
  translocator cases (1,000 events), including nil source, gaps, empty chunks,
  all-packet ordinals, admitted incomplete events and source payload boundaries.
  Legacy fire scanner regression and Clippy also pass. See
  `source-events-validation.json`; this does not establish complete fresh-entry
  or captured-film parity.
- Preserve caller chunk selection for biped band discovery. Native
  ScanBipedPositions derives its band from selected chunks and the following chunk;
  substituting the context's cached whole-film band changes that contract.
- Native ScanBipedPositionsForBand uses its option Layout, not automatically the
  context's imposed layout. scanFilmInputs explicitly copies ImposedLayout into
  that option. Keep the entry's setup distinct from the low-level scanner.
  `scan_context_world_positions` now connects this world-coordinate branch to the
  shared context's reader/observer. It preserves selected-chunk band discovery,
  explicit empty-band refusal and explicit option-layout precedence. It resolves
  the reader context after chunks/band/layout admission and retains profile errors
  in their full native u32 version domain. Native localized error strings remain
  represented by the existing Rust scan error categories. The fresh entry still
  needs to set catalog bounds, CaptureDirs and ImposedLayout before invoking it.
  Host validation compares 18,911 automatic/explicit selection observations and
  refusals; host/WASM also compare 16,914 companion-bearing world observations
  and mask hook counts against native fixtures. Clippy and format/diff checks pass.
  See `context-positions-validation.json` for the precise platform scopes.
- Connect all remaining scan DTOs and guarded channels, with the shared observer,
  calibrated MPP handoff, error/reset behavior, runtime logs and fallback timing.
- Exercise the full fresh scan/capture/assembly path against pinned native output
  before describing BuildFromFilmAvecFaits parity as complete.

## Loaded world-object census and tracking

`scan_source_world_object_keyframes` scans loaded source buffers directly and
shares census accumulation/finalization with the legacy wrapper. It retains
keyframe packet identities, file numbers, empty keyframes, and ordered recovered
anchors, including their bit offsets and heuristic recovery classifications.
The existing independently generated census oracle covers 64 cases; the loaded
path also runs with an extra negative-numbered source buffer to distinguish file
numbers from buffer indices. Consecutive timestamp deduplication happens before
sorting; nonconsecutive duplicate times remain.

`scan_context_world_object_tracks_for_band` uses the shared context's world-object
precision, with bounds checked first, then profile and chunk availability. It
retains source-linked position reads before lifetime segmentation discards short
segments. `WorldObjectSample.chunk` and `WorldObjectPadding.chunk` are now i64 to
preserve native file numbers. Legacy scanners explicitly widen their i32 file
numbers; the facts projection no longer needs that widening.

The new pinned-Go generator `halo_rust_context_world_tracks_test.go.txt` produces
112 loaded-source cases: 1,722 position reads, 138 tracks, 11 missing-bounds
errors and 30 missing-chunk errors. It covers gaps, duplicates and out-of-order
metadata, negative-numbered buffers, End packets, empty bands, and descriptor
values different from map defaults. Both new scanners pass the wasm32 harness.
The tracking adapter explicitly rejects descriptors outside the existing
selective decoder's supported widths (index <=32, axes 1..32); arbitrary-width
and overflow semantics still require a dedicated native audit.

Next: loaded placement calibration, creations and confirmation, preserving the
native distinction between measured calibration widths and format-selected
creation widths. Then finish the world phase and remaining fresh-entry phases.
These additions do not establish complete parser parity or start NEXT_PHASE.md.

Validation at this checkpoint: 15 native world tests passed (one ignored), the
facts-track codec regression passed, both selected wasm32 oracles passed, Clippy
passed with warnings denied, formatting/diff checks passed, and all 485 pinned
source hashes matched. See `context-world-tracks-validation.json`. No build
sessions remain live at this checkpoint.

## Native equipment creation reader prerequisite

Placement calibration must use native default-state reads, not the older strict
`decode_default_state` adapter. `NativeFilmReader::read_equipment_default_state`
now runs ti37 defaults with the shared signed MPP widths, native zero-tail reads,
live hooks, primitive fields and cursor restoration. It preserves unit-reference
observations in addition to equipment/MPP callbacks. The Go oracle covers 384
cases, 2,304 ordered hook publications and 110 reads past the source tail,
including zero-width and 33/64/65-bit MPP fields.

`read_native_equipment_creation` admits the body after a caller-validated 24-bit
NEW header. It retains the default-state read, partial mask, component start and
explicit refusal category, then returns the native creation DTO when accepted.
A private observer keeps speculative calibration attempts out of the caller's
shared callbacks. The pinned fixture covers 160 payload/profile cases and 488
attempts: 178 accepted, 54 default overflows, 239 mask refusals, 17 position
refusals. This is a body reader, not the loaded creation scanner or calibration.

The next integration should call this body reader for all 63 MPP candidates per
eligible anchor, then reuse the native candidate-order verdict. Calibration must
resolve the equipment archetype before scan profile resolution, stop only at
chunk boundaries when conclusive, and retain its measured widths independently
of the format-selected widths used later for creation scanning. The outer
placement pass still needs format selection, temporary MPP installation and
restoration, loaded creation scanning, confirmation, and facts publication.

Equipment-reader checkpoint validation: 17 native equipment tests, both new
wasm32 oracles, Clippy with warnings denied, formatting and diff checks passed.
All 485 pinned source hashes still match. See
`equipment-creation-native-validation.json`. No build sessions remain live.

## Loaded MPP calibration

`calibrate_context_equipment_mpp` now implements the loaded calibration pass.
Admission order is bounds, nonempty band, nonempty spans, equipment registry and
archetype, chunk numbers, then scan profile. Native silent refusals remain
inconclusive results with explicit Rust refusal evidence. Width candidates and
verdict logic are shared with the existing placement implementation.

The callback receives every attempted candidate in traversal order, including
source packet/file identity, anchor bit, candidate widths, the complete native
creation-body attempt and matched span index. This streams 63 attempts per
eligible anchor rather than forcing all speculative reads into memory. Each
candidate uses a local profile copy, leaving the caller's MPP and hooks intact.
Scoring considers every eligible bit without skipping accepted creations. The
conclusive verdict is tested only after the complete chunk, as in Go.

The independent loaded-source oracle has 32 cases, 549 anchors and 34,587
candidate attempts. Six cases are conclusive: three stop after the first chunk
and three after the second. It covers inconclusive score tables, positional
mismatches, missing bounds/registry, empty bands/spans and numbering gaps. Full
calibration statistics and scores are compared, not just the selected widths.
The installed MPP and shared observer are checked after every case.

Next: `ScanEquipmentCreationsForBand` over the loaded context, then compose the
outer placement pass. Preserve measured `Calibration.Widths` even when format27
selects [9,5] for creation reads; downstream pads/vehicles receive the measured
calibration widths. Keep the architectural refactor deferred.

Loaded-calibration checkpoint: native and wasm32 oracle checks, the existing
calibration regression, Clippy with warnings denied, formatting/diff checks and
485 pinned source hashes passed. Only an equivalent test-profile initialization
changed after the oracle runs. See `context-mpp-calibration-validation.json`.
No build sessions remain live.

## Loaded creations and placement composition

`scan_context_equipment_creations_for_band` now scans loaded delta packets using
the native body reader. Bounds and chunk admission precede slot statistics;
registry and archetype precede profile resolution. Accepted records skip their
bodies on the next scan step. Every attempted header is observable, including
refusals, while accepted full reads remain in the result. Native counters and
error strings are preserved for absent bounds/chunks/registry/archetype.
`EquipmentCreation.chunk` is now i64; legacy callers widen file numbers explicitly.
`FactsEquipmentCreation::from` projects the native DTO fields.

`scan_context_equipment_placements` composes census, tracks, lifetime spans,
format-only MPP selection, calibration, loaded creations and confirmation. It
retains intermediate evidence and streams calibration/creation attempts through
one typed callback. Temporary creation MPP installation uses a Drop guard, so
normal returns, errors and unwinding restore the caller's widths. Measured
calibration remains unchanged when the format chooses different decoding widths.
Facts placement/stat projections preserve the uninitialized calibration-map
state on early refusals and the initialized map after calibration.

The loaded creation oracle has 64 cases and 1,053 accepted creations. It reuses
the 32 calibration source layouts with two installed MPP profiles. The Rust test
adds an ignored negative-numbered buffer to separate buffer and file identities.
The composed placement oracle has 48 cases, 48 placements and 30 scanned results.
It includes successful calibrated fallback on formats24/28, format27 success
with inconclusive measured calibration, no movement witnesses, missing registry,
missing keyframe band, numbering gaps and missing bounds.

Next: loaded pad scans (weapons and powerups), vehicles and guarded world
channels, followed by the combined world phase. In the native replay wrapper,
`decodeFilmPlacements` logs an inconclusive-calibration warning but still returns
`pl`; despite the warning text saying no placements, do not clear successfully
decoded format-profile placements. Keep that native behavior in phase publication.

Loaded-placement checkpoint: four context-equipment native tests passed; the
creation regression passed ten tests with one ignored, including legacy vehicle
and ground-weapon creation checks. Both loaded scanners passed wasm32. Clippy,
format/diff checks and 485 pinned source hashes passed. See
`context-placements-validation.json`. No build sessions remain live.

## Native ground-weapon reader prerequisites

`NativeFilmReader::read_ground_weapon_default_state` now reads ti42 defaults
through the same live-width, signed-cursor implementation as ti37. It retains
raw magazine/default fields and MPP/creation callbacks; those default magazine
fields are not substituted for the later weapon-ammo component. The independent
ti42 oracle covers 384 cases, 1,920 ordered hook publications and 152 attempts
past the source tail, including zero and 33/64/65-bit MPP widths.

`read_native_ground_weapon_ammo` now reproduces the complete native production
component loop with a separate cursor: exact registry-name/index guards,
64-bit mask aliasing, calibrated widths before dispatch, stub widths after failed
dispatch, and corruption checks. It keeps attempted components, primitive reads,
skipped-width evidence, corruption fields and final/desync positions. It walks
components after i20 and then rereads the first 19 bits at i20's attempted start,
so a later failure does not erase already-located ammo. The third ammo field
remains in the complete component read without assigning it an unproved meaning.

The native ammo oracle covers 512 cases, 874 component attempts, 240 successful
ammo reads, 128 traversals through i9 multiplayer properties, 70 traversals past
i20 and 64 final cursors past the source tail. It validates component order,
ported status, starts, final cursor and ammo values, including refusal cases.

Next: parameterize the native creation body/loaded scanner for ti42 and include
the native ammo read with its full trace. Then compose weapon/powerup pad scans
with format/calibration width selection and restoration before vehicle/world
phase integration. Do not route the loaded path through the older strict
creation/default/ammo adapters.

Ground-reader checkpoint: all 12 selected native ground tests and three wasm32
oracles passed, including the expanded i9 fixture. Clippy passed before that
fixture-only expansion; Rust source was unchanged. Formatting/diff checks and all
485 pinned source hashes passed. See `native-ground-readers-validation.json`.
No build sessions remain live.

## Loaded ground-weapon creations

`read_native_ground_weapon_creation` now shares native NEW-body admission with
ti37 but reads ti42 defaults and retains `ammo_read`, including the complete
secondary traversal. Ammo reads do not change the creation scan's `AfterBit`.
Observed default and later component hook values are applied in native order.
`scan_context_ground_weapon_creations_for_band` shares loaded traversal and
counters with equipment while retaining ground-specific archetype admission.

The independent loaded oracle covers 96 source/profile cases, 184 accepted
creations and 114 with ammo. It reuses the existing native ground-creation profile
payloads under real loaded registries, varied numbering, End packets, absent
registries/bounds, empty bands, calibrated/stub widths and corruption settings.
Synthetic registry holes are populated with explicitly unported labels: an empty
slot followed by later names is structural registry termination, not a sparse
component list. Initial all-refusal fixture output exposed and corrected this
fixture framing error.

Next: compose the weapon and powerup pad scans in replay order. Width selection
must use format27's profile first; otherwise publish the unknown-format fallback
counter when applicable and use caller calibration only if both widths are valid.
Invalid calibration leaves inherited context widths in place. Always restore
any installed widths. Pad publication clears the whole native WorldObjectScan
on missing band, creation error or tracking error, while retained Rust evidence
can still describe the failed scan. Then finish vehicles/guarded world channels.

Loaded-ground checkpoint: the native loaded oracle passed, all three selected
wasm32 scans passed, 18 native equipment regressions passed, and Clippy,
format/diff checks and 485 reference hashes passed. See
`context-ground-creations-validation.json`. No build sessions remain live.

## Loaded pad scan composition

`scan_replay_pad_inputs` now composes ti42 weapons before ti37 powerups over the
loaded context. Each scan resolves registry format first, uses format27's [9,5]
widths when available, and otherwise installs calibration only when both signed
widths are positive. Invalid calibration leaves inherited widths in place. A
Drop guard restores temporary widths. Unknown-format counter events are returned
in selection order for the outer world phase to publish; global event timing is
not yet integrated.

Each result retains census, optional creation/track evidence and an explicit
failure. Native publication is wholly empty after a missing band, creation error
or tracking error. Successful publication includes complete creations, statistics,
keyframe times/lifetimes and tracks. Native structured log ordering is preserved.

The independent Go fixture contains 96 cases: 17 scanned weapon cases with 46
accepted creations, and 24 scanned powerup cases with 416 creations and 132 tracks.
The first fixture selected every other ground case; this accidentally aligned
all successful ground inputs with format27's mismatched widths and produced no
accepted weapons. The corrected generator uses consecutive ground cases and the
Rust test requires accepted creations in both channels. It compares complete
publication, ordered logs, fallback events, callback source identities, scan
order and width restoration. The fixture has no mobile weapon tracks and no
post-creation tracking failure; these are explicit coverage limits.

Next: native ti40 default-state reader with live MPP and vehicle media widths,
then loaded vehicle creation scanning. The native creation scanner admits bounds,
chunks, I0Layout, vehicle archetype, and profile in that order; it shares the
creation walk but uses dynamic biped i0 rather than world-object precision. The
vehicle replay wrapper clears publication on census/creation/position failure,
then adds events, occupant aims and march deaths/occupancy nonfatally. Continue
with guarded world channels and complete world phase, movement, identity bridge,
fresh-entry composition and the final all-data audit. NEXT_PHASE.md stays deferred.

See `pad-scan-validation.json` for validation status and fixture hash.

Pad checkpoint validation: corrected 96-case fixture passed on host and wasm32;
Clippy with warnings denied, formatting/diff checks and all 485 pinned source
hashes passed. No build sessions remain live. Full v41 parity remains incomplete.

## Native vehicle default state and loaded creations

`NativeFilmReader::read_vehicle_default_state` now enables ti40's existing
media-frame grammar using the native profile's switches and live raw widths.
The switches adapter is shared with direct component reads; unused width values
are not converted eagerly. The ti40 oracle has 384 cases, 1,536 MPP publications
and 145 reads beyond the source tail, including zero/33/64/65-bit widths and
full-precision/baseline scopes.

The new fixture found a shared absolute-width bug at case47: a 65-bit region
read yielded a negative signed native index, so Go selected build-default axes
but Rust selected map axes (432 versus 401 final bits). The native branch now
uses the signed low64 word to choose axis widths while retaining the raw field.
Existing absolute-index diagnostics still narrow to i32; auditing that reporting
domain remains necessary before claiming complete all-data parity.

`read_native_vehicle_creation` shares the native NEW body reader with equipment
and ground weapons. It uses the cached dynamic i0 layout for position admission
and AfterBit, independently of media-frame map precision, and retains the full
default read, mask, partial/refusal evidence and accepted creation. Vehicle
bodies do not run the ground-weapon ammo traversal.

`scan_context_vehicle_creations_for_band` shares loaded traversal and statistics.
Its admission order is bounds, chunks, slot count, I0Layout, registry/archetype,
profile. Layout errors preserve native messages; the complete detection remains
cached in the context. The independent 128-case fixture contains 393 accepted
creations and missing-bounds/chunks/layout/registry/archetype cases. It also covers
empty bands, End packets, numbering gaps and alternate source-buffer order.

Next: loaded vehicle-event and aim-only channels, then compose the vehicle replay
pass with its native position options and additive march deaths/occupancy. The
current event and aim DTO chunk numbers are still i32 and need widening for
loaded source identity. Continue with guarded world channels and the complete
world phase. Keep NEXT_PHASE.md deferred and keep full parity incomplete.

See `context-vehicle-creations-validation.json` for final validation status.

Vehicle-creation checkpoint: 30 native vehicle tests passed (one ignored), all
four selected default/loaded WASM oracles and five shared-path WASM regressions
passed. Clippy, formatting/diff checks and 485 pinned hashes passed. No build
sessions remain live. Full v41 parity is still incomplete.

## Loaded vehicle event and aim-only channels

`scan_context_vehicle_events` now uses the context's numbered source prefix and
biped census without legacy chunk copies. It allows an empty biped band (base0),
counts readable empty chunks, preserves all-packet ordinals, and retains source
packets. Malformed guarded references, which panic in the native scanner, return
an explicit source-linked Rust refusal plus preceding evidence. This is a
reported runtime difference, not a successful empty scan.

`scan_context_biped_aim` checks chunks then biped band, resolves the read context
at each native delta boundary, and retains source packet and record bit ranges.
`scan_biped_aim_records_with_sources` shares the existing payload grammar while
retaining companion fields traversed before/with the primary aim. Primary-end
bits preserve the native overlap rule independently of optional aim tails. The
canonical native vitality readers used by this scan have no profile-dependent
fields or observer publications. VehicleEvent and BipedAim chunk numbers are
now i64; legacy callers widen explicitly.

The independent combined oracle has 128 source layouts, 346 published vehicle
events, 862 aims, and 54 native panic cases represented as explicit Rust refusals.
It covers missing biped census, missing/readable chunks, reversed timestamps,
numbering gaps/duplicates/negative buffers, non-delta packets, End boundaries,
truncated references and source identity. Full nonpanic DTOs and native ordinary
errors are compared; on panic the test checks the distinct refusal category.

Next: compose `decodeFilmVehicleScan` over these loaded channels, loaded vehicle
creations, native vehicle position options, and `context.scan_march_facts()`.
Positions must pass `context.imposed_layout()` explicitly, allow all generations,
capture directions and use dynamic i2/i3 at level2 with production filters.
Census/creation/position failure clears the whole published scan. Events, aims,
and march deaths/occupancy are additive after position success. Preserve native
logs and death statistics and project the full native facts section. Unknown
format width selection/restoration follows pads. Then finish guarded world
channels and the complete world phase; NEXT_PHASE.md remains deferred.

See `context-vehicle-channels-validation.json` for validation status.

Loaded-channel checkpoint: host vehicle suite, both aim regression tests, the
combined wasm32 oracle, Clippy, formatting/diff checks and all 485 pinned hashes
passed. No build sessions remain live. Full v41 parity is still incomplete.

## Loaded vehicle replay composition

`scan_replay_vehicle_inputs` returns `ReplayVehicleInputScan` (the existing
`ReplayVehicleScan` is the publication-layer borrowed input). It resolves format
MPP widths before census, temporarily installs valid widths, and restores them
on return/unwind. The result exposes format/fallback metadata for the outer
world-phase counter owner and retains census, creation/position evidence,
events/aims, march facts and typed failures separately from publication.

Census, creation or position failure clears the complete native vehicle section.
After positions succeed, event, aim and march failures suppress only their own
channels. Position scanning uses imposed layout explicitly, all generations,
direction capture, dynamic i2/i3 level2 and native production filters. Native
vehicle logs are emitted in scan order. Death filtering uses archetype40 rather
than the census band; occupancy remains the march's full biped read list.

`facts_from_march_scan` projects the full calibrated frame/profile, all thirteen
statistics, initialized coverage maps and optional death list. An unattempted
channel keeps the default nil maps; an attempted failed/empty march has allocated
empty maps. `FactsScanProfile::from` snapshots optional width maps and preserves
raw profile fields. Vehicle event and aim projections preserve native cache data.

The independent composed oracle has 128 cases, 79 scanned outputs, 66 creations,
271 positions, 70 events, 30 aims, 15 vehicle deaths and 106 occupancy reads.
It compares complete native cache bytes plus uncached world positions/mask bits,
ordered logs, callback source identity and MPP restoration. The initial96 cases
had no positive deaths/occupancy, so32 loaded-march cases were added with real
serialized registry names/levels before declaring this checkpoint validated.
There are no native panics in this composed fixture.

Coverage limits: no post-creation position failure or march-error case occurs in
this fixture. Position-admission errors still use existing Rust categories rather
than exact native localized text. Outer unknown-format counter timing and final
all-data domain/runtime/combined-corpus audits remain open. Synthetic parser
agreement does not replace independently annotated gameplay expectations.

Next: guarded carrier marks, shared ti13 zone/flag reads and bomb arming reads,
then compose the entire world phase (zoom, placements/logs, spawn/stats, pads,
vehicles and guarded channels) with native observation/counter order. Movement,
identity bridge and fresh-entry/capture composition still remain. NEXT_PHASE.md
is deferred until current parity is complete.

See `vehicle-scan-validation.json` for validation status and fixture identity.

Vehicle-composition checkpoint: expanded128-case oracle passed on host and
wasm32, including positive deaths/occupancy. Clippy with warnings denied,
formatting/diff checks and485 pinned hashes passed. No build sessions remain
live. Full v41 parity remains incomplete.

## Loaded carrier-marker prerequisite

`scan_source_carrier_marks` now runs the existing carrier keyframe decoder over
loaded source buffers and numbered-prefix traversal. It retains ordered keyframe
packets even when they publish no marks. Readable empty chunks succeed; absence
of any readable data chunk returns the native error. `FactsCarrierMarkScan::from`
projects native nil/empty lists and statistics. No mode guard is inferred here;
the outer guarded world phase still decides whether to call this scanner.

The independent source oracle covers96 layouts,2,112 marks and291 keyframes,
including End packets, negative buffers, missing/out-of-order chunk numbers,
missing source and readable empty chunks. Complete scan DTOs and source ranges
are checked. This adds no architecture from NEXT_PHASE.md.

Next prerequisite: native loaded ti13 managed properties. The current
`ManagedPropertyScan::scan_delta_at` consumes legacy FrameEncoding. Share its
header/mask/traversal logic with a native component-reader callback, retaining
partial successful reads and every failed component attempt. Native setup order
is chunks, observed ti13 band, slot count, registry/archetype, then scan profile.
The native managed walk creates a LOCAL observer and installs only its managed
property hook, rather than forwarding the caller's observer; it creates a reader
per component and carries full profile widths. Do not substitute legacy encoded
widths. Native unported/overflow failures break that record but preserve earlier
reads, and only fully walked records can mark their emitted properties chained.
Then adapt loaded navpoint-radial scanning before composing guarded channels.

See `source-carrier-validation.json` for this checkpoint's validation status.

Carrier-source checkpoint: both host tests, wasm32 oracle, Clippy, formatting/
diff checks and485 pinned hashes passed. No build sessions remain live. Full
v41 parity remains incomplete.


## Loaded native managed properties

`scan_context_managed_properties` now scans loaded ti13 records using the full
native scan profile, a fresh reader per component and a private managed-property
observer. The legacy and loaded paths share header/mask traversal. Typed component
diagnostics supply the properties, retaining failed attempts and hook evidence
while publishing only successful in-bounds reads. Earlier successful reads survive
a later component failure; chaining is assigned only to complete walks. The scan
resumes at header/mask end plus one, as native does.

Admission is chunks, observed slots excluding competing archetypes, slot count,
registry/archetype, then profile. Errors retain the established census. The native
loaded oracle uses serialized registry bytes, replacing interior empty names with
explicit unported names so they cannot accidentally terminate registry parsing.
Across128 cases it publishes4641 reads from2119 records:1809 walked,310 broken,
382 chained. Source packet/range evidence and isolation from caller hooks are
checked. Nine host tests passed (one local corpus test ignored), the loaded WASM
oracle passed, Clippy/fmt/diff checks and485 pinned source hashes passed.

See `context-managed-validation.json`. These synthetic comparisons do not add
independently annotated gameplay coverage, and the loaded fixture does not vary
arbitrary raw width domains. Full parity remains incomplete. No build sessions
remain live at this checkpoint.

Next: loaded native navpoint radial. Its admission differs: no observed slots is
successful empty output, with census counters retained. Clock projection uses the
first delta timestamp per chunk and a caller manifest-start map; absent either
counts every packet as lacking a clock. Use native full-state keyframe reads and
retain rollback, sticky truncation, closure counters, private observer and partial
attempts. Then shared guarded objective composition and the remaining world,
movement, identity and fresh-entry work. NEXT_PHASE.md remains deferred.


## Loaded native navpoint radial

`scan_context_navpoint_radial` now uses loaded source traversal, raw native
component readers for deltas and `NativeFrameConfig::read_keyframe_record` for
full-state keyframes. Legacy and native paths share traversal/publication code.
The private navpoint observer never forwards caller hooks. Empty observed bands
succeed before registry/profile access, retaining all census counts. Missing
chunks and archetypes retain native error messages.

The caller supplies signed chunk-number/manifest-start mappings. Each chunk uses
its first delta packet timestamp as base, with native signed wrapping arithmetic
and i32 time projection. Source packets keep their original wire timestamp.
Packets without either clock input increment PacketsNoClock. Keyframe rollback,
sticky truncation, blocked component counts and exact closure/chaining are shared
with the existing scanner.

The independent loaded oracle uses128 serialized source/registry cases with1934
reads,1069 delta records,770 walked,299 broken,98 chained,236 keyframe records,
121 keyframe walks,115 broken keyframes,94 exact closures of177 bounded records,
and60 packets without clocks. It covers native errors, no-slot success, negative
manifest offsets, missing clocks, keyframe-only chunks, End boundaries and chunk
number/order variants. It checks every scan field, source packet identities and
caller-observer isolation. Five host tests passed. See
`context-navpoint-validation.json` for remaining validation status.

Coverage limits: this loaded fixture uses the default native profile and does not
exercise arbitrary raw width domains or the three-million-read cap. Synthetic
reference agreement does not replace independently annotated films.

Next: compose guarded channels in native observation order: carrierMarks,
zoneReads, flagGauge, bombReads. Share exactly one lazy managed-property scan
between enabled zone/gauge consumers, including failed and successfully empty
scans. Set each scanned flag from its own guard. Preserve native logs once for
the shared scan, and publish bomb keyframe closure counters after its success
log. `FactsModeGuards` already provides publication storage. Carrier guard uses
Flag.Scanned and flagFilmSignalsOf; zone guard uses the supplied nonempty zone
catalogue; bomb guard uses Bomb.Scanned then nonempty manifest clock map. Keep
scan evidence separate from nil-on-error publication. Complete world composition,
movement, identity bridge, fresh entry and final audits afterward. NEXT_PHASE.md
remains deferred; full v41 parity is incomplete.

Navpoint checkpoint: the128-case loaded oracle passed on host and wasm32. All
five host navpoint regressions, Clippy with warnings denied, and formatting/diff
checks passed. No build sessions remain live. Full parity remains incomplete.


## Guarded world-channel composition

`scan_replay_guarded_inputs` composes native carrier marks, shared managed-property
reads and bomb radial reads over one loaded context. Its four channel observations
always occur in native order, even when disabled. Each zone/gauge scanned flag is
its own guard. A single optional managed result caches successful, empty and failed
scans, so both guards never cause repeated decoding/logging. Errors clear only the
corresponding published channel; raw attempts and typed errors remain available.

`ReplayGuardedObservation` delivers diagnostics and process-counter additions at
native boundaries as well as borrowed channel values. The embedding runtime owns
those sinks; the full world runtime connection remains open. Flag guards each run
the native named-event pass independently, preserving warning order and duplicate
warnings. Bomb closure counters follow the success log and precede bombReads.

A new edge case was found while composing publication: failed keyframe walks can
allocate a radial-read slice and then roll it back to empty. Such native output is
an allocated empty list, not nil. The wrapper now derives this distinction from
retained keyframe hook diagnostics. Six controlled cases exercise it. This does
not change the lower scan DTO or discard failed-read evidence.

The independent native oracle covers224 cases:696 carrier marks,1113 zone reads,
1094 gauge reads,208 bomb reads,76 cases with both managed guards enabled, and
closure counter additions [16,51]. The first192 cases cover mode combinations and
loaded managed/radial/carrier fixture families. Sixteen more suppress delta bodies
while retaining clock headers/keyframes to exercise rollback; sixteen use native
chronology-oracle records with warnings to exercise both flag guard passes.

The generator initially read expvar counters from the root namespace, which
returned zeros. It now reads the actual `levelup` expvar map. Full channel/log
ordering, counter values at each boundary, nil versus empty publications, complete
guarded-cache bytes, and exactly one shared-scan log are compared. The208-case
host check passed; see `guarded-scan-validation.json` for final224-case status.

Generator inputs are the previously generated context-managed, context-navpoint,
and source-carrier JSON files. `/private/tmp/halo-guarded-chronology.json` is the
first16 nonempty-event_logs rows' `records` arrays from the independent
`chronology-v41.json.zlib` oracle. Expected guarded outputs are generated by
`filmScan.balayerCalquesGardes` in the pinned Go checkout, not by Rust.

No three-million-read truncation case is included. Profiles cover defaults and
simulation completion, not arbitrary width domains within this composed fixture.
No new independently annotated gameplay claims are made. NEXT_PHASE.md remains
deferred and full v41 parity remains incomplete.

Next: full `balayerMonde` composition. Order is source zoom, loaded placements,
placement stats, spawn events, pads, vehicles, guarded channels. Spawn stats are
retained but have no separate native observation. Placement replay wrapper emits
its unknown-format counter AFTER the scan and BEFORE its log; pads/vehicles emit
their counter at width selection BEFORE census. Existing pad/vehicle APIs expose
fallback metadata only on return; add boundary callbacks rather than delaying
these counters until publication. Use measured placement calibration for both pad
and vehicle inputs, retain evidence on failures, and emit native placement/spawn
logs. Source zoom and loaded placement/spawn primitives already exist. Do not
capture facts until all six scan phases have finished. Movement, identity bridge,
fresh-entry composition and final audits remain afterward.

Guarded-phase checkpoint: expanded224-case host and wasm32 oracles passed.
Clippy with warnings denied, formatting/diff checks and485 pinned source hashes
passed. No build sessions remain live. Full v41 parity remains incomplete.


## Full loaded world phase

`scan_replay_world_inputs` now composes source zoom, loaded placements and stats,
spawn events/stats, pads, vehicles, and guarded channels over the same context.
All ten native channel observations preserve their order. Spawn statistics are
retained without inventing a separate native observation. Complete world/cache
projection is available, but this function does not perform final facts capture.

Pads and vehicles now have `_observed` entry points using
`ReplayWorldObjectScanObservation`. Unknown-format notifications occur at width
selection, before temporary profile installation/census; the existing APIs remain
wrappers and preserve their prior behavior. Placements publish their format
counter after scanning and before logging. World forwards measured placement
calibration into both later scans; invalid calibration leaves inherited widths
in place under the existing native selection policy. Temporary widths restore.

World-level diagnostics and counters use `ReplayWorldObservation`; existing
pad/vehicle tracing remains at its native boundary. The test-only tracing helper
can now observe each emitted log so counter snapshots are checked against native
log timing rather than just final totals. Guarded diagnostics/counters forward
through the same world observer. The embedding runtime/fresh entry remains to be
wired to these sinks.

Channel borrows preserve nil versus empty for zoom, placements, spawn and guarded
lists. Successful placement confirmation allocates a list even when empty, whereas
earlier refusal/inconclusive paths return nil. Composed case15 exercises that
allocated empty list. A misleading native warning about inconclusive calibration
is intentionally preserved: known-format creation decoding can still publish
placements afterward; native decodeFilmPlacements returns those values.

The independent128-case oracle has543 zoom events,60 placements,1407 spawn events,
36 weapon creations,448 powerup creations and48 vehicle creations, with no native
panics. Sources combine placement, vehicle, guarded and ground-weapon fixture
families plus head-event inputs. Complete phase cache bytes, channel order,
ordered logs, native counter snapshots at every boundary, source-list nil state,
and MPP restoration are checked. The first96 cases lacked positive weapons and
vehicles; the final fixture adds ground inputs and uses vehicle creation cases.
Seven host phase regressions passed before the nil-state extension; final status
is in `world-scan-validation.json`. The WASM harness initially omitted its local
log-capture helper; that harness import was corrected without changing parser code.

Coverage gap: this composed fixture has no nonzero conclusive placement
calibration. Six lower-level context-MPP cases do have conclusive calibration,
but their lifespan inputs are caller supplied. Add a composed case with real
loaded tracks that agree with creations (or a pinned corpus slice) before claiming
that part of end-to-end parity. No radial read-cap or native panic case is added
here. Reference-generated synthetic evidence is not independent gameplay annotation.

Next: strengthen conclusive world-calibration evidence, then movement-phase
composition and identity bridge (grenades/projectiles/deaths/player tables/teams,
index and clock). Finish fresh entry, runtime sinks/counters and capture only
after all six phases, then the combined corpus/domain/null/export/runtime audit.
NEXT_PHASE.md stays deferred; full v41 all-data parity remains incomplete.

World-phase checkpoint: final128-case oracle passed on host and wasm32, with
seven phase regressions passing. Clippy, formatting/diff checks and485 pinned
source hashes passed. No build sessions remain live. Full parity is incomplete.


## Calibration and anticipation follow-up

The world oracle now has144 cases. Loaded placement cases134,136,137,138,142
conclusively select9/5 with16 agreements and zero runner-up agreements. These
use actual decoded tracks and creations, including unknown formats; downstream
pad/vehicle widths are asserted to replace inherited11/7 with measured9/5. This
closes the composed calibration gap described above. The144-case host and wasm32
checks passed. See world-scan-validation.json for updated counts and hashes.

AnticipatedDeclaration.chunk_index and FilmWorld.current_chunk now retain native
signed64-bit chunk numbers. build_source_anticipated_bindings walks loaded numbered
chunks rather than dating declarations by buffer ordinal. Sorting uses the native
unstable tie order. The160-case independent Go oracle covers128 conflicting tables
and32 loaded sources; whole tables, lookups and world binding agree. Host four
anticipation regressions and wasm32 passed. Clippy, formatting/diff checks and485
pinned source hashes passed. See anticipated-domain-validation.json.

Movement remains next. Its strict event locator must not use the death march's
fallback or generation recheck. Its hooks must inspect world bindings at publication
time and preserve native inference policies. NEXT_PHASE.md remains deferred.


## Loaded movement prerequisite: full-profile strict locator

NativeFrameConfig::locate_strict_entity_view now reuses the native march reader
with strict-only selection. It accepts only the first35-bit single-component
slot123 delta preceded by zero, with no fallback or generation recheck. Movement
hooks are scoped off during trials; other callbacks remain live and hooks restore.
The independent256-case Go oracle has80 located packets,268 trial probe
publications and256 post-locator movement publications. It varies corruption,
generation policy, extra fields, ID widths, missing bindings, truncation, and
unused position widths through u64::MAX. Final validation is recorded in
context-strict-locator-validation.json when complete.

Loaded movement implementation remains outstanding. NativeFrameConfig's
decode_inference_views is the correct general DecodeFrameViews traversal (class
and generic modes both use decodeInferLoop); decode_production_views has narrower
policy admission. InferenceFrame.records contain decoded and inferred rows;
count native type35 records including desync rows without losing raw evidence.

The movement hook must test the world at publication time, including inherited
capture slots on New records. Production callbacks already run before binding
New, but the native inference API has no equivalent callback. Do not apply a
packet-final world retroactively to buffered hook values. A scoped publication
context or observed inference traversal needs independent New/Delete/inference/
anticipation tests. read_bound_record_contextual receives both the live world and
capture slot, and chain rereads use it too; investigate it as an instrumentation
boundary without changing native hook neutralization.

MovementStateStats.map_widths currently uses usize: loaded native dimensions are
u64 and must retain that domain on wasm32. MovementStateRead.chunk and private
VerticalVelocity.chunk remain i32 and need the same i64 treatment as anticipation.
Map widths are read through native scan-profile resolution before chunk admission.
Native successful scan allocates an empty list; early absent/error paths return nil.
Replay failure clears published reads AND stats but must retain raw scan evidence.


Strict-locator checkpoint: final host256-case oracle, wasm32 oracle, Clippy with
warnings denied, formatting/diff checks, and the256-case loaded march regression
all passed. The earlier nine locator-related regressions passed too. All build
sessions are terminal. The changes remain uncommitted on simbleau/theater-experiments.
Continue with loaded movement and its publication-time world context; full v41
parity remains active/incomplete and NEXT_PHASE.md remains deferred.

## Loaded movement implementation

`scan_context_movement_states` now walks loaded numbered chunks with native
anticipation, keyframe/datum binding, strict event-packet location and the general
native inference-view traversal. It retains nil versus allocated-empty reads,
all attempted frames, and every raw movement publication with its source packet,
file number, packet ordinal, and publication-time archetype. A private observer
leaves the context's external observer untouched. Crouch/slide/clamber/sprint
reads deduplicate through the existing native scanner; velocity-derived jumps
retain their distinct name and existing derivation.

`MovementStateRead.chunk` and internal velocity chunks are i64; map widths are
u64 so metadata survives wasm32. Legacy conversions explicitly widen values.
`NativeFilmObserver` now carries a scoped movement binding context; the bound
record reader installs capture-slot/archetype lookup evidence before consuming
components and restores it afterward. This avoids checking a packet-final world
against earlier publications. The callback captures only this small context,
not an observer cycle or copied world.

The first loaded oracle found a real discrepancy at case96: generic views share
one native reader, but Rust reset the inherited capture slot for each view.
`decode_inference_frame_with_capture` now shares that slot across the views of
one packet. Single-view entry points still start with zero. The native case has
New-record hooks after a view boundary; the old code wrongly counted two as
unbound instead of duplicate observations. The expanded fixture retains the case.

Independent Go expectations now cover160 cases:583 biped records,92 desyncs,
467 raw publications,122 reads (69 crouch,24 clamber,17 slide,12 sprint),161
duplicates,13 unbound state publications and42 vertical velocity reads. Four
of24 event packets locate;20 do not. There are24 absent cases,59 errors and24
allocated-empty successful read lists. Extra32 cases preserve unused widths
through u64::MAX on absent/error paths. Native instrumentation is checked against
ScanMovementStates itself before emitting raw observation expectations.

The loaded fixture has two rejected rise episodes and NO positive derived jump.
Add loaded positive/negative jump evidence; lower-level derivation tests alone
are not the complete composed validation. Recorded corruption=true and panic
cases are also not included here. Validation status is in
context-movement-validation.json; final host/WASM/Clippy were running when this
section was written.

Next replay wrapper: `filmScan.balayerEtatsDeMouvement` clears BOTH reads and full
stats on error, warns with err/match_id, otherwise logs the native movement
counts, then publishes movementStates followed by movementStates.stats. Keep raw
scan evidence separately; cache FactsMovementStats intentionally stores only the
native encoded subset, while channel observations publish the full stats.
`halo_rust_movement_scan_test.go.txt` has already generated the160-case native
wrapper oracle at /private/tmp/halo-movement-scan.json (not yet included as a Rust
fixture or implemented). It uses haloGuardHandler from the guarded generator,
compares wrapper channel/log ordering and complete encodeEtatsDeMouvement bytes.

Identity bridge, six-phase fresh entry/runtime sinks, final capture, combined
corpus/domain/null/export audits still follow. NEXT_PHASE.md remains deferred.

## Movement replay phase

`scan_replay_movement_inputs` now implements the native movement phase over the
loaded scanner. It retains ContextMovementScan and its error separately, clears
both published reads and full statistics on failure, preserves successful nil
versus allocated-empty lists, and delivers the native diagnostic before
movementStates and movementStates.stats. The embedding runtime owns the sink.
`apply_to_facts` projects only the native encoded movement subset; the complete
stats remain available on the phase result and in channel observations.

The independent160-case wrapper oracle compares the entire ordered diagnostic/
channel timeline and complete cache bytes, including error-path resets. The
same test rechecks the raw observations/statistics against the scanner oracle,
so resetting publications cannot erase underlying evidence. The wrapper fixture
is movement-scan-v41.json.zlib. Its WASM checks have passed; final host/Clippy
status is in movement-scan-validation.json. A type-complexity lint on the small
binding context was resolved with a type alias; it did not change behavior.

Remaining movement evidence: positive loaded velocity-derived jumps with negative
controls, corruption=true profiles, and any relevant native panic/anticipation
cases. Then proceed to identity bridge (grenades, projectiles, deaths, player tables,
teams, index and clock), six-phase fresh entry/runtime sinks and capture, and the
combined corpus/domain/null/export/runtime audits. Do not start NEXT_PHASE.md.


Movement checkpoint: final host20 tests passed/one external-corpus test ignored;
loaded scanner and replay phase160-case WASM oracles passed. Earlier five selected
WASM scanner/view/inference calls and nine generic host regressions passed.
Clippy with warnings denied, formatting/diff checks and485 reference hashes passed.
All build sessions are terminal. No commits or pushes. Full parity remains active
and incomplete. Next strengthen loaded jump/corruption/anticipation evidence,
then implement balayerPont and finish the fresh-entry composition and audits.

## Loaded jump/corruption coverage extension

Movement and movement-replay oracles now have224 cases. Added64 loaded velocity
scenarios: eight accepted jumps and56 negative controls, covering insufficient/
excessive integrated height, gaps beyond the hold limit, an unfinished rise,
duplicate timestamps retaining the first sample, full-precision samples, and an
unbound entity. Each accepted jump produces two explicitly jumpDerived reads.
Half of these64 cases have corruption control enabled by a decoded identification
header, not an overridden cached flag; absent/present sentinel tails are consumed.
The Go generator asserts the positive/negative scenario outcome before comparison
and the Rust test asserts it independently of the serialized expected reads.
These are synthetic native-parity checks, not independently observed gameplay.

The224-case loaded/replay checks passed on host and wasm32, plus Clippy. Totals:
919 biped records,138 reads,330 vertical velocity reads,42 closed rise episodes,
eight derived jumps. Source metadata, raw observations, channel/log ordering and
complete movement cache bytes remain compared. Existing native-panic coverage is
separate; no panic scenario was added here. Reports have updated hashes/counts.

## Loaded grenade scan

`scan_context_grenade_throws` now implements native loaded traversal, profile
selection and coverage publication. `GrenadePreamble` retains bit width, default
state prefix and author offset. The decoder is shared with the existing recent
v41 payload API; the loaded path additionally respects the recorded build's
native prefix selection. All source headers in its fixture declare41. Earlier
build names on synthetic v41 sources exercise native selection logic; they do not
claim captured historical examples or support for other source major versions.

Projectile archetype resolution uses the native component-name vote and earliest
winner on ties. Registry failures remain inspectable while taking the native
fallback. File chunk numbers are i64 on FilmGrenadeThrow. Packet scans retain
source references and counters. No throws means a native nil list, including a
successful empty scan; no readable chunks errors before coverage logging. The
native fallback warnings and coverage diagnostic are delivered after the scan to
the embedding observer. The context's registry diagnostics retain their existing
tracing boundary.

Independent128-case oracle:512 throws,1022 candidate patterns,232 rejected
matches with recognized IDs,60 selected older-build prefixes and37 source errors.
Ordered diagnostics, grammar/prefix selection, every throw, coverage and packet
source ranges are checked. Generator is halo_rust_context_grenade_test.go.txt;
fixture/report use context-grenade-v41 and context-grenade-validation.json.
WASM passed; host/Clippy final status is in that report.

Next bridge prerequisite: ScanProjectiles delegates to ScanWorldObjects(ti41).
Admission order is source chunks, derived world-object slot band (empty is an
explicit archetype error), then ScanWorldObjectsForBand (bounds, profile, chunks).
The latter already has a loaded adapter in context_world_tracks.rs, but its
WorldObjectPrecision conversion rejects wide/overflow descriptors: audit native
semantics before treating that adapter as full-domain parity. ScanWorldObjects
returns an allocated-empty track list after a successful scan. Follow this with
deaths, film player table, teams, injective player indices and exact packet clock;
then finish balayerPont, the six-phase entry and final audits. NEXT_PHASE stays
deferred and the full goal remains incomplete.


Checkpoint:224-case loaded movement/replay host and WASM checks passed; the128-case
loaded grenade oracle passed on host and WASM, with eight host grenade regressions
passing. Clippy, formatting/diff checks and485 pinned reference hashes passed.
All build sessions are terminal. Changes remain uncommitted, on the existing
branch, under src/theater; unrelated Cargo.toml/experiments edits were preserved.
Full v41 parity remains active/incomplete.

## Loaded projectile entry and precision audit

`scan_context_projectiles` now delegates to `scan_context_world_objects` with ti41.
Admission order matches ScanWorldObjects: numbered chunks, nonempty recovered
archetype band, then the existing bounds/profile/chunk checks. A successful scan
keeps a Vec of tracks (including an empty list), its complete position reads,
and the source keyframe census used to select the band. Grenade name votes do
not change the projectile archetype used by this native entry.

The loaded ForBand path now consumes NativePrecisionDescriptor directly instead
of narrowing through WorldObjectPrecision. The new descriptor decoder follows
the pinned Go signed64/unsigned64 arithmetic on host and WASM: index truncation
to u32, zero/negative-width reads, last64-bit accumulation for wide values,
zero-producing shifts at widths64+, and wrapping length calculations. It retains
native infinity/NaN coordinate results rather than inventing a precision error.
The bounded convenience APIs retain their prior descriptor validation. Native
sample and track comparators have distinct NaN branches; both are reproduced.

Independent oracle:180 loaded sources and128 direct position cases, including
97 admission errors,3422 retained records and164 tracks. Tests compare native
track order, all sample fields, packet/file identity, accepted bit starts,
padding counts and direct position endpoints. The fixture includes index widths
33/64/65 with high bits discarded by native u32 conversion, saturated axes,
zero-width/negative-width rejection, and non-finite positions. IEEE bits are
compared except that NaN payload/sign is normalized across targets. Native scans
with impractically large overflowed limits are not run in Go: those descriptor
edges are exercised through the direct position primitive. No claim of controlled
or independently annotated gameplay follows from this synthetic oracle.

Validation state lives in context-projectile-validation.json; do not infer a
finished host/WASM run from the fixture generation alone.

Next identity bridge notes from pinned source inspection:
- ScanDeaths uses the last native numbered chunk, the narrow header-derived
  highlight profile, and native chunk-number-prefixed error strings. Missing
  registry uses the historical layout, but must not claim a recorded v41 header.
  Existing scan_film_deaths is a legacy chunk-array/version adapter, not this
  exact loaded entry. Keep the v41-only readable-version gate.
- ScanPlayerIndices checks empty roster BEFORE chunks, excludes the last numbered
  chunk, and preserves allocated-empty ByXUID even on error. Current legacy
  adapter has Rust-specific error strings; loaded bridge needs native ones.
- ScanClockOrigin looks up chunk1 directly; it does NOT require numbered traversal
  to have found a contiguous prefix. Keep successful zero distinct from failure.
- ScanPlayerTeams must use fc.Registry and the native reader context. Legacy
  scan_film_player_teams_with_context filters metadata chunk_type and uses the
  convenience FrameEncoding; do not substitute it for the loaded native path.
  Native lireEquipeDuRecord additionally requires the first component Ported,
  then independently replays the managed-player default state and compares the
  component boundary. Registry failure/mismatch and empty final map return nil.
- Bridge channel/log order remains grenades, projectiles, deaths, filmTable,
  playerTeams, conditional playerIndices, clockOrigin. No identity bridge edits
  have been made in this checkpoint. Architecture remains deferred.

Projectile checkpoint:13 host tests passed, one external-corpus research test
ignored; the180-case projectile/128-case precision oracle and112-case loaded
world-track regression passed in WASM. Clippy with warnings denied, formatting,
diff checks and485 pinned reference hashes passed. No build sessions remain.
Changes are uncommitted under src/theater on simbleau/theater-experiments.
The identity bridge, six-phase entry and combined final audits remain open;
full v41 parity is incomplete and NEXT_PHASE remains deferred.

## Loaded player-index and clock entries

`scan_source_player_indices` implements native ScanPlayerIndices over FilmSource:
empty-roster refusal precedes source selection, the final numbered chunk is
excluded, all readable replication chunks are retained, and conflicting XUID
readings are withheld without voting. SourcePlayerIndexScan preserves the raw
PlayerIndexTable even on error, using native error text. It does not apply the
later injective-or-empty bridge guard. The caller can retain raw evidence while
publishing the guarded table; an uncalled scan is distinct from an empty result.

The existing pure resolver now shares `resolve_player_index_reads`, retaining
each first XUID match and its five-bit preceding index, source bit start, and
leading zero-padding count. These are pattern matches, not claimed complete
player records. Each loaded chunk records both its file number and source-buffer
position. The map convenience resolver preserves its existing result contract.

`scan_source_clock_origin` looks up numbered chunk1 directly, returns its first
cached FilmPacket, and uses the two native failure strings. This intentionally
works even when contiguous-prefix discovery stops before chunk1. Successful
zero timestamps remain successful reads; no origin is guessed from positions.

Independent256-case Go fixture includes754 pattern reads,68 with leading padding,
45 collision cases,22 disagreement cases,130 successful clock reads (ten recorded
zeros),126 clock failures and167 index errors. It compares raw and guarded
identity tables, errors, ordered source reads, and clock packet ranges. Missing
metadata, nil sources, negative/large/duplicate numbers, gaps, empty buffers,
truncated/oversized packet headers, duplicated roster entries and zero/max XUIDs
are covered. See source-identity-scans-validation.json for final validation state.

Next: loaded ScanPlayerTeams and ScanDeaths, then balayerPont. For teams the
existing native reader already exposes read_keyframe_record; a managed-player
wrapper over its internal read_world_default_state(9) can independently replay
the default using the same signed reader machinery. Check the first component's
ported result before accepting its raw four team bits, preserve native hook
emissions, and select packets via context.chunk_numbers, not metadata chunk_type.
The generic scan profile's corruption flag is overwritten by the decoded header;
synthetic corruption fixtures must install that header, as the movement fixture
already does. For deaths, native ParseHighlightEvents wraps source decompression
errors (including underlying Go error text); don't equate Rust-specific inflate
messages with native diagnostic parity. No deaths/team implementation was changed
in this checkpoint. Architecture remains deferred and full parity incomplete.

Identity-index/clock checkpoint:256-case loaded oracle passed on host and WASM;
three existing index/evidence/clock host regressions passed. Clippy, formatting,
diff checks and485 reference hashes passed. All build sessions are terminal.
The user asked for a realistic parity ETA after3d7h. No defensible ETA exists;
PARITY_REMAINING.md now records the known integration gaps and priority: finish
the complete path and produce a real-film differential report before expanding
more helper-only suites. This changes sequencing, not the full v41 objective.
