# Runtime resolution and playback

`TheaterRuntime::load(film)` owns a canonical Film through `Arc` and constructs a
private `ResolvedFilm`. Loading indexes supported structural records, resolves
entity generations and component updates, and creates checkpoints. It also
associates guarded summary fields into higher-level typed events. It does not
mutate Film, reread structural packet boundaries, interpolate, or infer physical
jump/shot actions.

## Summary events

`summary_events()` returns `&[SummaryEvent]`, sorted chronologically. Each event
has its candidate timestamp, stable order within the complete runtime stream,
actor, typed payload, original codes, source reference, and derivation.

- The actor's XUID and gamertag come from the guarded summary read. A bootstrap
  roster link is separately `Unique`, `Missing`, or `Ambiguous`. A unique XUID
  whose roster name differs retains the summary name and flags the mismatch.
  Duplicate roster indices are never silently selected for lookup/linkage.
- `SummaryPayload` distinguishes kill, death, mode, medal, and unknown categories.
  Kill/death payloads do not fabricate an opponent or weapon from an opaque body.
- Medals resolve by `(type_code, metadata)` using the 124-pair empirical v41 table
  pinned to LevelUp `d61443ef59268ad734355db8e9974f68db5ca6d0`. The medal flag and
  supported sorting-weight category select a medal payload. Unknown pairs retain
  both codes as `Medal::Unknown`; no nearest-code substitution occurs. These
  byte pairs are not the Halo REST API's medal NameIds.
- The summary timestamp is the source-read millisecond value converted exactly
  to microseconds. Packet timestamps remain separate. Semantic meaning and record
  association use `SummaryDerivation::GuardedV41Layout`; matching full-stream
  entries use `Provenance::DerivedSummary`, never `RecordedRead`.

The canonical summary body still owns its count and opaque record-stream range.
No sequential summary grammar has been established. `summary_reports()` retains
an optional declared count and the candidate count per packet. `count_matches()`
is `None` when the count is unreadable, `Some(false)` for disagreement, and
`Some(true)` for agreement. Agreement does not prove grammar completeness.
`record(event.source)` returns the owned bounded candidate read, including all
UTF-16 units, codes, and bit ranges. It is not a fabricated Film record.

`SummaryFilter` combines inclusive time ranges, kind, XUID and medal. XUID queries
work independently of bootstrap linkage. Summary lookup starts with the smallest
applicable index, binary-searches its time bounds, then intersects the remaining
predicates. It preserves chronology and does not affect playback. An empty or
reversed time interval produces no events. Generic `EventFilter` covers the full
structural/lifecycle/state/input/action/summary stream.

`interpretations()` exposes explicitly labeled bootstrap, player-slot, event-gate,
bot, packet, fire/aim and whole-chunk evidence. An inferred event-gate selection
is not a recorded bit and never repairs Film or applies guessed world updates.

## Costs and playback

- Loading walks fields/events, sorts chronology, builds indexes, and stores
  checkpoint snapshots. It is not O(1), and resolved state can copy field values.
- `current()` borrows the materialized world in O(1). Enumerating/copying that world
  scales with its size; this method performs no event application.
- `advance_to(t)` binary-searches the event boundary and applies intervening updates.
  Moving backward uses checkpoint seeking.
- `seek(t)` binary-searches event/checkpoint indexes, copies a checkpoint world map,
  and applies at most 1,024 remaining events. Restoration scales with entity count;
  the operation is not O(1).
- Runtime clones share original Film buffers, event/summary indexes, evidence,
  reports and checkpoints, and copy the current world map in O(world size).
  `rewind()` clears that cursor without changing another runtime.

Loading leaves an empty current world until advance/seek. Unsupported or partial
records remain explicit; incomplete NEW records do not establish a lifetime, and
partial keyframes do not create unsupported state. Playback tests require seeking
and sequential advancement to yield the same world at selected times and ties.

Browser rendering, map geometry/assets, physical-action annotation and visual
interpolation remain future work. The runtime provides the typed summary events
and source-linked state/query interface those consumers can build on.
