# Canonical structure and fidelity

A v41 recording consists of a component registry/bootstrap chunk (type 1),
replication packet-stream chunks (type 2), and summary chunks (type 3). `Film`
retains the required registry separately and stores every following chunk in one
ordered `chunks` vector. Filtering helpers do not create a second ordering.

The bootstrap begins with version words and archetype/component registry blocks.
`registry.body` retains the ordered decoded registry, the two recorded header
words, the accepted archetype blocks, the terminal byte, and its stop. The chunk
also retains all bootstrap bytes beyond the structurally decoded registry.
Identity and player-table searches over those bytes belong to resolution because
their boundaries are not established by the registry grammar.

Each archetype owns ordered `RegistryComponent` entries. Each component retains
all 256 name bytes, its recorded precision level, and its complete 260-byte source
range. `name()` borrows the accepted ASCII name without replacing the raw bytes.
The archetype records its complete block range and zero-padding range. A rejected
boundary block is not an archetype; its bytes remain in the bootstrap after
`registry_end_byte`. There is no duplicate slot trace or parallel levels array.

`RegistryComponent::precision_level` is the little-endian 32-bit value immediately
after the 256-byte name. It is a component-specific encoding parameter, rather
than a universal bit count. Some v41 quantized vector readers use
`min(6 + level, 26)` bits per axis; other readers use it to gate additional fields
or select layouts, and some ignore it. The parser retains the recorded value
exactly. Converting quantized values into world coordinates belongs to resolution.

Replication packets use 16-byte headers. `PacketSource` identifies each header
and payload byte range in the enclosing decompressed chunk. `PacketRead`
distinguishes `Decoded` (a supported body decoder ran) from `Opaque` (no decoded
body). `Decoded` does not promise a complete payload: nested typed bodies retain
their own decoding stops. `PacketStream::opaque`
records ranges that framing could not classify as packets. Unsupported packet
layouts never become successful placeholder bodies.

Packet type 0 retains its configuration bit and reads the message view at bit
one, followed by entity and control views at the established message terminator.
Messages own their ordered fields; the list owns its recorded terminator. The
parser stops before entity traversal when the message boundary is unavailable or
when supported message-body readers disagree on its extent. It does not publish
an independent overlapping frame attempt as a second recorded section. Type 1 retains the separate wire sections: all
79-bit datum headers, then all 256-bit component masks in matching slot order,
five tail words, and alignment bits. Each section owns its source range; mask
words retain their MSB-first wire values rather than reversed semantic indexes.
Type 2 contains
keyframe/baseline records. Types 7 and 8 are represented explicitly as end and
roster packet layouts; the roster body remains opaque until its grammar is known.
Component records directly own their component fields. `RawBits` retains the full
recorded bit sequence of every decoded field, including fields wider than 64 bits.
Readers stop at source bounds and never synthesize zero padding.

NEW records and keyframes own `DefaultState` sections separately from their
surrounding guards, masks, and component updates. Each default section retains its
ordered fields, source extent, and complete/opaque/unsupported/truncated status.
A closed default-state guard produces no section. An entity archetype is stored
only when NEW explicitly records it; the schema used to decode a later DELTA
remains private parser context. Reader callback categories,
synthetic absent-reference zeros, and reversed action-mask projections are not
canonical fields; their recorded gates and words remain in the owning fields.

`Packet::payload` validates its envelope against the enclosing chunk's recorded
header and payload size. Public construction and serde do not automatically
validate nested model invariants. `Packet::source_coverage` computes a complete
payload partition from owned structural fields: `Fields`, `Opaque`, `Padding`,
and `Unparsed`. Uncovered gaps are never assumed to be alignment. Coverage costs
O(n log n) for n region boundaries and allocates a result; it performs no second
byte decode. Invalid source ranges or conflicting classifications return `None`.
Opaque packet bodies cover their entire retained payload. Known roster envelopes
have no payload grammar and their payload remains unparsed.

Event `end_bit` bounds the furthest source-backed field, including partial nested
reads. An `Unavailable` field retains the requested position/width as a read
attempt, not an actual source extent; it contributes no decoded coverage.
Field publication stops at the first failed primitive. Subsequent shorter reads
at that unchanged position do not become recorded flags or reference gates.
All remaining bytes/bits stay available through the enclosing chunk payload.

Summary packets retain their recorded count and the entire opaque record-stream
range. A sequential summary record grammar is not established, so the canonical
parser does not invent records by scanning marker patterns or using an observed
identity-to-tail offset. `ResolvedFilm::summaries()` retains the guarded v41
candidate search, timestamps, XUIDs, all 16 UTF-16 code units, and raw flags with
source references. Its associations are marked `GuardedV41Layout` and its event
index entries use `DerivedSummary` provenance, even when individual values were
read directly from bytes. Text, kinds, and player linkage remain interpretation.

## Source coordinates

Every parsed `Chunk<T>` retains its original `FilmChunk`, decompressed `data`,
position in the supplied list, transport/decompression outcome, and typed body.
Source positions are never replaced by manifest indices. Nested packet bit offsets
are payload-relative; `PacketSource::payload` locates that payload in the chunk.
All source bytes remain available even when decompression, framing, or body decoding
is partial.

Byte and bit ranges are half-open. Unknown chunk kinds are retained as
`FilmDataChunk::Unknown`. Known packet envelopes with unsupported body layouts are
retained as `PacketRead::Opaque`. Typed bodies carry their own ordered stop and
opaque-region models where decoding can end partway through a known layout. This
keeps recorded absence distinct from parser inability.

## Decoder scope

The v41 reader uses fixed grammar rules and does not search ahead for plausible
record boundaries. Unknown component layouts and stopped reads remain explicit.
Runtime-dependent event layouts stop when their required gate is unavailable.
Vehicle physics stops at the independently checked component boundary with
`RuntimeContextUnavailable` because the game flag `vehicle+0x818` is not recorded.
The pinned reference assumes that flag is set; canonical decoding does not.
Roster layouts needing an unestablished personalization width remain opaque.
Map-relative values remain quantized unless external map data is supplied during
higher-level interpretation.

Bootstrap/player searches, event-gate selection, bot candidates, fire/aim
interpretation, and whole-chunk highlight scans run only during resolution. Their
results are source-linked interpretations and do not repair or replace canonical
records.

## Fidelity contract

Structural fidelity means preserving supplied bytes, hierarchy, order, recorded
identities and timestamps, masks and quantized values, source ranges, unknown
regions, and explicit decoding stops. It does not mean every bit already has a
typed schema. Original input bytes can be retrieved unchanged. Editing the model
and byte-for-byte re-encoding are separate work and are not implemented. JSON is
an inspection format rather than a replacement for source-byte identity.

Scope is v41. Unsupported major versions fail before a v41 body reader is selected.
Map assets, rendering, interpolation, and inferred physical actions are outside
`Film`.
