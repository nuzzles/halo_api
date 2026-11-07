# Contextual player replay

`TheaterRuntime::resolve_player_replay(map_asset_id, &maps, duration_us)` returns typed
player tracks, source-linked observations, observed life intervals, diagnostics,
and immutable random-access timelines. The adapter exports at the original packet timestamps. This interpretation leaves canonical Film records and the
existing default-context `current()/seek()` world snapshot unchanged.

## Map calibration

`halo_api::theater::film::maps::MapGlossary` owns verified map quantization bounds,
region-index widths, names, and provenance, keyed by the match's map asset UUID.
Its default includes 20 developer-map calibrations and one shared Forge-canvas
calibration, registered against 109 verified asset IDs. `film::maps::known_maps`
exposes `KnownMap` and a constant `MapDefinition` for each calibration, matching
the `film::weapon::known_weapons` organization. Different Forge assets share
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

The contextual v41 resolver uses the supported event-15, quantized event-vector,
and v3 weapon-fire layouts internally. Canonical parsing retains its conservative
reader. Invalid bounds and unsupported regions are refused.

## Admission and gaps

The resolver reuses the sequential v41 packet/entity/component readers. It does
not scan for plausible entity boundaries or use the Octagon pose index.
Completed biped NEWs must contain matching default `player_index` and unit-control
indices before ownership is established. Bootstrap roster identities and bounded
type-8 roster interpretations supply names/XUIDs; type-8 requires the complete
list, explicit unique indices and only zero byte-alignment padding. The latter
is an interpretation of the pinned reference layout, not a newly canonical
packet grammar. Different-XUID reuse of a roster index is currently refused.

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

Incomplete/refused frames clear held state before any supported prefix updates.
Unknown delta ownership never becomes a player pose. Position and aim independently
expire after 100 ms; sampling never interpolates or looks ahead. Exact life-end
markers clear both channels. The same immutable timeline supports chronological
sampling and reverse/random lookup through `PlayerReplay::at`.

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
fire-header interpretation. Both its recorded shooter index and unit reference
must agree with an already established player binding, including generation.
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

Inventory is held causally until a change or life boundary; it has no pose-style
100 ms expiry. An incomplete bound player record or refused roster clears prior
inventory. Unrelated incomplete frames and refused event frames retain known
sparse slots so later player deltas can still resolve against them. Their unread
suffixes might hide a weapon change, so held state across those frames is
provisional. No later keyframe, spawn or shot reconstructs earlier inventory.
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

`PlayerReplaySet.projectiles` exposes partial, source-linked frag, plasma and
spike tracks. A complete archetype-41 NEW must contain a recognized projectile
tag, a recorded owner reference matching a currently bound player biped, and a
supported absolute position. Complete position records for the same full entity
ID extend the track. Unknown tags, unmatched owners, unsupported position forms
and tracks with fewer than two recorded positions are omitted. Dynamo is outside
the requested CSV, though it remains represented in carried grenade counts.

`grenades.csv` samples the tracks on the player sample grid. It uses the most
recent recorded position for at most 100 ms and never interpolates or predicts
an arc. A grenade ID combines the full entity ID and creation timestamp to
distinguish observed reuse. Observed deletion, replacement and refused frames
end visibility; a missing entity suffix contributes no position, while a later
complete position for the same ID can resume a still-fresh track. The exclusive
end bound is not a detonation timestamp. An unread suffix could hide deletion or
reuse, so this is not complete lifetime evidence. Octagon currently has no
decoded projectile creations; its header-only CSV does not establish zero throws.
Oddball provides a source-backed frag/plasma fixture, but spike, bounce,
attachment and detonation still need controlled validation.

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

## Active effects

`camouflage_samples` holds the preceding transmitted `unit-active-camo-state-component`
sub[1] value: 4095 is active, 0 inactive, any other value clears the state to unknown.
A gated-out subchannel is not an observation. `camouflage_observations` retains
raw q and source spans. Only complete records bound to an observed player life
are admitted. Decode gaps and life boundaries clear state with packet order
preserved. Overshield is inferred from the vitality sample's shield value > 1.0,
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
