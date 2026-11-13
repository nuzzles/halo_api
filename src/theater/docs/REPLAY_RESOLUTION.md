# Contextual player replay

Recorded movement controls are available through `input_observations`,
`input_samples` and `input_at()`, backed by `diagnostics.inputs` and explicit
input gaps. They retain raw fields and source-time holder evidence independently
of biped lifetimes; see [INPUTS.md](INPUTS.md).

Recorded ride evidence is now available through each player's
`ride_observations`, `ride_samples` and `ride_at()`, backed by
`diagnostics.attachments` and `diagnostics.rides`. It follows current ownership
and full ancestor paths with source-linked uncertainty; it does not supply
vehicle-center or seated-player poses. See [RIDES.md](RIDES.md) for admission,
source ordering, life-bound sampling and verification scope.

`TheaterRuntime::resolve_player_replay(map_asset_id, &maps, duration_us)` returns typed
player tracks, source-linked observations, supported ownership intervals, diagnostics,
and immutable random-access timelines. The adapter exports at the original packet timestamps. This interpretation leaves canonical Film records and the
existing default-context `current()/seek()` world snapshot unchanged.

`TheaterRuntime::scoreboard()` separately exposes source-linked statborg history
from canonical components. Its independent timestamp queries preserve full-width
statistics, raw ownership fields and explicit gaps; see [SCOREBOARD.md](SCOREBOARD.md).
This view does not use contextual replay's schema recovery or assumed event settings.

## Map calibration

`halo_api::theater::film::catalog::maps::MapGlossary` owns verified map quantization bounds,
region-index widths, names, and provenance, keyed by the match's map asset UUID.
Its default includes 20 developer-map calibrations and one shared Forge-canvas
calibration, registered against 109 verified asset IDs. `film::catalog::maps::known_maps`
exposes `KnownMap` and a constant `MapDefinition` for each calibration, matching
the `film::catalog::weapons::known_weapons` organization. Different Forge assets share
`KnownMap::Forge`; Forge edits of a developer map retain its native bounds.
LevelUp lists House of Reckoning (`pve_house`) in its UGC inventory but publishes
no quantization bounds for it, so that asset remains unsupported. Unknown IDs
are not assumed to be Forge maps.

The runtime
selects and validates the calibration from that glossary; callers can register
additional verified `MapDetails` with `insert`. `ReplayError::Map` preserves the
typed `MapError` for unknown IDs, invalid bounds, and unsupported regions.
The origin is derived from the
earliest nonzero parsed replication packet timestamp. This is a relative clock
convention, not independently validated video synchronization. The CLI persists
the map asset ID in its source manifest and reports the selected bounds. Unknown
map assets fail before export. It does not infer map identity from bit widths.
Display labels, camera framing, geometry, and rendering transforms remain with
the consuming application; the bounds here describe coordinate compression.

The two bundled fixtures use map bounds from LevelUp
`d61443ef59268ad734355db8e9974f68db5ca6d0`,
`data/titles/halo_infinite/reference/map_quant_bounds.json`:

- fo09_academy / Octagon: widths 15/15/17.
- Recharge / Oddball: widths 18/18/15.

The contextual v41 resolver uses quantized event-vector and v3 weapon-fire
layouts internally. Script event 15 uses the same recorded version/simulation
admission as canonical parsing; replay no longer supplies an unconditional
prefix assumption. Missing script context, invalid bounds and unsupported
regions are refused. Other contextual event policies remain under audit.

Variant-derived teleport orientation and kill-tail exclusions are shared with
canonical parsing. Replay retains the old disabled-playback and non-engine-1
fallback policies only as explicit hypotheses: `diagnostics.event_assumptions`
records each affected event list's source range, timestamp and assumptions.
Known engine type 1 is never overridden. Canonical parsing stops at an uncertain
kill tail or unavailable teleport context; replay's fallback result does not
modify the stored `Film` or turn those assumptions into recorded fields.

## Admission and gaps

The resolver reuses the sequential v41 packet/entity/component readers. It does
not scan for plausible entity boundaries or use the Octagon pose index.
Completed biped NEWs still require decoder admission and complete defaults.
Ownership comes from the shared v41/27 `unit-control-component` reader and the
current listing-derived holder resolution under `AssignmentSchema::Reference8f0f8d0`.
The default-state `player_index` is not equated with either the controller or the
replay output index. Both human and bot populations must establish a unique holder;
unobserved, incomplete, missing-ID and conflicting populations remain unknown.

Bootstrap identities supply initial output indices only. Replay no longer requires
a usable bootstrap player table when packet listings can establish participants.
Human XUIDs and bot IDs occupy separate `ParticipantId` variants. Reusing an engine
ID does not overwrite a participant or fail the whole replay. `film_index` is a
stable output index, preserving bootstrap indices for initial humans and assigning
fresh indices to later participants. It is never an engine player ID. `xuid` is
optional (None for bots), and `gamertag` is optional for unavailable/invalid UTF-16
names. These labels describe initial identity evidence; canonical roster history
retains later names and original UTF-16 units.

Full entity IDs retain their generations. Deletes, replacement declarations,
and observed ownership changes clear state. Supported absolute positions,
resolved object-relative positions, and desired aim are published from complete
components. Each observation
has the original chunk index, payload byte, and record bit range. Contextual
source ranges are not misrepresented as canonical `SourceRef` indices.

