# Unit-controller observations and ownership continuity

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


`TheaterRuntime::controllers()` reads canonical `unit-control-component` fields
within the existing film / parser/v41 / runtime organization. The interpretation
follows libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`, under v41/27 and
recorded row parameters 1/2. The retained Octagon registry records parameter 2.
Unknown versions, unbound archetypes, duplicate component rows and mismatched
registry rows refuse attribution. No biped slot-range heuristic supplies a type.

`observations()` retains every named component occurrence: baselines, updates,
repetitions, explicit clears and partial/unsupported candidates. Each observation
borrows the original Event, raw entity ID, component fields and completion status.
The runtime and keyframe identity variants remain separate: keyframe high bits
identify namespaces, not runtime generations. No baseline sample invents a spawn.

`observation.state()` exposes the controlling-player presence bit, optional
five-bit player, optional secondary six-bit word and independent optional 32-bit
reference. All values retain their canonical source fields. The last two fields'
roles remain unestablished; neither supplies another owner or an entity link.
An absent controller is different from a missing component or incomplete read.
A partial component cannot publish a clear even if its presence-bit prefix is
readable. Known corruption-check trailers are validated and remain in `fields`.

`observation.holder(AssignmentSchema::Reference8f0f8d0)` links a present player
to the listing-derived holder view at that exact event. Later same-timestamp
listings cannot backdate identity. Missing/ambiguous populations remain in the
returned HolderResolution; absent control returns None. The interpretation does
not equate bootstrap order with engine IDs, infer a takeover or select the first
later assignment. Queries preserve canonical Film and playback position.

`history(ControllerEntity)` includes matching component observations and lifecycle
boundaries: a NEW/keyframe without a controller component, complete/incomplete
deletions, partial records without readable control, and records naming another
generation of the same runtime slot. Other-generation records do not themselves
prove replacement. Unrelated complete DELTAs do not replay cached values, and
identical controller observations are not deduplicated. Runtime/keyframe histories
never merge solely because their raw numbers match.

The direct `history` remains a record-only view. For controller-holder state
through packet gaps and population changes, use the ownership queries below.
A controller clear is not inherently a recorded death. Upstream `owners::reader`
retains only the first controller/end per generated slot; the local views retain
later changes without assigning the first controller to an entire life.

## Ownership history and queries

`controllers.ownership(entity, assignment_schema)` performs one event-stream pass
and returns evidence after every relevant controller record, population listing
and continuity boundary. `ownership_at(entity, schema, timestamp_us)` returns
the last evidence through that time, including unread suffixes.
`ownership_at_event(entity, schema, &event)` requires an actual borrowed event and
excludes later same-time listings and the current packet's unread suffix. None
means the prefix contains no relevant evidence or boundary. Queries preserve
Film, source pointers and playback position.

Each entry exposes the direct controller evidence, triggering cause, current
session/bot listings and optional HolderResolution. Status distinguishes
Unobserved, Unresolved, Deleted, Uncontrolled and the holder's Unique, Unassigned,
Ambiguous or Unknown result. A listing can change the current holder even when
the controlling engine ID is unchanged; its source does not overwrite the earlier
controller observation or retroactively rename earlier entries. In contrast,
`observation.holder(schema)` always describes listings at the original component
event, so the two queries can correctly return different identities.

NEW without a controller, partial/unbound/duplicate components, incomplete
records/deletes and another generation invalidate the previous controller. A
complete controller value can reestablish observed control, but no skipped update
is reconstructed. Keyframe packets invalidate accumulated state and seed only
the separate Keyframe domain. A later NEW with the same generation remains a
separate observation; spans are never merged across lifecycle boundaries.
No biped slot range, baseline position or missing controller implies a life,
spawn, death or owner.

A shared private `runtime/continuity` visitor now supplies attachment and
controller queries with the same boundary order. Unread entity regions close
control after their readable prefix; opaque/refused entity packets invalidate
at their envelope. Entity-only gaps retain independently framed populations.
Unread chunk suffixes, incomplete decompression after known packets and source
sequence discontinuities may hide listings, so they invalidate both populations
as well as control. `population_gap` retains that boundary until both domains
have a new listing. A missing/malformed replacement list cannot become empty or
unique, and rereading control alone cannot reuse pre-gap population identity.

Unread chunks that produce no packet event now create an explicit
`ControllerBoundary::UnframedChunk`, and invalidate controller state and both
population domains. The boundary borrows the original chunk; `event()` returns
None and `unframed_chunk()` supplies its source bytes, transport and metadata.
Complete empty clear/zlib chunks remain distinct from unread chunks. Attachment
unknown evidence now uses this same source-boundary representation.

Source order places an untimed gap after the preceding known prefix, before the
following source data. If source clocks reverse, the earlier bounding event
index is used conservatively. No timestamp is assigned to the chunk, and no
manifest offset substitutes for the packet clock. A timestamp query therefore
reports uncertainty through that interval, while an exact query in the preceding
readable prefix can still expose its controller. Full histories include leading,
trailing and adjacent unframed chunks, including recordings with no packet events.
Computing these source bounds requires a full event-index pass only when such
chunks exist. Fresh controller and human/bot listings can establish later state.

`ownership.spans()` pairs every entry, including uncertainty and repeats, with
its next boundary. `end: None` means no further boundary was recorded. A
PacketEnd boundary references the packet envelope but follows its readable
records: use vector order, not a resort by the referenced event's order.
No synthetic film end, duration, first-later identity or inferred death is added.
These are source-linked controller-holder spans in the indexed event stream;
contextual replay now consumes the same component admission/value reader and
holder resolver. Its own contextual gaps remain distinct from canonical stops;
see REPLAY_RESOLUTION.md. The separate [ride view](RIDES.md) now joins this
ownership history with direct vehicle attachments and ancestor changes. Full-life
reconstruction and contextual ride/replay integration remain work toward complete
format/runtime coverage.

```rust,ignore
use halo_api::theater::runtime::roster::AssignmentSchema;
for observation in runtime.controllers().observations() {
    println!("{} {:?}: {:?}", observation.event.timestamp_us,
        observation.entity, observation.state());
    println!("{:?}", observation.holder(AssignmentSchema::Reference8f0f8d0));
}
let ownership = runtime.controllers().ownership(recorded_entity, schema);
for span in ownership.spans() {
    println!("{:?} until {:?}", span.start.status(), span.end.map(|e| e.boundary));
}
```

See [controller fixtures](../fixtures/controllers-v41/README.md) for independent
original-reader comparison and explicit local-policy tests. The 130 authored
bodies cover all IDs and optional branches; all eight alignments and every-bit
truncation of the longest body are checked. Captured and full-suite results are
recorded in VALIDATION.md.
