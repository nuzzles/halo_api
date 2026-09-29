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
distinguishes complete, partial, and opaque body decoding. `PacketStream::opaque`
records ranges that framing could not classify as packets. Unsupported packet
layouts never become successful placeholder bodies.

Packet type 0 contains ordered frame events, views, entity records, components,
and controls. Type 1 contains every datum-table slot in wire order. Type 2 contains
keyframe/baseline records. Types 7 and 8 are represented explicitly as end and
roster packet layouts; the roster body remains opaque until its grammar is known.
Component records directly own their component fields. `RawBits` retains the full
recorded bit sequence of every decoded field, including fields wider than 64 bits.
Readers stop at source bounds and never synthesize zero padding.

Summary packets retain declared counts and an ordered sequence of decoded events
and opaque gaps from the guarded v41 footer layout. Each event stores its recorded
timestamp, numeric XUID, UTF-16 code units, raw flags, and bit ranges. Text decoding,
summary kinds, and player linkage are exposed by `ResolvedFilm::summaries()`.

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
