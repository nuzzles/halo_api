# Source-linked networked properties

`runtime.properties(PropertySchema::V41Registry27)` exposes canonical class 13
property occurrences and source-supported state. It preserves the existing
film / parser/v41 / runtime organization. The parser and Film models are unchanged.

- `observations()` returns recorded default bodies and name/scalar/viewer rows.
- `at(entity, timestamp_us)` includes every event at the requested clock, followed
  by any unread suffix boundary. It does not move the runtime playhead.
- `at_event(entity, &event)` includes that exact runtime-owned Event, excluding later
  records at the same clock. Cloned and foreign Events return `ForeignEvent`.

`PropertyEntity::Runtime` retains all generation bits. `Keyframe` is a separate
namespace; neither is converted to a stable player, controller, team or zone ID.
The 32 viewer indices remain the recorded film slots.

## Admission and values

Observations validate the v41/27 registry and class 13 declaration. Components
require the exact 34-row layout: name at 0, scalar at 1, and 32 parameter-1 masked
rows at 2–33. Repeated masked names are legitimate; duplicate record indices refuse.
Default bodies have their own source/status admission, independent of the component
row layout. Every field is contiguous, within its retained range, and matches the
original packet bits. Extra/missing fields, bad widths and source mismatches refuse.
Both native keyframe and DELTA corruption trailers are validated structurally and
retained; the default-state check remains distinct. Check values are not verified
against an unestablished checksum algorithm.

`PropertyDefault` retains version presence/value, name hash, array gate and all
one/32 ordered variants. Each `PropertyAtom` retains the scalar/masked domain,
original four-bit tag and optional raw payload. A tag with no payload is distinct
from an omitted row or an error; no zero is inserted. In particular, scalar tags
7–15 and masked tags 0–6 carry no payload. Tags11–15 remain distinct tags despite
sharing the corrected four-bit masked enum payload. `enum_value()` separately
projects the native code-minus-one convention, including code 0 -> -1.

## History and evidence

`PropertySnapshot` retains independent name, scalar and 32 viewer evidence slots.
NEW/full defaults and keyframes replace baselines; deletion, another generation,
partial records and shared packet/chunk gaps invalidate stale support. A complete
readable prefix remains inspectable at its exact event. An unread suffix closes
support at packet end. Opaque chunks retain source boundaries without invented
clocks. A malformed/partial default is `InvalidDefault`, with its original
observation and decode error retained. An absent default domain is `MissingBaseline`,
not an implicit zero or a copied value from the other domain.

`Observed(PropertySample)` retains the original observation and selects a specific
field. Its `value()` still returns schema/read errors; Observed is not a certificate
of successful decoding. Later component rows can override values from the same
record's default body. Scalar and viewer rows remain independent, and updating or
clearing one viewer row does not replace another.

A name update changes only name evidence. Existing value samples retain their
original earlier sources; the history does not relabel them retroactively.
Consumers attributing a value to a name must inspect these independent sources,
including default-versus-component order inside the same Event. No hill score,
team, current viewer or script applicability is inferred by this raw history.

## Evidence and remaining work

The property-v41 fixtures execute the pinned original Go readers and retain 33
captured vectors; their source claims and width correction are documented in
[the fixture README](../fixtures/property-v41/README.md). Runtime tests consume all
256 authored source-traced bodies and every truncation, verifying source extents,
raw bytes and missing payloads. Native configured keyframe/NEW/DELTA/DELETE fixtures
exercise all 32 viewer rows, name changes, scalar/array defaults, explicit clears,
checked trailers, same-clock ordering, partial reads, generation reuse and gaps.
Schema/duplicate/default-status mutations are explicitly model-policy tests.
Synthetic registries do not expand production bootstrap or NEW-envelope admission.

The retained-film audit reports source occurrences and errors without asserting
that raw property counts are scores. See VALIDATION.md for results.
`runtime.hill_scores(ZoneSchema::Reference8f0f8d0)` now builds ordered name/value
bindings and invalidations over these observations, with typed limits, duplicate
refusal and explicitly labelled reference normalization. It preserves namespace
and scalar/viewer scope and does not backfill earlier queries with later values.
See [ZONES.md](ZONES.md). Viewer/team identity, team score timelines, the separate
class 13 NEW-envelope context and full canonical-format proof remain open.
