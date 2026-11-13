# Tick weapon-object histories

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


`runtime.objects(ObjectSchema::V41Registry27).weapons()` builds class-42 object
tracks from sequential tick records. It preserves the film / parser/v41 / runtime
organization. These are weapon-object observations, not a declaration that each
object is resting on the ground or belongs to a particular player.

## Records, tags and positions

`tracks()` retains every interval, including unpositioned NEW records and untagged
observations. Full runtime IDs retain all generation bits. Class selection uses
an explicit NEW archetype or the existing sequential runtime schema state for the
same full ID. Inherited schema is not a new spawn or proof of lifetime continuity.
`Create` requires a complete NEW and source-validated current default tag;
`Observation` makes no spawn claim.

A tag comes only from the current NEW's complete default section, with its exact
`mpp.object_tag` field, original Event and source bytes. Unknown catalogue tags
remain valid recorded words. No catalogue membership filter discards them.
Resuming after a gap or snapshot does not inherit a tag from an earlier interval;
a DELTA-only interval is explicitly untagged. Snapshot tags and coordinates never
backfill runtime values.

Every position observation retains its Event, component, fields and decode result.
The reader checks v41/27, the unique parameter-1 position row, complete component
status, contiguous extents, original source bits and the structural check trailer.
A complete default or component prefix in a partially read record remains
inspectable. A partial component stays an error rather than retaining the previous
successful coordinate.

Position domains remain distinct:

- `Indexed` retains the recorded region and all three code fields/widths.
  `world_region(bounds)` requires explicit matching region bounds and matching
  quantizer widths before using the existing midpoint conversion. A declared
  zero-width axis retains its named field and empty raw bits; only code zero is
  legal and its coordinate is the supplied bin center, not an implicit zero.
- `Default` retains the fixed-width 22-bit codes and their default-domain marker.
  It is not silently assigned the current map region.
  `fixed_midpoint(FixedPositionSchema::Reference413971b)` projects it in the
  explicitly selected +/-20,000 reference domain.
- `Local` retains 14-bit parent-local codes. Handle/region gates and words remain
  in the original component; numeric coincidence does not establish a parent join.
  The same fixed-midpoint API returns `ParentLocal` values over +/-100, retaining
  that domain rather than silently transforming them to world coordinates.
- `Raw` retains the original little-endian float words and local/world distinction
  from the selected canonical full-precision branch. `raw_world()` refuses local
  and nonfinite forms; signed zero remains intact.

`fixed_midpoint` returns its selected schema, domain, original codes and f64
bin midpoints. It accepts only Default/Local forms and checks their 22/14-bit
widths and raw-word shape. Indexed/raw forms refuse. Selecting the reference is
an explicit applicability assumption; it does not identify the film's build.
These values match unchanged libfilmshell `LevelBox::engine` arithmetic under
LevelUp's recovered fixed bounds. They are not a claim of bit-exact engine f32
rounding or of a reconstructed original unquantized coordinate. The shared
`runtime::coordinates` API also validates explicit domain/codes/width inputs at
precision 16; it never uses a missing map as a default-box signal. Independent
sources and test scope are in
[fixed-positions-v41](../fixtures/fixed-positions-v41/README.md).

No later resting coordinate overwrites earlier samples. Omitted DELTA rows hold
the last observation only within a supported interval; invalid observations
replace success with their error. Initial NEW positions are retained even though
the upstream ground-weapon reader waits for a subsequent UPDATE.

## Recorded item-rest flags

Each track's `rest_flags` retains every `item-at-rest-component` occurrence.
`at(...)` and `at_event(...)` expose `rest_flag`, the last supported observation
within that interval. `None` means no recorded flag; `Ok` retains the original
one-bit field and explicit `at_rest` boolean; `Err` retains the invalid component
and its source. A false flag is not missing evidence. An invalid update replaces
prior success, and omitted DELTA rows hold the prior observation with its original
timestamp rather than creating a new sample.

Admission requires v41/27, the unique class-42 parameter-1 registry row, complete
component status, correct name/index, contiguous fields, source-bit equality and
a valid structural corruption-check trailer. The reader accepts a complete rest
component in an otherwise partial record at its exact event; the unread suffix
closes support. NEW records retain initial flags without requiring a position.
Snapshot flags do not seed runtime IDs, and no flag crosses a shared history gap.

This reports a recorded engine flag, not an inferred rest onset, world position,
ground placement or ownership. No coordinate stability test synthesizes a flag,
and later positions are never backfilled to the flag's timestamp. Existing
components-v41 original-reader traces cover both flag values in snapshot and delta
modes (original fixture parameter 4); authored native registry-parameter-1 tests
exercise the supported runtime row. The reader's one-bit grammar is independent
of that parameter. LevelUp's pinned `dispatch_item.go` independently agrees on
ReadBit for class 37/42 row 18 (FUN_140ff91e4). This runtime API currently projects
class 42 only.

## History boundaries

`at(entity_id, timestamp_us)` includes all events at that clock and closes unread
suffixes. `at_event(entity_id, &event)` stops at the exact borrowed Event, rejects
clones/foreign Events and allows inspection of a readable partial prefix before
its support closes. Neither query moves the runtime playhead.

NEW, DELETE, generation changes, reclassification, incomplete records and all
shared packet/chunk/keyframe gaps end an interval. A fresh readable prefix starts
with independent evidence; absent tags/positions do not become zero. Untimed
opaque chunks retain source boundaries without invented clocks. Ends distinguish
complete and incomplete deletion, ordinary replacement and gaps. No open interval
gets a synthetic destruction, pickup, rest or expiry time.

## Pickup candidates

`pickups()` retains every direct code-9 occurrence, its original record, reference
interpretations and body result, including absent tags, partial reads and unknown
actors. `pickup_candidates(track_index)` returns every same-packet/same-tag match
for a complete recorded deletion of a tagged interval. Equal timestamps in another
packet do not match. Lookup uses a packet/tag index while preserving duplicate
matches in source order. Unpositioned but tagged objects can still have candidate
events; missing positions are not manufactured to support a match.

These are candidate relationships only. Multiple deleted objects can share a tag
and multiple pickup events can name it. All candidates survive; the API does not
consume a pickup to select one arbitrary object or expose a `taken=true` claim.
Even a single candidate cannot establish a physical-object/actor association.
The original action references remain available for a future proven identity join.

## Evidence and remaining scope

[ground-weapons-v41](../fixtures/ground-weapons-v41/README.md) executes the unchanged
upstream inventory reader on seven authored operation/event sequences. Native
fixtures compare its tag, position, start/end and candidate evidence while testing
the differences above. Additional tests exercise checked packets, same-clock
queries, all position domains, malformed source/schema/default evidence, generation
reuse, partial prefixes, unknown tags and snapshot/chunk gaps.

The retained-film audit reports track and position occurrences, missing tags and
candidate counts without asserting pickup ownership. See VALIDATION.md. This
supplies tick history, recorded rest flags and the candidate layer. Physical rest-onset interpretation,
automatic fixed-domain build admission, parent/world transforms, physical pickup
identity, wider writer admission and whole-format proof remain open. The parser
and canonical Film models are unchanged by this layer.