The bounded absolute position branch consumes three 14-bit axes, an optional
object handle, an optional 11-bit region, and the two-bit finite tail. These
coordinates use the fixed +/-100 bounds and a referenced coordinate frame.
Category-0 handles add 0x200 to the slot value and retain the two-bit generation.
Full-state keyframes provide the referenced object's raw position and orientation.
Admission requires a live datum and a complete component walk ending exactly at
the next slot's header. Baselines become available at their source packet, never
before it. Parent transforms, non-default scale, malformed orientation, and later
transform changes remain unresolved until another supported baseline is recorded.
Without a proven transform, the resolver reports
`frame/position/unresolved_bounded_reference`. Neither neighboring player samples
nor a guessed translation can establish a frame.

Schema inference and NEW admission reject masks that address components outside
the declared archetype. An inferred schema also needs a complete known-schema
delta immediately afterward, with a strictly greater slot: the film writer emits
deltas in slot order. An apparent successor at an earlier slot cannot corroborate
the guessed layout, even when both component bodies decode completely.
A conflicting NEW for a different full ID must have
complete decoded defaults, at least one decoded component, and a continuation
that consumes the entity/control views through zero byte-alignment padding.
The continuation runs on a copy of the bindings without publishing observations;
failed validation cannot mutate the live decoder. An opaque, empty declaration
is insufficient even when it happens to align with the packet end. See the
[invalid declaration fixture](../fixtures/movement-schema-v41/README.md).

A NEW rejected by schema admission cannot invalidate runtime ownership, even
when its raw header reuses a live player ID. A delta for an unobserved generation
can invalidate the old life only when the entire packet closes cleanly and
does not also contain a delta for the admitted generation in that slot. Raw
traversal can use the old schema to find boundaries; that permissive traversal
alone does not establish a replacement entity. An unclosed cross-generation
DELETE likewise cannot retire an admitted biped schema or end its life. A
matching DELETE, a closed cross-generation DELETE, or an admitted replacement
still invalidates it.

Incomplete frames retain supported prefix observations, then clear held state at
the unread entity suffix. Refused frames clear state after any separately supported
event headers. No subsequent DELTA or shot inherits a pre-gap player binding.
Unknown delta ownership never becomes a player pose. Position and aim independently
hold their last decoded delta state until an explicit invalidation; silence alone
does not expire a stationary player. Sampling never interpolates or looks ahead. Exact life-end
markers clear both channels. The same immutable timeline supports chronological
sampling and reverse/random lookup through `PlayerReplay::at`.
For playback that should exclude ended lives, `PlayerReplay::life_at(time)`
requires one unique supported life, `sample_at(time)` returns its latest pose,
and `alive_at(time)` reports whether such a sample exists. These use exclusive
life ends even when a source observation at the same timestamp preceded an
unread gap; timestamp-only playback cannot resolve within-clock source order.
`PlayerReplaySet::samples_at(time)` yields those samples in stable player order.
A life may be supported while both position and aim are unknown, so a sample
with empty coordinates differs from no active life. Overlapping or publicly
mutated life intervals refuse the convenience queries; `at(time)` remains the
unfiltered raw timeline lookup. `alive_at` describes supported visibility, not
proof that the game character survived an unread region.

An unchanged position while aiming or firing is an audit candidate, not proof of
missing movement. The stream can omit unchanged transforms. Recorded zero
velocity and repeated equal source positions can corroborate standing still;
held CSV positions alone cannot distinguish that from a decode gap.

Life IDs enumerate observed spawns. Their end is the earliest subsequent spawn,
matching DELETE, ownership change, recovered death summary, or film end. Missing
spawns and summary/packet alignment uncertainty mean these intervals do not prove
complete alive/dead coverage. A missing value is never zero.

Aim uses an approximate desired-direction convention: yaw zero +X toward +Y,
4096 counts/turn; pitch 1024 level, 2048 counts/turn, positive toward +Z. Controlled
turn/pitch clips support direction and monotonicity. Exact quantizer endpoints,
camera transform and firing direction remain distinct unresolved questions.

## Validation and remaining work

Runtime tests cover bounds, unsupported branches, expiry, gaps, death filtering,
same-time respawn, and malformed roster input. Adapter fixture tests check all
100 observed Octagon lives, both identities including the late roster update,
source ranges, corruption of a spawn binding, and equality of sequential versus
random/reverse sampling at fractional Hz. Overlapping source observations agree
with more than 23,000 historical position reads and 20,000 historical aim reads.
That is regression agreement, not an independent golden truth. The historical
bridge is compiled only for tests.

Oddball exercises a different map and all eight players through the same runtime
and CLI path. Its incomplete records and missed spawns remain visible in the
report. Future work includes those unsupported grammars, dynamic roster reuse,
automatic verified map context, independent clock/life validation, predicted and
delta transforms, and exact aim calibration. Complete vitality coverage, powerups,
projectile coverage and complete medal decoding remain unfinished.

The event-vector continuation recovers Nighted Zion's spawn at 1034.948531s
(chunk 52, payload byte 754329, biped record bits 1999..3688). At that stage Oddball had
254 observed lives; ten of the historical 264 were missing: seven behind
unsupported event 36, one behind event 39, and two behind unbound entity deltas.
At 30 Hz, position rows increased from 114601 to 116338 and aim rows from 92290
to 93526. Refused frames fell from 4003 to 3659; many admitted frames still have
an incomplete entity suffix. This is improved coverage, not complete decoding.
See [captured packet evidence](../fixtures/oddball-event-vector-v41.md).

## Firing events

