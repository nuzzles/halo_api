# Source-linked vehicle tracks

`runtime/replay/vehicles.rs` adds `PlayerReplaySet::vehicles` and
`vehicle_snapshots`, retaining the film / parser/v41 / runtime organization.
These are contextual replay projections over sequentially decoded records, not
new canonical Film fields or proof of complete vehicle simulation decoding.

## Runtime tracks

A preceding sequential declaration must establish archetype 40. The declaration
comes from a NEW header or an admitted slot in the selected keyframe namespace;
future headers, scanned candidates and inferred slot ranges do not qualify.
Keyframes supply schema context only. Their namespace bits never supply a runtime
generation, tag or held position. Full runtime IDs include generation zero.

`VehicleTrack` retains start/source/declaration, optional tag/tag source, ordered
position observations, exclusive support end and an optional recorded boundary.
`VehicleStart::Create` requires a complete NEW and complete default state;
`Observation` makes no spawn claim. A tag requires one valid 32-bit
`mpp.object_tag` inside a complete current NEW default section. Unknown tags
remain unknown. Later snapshots or DELTAs never backfill an earlier track.

`position_at(time)` returns the latest supported position with no interpolation
or freshness timeout. Repeated observations retain source order; the last read
at an equal clock controls the sampled value. Unrelated complete records hold
sparse state, while an unreadable position explicitly clears it. NEW, DELETE,
a generation change, missing schema, incomplete records and replay source gaps
close support. Gaps clear tags with the old track; fresh DELTAs start untagged
tracks. Zero-length support intervals retain their source observations.

`PlayerReplaySet::vehicle_at(full_entity_id, time)` finds the latest runtime
track whose exclusive support interval contains the requested replay time.
It preserves the full generation, returns no stale track after a gap/delete,
and never substitutes a similarly numbered keyframe snapshot. The lookup does
not prove a physical lifetime between observations.

Opaque non-frame bodies and unread control suffixes are now general replay
gaps, including for vehicle tracks. A later complete vehicle record can begin
fresh support; no old position or tag crosses the boundary. Authored native
packet tests verify track closure alongside rider ownership and parent paths.

A missing `end` means no ending event was recorded within the supplied replay
window. `end_us` then equals the caller's duration; it is not a destruction time.
Untimed gaps retain their original absent clock and conservative visibility bound.
Partial records can retain an earlier complete position observation while closing
sampled support at that boundary.

## Position admission

The typed projection requires registry v41/27 and exactly one
`object-position-dynamic-precision-component` row with parameter 1. It checks
component completion, duplicate occurrences/indices, contiguous raw fields,
widths, padding, component extents and any known corruption trailer.

Runtime world coordinates currently require the ordinary absolute branch:
`use_prediction=0`, `delta=0`, `precision_high=0`, `default_region=0`, an explicit
known region, and axis widths matching that region's precision-16 quantizer.
The returned encoding includes the supplied map bounds. Predicted, delta,
writer-specific and unsupported layouts retain raw fields with an explicit
error. They cannot hold the previous world coordinates.

Complete ordinary default-region and high-precision local branches now expose
`VehiclePositionRead::Fixed { domain, codes }`. Default-world codes must have
22 bits per axis; parent-local codes must have 14. Local handle presence,
13-bit index, two-bit generation and optional 11-bit region are validated in
source order and retained in the observation fields. They do not establish a
parent entity or a transform. Missing fields, mismatched widths and incomplete
or malformed trailers refuse the entire typed position.

`VehicleTrack::position_read_at(time)` returns the latest typed read within the
same sparse/exclusive support bounds as `position_at`. Call
`read.fixed_midpoint(FixedPositionSchema::Reference413971b)` for an explicit
reference conversion using the shared +/-20,000 default-world or +/-100
parent-local bounds. The result includes its domain, original codes and reference
schema. Choosing that schema is an applicability assumption, not automatic build
admission. Its f64 midpoint convention is not bit-exact engine f32 arithmetic.
`position_at` still returns no world position for either fixed domain; this
prevents local or externally selected coordinates from entering ordinary world
samples silently. A fixed read clears any preceding ordinary world position.
Unknown regions never fall back to the default domain, and snapshots retain their
separate float32 branch. Publicly constructed out-of-range codes refuse conversion.

