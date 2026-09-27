# Next: scan/cache conversion and BuildFromFacts integration

The complete REPLAYINPUTS25 blob and LEVELUPFILMFACTS file are implemented.
See FACTS_ASSEMBLY_PORT.md, FACTS_FILE_HEADER_PORT.md, FACTS_IDENTITY_JSON_PORT.md,
FACTS_STATBORG_JSON_PORT.md, FACTS_KILLS_JSON_PORT.md and FACTS_FILE_PORT.md.
Combined platform results are in facts-file-validation.json. Native cache omissions
remain explicit; this is not canonical Film fidelity.

## Native integration contract inspected

Continue replay/build_from_facts.go, replay/film_inputs.go and the capture boundary
in replay/build_from_film.go. BuildFromFacts:

1. Creates a fallback counter only if the caller supplied none, then accumulates
   persisted scan fallbacks before assembly. Persisting after assembly would count
   assembly fallbacks twice on replay.
2. Restores identity from the optional identity section.
3. Applies Facts.FilmInputs to caller Options and calls BuildFromPositions using
   positions and fire events. It performs no source reads or freshness check.
4. It does NOT itself apply file.Statborg or file.Kills. Preserve the pinned call
   contract. The native application adapter internal/replaybuild/filmfacts_cuisson.go
   restores these through entreesDeLaCuisson and fills them after fresh scanning
   through completerLesFaits. This outer adapter is context for integration, not
   an instruction to port its database/application layer.

FilmInputs.applyTo restores film version, translocations, creations, loadouts,
weapon/pickup/equipment changes and counters, keyframe/delta inventory and ammo
refusal, ability ranks/impulses/charges and counters, camo/grapple, movement states
and counters, placements/spawns/pads/vehicles, grenades/projectiles, deaths,
player indices/seats/teams and team counters, and clock origin. Zoom lookup is
rebuilt purely from cached zoom events and position-derived lifetimes. Guarded
flag/zone/bomb fields update individual nested fields, preserving caller catalogs
and mode configuration. Caller geometry, match evidence and catalogs remain inputs.

faitsDuBalayage captures immediately after scanning and before assembly. It obtains
actual I0 axis widths from FilmContext.I0Layout and LayoutDetected from whether
ImposedLayout is absent. If layout resolution fails it returns no facts. Coverage
comes from the existing decoder coverage builder. Identity is copied into a fresh
optional object; do not alias an identity object being mutated during assembly.

## Player integration completed, document integration remaining

`facts_player_inputs.rs` now provides build_facts_replay_players, using the same
internal pure player assembly as the recording path. See FACTS_PLAYER_INPUTS_PORT.md.
The native comparison covers 128 cached player documents and 1,024 signed-scope
cases. Scope levels now retain native i64 rather than narrowing cache levels to
u8. Full document assembly and capture remain pending. This bridge has explicit
legacy replay-domain errors for raw non-UTF-8 strings and unrepresentable counters;
those limits must not be mistaken for native decoder rejection rules.


NativeFilmFacts uses exact cache DTOs, retaining arbitrary native i64/u64 and raw
byte strings. Existing recording/replay DTOs sometimes use narrower host-sized
fields and UTF-8 strings. Audit conversions field-by-field; never silently narrow
on WASM or normalize arbitrary cache strings. Keep cache-only fields separate from
recording source provenance and do not fabricate source ranges by constructing a
synthetic Film just to reuse source-facing builders.

The existing build_film_replay_document/build_film_replay_players paths take Film
and retained source evidence. Reuse their pure assembly passes for a cache-facing
entry point, preserving native applyTo precedence and empty/absent scan semantics.
This is completion of current reference parity, not the deferred Film/ResolvedFilm
redesign. Compare native BuildFromFacts with controlled input fixtures and captured
films; verify native BuildFromFilmAvecFaits capture timing and cache roundtrips.

Then reconcile all 485 pinned source files, 5,253 declarations, field coverage,
public exports, runtime/error behavior and captured-film documents. Positive v41
VIP/Assault and full equipment-recovery evidence remain open; do not repeatedly
request the same recordings. Current all-data v41 parity remains incomplete.

Only after current parity is complete, propose the types, migration plan, fidelity
contract and validation matrix in NEXT_PHASE.md before starting that refactor.