Player tracks also retain `shots`, each with a relative timestamp, full biped ID
and packet/header source range. The resolver reuses the bounded packet-leading
fire-header interpretation. Its recorded engine player ID must match the current
controller, and the unit reference must match the established participant binding,
including generation. The engine ID need not equal the stable replay output index.
Wide unit references use their absolute slot; narrow domain-1 references use the
reference reader's 512 base. Headers are processed before entity updates in the
same packet, so a later NEW cannot retroactively attribute a shot. Lifetime
filtering removes events outside the final observed life (including summary
death cutoffs). Fire-header recovery does not recover a missing spawn or prove
the boundary of the remaining event body.

The CLI's nullable `is_firing` is true for a shot in the trailing sample interval
within the current life, with no carry-forward. Blank means unknown; the partial
event reader cannot establish false. Nested fire headers are also recovered when all preceding event boundaries
are supported, including corrected damage-event boundaries. Unattributed
headers and unsupported preceding events remain gaps. Raw source-linked shot observations in the
report remain independent of the CSV timeline.


## Weapon-fire continuation

The contextual resolver uses the weapon-fire v3 layout described in the
[captured packet evidence](../fixtures/oddball-fire-v41/README.md). It includes
short/long records, variable target/component loops and optional tail fields.
Configuration=0 and unknown/default vector regions remain unsupported.

Oddball increased from 254 to 258 lives with fire continuation alone, and to
261 after resolving the subsequent damage-event conflict. All seven target
spawns are now recovered. Three of the historical 264 remain missing: one behind
event 39 and two behind unbound entity deltas. Octagon remains at 100 lives.

## Corrected damage gate