Snapshot coordinates require the three-zero prefix followed by three raw
little-endian float32 words and the recorded finite field. Nonfinite values are
refused; signed zero and ordinary zero coordinates are preserved. Zero alone
never proves a holder relationship. `VehicleSnapshot` retains its namespace ID,
keyframe stop, optional clock, tag and position separately. A complete position
prefix remains readable when a later component stops the snapshot body.

## Relationships and remaining scope

Ride observations retain separately recorded parent/seat/holder evidence; vehicle
tracks do not replace player poses with vehicle centers. No seat transform,
proximity-based part link, inferred exit, later owner, AI classification or full
biological life is added. Vehicle construction/physics contexts that already stop
the canonical reader remain unresolved. Snapshot object evidence and labelled stationary reference candidates are now
available through runtime.objects() (OBJECTS.md). Tick weapon-object histories and explicit pickup candidates are documented in
GROUND_WEAPONS.md. Rest-time/physical pickup attribution, zone timelines and
field-driven whole-format round-trip proof remain separate work.

Reference: libfilmshell
[`f050371a6cda83b9887eeb83a1884336fce28e53`](https://github.com/OpenSpartan/libfilmshell/tree/f050371a6cda83b9887eeb83a1884336fce28e53),
`src/vehicles.rs`, `src/assemble.rs`, `src/components.rs` and
`src/snapshot.rs::raw_position`. Class/position evidence is reused; first-later
tag fallback, generation-zero omission, singleton filtering and snapshot-part
proximity associations are not imported.

A prior vehicle inventory walked the separately retained 32-film corpus
through `Film::parse`. It found
451 keyframe tables and 207 vehicle-class headers/bodies: five in each
appearance film and 197 in the raid film. All 207 have source-backed tag fields
and complete position components, but every vehicle record stops later at the
unrecorded `vehicle+0x818` flag; none is a complete record. The f050371 tag
catalogue classifies 38 of the raid tags as vehicle parts. No decoded table
reaches two vehicle records, so the current canonical traversal cannot pair
a part and carrier within one snapshot. Complete position-component status does
not by itself establish a usable world coordinate or the missing carrier.
`vehicle_snapshot_branches` separately exposes each candidate's next-slot
evidence and a reference-style layout hint. The hint is bounded to recorded
bits and never changes canonical traversal. Even a unique plausible successor
does not establish the vehicle's unrecorded construction flag.

## Validation scope

Authored native component bodies exercise all eight bit alignments, explicit map
quantizers, snapshot float endianness, signed zero and nonfinite refusal. Tests
also cover sparse/same-clock state, malformed/duplicate/incomplete fields,
unsupported prefixes, schema refusal, lifecycle/generation changes, source gaps,
partial-prefix retention and tag isolation. Tag tests consume the existing
independent native creation fixture. Model mutations test refusal policies and
are not additional independent wire evidence.

An end-to-end native packet fixture uses a synthetic one-component class40
registry and an explicitly supplied no-check bootstrap model, as in the ride
integration tests. It verifies snapshot/runtime separation, a generation-zero
DELTA, a DELETE, exact field source locations and no player-pose substitution.
This fixture does not establish public bootstrap admission for that synthetic
registry. The retained-film replay audit uses unmodified Film input and reports
actual vehicle observation counts; see VALIDATION.md for results and limitations.


The fixed-domain extension reuses the 150-vector independent original-Go bounds /
original-Rust midpoint oracle in `fixtures/fixed-positions-v41`. Vehicle-native
bodies cover every vector at all eight alignments and four local handle/region
gate combinations (4,800 cases, including repeated default-domain combinations).
Malformed/missing fields, corruption suffixes, same-clock replacement, sparse
holds and support gaps have additional policy tests. Native packet integration
now covers indexed, default-world and parent-local DELTAs through replay and
DELETE, preserves Film serialization and keeps snapshots separate. The captured
replay audit reports fixed-domain counts separately from ordinary world poses.
