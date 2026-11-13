# Mode-qualified zone event observations

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


`TheaterRuntime::zone_events(mode, schema)` interprets canonical
PlayerGameEventSmall (code 82) messages under an explicitly selected `ZoneMode`
and `ZoneSchema::Reference8f0f8d0`. It keeps the existing
film / parser/v41 / runtime organization and shares `runtime/actions` parsing
and source validation. No bytes are rescanned or reparsed by this view.

```rust,ignore
use halo_api::theater::runtime::zones::{ZoneMode, ZoneSchema};
for event in runtime.zone_events(ZoneMode::Oddball, ZoneSchema::Reference8f0f8d0) {
    let meaning = event.meaning();
    let recipients = event.recipients();
    // event.action retains the original Event, EventRecord, references and fields.
}
```

The mode is an applicability choice, not a newly recorded fact. Existing script
filename/mode hypotheses remain in the variant API; this view does not silently
promote those candidates or event IDs to proof of an active mode. The same ID can
mean different things: 0x320 is an Oddball score tick and a KOTH capture message.
Unknown IDs remain `Ok(None)` from `meaning`; partial/malformed or unsupported
reads retain explicit `ActionReadError` and their original source prefix.

The pinned reference supplies 23 mode-qualified table rows for Strongholds,
Firefight KOTH, Oddball and PvP KOTH. All 17 reference event kinds are represented,
including NearlyWon. Strongholds' separately handled 0x322 broadcast is also
retained as its own `StrongholdCaptured` message rather than supplying a target
to another record. `ZoneAudience` reports the reference's Player/Team/Match scope;
it does not resolve an actor or team from recipients.

Each `ZoneEvent` contains its `ActionEvent`, mode and schema. `message()` exposes
the exact ID, small field, ordered keyed values, extras and final recipient mask.
`recipients()` returns every set high-bit-first recipient index (0–31), with no
actor, XUID, holder or team substitution. Equal-clock and repeated messages keep
their original order. No frame-level grouping or per-recipient multiplication
changes the recorded event count, and no game event is dropped because team
attribution is unavailable or ambiguous.

`zone_reference()` requires a recognized zone-naming event kind, exactly one
ZONE_KEY entry (0x436d4d78), value kind 6 and marker bit 0x40000000. The key and
encoded value retain their original field indices. `index_bits` removes only
that marker; it is not a proven runtime ID, generation or keyframe namespace.
Repeated keys remain ambiguous even if identical; absent keys remain absent.
Wrong types and missing markers have distinct errors. The same key on an Oddball
message can refer to a skull; it is never promoted to a zone reference.

`positions()` returns every keyed point in source order, including repeated keys,
with exact region and axis codes. It does not select the first position, infer
its role, calibrate a world location or link it to a later event. In particular,
a Firefight hill appearance lacking a target remains targetless even when a
later HillLive names a zone. Original unknown keys and tails remain accessible.

## Evidence and remaining work

[The independent fixture](../fixtures/zones-v41/README.md) executes the unchanged
pinned upstream table and ZONED predicate for 4,096 mode/ID combinations. Seven
runtime tests use authored native code-82 packets through public Film parsing,
plus labelled model mutation for unsupported format, to verify sources, unknowns,
mode collisions, all 32 recipient bits, duplicate/wrong/missing keys, multiple
positions, equal-clock order, truncation and refusal of cross-record/future-target
backfill. The body grammar's existing independent events-v41 fixtures are unchanged.
The retained direct-action audit also checks this view; see VALIDATION.md.

This adds direct mode-event interpretation. It does not complete zone geometry,
manager/navpoint relationships, ownership/contest timelines, hill scores,
current recipient-holder/team attribution or calibrated zone locations. Those
require their own source, lifecycle and context evidence. Upstream scans,
first-position selection, recipient/team aggregation and later-target backfill
are not imported as canonical facts. Ground objects, complete lives, unresolved
writer/physics context and field-driven whole-format proof remain open.

## Navpoint source components

`TheaterRuntime::navpoints(ZoneSchema::Reference8f0f8d0)` exposes actual
archetype-12 docking-filter, radial-progress and visual-group occurrences. The
registry must be v41/27 with a unique matching row/name and parameter 2 for docking
or 1 for progress/groups. Row numbers are retained, but class or layout is never
inferred from a slot range. Missing archetypes, duplicate rows and unsupported
registries remain inspectable with errors. Keyframe namespace IDs and full runtime
IDs have distinct variants. An unrelated DELTA never replays an earlier group.

`NavpointObservation` borrows the Event and all fields, retains component status
and exact optional source extent, and validates raw widths, padding, field
contiguity and the component boundary before interpreting values. An invalid
signed extent stays absent instead of becoming a fabricated zero span. A complete
component prefix may be observed when a later part of its record stops; an
incomplete component never becomes an absent group or zero progress.