The separate weapon-damage projection inverted FUN_1407f2058: bit 1 means absent;
bit 0 carries five bits. The existing structural event-0 reader already used
that polarity. LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`,
`grammar/vue_a_charges.go` (`chargeDegatsApres`, D-LN-2), confirms it against the
game reader. Correcting the projection resolves the conflict; the blanket event-0
refusal and nested-fire barrier are removed. Event layouts and bounds still must
be complete before entity continuation. The damage projection still explicitly
reports any synthetic padding; it is never used to establish event boundaries.

Tests cover both absent-flag values with known downstream fields, agreement with
the structural decoder over 2,048 retained legacy inputs, and the three captured
nonmodal packets. Old fixture values after the inverted gate are not golden
truth. CLI tests check each recovered life against its full entity ID and source
NEW range. Following valid damage now applies entity state; truncated damage
cannot do so. This improves replay coverage without establishing health/shield
semantics or complete match decoding.

Verified default-60-Hz exports after this change:

| Fixture | Lives | Player rows | Position rows | Aim rows | Shots (nested) |
|---|---:|---:|---:|---:|---:|
| Octagon | 100 | 52290 | 25799 | 24476 | 570 (23) |
| Oddball | 261 | 577944 | 254395 | 204327 | 3109 (314) |

Shots are raw attributed events; sample rows with `is_firing=true` number 564 and
3097 respectively. Multiple shots can share a sample interval. These numbers do
not establish complete shot coverage or continuous trigger state.


## Weapon inventory and selection

Player tracks now expose `weapon_observations`, `weapon_samples`, and
`weapons_at(timestamp_us)`. Complete, bound biped records supply initial inventory,
pickups/drops and desired selection. Repeated `weapon-state-type-info` components
map to slots by their order in the recorded archetype registry, never by the order
of fields present in a sparse delta. The identity presence gate is the first field;
a nested frame-configuration gate has the same name and must not hide the identity.

Asset IDs combine recorded high32 family and low32 variant. CSV serializes the
full value as `0x` plus 16 hexadecimal digits, not a weapon entity instance or an
API name index. `active_weapon_id` resolves the desired principal slot; draw-animation
completion is unavailable. The selection R3 is a request counter, not a slot.
`secondary_weapon_id` is the sole other carried asset, not the desired off-hand
slot. It requires all four inventory slots to be known. Unknown selection,
dual-wield selection, empty slots and ambiguous alternate inventory remain blank.

Inventory is held causally until a change, life boundary or continuity gap.
Incomplete bound player records, unread entity suffixes, refused frames/rosters,
keyframe resets and source loss clear prior inventory. A sparse channel can have
an omitted change in unread source, so it cannot survive that gap.
No later keyframe, spawn or shot reconstructs earlier inventory.
Gap-only or selection-only updates cannot restore unknown asset IDs. Packet
ordering is preserved for equal clocks, and summary death cutoffs filter source
observations by full generation.

Validation covers registry-ranked sparse identity, nested presence gates,
controlled switch excerpts, pickups, drops, ambiguous/dual-wield selection,
coincident gaps, deaths, generations, fractional-Hz CSV sampling and 64-bit ID
formatting. The original Octagon fixture exercises source ranges and both
sequential and reverse/random lookup. See the
[controlled evidence](../fixtures/weapon-selection-v41/README.md).

Verified 60 Hz weapon coverage (rows, not complete-match coverage):

| Fixture | Player rows | Active weapon rows | Secondary weapon rows |
|---|---:|---:|---:|
| Octagon | 52290 | 431 | 428 |
| Oddball | 577944 | 306 | 31 |

Position, aim, observed lives and firing-row counts remain unchanged. Stronger
weapon guarantees require recovering the unread parts of refused frames.


## Carried grenade counts

`grenade_observations`, `grenade_samples` and `grenades_at(timestamp_us)` expose
source-linked carried counts from complete, bound biped records. The existing
`unit-grenade-counts-component` grammar reads a three-bit length followed by byte
counts. Exactly four entries are admitted in rank order: frag, plasma, dynamo,
spike. No throw-event subtraction, default spawn inventory or future keyframe is
used. The Twin player CSV exports all four carried counts.

An observed zero is exported as `0`; missing data remains blank. Counts are held
until the next update, life boundary or decode gap. The same partial-prefix
policy as weapons applies. Equal-clock packet order, summary death cutoffs and
full entity generations constrain the timeline. Unsupported array shapes clear
previous counts.

The resolver uses a conservative per-type admission limit of 2 for these
supported maps. Raw byte values are
preserved in `raw_counts`; values exceeding the limit are not clamped or exported.
They clear the timeline and increment `refused_grenade_counts`.
Oddball contains two such suspicious readings; their underlying decoding cause
is still unresolved. See [captured evidence](../fixtures/grenade-counts-v41/README.md).

Validation includes source bytes, nonzero counts, zero versus unknown, the fourth
slot mapping, configurable limits, truncated fields, gaps, deaths, entity reuse,
random/reverse lookup and fractional-Hz CSV sampling. Carried counts do not prove
complete inventory coverage or establish when a grenade was thrown.

## Grenade projectile positions

`PlayerReplaySet.projectiles` exposes partial, source-linked frag, plasma,
dynamo, spike and rocket tracks. A complete archetype-41 NEW must contain a
recognized projectile tag, a recorded owner reference matching a currently bound
player biped, and a supported absolute position. Complete position records for
the same full entity ID extend the track. Unknown tags, unmatched owners and
unsupported position forms are not admitted as tracked projectiles; their raw
canonical records remain available.

Tracks retain every admitted source position, including the creation's only
position, repeated timestamps and a zero-length support interval. Equal-clock
queries select the last observation in source order, while all earlier reads
remain inspectable. A 100 ms sampling age limit does not establish deletion:
a later complete sparse update can supply a fresh sample after that age expires.
Queries remain unknown between the old sample's age limit and the later update;
they never interpolate or backfill that interval.

`grenades.csv` samples tracks on the player sample grid using the most recent
recorded position for at most 100 ms. Singleton tracks can contribute a bounded
sample but do not establish an arc or continuous flight. A grenade ID combines
the full entity ID and creation timestamp to distinguish observed reuse.
Deletion, replacement, refused records and general source gaps close support.
An ordinary DELTA cannot restart a track after a source gap; a new complete
creation and current owner evidence are required. The exclusive end bound is
not a detonation timestamp. Zero-length intervals remain in history but never
produce a sampled position. The `deleted` flag indicates a deletion ending a
still-fresh supported track; raw delete records remain independently available.

Octagon's historical absence of decoded projectile creations does not establish
zero throws. Prior Oddball frag/plasma evidence and exports below are historical
checkpoints; current captured source counts are in VALIDATION.md. Spike, bounce,
attachment and complete detonation/lifetime behavior still need controlled
validation. Keeping more observations does not resolve unknown owner, tag,
position, source-gap or projectile-to-shot relationships.

Verified default-60-Hz exports after grenade admission:

| Fixture | Raw count observations | Rejected observations | CSV rows with all three requested counts |
|---|---:|---:|---:|
| Octagon | 100 | 0 | 431 |
| Oddball | 588 | 2 | 631 |

Octagon admitted counts are all zero. Oddball has 534 sampled rows with nonzero
frag and 118 with nonzero plasma counts. Neither fixture recovers a nonzero spike
count. These counts do not establish complete-match inventory coverage. Existing
position, aim, life, firing and weapon-ID coverage remains unchanged.

## Health and shields

`PlayerReplay::vitality_at` and `vitality_samples` expose independently held health
and shield scalars. Only complete, owned, sequentially located biped records are
admitted. A missing component leaves its own previous channel unchanged; invalid
values clear that channel. Decode gaps and life boundaries clear both. Coincident
observations and gaps preserve packet order. Spawn defaults do not yet establish
initial vitality. We never assume full health, simulate regeneration or backfill
from future observations. The same partial-prefix limitation as weapon inventory
applies: supported prefix readings do not prove an unsupported suffix had no change.

The endpoint-exact float32 quantizer follows LevelUp commit
[`413971bcc485be53a2fa8cf229e170539a5146f8`](https://github.com/JGtm/LevelUp/tree/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar),
`vitality.go` and `quantize_endpoint.go`. That reference attributes body to
`FUN_140fb8978` and shields to `FUN_140d50cbc`; this is source comparison plus
fixture validation, not an independent disassembly audit.

- Body: 8-bit q plus three flags. Range [-1, 1], 255 levels, exact endpoints
  and exact q127 midpoint. q254 = 1. q255 is refused. Negative decoded values
  have unverified meaning, remain in observation evidence, and export blank.
- Shields: 8-bit q over [0, 4], 256 levels, exact endpoints. q64 = 1, q0 = 0.
  Preserve overfill; `is_overshield_active` uses the measured q > 64 threshold.
- Shield tails contain a regeneration gate, two independently optional 12-bit
  words when gated, a 16-bit word, and four flags. Lengths are 29/31/43/55 bits.
  These fields are retained without assigning a regeneration model.

CSV values use 1 as the **normal full reference**, not 100. These are serialized
scalars, not proven percentages of a custom mode's maximum. Maximum-vitality
component fields are retained as `MaximumModifiers` observations, but their HUD
normalization semantics are unresolved. The Twin saturates its normal shield bar
at 1 and retains unknowns as gray dashes. The Overshield column uses the measured shield-overfill threshold; this is not a separate activation flag and custom shield maxima can affect its meaning.

Captured component fixtures cover damage, depletion, recovery and full endpoints;
synthetic tests separately exercise all optional regeneration branches. Full
Octagon regression rereads every observation field from retained packet bytes and
checks random/reverse timeline lookup against fractional-Hz sampling. As of this
change, 60 Hz Octagon has 235 known health rows and 12,628 known shield rows out of
52,290; Oddball has 1,548 and 106,507 out of 577,944. Sparse health coverage and
unknown initial values are still substantial gaps, not proof of full HUD parity.

### Vitality coverage blockers

`ReplayDiagnostics::blockers` groups contextual keyframe stops, refused frames
and incomplete entity frames. Each entry retains its count, first relative
timestamp (or null before the origin), and first source cursor. The cursor is a
zero-width bit range within the original packet payload, not an accepted record
span. Diagnostic reason strings are not a stable enum. Counts measure parser
stops, not missing vitality events; collecting them does not change admission,
gap clearing or sampling.

All 22 Octagon keyframes stop at archetype 34 component 10,
`tacmap-mapdismissallock`, before traversal reaches player archetypes. The first
cursor is payload bit 2279. The pinned LevelUp reference's
`testdata/ecs_table.tsv` also marks this component `non_porte`, without a
deserializer address or width. Keyframe guards describe memory sizes, not wire
bit lengths, so they cannot safely skip this barrier. A validated layout is
needed before later keyframe barriers and initial player vitality can be tested.

For player replay only, repeated keyframe headers can supply a missing entity
schema without decoding the blocked component. A candidate must match the
paired type-1 datum table's live slot and generation, fit the keyframe's
increasing slot order, and recur with the same full ID and archetype in the
next keyframe. This permits later frame deltas to decode; it does not turn a
keyframe into a spawn, position, or vitality observation.

Octagon has 1,613 frames with rejected delta headers, 89 refused event lists and
one truncated entity frame. The refused event lists read code 9 and later stop
at an unsupported reference labelled code 116; the latter boundary is unresolved,
not proof that the stream actually contains event 116. Unknown suffixes may
contain player changes, so gap clearing remains necessary. No additional health
or shield readings were recovered by this investigation.

## Held equipment selection

`equipment_observations`, `equipment_samples` and `PlayerReplay::equipment_at`
expose `biped-desired-ability-set-component` (also accepting its unsuffixed
registry name). The existing parser reads a three-bit request counter and a
one-bit gate: zero introduces the six-bit match-palette rank, one explicitly
clears the selection. The counter is not an equipment ID. Rank zero remains a
rank. Only complete sequential records of currently owned bipeds are admitted;
there is no bit scanning, proximity ownership or future-state backfill.

The timeline distinguishes unknown, empty selection and recorded rank. It clears
at decode gaps, deaths, respawns and ownership changes, preserves coincident
packet order, and uses the same supported-prefix limitation as weapons.
Malformed complete observations clear selection rather than holding stale state.
The CSV column `player_equipment` includes recorded selections and generic
inventory references. It does not classify those references as powerups.

CSV exports a recorded desired-ability rank as `ability_rank:N` without
assigning a potentially incorrect equipment name. Explicitly empty selection
alone does not prove the inventory is empty. Ability-rank interpretation uses
LevelUp `internal/grammar/ability_rank.go` and `equipment_changes.go`;
its statistical palette classifier and recovery scans are not used. A rank
alone never sets a powerup activation flag.

The complementary `Inventory` channel reads `unit-equipment-component`: a
three-bit head (retained without assigning selection semantics), a three-bit
count, then optional category-0 handles (presence + 13-bit value + two-bit
generation). The reference base is 512, per LevelUp `equipement_porte_lecteurs.go`.
The full reference, including generation, joins only a previously complete
archetype-37 NEW's `mpp.object_tag`. That tag's original source is retained.
Global decode gaps discard identity mappings; NEW/DELETE and generation reuse
invalidate the affected carried state. No object proximity or creator-index
heuristic establishes ownership.

When no desired rank is known, `player_equipment` exports generic inventory
references as semicolon-separated `equipment_tag:0xXXXXXXXX` or unresolved
`equipment_entity:0xXXXXXXXX` values. Their object-tag names remain unmapped.
An explicitly empty inventory exports `none`; unknown inventory stays blank. A later object
creation does not backfill an earlier reference or row. Source regression slices
for inventory and equipment defaults are in `../fixtures/equipment-v41/`; they
establish bytes and joins, not independent HUD-name validation.

In particular, `0xa53cd143` is not a verified powerup: LevelUp's
`.ai/V7.5/replay2d/PLAN_R3_IDENTITE_TI37.md` at `413971b`, phase 2(d), reports it
in a negative-control film without player equipment and describes it as a generic
world item. Its exact name remains unresolved. Octagon and Oddball currently have
raw inventory evidence but no verified powerup identity exported.

## Equipment energy and delay

`energy_observations` indexes source-time attributed scalar readings in
`ReplayDiagnostics::equipment_energy`. Every contextual candidate remains in
diagnostics, including errors, partial prefixes, unassigned readings and separate
keyframe observations. Candidate `ownership` is distinct from admitted
`participant`; stale support after a gap retains evidence without attribution.

`energy_samples` and `PlayerReplay::energy_at` retain all currently supported
equipment objects. Energy and delay independently reference their original
readings, alongside current ownership and attachment-path indices. A later holder
can receive supported object state without rewriting earlier source attribution.
Empty samples mean no supported readings, not proven empty inventory. There is
no arbitrary selection among multiple objects or implicit scalar default.

The channel shares native source-byte/schema validation and the unchanged
libfilmshell scalar oracle. Energy remains a fourteen-bit word with optional
reference fraction conversion; delay remains a ten-bit word with no established
physical time unit. NEW/DELETE, generation, partial records, source/life gaps,
opaque packets and unread control suffixes interrupt support. Fresh direct
equipment-to-biped parent and controller evidence is required after gaps.
Untimed packet boundaries use the preceding envelope's support bound while
retaining an absent observation clock. Snapshot reads never seed runtime values.
See [EQUIPMENT.md](EQUIPMENT.md) for API and evidence limits.

## Active effects

`camouflage_samples` holds the preceding transmitted `unit-active-camo-state-component`
sub[1] value: 4095 is active, 0 inactive, any other value clears the state to unknown.
A gated-out subchannel is not an observation. `camouflage_observations` retains
raw q and source spans. Only complete records bound to an observed player life
are admitted. Unreadable records, unread entity suffixes, source loss and life
boundaries clear state with packet order preserved. Sparse camouflage and inventory
cannot survive a gap that might hide a change. Overshield is inferred from the vitality sample's shield value > 1.0,
which is equivalent to raw q > 64. Both follow LevelUp 413971b; neither supplies
spawn defaults or claims exact activation/pickup timestamps.

## Projectile events and player continuation

Projectile detonation (event 5) and impact (event 6) have variable payloads.
Their weapon/effect gates and direction branches must be consumed before the
next event. Positions use map-derived quantization widths at precision levels
15 and 12, not the player position's level 16. Widths are recomputed from the
bounds before applying the quantization cap. Missing map context or an
unsupported region refuses the event rather than skipping a fixed bit count.

The captured Argyle detonation at 811.025027s ends at bit 140, followed by damage,
kill, and damage-aftermath events. The entity list starts at bit 535 and includes
adarkfortress's dead-state and control detachment. The previous fixed-length skip
hid that record and retained the previous life until its respawn at 818.938234s.
The regression tests both the complete event list and the resulting life gap.
See `fixtures/movement-schema-v41/argyle-detonation.bin` for the bounded capture.

Detonation events (code 5) are also exposed as `PlayerReplaySet::detonations`,
using the complete sequential event body and map quantization at precision 15.
They retain source spans and do not depend on a projectile surviving every movement
update. Weapon identity is not inferred from proximity to another tracked object.
Team observations must fit the managed-player schema mask before updating durable
team state; traversable out-of-schema bits cannot establish a team change.

### Live Fire secondary-region packet alignment

Live Fire's region 3 absolute positions use a verified 54-bit aggregate payload.
The region 1 map bounds supply only 35 coordinate bits and must not be reused for
that payload. Doing so misaligned later records and manufactured a team 6 update
for FunkyK1ngCold at 291.926900s in KOTH Live Fire. The contextual map calibration
now preserves region 3 as opaque position bits: following records can be decoded,
but unknown per-axis bounds cannot produce player or object coordinates. See
[`live-fire-region-3`](../fixtures/live-fire-region-3/README.md) for captured packet
boundaries and the 2,250-command-tick cross-check. The runtime mask guard remains
defense in depth; the corrected packet contains no team update at all.

### Weapon charge, heat and defaults

Complete `weapon-state-rounds-inventory` components now add
`WeaponChange::Reserve { slot, rounds }` to `weapon_observations`. The eleven-bit
unsigned count uses its own component-family rank among exactly four registry
rows. Zero and repeated values are retained; absent components provide no new
value. Complete optional corruption-check trailers are accepted. Partial records
and invalid slot layouts do not produce reserve observations.

`WeaponSample::active_weapon_reserve` and `secondary_weapon_reserve` follow the
existing sparse magazine convention: observations at that time are published,
and retained slot readings are republished when a weapon becomes visible after
selection/identity changes. A missing sample value is not zero or a new count.
All four slots remain inspectable through source observations even when the
two-weapon presentation cannot select an active/secondary asset. Reserve state
is independent of magazine, charge, heat and asset identity. Life and decoding
gaps clear it; no capacity-derived or empty-slot reserve baseline is invented.
Opaque non-frame bodies clear reserve support together with all other held
replay channels and ownership. `diagnostics.reserve_gaps` additionally records
packet boundaries that interrupted live reserve values; the complete boundary
list is in `diagnostics.gaps`. Untimed packets retain no timestamp and use the
preceding envelope's exclusive support bound. `reserve_slots`
on every weapon sample exposes the latest supported readings for all four slots:
`None` explicitly means unknown, so a support boundary is distinguishable
from no new sparse active/secondary reading. Later source observations can
re-establish support, including after a gap at the same clock.
The [reference sequence](../fixtures/reserves-v41/README.md) executes unchanged
component and ammo readers. Captured audits check original eleven source bits
and packet timestamps. This adds no canonical reader or life/owner inference.

Complete `weapon-state-ammo` records publish magazine and charge independently:
closed gates encode zero (native FUN_140ea1018 explicitly assigns zero), whereas
an absent component supplies no observation. The two forms must not be conflated.
`WeaponSample` preserves the 12-bit consumed charge code and `WeaponHeat` preserves the
7-bit heat code plus both flags in wire order. Charge also resolves the initial zero baseline described below; other values come from wire readings.
`WeaponCharge::reference_fraction()` and `WeaponHeat::reference_fraction()` now
reproduce libfilmshell 8f0f8d0's exact endpoint-aware f64 conversion; their
`reference_percent()` methods apply its lossy rounding. Interior values are bin
midpoints using 4,094 or 126 interior steps, not the approximate `q / max` law.
Out-of-range caller-constructed words return None. The original raw fields remain
unchanged. `WeaponSample::active_weapon_charge()` / `secondary_weapon_charge()`
wrap only the existing sparse sample values; `WeaponChange::charge()` wraps
transmitted charge observations and excludes empty-slot baseline variants.

`WeaponHeat::raw_flags()` reproduces the unsigned two-bit word in wire order.
Flag gameplay meanings and bit-exact engine f32 arithmetic remain unproven.
The charge conversion is not inverted to remaining battery or treated as an
ammunition count. An exhaustive unchanged-reader
[oracle](../fixtures/weapon-quantization-v41/README.md) covers all 4,096 charge
codes, 128 heat codes and all four flag combinations. Native gate-zero semantics
remain distinct from upstream's absent fraction and from an omitted component.
In the bundled KOTH Streets capture, KiWi
DiPSTiC has consumed-charge codes 254, 508 and 762 on shots at 93.576972,
94.043936 and 94.511231 seconds. Between the first two shots, heat falls
from 21 to 20 without a charge update.

Wire semantics reference: LevelUp `grammar/unit_weaponstate.go` at commit
`413971bcc485be53a2fa8cf229e170539a5146f8`, documenting the native assignments
at `140ea1129` (magazine zero) and `140ea10af` (fraction zero).

Readings are associated with all four registry-defined slots, apply after slot
identity changes, and publish at their observation timestamps. Readings retained
in a slot are also republished when selection changes or an objective drop makes
that slot visible again; a repeated selection stays sparse. An objective hiding
ordinary weapons from the two-weapon view does not discard their inventory state.

A complete biped NEW that explicitly declares an empty or Unarmed slot establishes
its initial zero magazine and zero consumed charge (provided no nonzero ammo
component contradicts it). The first pickup can omit those unchanged zeros.
`EmptyAmmoBaseline` records this separately from transmitted `Ammo`/`Charge`
readings; the pickup publishes zero in the resolved sample. An explicit pickup
reading overrides it. Identity, ammo and heat are independent delta components: an identity-only
update preserves recorded slot values, including drops/replacements. Only explicit
ammo updates or life/decoding gaps replace or invalidate those values. The initial
baseline is never recreated by a later drop, replacement, or decoding gap.
Slayer Streets verifies fresh Stalker pickups at 80.916552 and 285.875348 seconds,
their first 254-code shots, and a partly used pickup with code 1016 at 112.986021.
Record gaps and life boundaries clear prior state. `weapon::WeaponDefaults`
provides known standard ammo kinds and consumption. Magazine capacity is metadata,
not evidence that a pickup or reload supplied a full magazine. Observed counts
and game-mode respawn configuration take precedence; unknown variants stay unknown.

The Slayer Live Fire recording demonstrates empty-magazine pickups rather than
capacity defaults: ElMariachi S128 picks up an empty sniper at 245.280315s,
reloads one round at 247.381631s (no shot event), then fires it at 248.983995s,
returning to zero. KOTH Lattice likewise reloads a picked-up empty sniper to
three rounds at 82.512451s. Do not replace these initial zeros with magazine
capacity. Oddball Streets verifies the explicit Unarmed baseline for JE TRICHEEEE
at 382.557002s and CadillacDrac at 781.452837s.

An identity-only replacement is not an ammo reset. In Oddball Lattice, Furturn's
slot 1 records 20 rounds at 270.210299s with the Commando, then changes to Vestige
Carbine at 275.448232s without an ammo component. Preserve the 20-round slot
state. The native biped schema even serializes ammo before type-info; clearing
ammo when interpreting type-info can erase the same record's valid reading.
Regression tests cover independent identity, magazine, charge and heat deltas,
explicit used-pickup readings, objective visibility and reset-on-gap behavior.

### Runtime schema recovery

Contextual replay explicitly opts into `replay/schema_recovery.rs` for unbound
DELTA schemas and for a DELTA whose full ID has a different generation from the
slot's recorded binding. The latter can follow an unread NEW with a different
archetype; if no unique candidate is found, replay retains its existing slot
traversal and source-gap behavior. Canonical Film parsing leaves this hook absent
and never promotes a recovered schema to a recorded declaration. Recovery still requires a unique complete
candidate and a complete later DELTA with an increasing slot, a non-soft schema
and a compatible generation; apparent packet closure alone is insufficient.
`ReplayDiagnostics.schema_recoveries` exposes each hypothesis's full ID, proposed
archetype, timestamp and the candidate/successor source extents. These are runtime
hypotheses, not canonical declarations, ownership, or complete format coverage.
Later recorded declarations supersede inferred bindings. Existing generation-
reuse, earlier-successor and ambiguous-candidate checks remain in place.

`ReplayDiagnostics.assumed_creations` separately lists contextual NEW reads in
packets whose event grammar used an unrecorded assumption. Each entry retains
the full ID, archetype, exact packet/bit source, record stop, frame closure and
assumptions. It is not a canonical Film declaration. In the retained long raid,
the `KillPlaybackDisabled` assumption reaches a complete generation-2 type-42
NEW for slot 1333 in chunk 140, packet 1182; the canonical packet refuses its
event list. This is independently comparable with libfilmshell's resumed entity
walk without making the skipped kill-playback branch source-backed.

## Source continuity and visibility bounds

`ReplayDiagnostics::gaps` records source-ordered `ReplayGap` values separately
from decoder blocker counts. Each gap identifies the manifest
`chunk_index`, optional packet index, optional relative recorded packet clock,
reason and `visibility_end_us`. Contextual entity stops come from the contextual
reader; the more conservative canonical packet body is not substituted for it.

Structural gaps include missing/unknown chunk-index continuity, unread transport
or trailing bytes, packetless nonempty/incomplete chunks, and keyframe boundaries.
Complete empty clear/zlib chunks preserve known index continuity. Keyframes clear
old runtime ownership and reference transforms before their available baselines.
An incomplete keyframe also invalidates those transforms after its readable prefix.
All nonzero packet clocks must be monotonic in source order; replay refuses an
unreconstructed clock reversal rather than sorting it into a different history.

A gap clears pose, camouflage, weapons/ammo/heat, grenades, vitality, equipment,
team bindings, reference transforms and projectile tracks. It also removes player
ownership bindings. Source loss that could hide a roster clears available player
assignments; a complete subsequent roster restores only population knowledge.
A fresh complete controller can restore ownership of an admitted biped when the
current populations establish a unique holder. After source loss, both populations
also need fresh evidence. A later listing can resolve a previously observed
controller without backdating the resulting ownership interval. These rules may
reduce coverage compared with output that bridged unread source or borrowed a
first-later identity.

Unread non-frame bodies (including types 6/10) and unread control suffixes now
use that same general boundary handling. A control suffix is applied after its
readable entity/control prefix; when both sections are partial, the packet adds
one general `UnreadFrame` gap. Opaque packets retain their original packet
address and optional clock. This is an end to supported reconstruction, not an
observed death, destruction or physical state change. Prior raw fields and
source observations remain available, but later sparse player fields cannot
reuse the old owner without fresh controller/holder support.

Every gap remains in diagnostics. Repeated gaps without intervening supported
state do not append redundant player clears or ride transitions. Inputs can
resume independently of a biped life and still clear at the next gap. Population
preservation continues to follow the explicit reason-specific roster policy;
this change does not invent new roster observations or map unknown packet bodies.
Captured ownership intervals are checked against every general gap, and authored
tests distinguish same-clock restoration from unsupported continuity.

`LifeEnd::ContinuityGap` means the end of supported visibility, not a death or
DELETE. For untimed source loss, the last preceding packet clock (or origin for
leading loss) conservatively bounds visibility; `timestamp_us` remains absent.
Manifest offsets never supply a packet clock. A packet-only timestamp query cannot
separate a readable prefix from its unread suffix at the same clock, so the suffix
wins. Prefix observations, including a spawn immediately followed by a gap, remain
inspectable even when the supported visibility interval is empty. A later supported
controller/holder observation at that same clock can restore ownership in source
order; pose and other held channels still require new observations. No interpolation
is added.

These changes concern contextual replay safety, not proof of complete film decoding.
Canonical bytes/records and `film / parser/v41 / runtime` ownership are unchanged.
Complete-life reconstruction, ride projections and unread format regions remain
tracked in LIBFILMSHELL_ROBUSTNESS.md.

## Controller and holder evidence in replay

`diagnostics.ownership` retains controller/holder reevaluations, including repeated
control observations, population changes, refusals and competing biped claims.
Each entry has its triggering `ReplaySource`, original controller source, current
human/bot listing sources and the uniquely resolving assignment record, if any.
`ReplaySource` adds manifest chunk index and packet index to the contextual
byte/bit span. These sources remain separate from canonical Event/Record indices.
Missing or malformed control is Unknown; an explicit absent controller is
Uncontrolled. A unique holder is not selected when two observed bipeds compete for
the same engine controller, because this projection cannot pick one player pose.

`PlayerLife::ownership_observation` indexes the evidence that opened that interval.
`start_reason` distinguishes `LifeStart::Spawn` (an admitted complete NEW at that
boundary) from `LifeStart::Ownership` (a later controller/listing establishes
ownership). The latter does not imply a spawn, resurrection or earlier lifetime.
The existing name PlayerLife is retained for API organization, but these are
supported control intervals, not proof of a player's entire biological life.
An ownership transfer closes held state without copying the old holder's pose,
inventory or vitality. Same-clock earlier observations remain available while
sampled state follows the actual boundary order. Controller clears end ownership
without claiming a death. Observed DELETE and summary-death reasons stay distinct.

Fire attribution checks the current engine controller and full-generation biped
before publishing the stable output index. Projectile owners use that same current
binding. Team entity defaults resolve through current holder assignments; after a
holder change, a fresh team value is required rather than carrying the old one.
Reference transforms, schema recovery, event assumptions, canonical Film bytes and
packet/entity readers otherwise keep their existing responsibilities.

## Vehicle position evidence

`PlayerReplaySet::vehicles` retains sequential class40 runtime tracks with
source-linked positions, optional current-NEW tags and explicit support boundaries.
`vehicle_snapshots` retains separate namespace observations. Unknown encodings
clear held coordinates; source gaps and lifecycle changes close tracks.
`position_at` holds sparse state only within its support interval. No tag backfill,
interpolation or rider-pose substitution is added. See [VEHICLES.md](VEHICLES.md)
for schema/position admission, public API semantics and validation limits.

`PlayerReplaySet::ride_carrier(ride_index, Catalog)`
classifies a direct vehicle-class parent from its recorded tag. If the tag is a
catalogued part, it requires the recorded attachment path to reach a detached
class-40 carrier and a supported full-ID vehicle track whose tag matches the
part's `part_of` tag (or a catalogued descendant). The returned seat still
belongs to the part. Missing tracks, tags, attachment links and mismatched
carriers have distinct errors. This does not use the upstream snapshot-proximity
join, so rides in parts without a recorded carrier path remain unresolved.
