# Recorded attachment state, histories and root paths

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


`runtime.attachments(AttachmentSchema::Reference8f0f8d0)` reads canonical
`object-parent-state-component` occurrences under v41/27 and recorded parameters
0–4. The meaning of the parent, node and seat follows the explicitly pinned
libfilmshell revision. The existing film / parser/v41 / runtime organization is
preserved; this adds no wire scans, guessed component boundaries or world poses.

`observations()` retains baseline/DELTA occurrences and their original events,
entity IDs, fields and completion status. Schema ambiguity and partial reads
remain inspectable. `state()` requires a unique matching registry row and an
established archetype. An unbound DELTA cannot infer its archetype from which
suffix would fit. Keyframe high bits are namespaces, and their identity domain
remains distinct from runtime generation IDs.

States retain attached/detached, handle class/index/generation, parent node,
optional secondary node, flags, three 16-bit local-offset codes, optional
orientation code, magnitude, optional seat and tail fields. The older detached
layout retains its separate free-reference/alternate fields. Those fields do not
become an attachment. A detached component with a present seat remains exactly
that combination. No local code becomes a copied vehicle-center position.
Known corruption-check trailers are validated and remain in the original fields.

Parent handles remain typed raw words. The reference's Unit-handle mapping is
`slot = 512 + index`, retaining all two generation bits in the runtime ID. Object
handles stay unmapped until their domain/base is established; a numeric match to
an unrelated entity cannot supply that relationship. The pinned LevelUp
`varwidth.go` initializer notes set category 1 to base 512, but also identify
additional category-1 table writers (`FUN_1408f1618`, `FUN_142f2f0cc`). Their
applicable state has not yet been established here; the invariant category-4
Unit mapping does not establish the broader category by itself. No vehicle or biped type is
inferred simply from a parent handle or seat value.

## Snapshot and source-boundary behavior

`at(timestamp_us)` builds the latest observed attachment state through that time.
`at_event(&Event)` stops after that exact borrowed event and rejects foreign or
cloned events, excluding later same-time records. They leave Film and playback
unchanged. A component update replaces the preceding value; a missing component
on a NEW is unknown, and a partial component cannot become detached. Complete
deletes remain explicit. Records for another generation invalidate the older
slot incarnation; they do not prove a particular death or replacement time.

A NEW, deletion or generation boundary prevents an old reference from silently
binding to a later incarnation with the same raw ID. If the target has readable
state again, an older child reference yields `LifecycleBoundary` until a fresh
child observation establishes the link. This also avoids treating a later NEW
as proof of an earlier relationship.

Unread entity suffixes invalidate accumulated state **after** their known packet
prefix. Thus `at_event` can expose the exact prefix observation, while `at(time)`
includes the unresolved suffix. Opaque/refused entity views invalidate at their
packet event. Unframed chunk tails and incomplete decompression also invalidate
state after the final known packet. Discontinuous packet order, nonconsecutive
chunk indices and missing chunk-index metadata cannot carry prior links across
the boundary. Keyframe packets invalidate prior state; their rows establish only
the separate keyframe identity domain. No namespace-to-generation fallback
supplies a runtime baseline.

Chunks with unread bytes or incomplete transport and no framed packet now
supply `AttachmentBoundary::UnframedChunk`, borrowing the exact canonical chunk.
`boundary.event()` and `evidence.event()` return None for that case;
`boundary.unframed_chunk()` returns the source chunk with its original bytes,
transport status, manifest identity and optional offset. Unknown attachment
evidence carries a `boundary`, rather than requiring an invented packet Event.
Complete empty clear/zlib chunks do not become unread boundaries.

Untimed gaps are placed after the preceding source chunk's indexed prefix and
before later source data. If source timestamps reverse, the earlier bounding
index is used conservatively. This orders evidence, not the unknown chunk's
occurrence time. Timestamp queries cannot carry prior links through that uncertain
interval; exact-event queries can still inspect the preceding readable prefix.
No manifest millisecond offset is substituted for a packet timestamp. Leading,
adjacent and wholly unframed chunks retain their own source evidence in history.

