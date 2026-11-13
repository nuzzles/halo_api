# Recorded rides and current ownership

`runtime.rides(RideSchema::Reference8f0f8d0)` joins the existing attachment and
controller/holder views. It keeps the film / parser/v41 / runtime organization:
Film owns the original bits, the parser reads them, and `runtime/rides.rs`
interprets their relationships under an explicit reference schema.

The reference is libfilmshell
[`8f0f8d078e513c800baa21fac416fd4c0c127d03`](https://github.com/OpenSpartan/libfilmshell/tree/8f0f8d078e513c800baa21fac416fd4c0c127d03),
particularly `src/attach.rs`, `src/vehicles.rs` and `src/assemble.rs`.
Its parent-state reader establishes direct parent/seat relationships; its public
ride assembly also uses copied vehicle positions, fallback exit timing, inferred
vehicle-part associations and aggregated ownership. Those additional hypotheses
do not enter this source-linked view.

## Admission and independent uncertainty

A direct `RideState::Riding` requires an established biped archetype 35, a complete
admitted parent component, a supported Unit handle and an immediate parent with
established vehicle archetype 40. Parent-index ranges or a seat alone cannot
establish either class. Object handles remain unmapped. All generation bits are
retained, and keyframe/runtime identities stay separate.

`RideLink` borrows the rider and vehicle observations and the optional raw seat
word, including its source field. A present seat on a detached component stays
detached. Absence of a seat is not seat zero. Parent node and local offset words
remain available through the observations; they are not world-space poses.

The full `RootPath` retains missing, unresolved, unmapped, cyclic and lifecycle
outcomes. A known direct ride can coexist with an unknown higher ancestor or a
cycle farther up the path. `root_vehicle()` only returns an explicitly detached
terminal vehicle; it never substitutes the immediate vehicle for an unknown
root. There is no four-level depth limit. A lifecycle boundary before the
immediate parent prevents an older child handle from binding to that later
incarnation, even when its raw ID repeats.

`ownership` retains the current controller plus both independent populations,
their source boundaries and resolution status. `participant()` returns only a
unique current human or bot holder. Unknown, ambiguous, deleted and explicitly
uncontrolled states remain distinguishable through `owner_status()`. A known
ride does not require a known participant, and an actor-control occurrence does
not classify the entire interval as AI.

## Queries and source intervals

- `entities()` lists candidate biped subjects, including partial observations
  and separate keyframe/runtime IDs. It is not a list of proven riders.
- `history(entity)` retains relevant rider, ancestor, controller, population and
  gap boundaries. Repeated source updates remain visible.
- `at(entity, timestamp_us)` includes all indexed data through that timestamp
  and the applicable unread packet suffixes.
- `at_event(entity, &event)` stops after the exact borrowed event, excluding
  subsequent same-time data and that packet's later unread suffix. Cloned or
  foreign events return `RideError::ForeignEvent`.

Queries leave Film and playback unchanged. History uses two chronological passes
over the same continuity steps, merging by their private ordinals. This preserves
the position of PacketEnd and untimed chunk gaps, whose referenced envelope
orders cannot safely be sorted with ordinary event orders. The attachment walk
watches the current path and its unresolved endpoint, so ancestor-only updates
can change a ride without a new rider component. Unrelated entity updates do not
emit a ride boundary. Ordinary attachment queries skip touched-entity collection;
ride queries reuse that temporary buffer across steps.

`spans()` pairs each observation with the next source boundary. `riding_spans()`
selects spans whose direct relationship is established, including ones with
unknown owners or roots. These are evidence intervals, not measured durations.
Equal timestamps do not collapse their ordering. An unread chunk can end one
without a packet timestamp. `end: None` means no later boundary was observed;
there is no fabricated film-end timestamp, infinite-life claim or inferred exit.

Deletion, generation changes, partial records, source loss and keyframe resets
can make a ride unresolved. Fresh parent and controller observations restore
only what they establish. Entity-only unread suffixes preserve independent
roster packets; loss that could hide listings requires fresh human and bot lists.

## Verification and remaining scope

Native packet/table/component bodies under an explicitly configured synthetic
registry test direct seats, same-time order, long/cyclic/missing chains, wrong
classes, Object handles, current human/bot ownership, ambiguity, clears, unread
suffixes, transport/sequence loss, deletion and identity domains. Separate,
explicitly labelled model mutations test lifecycle and malformed-component
policy; they do not claim wire NEW/creation coverage. Retained-film audits check
original fields and merged ownership histories. Results and positive-capture
limits are recorded in [VALIDATION.md](VALIDATION.md).

Contextual replay now exposes the separate evidence described below. Vehicle
tracks, full-life attribution and complete canonical field-driven round-trip
proof remain open. This view does not supply
vehicle-center or seated-player positions, proximity-based associations, inferred
exit events, or unrecorded physical simulation context.

## Contextual replay

`runtime/replay/rides.rs` uses the shared attachment schema/value reader with
contextually decoded records. It keeps `ReplaySource` addresses rather than
inventing canonical Events or joining against canonical stops that differ from
the contextual decode. Its graph contains only sequentially observed parent
components. It uses preceding NEW declarations and native keyframe headers for
layout admission; a later snapshot, scanned declaration candidate or inferred
DELTA schema cannot supply a class. It retains the first keyframe namespace used
for replay view zero, rejects duplicate slots in that namespace, and does not
import keyframe parent state as runtime attachment state. A snapshot resets the
layout table without switching namespaces. Layout declarations can persist
across other gaps, as contextual decode context; parent values and ownership
cannot. A full runtime declaration remains generation-specific.

`ReplayDiagnostics::attachments` stores each admitted component once, with its
raw fields, parent mapping, raw optional seat, source and declaration source.
`ReplayDiagnostics::rides` stores ordered path indices, direct ride state, the
full root outcome and the current ownership-observation index. Unknown or
uncontrolled subjects remain in diagnostics even when no player can be selected.
An attached Object handle stays unresolved, and a known direct ride can retain
an unknown or cyclic higher root. A reused parent ID requires fresh child evidence.

Each `PlayerReplay` exposes `ride_observations` (indices into diagnostics.rides)
and `ride_samples`, plus `ride_at(timestamp_us)`. Observations retain source
order at equal timestamps. Samples hold the latest supported diagnostic index;
`observation: None` explicitly clears support after a gap, ownership change or
life bound. This does not mean an exit was observed. Source gaps keep their
original optional clock and separate conservative replay visibility bound.
Summary-death/film-end bounds limit sampling, but do not create ride exit events.
`PlayerReplaySet::ride_observations_in_life(player_index, life_index)` filters
those source-ordered observations to the unique supported life and its full
entity ID. Life ends are exclusive; an equal-clock transfer cannot attribute
the preceding player's observation to the next life. This is the source-backed
counterpart to latest libfilmshell's `life_rides` convenience query, not a list
of inferred physical ride intervals.
`riding_support_in_life` pairs each latest direct Riding sample with the next
sample boundary or exclusive life end. It retains the diagnostic observation
index, excludes zero-length same-clock changes and cannot span a recorded
continuity gap. An interval measures supported replay evidence, not physical
occupancy or a proven exit at its end.
Updates retain both the entity and the opening ownership evidence of their life,
so same-clock respawns cannot admit an old update through a new life, even when
the raw entity ID repeats.

Current unique holders receive observations through the existing replay binding.
A same-clock transfer preserves the former holder's earlier observation and
clears their sampled state, then starts the new holder's support at the transfer.
Missing or conflicting populations, competing bipeds and controller clears do
not select a player. Ancestor updates can change a player's ride evidence without
a new biped update. Unrelated records do not replay cached parent fields.

The attachment graph uses epochs for gap invalidation and watches current path
dependencies; it does not rescan the film for every player or event. Public
observations store diagnostic indices instead of repeated copies of paths' raw
components. Existing pose reconstruction is independent: this integration adds
no vehicle-center position, seated pose, inferred seat or proximity relation.

Authored end-to-end tests use explicitly configured native packet fixtures. Their
synthetic registry lacks a publicly admitted bootstrap, so a separately parsed
no-corruption-check bootstrap is supplied as a **test model policy**. This is not
new build-admission or bootstrap-provenance proof. Lifecycle/error tests label
their separate public-model mutations. Retained-game audits use unmodified
public Film parsing and verify source ranges, raw fields, ownership attribution
and gap references; see VALIDATION.md for counts and limitations.

Vehicle tracks are now available separately through `PlayerReplaySet::vehicles`
and `vehicle_snapshots`; see [VEHICLES.md](VEHICLES.md). They preserve runtime
generations and snapshot namespaces separately and never substitute vehicle-center
coordinates for rider poses. Seat transforms and complete seated pose reconstruction
remain unresolved.

`PlayerReplaySet::ride_carrier_position_at(player_index, time, catalog)` now joins
one uniquely supported life and direct ride to the exact current carrier track.
It returns the source-owning sparse vehicle position observation, optional world
point, direct part, carrier and recorded seat. A fixed/unsupported position read
is returned without a world point. Missing life, ride, carrier, track or position
has a distinct refusal; a track that ended or was replaced after the ride cannot
supply a stale point. The point is the **vehicle center**, not the rider's seated
pose. The query does not add to `PlayerReplay::samples` and uses no proximity or
vehicle-center splice. Synthetic native ride fixtures exercise direct and part
paths, detachment, missing position and track-end refusal. Captured positive
carrier-position joins and seat transforms remain unverified.