Filters retain the recorded mask and flag, every sparse clause slot, tagged kind,
negation and payload. Scalar forms have typed source words. Complex string,
reference, enum and optional payloads retain their complete canonical named fields;
this view does not map their handles or evaluate their conditions. Unsupported
clause tags keep the partial original component with a refusal. An all-ones player
mask is preserved; it does not silently become an empty set of occupants.

Visual groups retain the presence bit, group name, filter, default state and every
per-clause state/order pair. Item ordinals and recorded order values remain
separate from filter mask slots; repeated orders and conflicting/negated clauses
are not resolved by selecting the first clause. Explicit absent groups are distinct
from missing updates. `reference_name` recognizes HasProgress, HasText and
ControlPoint. `reference_state` labels a supplied default/item word as Contested,
Incoming or A–Z only for the matching group name. These labels do not establish
which viewer sees the state, an owner, a capturing team or a zone relationship.

Radial progress retains all 256 original codes. `reference_progress()` exposes
upstream's saturating subtraction/clamp as a labelled reference projection. It
does not claim capture progress: a group may describe an incoming-hill countdown,
and this component alone cannot resolve that role. No default neutral owner,
first team/letter, inferred clear, frame clock or continuous zone lifetime is added.

Five navpoint tests reuse 56 existing independently traced component cases at all
eight bit alignments, cover all progress codes, sparse/conflicting clauses,
per-item states/orders, every truncation of an authored group, malformed retained fields,
registry/duplicate refusal and native packet observations. Group fixtures' original
parameter-2 traces have the same level-independent group grammar; separate authored
parameter-1 packets exercise the admitted registry. The packet fixture uses a
synthetic registry and explicit internal decode configuration, not a new public
bootstrap admission. The retained-film audit uses unmodified public Film parsing.

## Crate geometry

`TheaterRuntime::crate_properties(ZoneSchema::Reference8f0f8d0)` observes
class38 multiplayer-properties components under the exact v41/27 registry with
a unique matching row and parameter1. It borrows original events, fields and
component status and keeps keyframe namespaces separate from full runtime IDs.
Wrong/unknown classes, duplicate rows, partial components and source mismatches
remain refusals. An unrelated DELTA does not replay previous properties.

`decode()` validates contiguous fields against the original packet bits, excludes
any corruption trailer, and uses the existing Bond reader within the exact
retained component body. Root/nested lengths, terminators, integer widths and
resource limits are enforced. `CrateProperties::bond` retains even an incomplete
nested read, with packet-relative source ranges; `items()` admits semantic
selection only after complete, exact body consumption. The original canonical
TLV fields and component boundaries are unchanged. In particular, an inheritance
marker can stop the canonical component before Bond finishes; the adapter reports
that limit rather than borrowing bytes from the following component.

`items()` returns every root field0 structure/list/set item in source order.
`CrateItem::geometry()` similarly retains every geometry entry. Empty collections,
absent fields and explicit absent properties remain distinct. Shape codes2/3 are
labelled cylinder/box, while unknown codes survive. Fields5–8 retain the recorded
forward/radius, right, above and below integers, including zero/negative values.
`engine_units()` divides by 65536 while retaining the exact signed/unsigned code.
Missing dimensions stay absent; a cylinder's recorded right extent survives.
A present integer wrapper without field0 returns `MissingWrappedValue`, preserving
its structure separately from an absent dimension or recorded zero.
`team()` is the recorded map assignment, not a current owner. Duplicate requested
fields and unresolved inheritance refuse semantic selection.

The [geometry fixture](../fixtures/geometry-v41/README.md) executes the pinned
upstream reader independently. Native component/packet tests cover alignment,
truncation, malformed lengths, source mismatch, schema refusal, same-clock clears
and namespace separation. The retained-film audit checks all component fields
and nested Bond atoms against the original bytes; see VALIDATION.md for counts.
No first-item selection, missing-dimension zero, default orientation or world
calibration is imported. Orientation, manager/navpoint/zone relationships, viewer
condition evaluation, lifecycle-aware zone state and hill-score projection remain
open, alongside the broader canonical-format objective.

## Crate orientation

`TheaterRuntime::crate_orientations(ZoneSchema::Reference8f0f8d0)` exposes
class38 dynamic-precision forward/up components, requiring a unique matching
v41/27 registry row at parameter2. Each observation retains its component index,
Event, exact source extent, status and fields. All fields are checked against the
packet bytes before interpretation. Keyframe namespaces remain separate from
runtime IDs; duplicate/partial/unsupported components return explicit errors.