Global invalidation is O(1), using a private epoch unrelated to film generation
bits. State storage keeps source references rather than a second Film. Snapshot
construction visits the selected event prefix; when unframed chunks exist, it
also indexes source bounds over the full event stream. Root traversal visits only
its path. These queries describe the recorded prefix, not simulated motion or proof
of continuous game activity outside the recording.

## Root outcomes

`snapshot.root(entity)` follows established links until the terminal entity
explicitly records no parent. Every visited observation remains in the result.
There is no four-level cutoff. Outcomes distinguish:

- `Root`: the terminal entity explicitly records detached state.
- `Missing`: the target has no observation in the selected prefix.
- `Unresolved`: a read, schema, lifecycle, source gap or deletion blocks traversal.
- `UnmappedParent`: the raw handle lacks an established identity mapping.
- `LifecycleBoundary`: the target's later incarnation cannot accept the old link.
- `Cycle`: a repeated entity closes the path, including self-parenting.

No missing edge becomes an implicit root. A root is a recorded parent-chain
result; it is not proof of vehicle classification, a ride duration, a world pose
or player ownership. Ride projections and downstream replay integration remain
subsequent work, alongside unresolved wire/context coverage.

## Direct attachment histories and spans

`history(entity)` visits the recording once using the same state transitions as
snapshots. `entries()` retains repeated component observations, missing baselines,
deletes, other-generation boundaries, schema/read errors and global source gaps.
Global gaps are retained even before the entity's first observation. Keyframe and
runtime histories stay separate. Each entry borrows the original source event.

`AttachmentBoundary::Event` takes effect at that event;
`AttachmentBoundary::PacketEnd` marks an unread suffix after the packet's known
prefix. A packet-end entry refers to the packet envelope, so its event order can
precede the readable records it follows. The history's vector order is the
evidence order; sorting it by the envelope event would move the gap incorrectly.
This distinction also preserves readable-prefix, gap, recovery order when all
three share a timestamp.

`history.spans()` returns each admitted direct component observation and its next
evidence boundary. Detached values and repeats remain separate spans. A gap,
delete or invalid component ends the preceding span without inventing a detached
value. `end: None` means no later boundary was recorded; it does not supply a film
end timestamp, an infinite lifetime or evidence beyond the recording. No duration
is calculated from packet timestamps.

These are direct parent-state spans. An ancestor's update or lifecycle boundary
can change a root-path result without changing the child's recorded component.
Use a snapshot root query to inspect that chain; a direct span is not a continuous
root-path, ride or player-ownership claim. Local offsets and seat values remain
source fields throughout.

The separate [ride view](RIDES.md) joins these observations with current
controller/holder evidence and watches ancestor changes. `archetype()` exposes
the established class without inferring it from handle ranges. Ride spans retain
the full path and can therefore end at an ancestor or ownership boundary even
when a direct attachment span continues.

```rust,ignore
use halo_api::theater::runtime::attachments::{AttachmentSchema, AttachmentEntity};
let view = runtime.attachments(AttachmentSchema::Reference8f0f8d0);
let snapshot = view.at(timestamp_us);
let path = snapshot.root(AttachmentEntity::Runtime(recorded_entity_id));
println!("{:?}", path.outcome);
let history = view.history(AttachmentEntity::Runtime(recorded_entity_id));
for span in history.spans() {
    println!("{:?} until {:?}", span.observation.state(), span.end);
}
```

The [attachment fixtures](../fixtures/attachments-v41/README.md) execute the
unchanged native ParentState block and distinguish direct values from local
uncertainty policies. Full validation and retained-corpus limits are recorded
in VALIDATION.md. Whole-format completion remains unproven.