## Shot integration follow-up

facts_shot_inputs.rs now has cache-native shot attachment and vehicle recovery
entry points, preserving i64 player indices and complete cache orphan events.
Both use shared internal calculations with the original recording APIs; no
synthetic FilmFireEvent source coordinates are created. See FACTS_SHOT_INPUTS_PORT.md
and facts-shot-inputs-validation.json for actual validation status.

Cache loadout, grenade-read, projectile and grenade adapters are implemented.
ReplayGrenadeRead retains signed i64 selection ranks and nullable counts;
ReplayGrenade retains signed i64 player indices. Projectile publication consumes
borrowed cache points without inventing source coordinates. See
FACTS_CARRIED_INPUTS_PORT.md for their validation scope.

Inventory now has build_facts_replay_inventory, sharing a borrowed reduction with
recording inputs. ReplayInventory.gs, d and cand retain native i64; see
FACTS_INVENTORY_INPUTS_PORT.md for actual validation status. Its high-level player
publication wrapper now shares the recording publication pass; native cache/player
checks are tracked in facts-ability-inputs-validation.json. Carried ability ranks
now have a cache adapter too (FACTS_ABILITY_INPUTS_PORT.md). Remaining adapters
include equipment, vehicles, objectives,
complete cached document composition and capture/fallback handling. Preserve caller
catalogs and mode configuration, and distinguish missing input from empty scans.

## Equipment-change adapter

facts_equipment_inputs.rs now shares native selection/coverage with the recording
adapter. Published gaps and scanner counters retain i64. Native publication
normalizes every non-spawned, non-spent kind to taken; original arbitrary kind
bytes remain in FactsEquipmentChange. See FACTS_EQUIPMENT_INPUTS_PORT.md and
facts-equipment-inputs-validation.json for current checks. Remaining equipment
work includes weapon/pickup inputs, active states, impulses, charges, placements
and episodes, followed by vehicle/objective/document integration and capture.

Weapon and pickup cache adapters are now implemented with shared recording-path
reductions; see FACTS_WEAPON_PICKUP_INPUTS_PORT.md and its validation JSON.
Weapon kinds normalize at publication exactly as native does. Pickup scanner
counters retain i64, including wrapped refusal sums. Remaining equipment work
includes active states, impulses, charges, placement and episode integration,
followed by vehicle/objective/document composition and scan capture.

Translocation cache projection is now available through
build_facts_replay_translocations, sharing the native/source publication reducer.
See FACTS_TRANSLOCATION_INPUTS_PORT.md. The WASM harness supports --test-prefix
for incremental checks; report selected runs as selected, not as a full suite.

Cached grapple and camouflage/overshield episode adapters now share their pure
recording reducers. The extended grapple oracle exposed and corrected the native
float32 FMA accumulation order in arrival selection. See FACTS_STATE_INPUTS_PORT.md
and facts-state-inputs-validation.json for validation scope. Next inputs include
ability impulses/charges (their rank resolver still returns wire u8), movement
states, placements and vehicles. Full document/capture work remains pending.

Ability impulse/charge cache adapters now use the shared signed rank resolver.
ReplayAbilityRankIndex returns Option<i64>, ReplayAbilityCharge.charges is i64,
and impulse scan counters retain i64. See FACTS_ABILITY_ACTION_INPUTS_PORT.md and
facts-ability-actions-validation.json. Preserve outer stats.scanned coverage gates
when composing the document; computed builder coverage alone does not prove a
scan ran. Movement states and placement/vehicle/objective inputs remain next.

Movement cache details verified for the next adapter: FactsMovementStats stores
signed counters and u64 MapWidths but native filmfacts_encode/decode omit
JumpEpisodes and JumpsDerived; decoded native stats leave those at zero. Cache
movement kinds remain arbitrary raw strings, unlike Rust ReplayStance.kind String.
Group/sort by original bytes before any JSON string normalization; distinct invalid
UTF-8 kinds must not collapse into one timeline or by-kind count. Preserve
native nil/empty and signed counter behavior in the eventual publication bridge.

Movement cache publication is now implemented via build_facts_replay_stances,
sharing the original life/death accumulator. Raw kind identities use
ReplayByteString and remain distinct through grouping/sorting/counting; JSON
replacement happens only when serializing. Counters retain i64 and widths u64.
See FACTS_MOVEMENT_INPUTS_PORT.md and facts-movement-inputs-validation.json.