`decode()` preserves raw, delta and configuration selector words, then exposes
`OrientationEncoding`: six raw vector words, absolute direction/default-up and
roll, or keep/delta direction and roll-increment words. Configuration overrides
raw/delta selection as established by the existing instruction-backed parser.
No selector is dropped just because another selector wins. Clear increment gates
remain absent increments, not zero absolute angles; unknown prior orientation
cannot be supplied by a later component. Corruption trailers remain source fields
but do not become orientation values.

Two labelled projections distinguish different evidence:

- `reference_world_angle()` matches libfilmshell's unshifted 30-bit turn formula
  only for its compatible non-raw, non-delta, config/default-up branch.
- `engine_world_heading()` derives an angle in world XY from complete LE raw
  forward/up vectors or config-mode default-up midpoint rolls. It rejects
  nonfinite, parallel or vertically pointing raw forward vectors, nominal mode,
  nondefault up and unresolved incremental orientation. The config formula is
  computed in f64 and is not bit-exact emulation of the engine's f32 reconstruction.

The raw branch of upstream `angle_at` omits the parameter2 config bit and treats
copied memory words as MSB-packed float values. It also bypasses configuration
overrides. Those offsets are not copied into the local view; the raw reference
projection refuses while the engine projection uses the canonical fields. Both
original helpers are independently exercised in the
[orientation fixture](../fixtures/orientation-v41/README.md), with discrepancies
and numerical precision limits recorded.

An observation does not attach a heading to geometry, pick the first orientation,
carry it across updates/gaps, or supply a default for omitted components. General
nondefault-up, nominal-mode and incremental reconstruction, lifecycle-aware
geometry joins, manager/navpoint relationships and zone state remain open.

## Manager reference state

`TheaterRuntime::manager_links(ZoneSchema::Reference8f0f8d0)` exposes class10
creation/default target handles and the sixteen navpoint words at registry
rows2–17. These rows legitimately share `managed-object-navpoint-component`;
slot0–15 and original row index remain distinct. Admission requires the exact
v41/27 class declaration, all sixteen rows at parameter1, and no duplicate row
within an individual record. A repeated component name is not itself a duplicate.
Unsupported candidates and partial reads retain their original fields and errors.

`observations()` preserves every target/row occurrence. `decode()` validates
source extents and original packet bytes, then retains the version gate/value,
target presence gate, category-zero index and two generation bits, or the complete
32-bit navpoint word. No absent version becomes a recorded zero. Known default
and component corruption trailers are validated separately from the payload.

The upstream projections remain explicitly labelled: `reference_zone_index()`
adds512 to the raw creation index, while `reference_navpoint()` distinguishes
u32::MAX, the reference's marker/index convention, and uninterpreted words. The
marker projection retains every discarded high bit alongside the low16 index.
These are not canonical entity identities. The category-zero reference table has
additional writers whose applicable base is not established here; a13-bit width
does not prove base512. Likewise, stripping marker/generation bits cannot prove
which runtime or keyframe navpoint a word names. No numeric coincidence supplies
that missing identity evidence.

`at_event(entity, event)` queries through an exact borrowed event, rejecting
foreign/cloned events and excluding later same-time records. `at(entity, time)`
includes all observations through the requested recorded timestamp and closes
known unread packet suffixes. Each `ManagerSnapshot` retains independent target
and sixteen-row evidence. Explicit clears/replacements affect only their row;
a second row carrying the same word survives. NEW/keyframe baselines, deletes,
other generations, incomplete records and the shared continuity gaps invalidate
prior state with source-linked reasons. An untimed opaque chunk creates no clock.
Keyframe namespace IDs and full runtime IDs are separate domains. Missing fields
on a fresh baseline do not inherit values from a previous incarnation.

The six manager tests reuse four independent creation cases and two independent
component cases across all16 rows (36 case/row combinations), all eight bit
alignments and every-bit truncation. Authored native packets verify lifecycle and
same-clock behavior; schema mutation tests distinguish legitimate repeated names
from duplicate rows. A separate untimed-gap fixture checks source ordering.
A native checked-keyframe/DELTA fixture also verifies both canonical corruption-trailer spellings and the default-state check. The existing parser, canonical fields and fixture oracle bytes remain unchanged.

This provides manager reference evidence and lifecycle-aware state. It does not
join lossy reference indices to navpoint/crate identities, reuse a later heading
or label, infer an idle zone on a clear, or evaluate viewer/owner conditions.
Those identity and semantic obligations remain part of the full objective.

Geometry, orientation, navpoint and manager-row views share validation of the two
canonical component-check trailer forms (keyframe underscore names and DELTA
dotted names). Both remain unchanged in retained fields. Missing values, bad
widths/padding, mismatched names and extra suffix data refuse; the check word is
never part of a Bond body or semantic component value. Validation covers trailer
structure, not an unestablished checksum algorithm. Native checked packets test
both present and absent check-word gates; see VALIDATION.md.

