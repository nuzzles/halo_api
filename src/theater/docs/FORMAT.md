# Native structure and fidelity

The native recording consists of a component registry/bootstrap chunk (type 1),
a replication packet stream (type 2 chunks), and summary data (type 3 chunks).
`Film` represents those as `registry`, `replication`, and `summaries`.

The bootstrap begins with version words and archetype/component registry blocks.
The captured v41 corpus has 50 accepted registry blocks and 1,067 named slots.
`registry.definition` retains the decoded component registry and its read diagnostics;
`registry` retains the entire input, including bootstrap data beyond that
structurally decoded registry. Identity/player searches over those bytes belong to
resolution rather than being presented as established native boundaries.

Replication packets use 16-byte headers. Native packet reads preserve their
header, payload offset, timestamp, ordered views/records/component fields, partial
reads and stops. Keyframe traversal advances through decoded record boundaries;
it stops at an invalid header or unsupported component and never searches ahead
for a plausible replacement boundary.

Summary packets retain declared counts and entries from the guarded v41 footer
layout. Each summary has its own recorded timestamp and XUID. Player linkage is a
resolved interpretation. The separate whole-chunk highlight search is available
only through resolved interpretation evidence.

## Source coordinates

Every parsed chunk retains its original `FilmChunk`, decompressed `data`, and
`source_position` in the supplied list. Source positions are never replaced by
manifest indices. Nested packet bit offsets are relative to the header's
`payload_offset` in decompressed data; compressed transport has separate storage.
The reader warns when packet framing stops before the end of a chunk. All bytes
remain retained in `data`. The bootstrap is retained whole rather than
misrepresented as a replication packet sequence.

## Unknowns and runtime settings

The v41 reader uses its fixed grammar defaults, not inferred map calibration.
Unknown component layouts and stopped reads remain explicit. Runtime-dependent
code-15 event bodies stop with `MissingRuntimeGate15`; the native parser never
chooses a gate by scoring candidate parses. Roster bodies needing an unestablished
personalization width remain refused/opaque. Map-relative translocator reads may
stop with `MissingMap`. Quantized component fields remain available; default map
bounds are not used to publish inferred world coordinates into Film.

Bootstrap/player searches, event-gate selection, bot candidates, modal fire-aim
interpretation and whole-chunk highlight scans run during resolution. Their results
are separate from native records and do not silently repair or replace stopped
native records.

Some native readers retain padded lookahead diagnostics; synthetic bits and partial
fields must not be treated as backed source bits. Independent native packet-head
and damage reads can overlap the generic body, so their results do not establish
an exhaustive, nonoverlapping partition of every source bit.

## Fidelity contract

All supplied source information is retained, with structural decoding where known.
Unknown bytes are not discarded or replaced by guessed records. This does not
claim every bit already has a typed schema. Original input bytes can be retrieved
unchanged; editing models and byte-for-byte re-encoding is not implemented.
JSON is an inspection format, not a replacement for native byte identity.

Scope is v41. Unsupported major versions fail before a v41 body reader is selected.
Map assets, rendering, interpolation and inferred physical-action semantics are
outside the native model.