Placement origin and census-end adapters are now implemented:
ReplayEquipmentOriginIndex::from_facts/classify_facts and
replay_facts_equipment_ends share their pure reducers with the source path.
Full u32 spawn generations and validity flags survive indexing; only exact raw
b"taken" kinds count as written-taken evidence. See
FACTS_PLACEMENT_EVIDENCE_PORT.md and facts-placement-evidence-validation.json.

Full cached equipment placement publication is now implemented through
build_facts_replay_equipment_placements. It shares the publication loop and owner
selection with recording inputs; scanner counters and spawn-list count retain
i64. Native supplied placements still publish when calibration is unconfirmed.
See FACTS_PLACEMENT_INPUTS_PORT.md and facts-placement-inputs-validation.json.
The extended native fixtures compare full placement output and actual fallback
reports with signed-domain stats, full u32 aim, and caller family overrides.

Ground-object cache inputs and lifetime resolution are now implemented:
assemble_facts_ground_objects and resolve_facts_ground_weapon_pickup share their
recording reducers through consumed-field views. The native oracle covers full
u32 life generations, high MPP ID bits, empty tracks and extreme timestamps.
GroundPadClock.frames is now i64; its shared layer follows native signed frame
clamping and does not suppress zero/negative-frame output. See
FACTS_GROUND_INPUTS_PORT.md and facts-ground-inputs-validation.json.

Cached pad-scan and individual ground-item publication adapters are now complete
at their native function boundaries. build_facts_replay_ground_pads returns
ReplayGroundPads (pads, pickup intervals, coverage, retained weapon objects).
Source and cache share append_ground_pad_chain for count accumulation and pad
index offsets. Persisted scan counters are i64. build_facts_replay_ground_weapons
shares the item matcher and accepts only exact raw taken/swapped change kinds.
See FACTS_PAD_SCANS_PORT.md and facts-pad-scans-validation.json. The final cache
document composer still needs to call date_pad_pickups and attach these layers.

Vehicle census/death, position grouping, heading and sample cache adapters are
now implemented. build_facts_replay_vehicle_lives and
assign_facts_replay_vehicle_deaths preserve the already-selected death slice;
cache assembly does not re-filter archetypes or dead-state flags.
replay_facts_vehicle_positions_by_slot retains complete cache rows, filters world
presence, and stably sorts per slot. replay_facts_vehicle_heading and
build_facts_replay_vehicle_samples share native chassis/velocity math and source
sample publication. See FACTS_VEHICLE_INPUTS_PORT.md and
facts-vehicle-inputs-validation.json for actual validation scope.

Cache spawn indexing, drawable-life selection and track publication are now
implemented via replay_facts_vehicle_spawns_by_life,
replay_facts_vehicle_drawable_lives and build_facts_replay_vehicle_track. Complete
cache creation rows survive earliest-time selection. The dedicated native cache
track oracle checks spawn selection, drawable results and track publication;
additional fixture fields are not assumed validated.

Cached rider aim uses replay_facts_vehicle_aim_by_slot and
build_facts_replay_vehicle_ride_aim with shared source calculations. Written
occupancy rides use build_facts_replay_film_vehicle_rides and the cache-typed
FactsReplayVehicleRideContext; the shared reducer preserves source film provenance,
closure precedence, identity lookup and contradiction indexes. See
FACTS_VEHICLE_INPUTS_PORT.md and facts-vehicle-inputs-validation.json for checks.

Combined event/gap rides are now implemented via build_facts_replay_vehicle_rides.
Private consumed-field event values preserve signed i64 Kind and do not fabricate
source coordinates. Complete cache-event merging and episode helpers share source
reducers. Seat assignment reuses the unchanged VehicleOccupancy type. The native
oracle covers direct/nearest event attribution, geometry, gap fallbacks, explicit
contradictions and seat assignment; fixture log expectations are not implicitly
validated by output comparisons.

build_facts_replay_vehicle_publication now composes cached census, creation,
position, death, occupancy, aim and event inputs. Source/cache share final sorting,
relay merge, coverage and cycle calculation. Its native fixture passes through
encodeVehicleScan/decodeVehicleScan before native assembly and compares Rust cache
decoding and full publication. See FACTS_VEHICLE_INPUTS_PORT.md and validation JSON.