## Navpoint history

`runtime.navpoint_history(ZoneSchema::Reference8f0f8d0)` adds source-supported
point queries alongside the existing `runtime.navpoints(schema)` occurrence
iterator. `observations()` exposes the same original records; `at(entity, us)`
includes all records at that clock and closes unread packet suffixes.
`at_event(entity, &event)` stops at the exact runtime-owned event and rejects
cloned or foreign events. Queries do not move the runtime playhead.

`NavpointSnapshot` retains independent docking, progress and eight visual-group
evidence slots. Group slots follow the serialized suffix 0..7. Their name hashes,
conditional states and order values remain in each original observation. An
explicit absent group replaces only that group; an omitted DELTA row preserves
support only within an uninterrupted lifetime. `Observed` retains the original
decode result, including incomplete or unsupported reads, rather than certifying
a valid semantic value.

NEW/keyframe baselines, deletion, another generation, partial records and shared
packet/chunk gaps invalidate prior support. A readable prefix remains inspectable
at its exact event; an unread suffix invalidates held state at packet end.
Unframed chunks retain source boundaries without inventing timestamps. Runtime
IDs keep all generation bits and remain separate from keyframe namespaces.

This establishes source-supported navpoint history. Viewer filter evaluation,
manager/zone identity joins and owner/contest timelines remain unresolved. No
default owner, inside-player mask or idle-zone state is inferred. Authored native
packets test all eight groups, clears, ordering, gaps and lifecycle reuse; captured
films provide sampled exact-event/source regression checks. The existing parser
and canonical Film models are unchanged. See VALIDATION.md for results.

## Hill properties and reference score projections

`runtime.hill_scores(ZoneSchema::Reference8f0f8d0)` builds an ordered ledger over
the source-validated class 13 [property history](PROPERTIES.md). `changes()` retains
inputs and source-linked resets; `inputs()` iterates all attributed occurrences,
including invalid reads. The existing film / parser/v41 / runtime split is retained.

The four recognized hashes are LeftBank `0x7b7315d7`, RightBank `0x0a1ce07d`,
PointsToWin `0x29453215` and HillsToWin `0xc728669c`. Each `HillInput` retains the
original name and value samples and the value's decode result. Default names precede
their ordered values; component row 0 can establish a new name for subsequent
values. A changed/unknown name clears the earlier binding. A repeated identical
name applies to future values while existing values retain their earlier name
source. Values preceding a name are never backfilled, including defaults followed
by a name override inside the same Event.

`at(namespace, scope, timestamp_us)` includes all same-clock events and closes
unread packet tails. `at_event(namespace, scope, &event)` stops at the exact borrowed
Event, rejects clones/foreign events and excludes later same-clock records and its
unread packet suffix. Runtime IDs retain generation bits and remain separate from
keyframe IDs. Scope is explicitly `Scalar` or `Viewer(0..31)`; it does not identify
a roster participant or team. Queries do not move the runtime playhead.

NEW/default/keyframe baselines, deletes, another generation, partial records and
shared continuity gaps invalidate stale inputs. Unframed chunks retain their own
source boundaries without invented clocks. Bad name/default reads clear bindings;
a bad value under a known name replaces the prior value with its error. A readable
prefix can still be inspected at its exact event before an unread suffix clears
support. `unique(property)` refuses duplicate named entities even when values agree.

`reference_bank()` requires a masked tag-7 bank, projecting
`(q - 0x7fffff) / 0x7fffff * 100`. This is the exact upstream convention, distinct
from engine midpoint dequantization. `integer_limit()` requires an unsigned variant
(scalar tag 4/6 or masked tag 8), accepting 1..999 PointsToWin or 1..99 HillsToWin.
String IDs, booleans, enums, float quanta, missing payloads, zero and out-of-range
limits cannot supply scores. `reference_points(LeftBank | RightBank)` combines unique
bank/PointsToWin inputs in the requested namespace and scope, multiplies, rounds
and clamps at zero, and returns both source inputs. A later limit affects only
later queries. Viewer scores do not fall back to scalar limits or other viewers.

The [hill-score fixtures](../fixtures/hill-score-v41/README.md) execute the original
upstream reader for 60 arithmetic cases. Authored native packets test these values,
checked trailers and lifecycle/boundary policies; model mutations separately test
corrupt evidence. The retained-film audit reports named input/projection counts,
not team scores. See VALIDATION.md for results.

This supplies source-bound hill inputs and explicit reference projections.
Viewer/team identity, two-team applicability, owner/contest interpretation and
team score timelines remain unresolved. Upstream's `team_of(row - 2)` and `1-own`
mapping, cross-viewer aggregation and unchecked retention are not sufficient
evidence for these joins. The separate class 13 NEW-envelope and broader canonical
format requirements remain open.