Cache heading-source reporting is now implemented through
replay_facts_vehicle_heading_sources with the shared native log method.

Free-object lifetimes/publication and cached flag marker/carry inputs now share
source reducers. See FACTS_OBJECTIVE_INPUTS_PORT.md for native option precedence
and remaining zone/bomb/flag-gauge document wiring; typed managed/radial readings
are already shared between cache and source.

Next: complete objective/document composition, including flag named-event passes,
explicit cached scan gates, zone/bomb caller options, and vehicle-shot attachment.
Full BuildFromFacts capture/fallback handling and legacy domains remain open.
Native lifetime sorting is ambiguous for equal first timestamps within one slot
(Go map iteration); deterministic full-publication fixtures avoid those ties.
Keep the ambiguity explicit in the final runtime audit.

The vehicle test build exhausted disk on 2026-09-25. After all failed build handles
were terminal, cargo clean --package halo_api --profile dev removed regenerable
package artifacts; sources and fixtures were retained. Validation retries use
CARGO_INCREMENTAL=0. This is a runtime build setting, not a Cargo.toml edit.

Full BuildFromFacts composition, fresh scan capture before assembly fallbacks,
caller option precedence, legacy raw-string/counter reconciliation, final
source/field/runtime and captured-document audit remain incomplete. Keep the
architecture in NEXT_PHASE.md deferred until current v41 parity is complete.

See FACTS_DOCUMENT_ASSEMBLY_NEXT.md for the inspected native pass order and an
order difference in the existing Film wrapper that must not be copied into cached
assembly. attach_facts_replay_vehicle_shots_to_document now implements the explicit
document mutation boundary; full composer wiring remains pending.

## Complete cached document path now connected

`build_facts_replay_document` and `FactsReplayDocumentOptions` now connect the
cached stage helpers in native order. Whole typed documents are compared against
256 native BuildFromFacts outputs, including defaults, empty timelines, supplied
Statborg identity, score/mode gates, caller labels, populated channels, independent
scan flags and once-only fallback accumulation. See `facts-document-validation.json`.
The explicit cached player path preserves supplied identity without running the
convenience builder's completion logic.

Continue with full-composer runtime logging and fallback-trigger order, remaining
caller/legacy byte-string and counter/null domains, fresh scan capture, richer
world/vehicle combinations and captured-film validation. The existing helpers are
no longer merely disconnected document stages, but the connected entry is not yet
proof of complete all-data parity. Deferred architecture work stays deferred.

Shared layer coverage and caller objective counts now use signed i64 with native
wrapping arithmetic. The host/WASM oracle includes full cached documents and
ordered logs; see signed-layer-validation.json. Other legacy domains remain open.

## Actual-context scan snapshot boundary

capture_film_scan_facts implements the native faitsDuBalayage boundary over
already-scanned exact DTOs. It obtains I0Layout from the supplied NativeFilmContext,
returns None for layout errors even with a retained candidate, overwrites only the
capture header's film/module/axis widths/detected status, computes current decoder
coverage and snapshots optional identity and scan-only fallbacks. Nil counter and
an empty counter preserve distinct fallback sections. Extra Statborg/Kills sections
remain unset, matching the native helper.

The native fixture compares complete encoded files for 24 controlled contexts:
eight imposed captures, four detected captures and twelve layout refusals. Later
identity scalar mutation and fallback increments leave captured bytes unchanged.
See scan-capture-validation.json. The source-context fixture and already-scanned
player-input fixture are deliberately independent; this does not prove a full film
scan/capture roundtrip. The fresh scanner still needs to produce those DTOs and
invoke capture before document assembly. Do not mark BuildFromFilmAvecFaits done.
# Loaded source position bridge

`facts_from_world_position_scan` and `facts_from_quantized_position_scan` now
project accepted source positions into exact native facts DTOs. Host and WASM
checks compare 19,484 quantized and 16,914 world observations with the native
scanner fixture. Clippy, format/diff checks and all 485 reference hashes pass.
See `FRESH_SCAN_INTEGRATION.md` for entry order, remaining scanner wiring and the
native cache's documented information omissions. Full fresh entry parity remains
open; this does not start the deferred canonical/resolved/playback redesign.
