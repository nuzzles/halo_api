# Canonical structure and fidelity

A v41 recording has a component registry/bootstrap chunk (type 1),
replication packet-stream chunks (type 2), and a terminal
summary/highlights chunk (type 3). `Film` stores these as `registry`,
`replication_stream`, and `summary: Summary`. Parsing requires
exactly one registry first, at least one replication chunk, and one summary
last. Duplicate indices return `DuplicateIndex`; misplaced sections return
typed errors. Replication chunks
are sorted by manifest index during parsing. Replication after a summary returns
`SummaryNotLast`. A registry-only input returns `MissingSummary`; a present empty
or unread summary remains a valid section.
All 32 parser corpus manifests and 29 Twin fixture manifests contain a single
registry, replication chunks, and a single final summary; no captured exceptions
were found. Packet-level tests explicitly supply empty summaries where appropriate.
Complete-film admission checks framed type-0 payloads for the 27-bit
high-frequency update signature followed by its counter. All 32 captured
films have a marker. Marker presence is source evidence only, and this check
runs before parser packets are consumed into typed observations. Damaged zlib
and resource exhaustion still return errors. `Film` does not retain the tick
scan's packet locations.

Replication packet type 7 admits `EndPacketBody` only for an empty payload.
A nonempty body retains its envelope and exact bytes with `UnsupportedLayout`,
and its payload coverage is opaque. This is a refusal of unestablished grammar,
not proof that such packets are illegal. Framing follows fixed 16-byte headers
and declared payload sizes, admitting empty envelopes of any type and continuing
after type 7. Malformed or short suffixes remain opaque; no marker scan repairs
them. A later packet after type 7 is structurally framed, but runtime continuity
marks the transition as uncertain because the marker does not prove state carries
through it.

The bootstrap begins with version words and archetype/component registry blocks.
The private `ParsedRegistry` retains the ordered decoded registry, the two
recorded header words, the accepted archetype blocks, the terminal byte, and
its stop. The parser retains bootstrap bytes beyond the decoded registry until
it projects the finished `Film.registry`.
For the established v41/27 layout (50 blocks, known registry fingerprint,
endpoint `0xcb208`), `bootstrap_header` additionally retains the next 123 event-
version words, three fixed 32-byte build buffers, two words and the corruption-
check bit. Its fields own exact raw bits and ranges; truncation retains complete
fields and the first unavailable field. No printable build-string search selects
this boundary. Other layouts have no `bootstrap_header`, and later bootstrap
bytes remain retained. After a complete fixed header, `identification` reads the
next 12,848 bytes at their recorded one-bit shift: two 256-byte buffers, a
little-endian timestamp, three words, three 4,096-byte blocks and two map-variant
GUIDs. The buffers/blocks retain exact bytes without inferred internal meanings;
zero timestamps and GUIDs are not erased. Its completion establishes the game-
options body boundary. Its `game_options` read walks the ten prefix fields,
first Bond message, bounded NUL-terminated strings, gated blocks/second Bond
message and fixed tail in recorded order. Only a complete read establishes the
first of the 32 player records. Runtime identity uses these canonical fields on the supported
layout. Its older build-string fallback on other layouts is explicitly labelled
by `IdentityRead::boundary_source`; a truncated canonical section never falls
back to that search.

Both messages retain generic Bond Compact Binary v2 syntax: declared structure
lengths, ordered fields (including duplicate IDs), original extended headers and
varints, base/structure terminators, lists/sets and ordered map entries. All wire
types 2–18 are represented. Strings and floating-point words retain their raw
bytes; no semantic defaults, text replacement or numeric rounding enter `Film`.
Fixed-width scalar arrays retain one contiguous raw payload instead of one node
per element. Declared bounds, integer overflow and truncation are checked without
zero-padding or resynchronization. Nesting beyond depth 64 and a budget of 100,000
value/header nodes return explicit refusals; these are resource policies, not
claims about format maxima. Unread suffix bytes remain in the registry chunk.

Runtime player-table decoding uses the complete canonical game-options endpoint
and labels it `CanonicalGameOptions`. An incomplete canonical body refuses that
projection without scanning for later player-shaped bytes. Other layouts retain
the explicitly labelled `PatternSearch` fallback. Player-slot semantics and
build-specific personalization widths still live in runtime; this boundary does
not establish a complete canonical player-record grammar.

Script event 15 is admitted under recorded version 1 with complete canonical
game options and a valid recorder simulation. Values 0/1/3 select the 15-bit
prefix; value 2 omits it. Its following 13-bit word, 10-bit payload bit-count and
raw payload retain their exact source extents. Both canonical parsing and replay
use the recorded writer context; missing or unsupported context stops before the
body. The playback reader's current simulation is not the recorder's simulation.
The simulation-2 writer branch has synthetic/source evidence but no local
captured example. The whole-table admission rule below also applies.
If complete game options cannot establish the recorder simulation, ordinary
`Film::parse(chunks)` leaves script event 15 unresolved. Parser tests can supply
independent writer facts to exercise this branch, but the public parser does
not expose a writer-context argument. The internal read retains
`MissingRuntimeGate15` before the event body when that fact is unavailable.

Selected variant schema semantics are derived privately from the complete Bond
AST without replacing recorded values. Exactly one instance in the four optional
variant lists is required. Missing parents, wrong types or conflicting selected
fields remain unknown; omitted scalar defaults apply only inside the complete
owning schema structure. Field IDs are scoped to their inheritance segment, so
base fields cannot masquerade as the derived schema's fields. Raw duplicates,
base terminators and all original values remain unchanged in `Film`.

Event 116 requires a known engine type when its first orientation gate is set.
Missing context stops immediately after that gate. Engine 1 selects the inverse-
gated 30-bit direction and unconditional 30-bit roll of FUN_142e29bac; other
engines select the 19-bit direction / 8-bit roll. A clear first gate needs no
variant context. The engine-1 composition is source-analysis/synthetic evidence,
not a captured-film claim.
For event 85, the known kill fields precede a conditional 68-bit tail. Its absence
is proven only when play-of-the-game is disabled and either killcam is disabled
or engine type is 1. Otherwise `MissingKillPlaybackContext` preserves the base
fields and leaves following bits unparsed. The unrecorded runtime switches are
not silently assumed false in canonical parsing.
Private parser tests can supply `FilmParseContext { kill_event_tail: Some(true | false) }`
when the writer's branch is independently known. A present
tail reads three separate raw fields of 32, 32 and 4 bits; each truncated prefix
remains source bounded. `EventListRead.kill_tail_context_used` records the
external selection actually needed for that event list, separate from recorded
fields and replay assumptions. A supplied present tail that contradicts a
recorded variant proving absence stops at `ContradictoryKillPlaybackContext`.
The parser orders supplied chunks by manifest index. Public `Film::parse` retains its strict
missing-context stop.

Contextual replay can use its previous disabled-playback/non-engine-1 defaults
where recorded context is unavailable. Actual uses are labelled in
`EventListRead.assumptions` and retained with source ranges in replay diagnostics.
It never overrides a recorded engine type 1. Canonical event lists have no such
assumptions; fewer completed views after an uncertain kill tail reflect this
stricter boundary, not discarded source bytes.

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
Messages own their ordered fields; the list owns its recorded terminator. Before
any nonempty message's references or body, the supported v41/27 bootstrap must
record all 123 native event versions and the frame configuration must be one.
The first missing or mismatched table entry produces `MissingEventVersion` or
`UnsupportedEventVersion`; the error's event code identifies that table entry,
which may differ from the message code. Configuration zero produces
`UnsupportedReferenceConfiguration` because its global reference range is not
established. The complete continuation/code header is retained, without rewinding
or reading references. Earlier header truncation/unknown codes retain their stops.
An empty list terminator needs neither admission condition. Shorter tables and
other majors do not inherit upstream's numbering/closure hypotheses.

The f050371 event extension supplies source-field readers for codes 2, 8, 12,
13, 14, 80, 104, 105 and 106. Melee-clang (12) is conditional, replacing the
former fixed 94-bit skip: two optional players, an optional tag, a precision-12
point and a 19-bit direction. Collision damage (106) also uses precision 12;
networked AI effects (80) select zero, one or three precision-16 points. The
existing point readers retain fixed default-box widths and refuse unavailable
indexed context. These additions do not change version/reference admission or
interpret engine IDs, tags and numeric codes as gameplay outcomes. Independent
traces and branch coverage are in `fixtures/events-f050371-v41`.

A complete native version table can be used before unrelated later bootstrap
fields arrive; missing corruption policy still stops at its own boundary. Fire
v3 is admitted as part of this table. Layout admission is separate from map
context: short/map-independent
and default-box fire bodies decode canonically; indexed points retain their gate
and stop without supplied widths. Unknown fire versions are not treated as v3.
This establishes the selected native table, not all historical writer layouts. The
parser stops before entity traversal when the message boundary is unavailable or
when supported message-body readers disagree on its extent. It does not publish
an independent overlapping frame attempt as a second recorded section. Type 1 retains the separate wire sections: all
79-bit datum headers, then all 256-bit component masks in matching slot order,
five tail words, and alignment bits. Each section owns its source range; mask
words retain their MSB-first wire values rather than reversed semantic indexes.
Type 2 contains a configuration bit and 8,191 physical keyframe-table entries.
Each entry has a 108-bit header (three 32-bit words, four flags and one byte).
An all-ones archetype means the entry has no body; traversal continues to the
next physical slot. An all-ones ID is not an end marker, and nonmonotonic IDs or
generation zero do not determine the next boundary. `KeyframeTable.slots` retains
all complete headers in physical order; `records` contains the live bodies.
Only all 8,191 entries with complete bodies and fewer than eight remaining bits
establish `End`. Short tables are truncated; extra bytes are `TrailingData`.
Unsupported bodies stop traversal without searching for a later header. See the
[native table fixtures](../fixtures/keyframe-slots-v41/README.md).
For a type-40 vehicle snapshot, the writer's construction-time `vehicle+0x818`
gate may be absent from the film. Private parser tests can supply an externally
established gate in `FilmParseContext.vehicle_snapshot_gates`, keyed by supplied
chunk index, zero-based packet index and full snapshot ID. An unindexed chunk
cannot select an entry. The gate applies only to that snapshot record; its
`external_vehicle_type_gate_used` field reports the selected external value
when a gated component is reached. Both true and false branches have bounded
wire tests. Without a supplied gate, the ordinary parser retains its explicit
`RuntimeContextUnavailable` stop. A plausible following slot does not establish
the construction state.
Type 7 is an explicit end packet. Types 8 and 12
own count-driven roster/bot metadata with raw player fields, build context and
explicit partial stops; see [ROSTER.md](ROSTER.md).
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
In a type-0 NEW record, a clear initial-component gate has no following mask;
the next bit begins the next entity header. Creation archetypes 7, 13, 14, 20,
23, 28, 29, 46, 47 and 49 omit that gate entirely. A mask and component fields
exist only when a gate-bearing creation records a set gate. The retained first
Octagon frame verifies the gate-clear successor boundary and all 207 entity
entries against the current libfilmshell entity view.
For type-9 managed-player creation, complete NEW and keyframe records expose
`managed_player_participant()` only when the recorded six-bit `player_index` is
below 32. Values 32–63 remain exact raw default-state fields; the outer entity
ID and roster position are separate domains.

`Packet::payload` validates its envelope against the enclosing chunk's recorded
header and payload size. Public construction and serde do not automatically
validate nested model invariants. `Packet::source_coverage` computes a complete
payload partition from owned structural fields: `Fields`, `Opaque`, `Padding`,
and `Unparsed`. Uncovered gaps are never assumed to be alignment. Coverage costs
O(n log n) for n region boundaries and allocates a result; it performs no second
byte decode. Invalid source ranges or conflicting classifications return `None`.
Opaque packet bodies cover their entire retained payload. Roster/bot records
classify scalar fields and bounded opaque blobs separately; unavailable fields
and type-8 trailing bits remain unparsed. Only closed type-12 reads establish
up to seven padding bits.
Native keyframe headers count as fields, including vacant entries. Keyframe
alignment is classified only after complete table closure; other suffixes remain
unparsed. Exact header words and the original packet bytes are retained.

Event `end_bit` bounds the furthest source-backed field, including partial nested
reads. An `Unavailable` field retains the requested position/width as a read
attempt, not an actual source extent; it contributes no decoded coverage.
Field publication stops at the first failed primitive. Subsequent shorter reads
at that unchanged position do not become recorded flags or reference gates.
All remaining bytes/bits stay available through the enclosing chunk payload.

The recorded bootstrap corruption-check flag overrides Film and replay profiles.
When enabled, every nonempty event body is followed by R1 and, when set, R32.
The 13 empty descriptors omit this trailer. `EventFieldStage::CorruptionCheck`
retains its gate and arbitrary sentinel word; incomplete trailers stop the list.
Component trailers have the same optional form. Positive snapshot default blocks
instead contain an unconditional R32, which is absent from NEW defaults.
`BootstrapHeaderRead::corruption_checks()` preserves missing flags as `None`.
Film and replay retain that unknown policy and stop at the first affected body
boundary with `MissingCorruptionCheckContext`; no guard or successor is read.
Known prefix fields remain available, and default/component sections carry
`ContextUnavailable`. Empty events, empty masks, vacant slots and nonpositive
snapshot body guards can complete without that policy. A NEW header's established
schema can still bind a later empty DELTA. Internal configured research readers
remain separate from public Film/replay admission.

The recorded packet configuration also governs shared variable-width handles in
NEW/DELTA bodies, snapshot defaults/components, and control actions. Configuration
zero requires a global reference range that is not established here. A present
handle retains its presence gate and, for category 1, its probe; parsing stops
before the value/generation fields. Entity and keyframe records report
`UnsupportedReferenceConfiguration`, with `ContextUnavailable` on the affected
section. Controls retain an unsupported-view reason and their exact prefix.
No corruption guard or successor is read after the stop. The legacy position
handle-tail path follows the same rule even when a profile supplies a width.
Fixed-width posture words use a different native primitive and remain readable.
Absent handles, empty masks, vacant slots and other unaffected prefixes require
no global range. A known NEW schema can still bind a later empty DELTA. The flag
is selected anew for each packet; it does not modify the inherited profile.

Control views keep complete records in wire order as `ControlEntry` values with
`ControlPayload::Input`, `Commands`, or `Kind2`. Kind-0 projections live in
`ControlInput`; `ControlEntry::index()` returns only an explicitly recorded player
index. Kind 2 has no established player field. Command lists distinguish a
closed gate from a recorded zero count, and each complete command and secondary
unit block has a source range. The enclosing view owns all exact fields,
including optional gates, raw floats, delta prefixes and partial record prefixes.
Kind 3 stops immediately after its kind field because the references disagree
on the following layout. New control grammars have pinned upstream synthetic
boundary tests; they do not establish named physical inputs or full coverage.
The full secondary block now follows f050371's corrected branches: its second
optional reference uses category 1, a local-position flag selects three 14-bit
words instead of a world point, and a present target carries the handle followed
by an 11-bit raw word. Absent targets carry neither. Local and default-box
positions are self-contained; indexed positions still require explicit context.
Configuration-one reference admission remains mandatory for present handles.
The 336-case `controls-f050371-v41` oracle validates field and successor boundaries;
the three older full-secondary fixture traces remain explicitly historical.
The control reader's record cap is a configurable resource policy, not a format
maximum. `ParseLimits.max_control_records` defaults to 65,536, admitting the
independently traced 65- and 128-record lists. At any configured limit it still
reads the next continuation bit: zero closes the list, a missing bit is
truncated, and one retains that bit and returns `RecordLimit` before the next
kind/body. Complete records remain inspectable and the unread suffix stays
unparsed. A configured 64-record limit retains the earlier boundary behavior.

Replay diagnostics now report incomplete control views at their consumed-prefix
end, alongside existing event/entity/snapshot blockers.

Summary packets retain their recorded count and the entire opaque record-stream
range. A sequential summary record grammar is not established, so the canonical
parser does not invent records by scanning marker patterns or using an observed
identity-to-tail offset. `TheaterRuntime::summary_highlights()` retains the guarded v41
candidate search, timestamps, XUIDs, all 16 UTF-16 code units, and raw flags with
source references. Its associations are marked `GuardedV41Layout` and its runtime event
index entries use `DerivedSummary` provenance, even when individual values were
read directly from bytes. Text, kinds, and player linkage remain interpretation.

## Source coordinates

Each parsed registry, replication and summary section owns its inflated `data`,
manifest metadata and typed body. Compressed input is dropped as inflation succeeds.
Nested packet bit offsets
are payload-relative; `PacketSource::payload` locates that payload in the chunk.
Inflated source bytes remain available when framing or body decoding is partial.
Valid RFC 1950 window headers are admitted, and each compressed chunk may produce
at most 256 MiB of output. Invalid or incomplete zlib streams return
`ParseError::Inflate`. Packet framing never scans marker patterns through an
unwalked suffix.

`Film::parse` applies internal resource limits: 256 MiB of decoded bytes
per chunk and 65,536 envelopes per replication/summary chunk. These are
policies, not format maxima or a whole-process heap cap; decoded models also
consume memory. Chunk inflation exhaustion returns `ParseError::Inflate` with
the manifest index and `InflateError::Limit`. Packet-count exhaustion returns
`ParseError::PacketLimit`. Neither returns a partial `Film`. A limit reached
exactly at the chunk end is not exhaustion. Type-7 and malformed-envelope stops keep their existing framing
behavior. Unread framing suffixes interrupt runtime continuity without assigning
clocks to unread data.

Byte and bit ranges are half-open. Unknown chunk kinds return
`ParseError::UnsupportedChunkKind`; they are never silently discarded.
Known packet envelopes with unsupported body layouts are
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
Roster/bot entries needing an unestablished personalization width retain the
read prefix and stop explicitly before the personalization block.
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

Canonical DELTA traversal requires an inherited schema binding; it stops at the
header when that binding is absent. Plausible bodies and successor records do not
establish canonical component fields. Contextual replay may opt into runtime's
schema recovery, whose evidence is returned in `ReplayDiagnostics.schema_recoveries`
with full IDs, proposed archetype and both candidate/successor extents. That
recovery does not modify the stored Film. This separation does not imply every
other contextual/default/structural grammar has completed its canonical audit.


## Build-bound multi-region positions

A supplied region may have zero-width position axes. The original width law
returns zero bits for a positive range fitting one quantization bin; forcing one
bit would consume the following field. The named canonical axis remains present
at its exact cursor position, with width/bit length zero and empty raw bytes.
This is a schema-defined zero code, not an omitted/truncated field replaced by
zero. Bounds must still be finite and increasing. Unknown regions and missing
context stop before axes, and failed earlier reads cannot introduce zero fields.
Component and event readers retain the same policy. See
[zero-position-axes-v41](../fixtures/zero-position-axes-v41/README.md) for original
width/point traces, native continuation and every-bit truncation tests.

The v41 reader may derive position widths from the external libfilmshell
catalogue only when complete canonical bootstrap/identification/game-options
sections record engine BuildId 269225, branch HI_1_13_0 and a known LevelTag.
The product-version string uses a different namespace. The source catalogue
label `269225.26.04.08.1618 HI_1_13_0` is retained intact; its date/time suffix is
not a separately decoded film field. Other builds, branches, tags or incomplete
sections do not inherit this catalogue.

All 65 BSP regions across 33 levels retain exact source float words and axis
widths. Index selection precedes axis reads; known regions use their own widths
at object precision 16 and projectile precisions 15/12. Unknown indices stop
before axes, without primary-region fallback or opaque/stub substitution.
Default-box object positions use three fixed 22-bit words, including when no
map context is supplied. Default event points use 22/21/18-bit words at their
respective precisions. Canonical fields keep their codes and recorded regions.

Runtime converts player, object and detonation positions using the selected
region's bounds. It admits additional catalogue regions only if the caller's
primary bounds and index width agree exactly. Default-box positions and unknown
regions do not become primary-region coordinates. No render transform or
interpolation is introduced. This catalogue does not establish every event
version, component branch or game construction setting.


## Shared component descriptor boundaries

An attached `object-position-component` retains three 14-bit parent-local codes,
an optional 13-bit object index plus two generation bits, an independently
optional 11-bit region and the two-bit finite tail. Its complete length is
47, 58, 62 or 73 bits, including the first discriminant. These local codes do not
become world positions. The old fixed 60-bit path was not a valid substitute.

`high-frequency` has two descriptor tables: archetype 3 records 16/8/2-bit fields;
archetype 4 records an eight-bit counter. Other archetypes have no established
table and stop Unsupported before the body. The other names sharing libfilmshell's
internal Bits8 enum do not inherit this high-frequency-specific rule.

Dynamic orientation retains the instruction-addressed LevelUp layout: the
version>=2 configuration flag can override the initial raw selector, and a clear
delta-roll gate has no payload. libfilmshell disagrees on those branches, so its
readers cannot serve as the sole oracle there. The existing parser behavior is
preserved and independently traced through the pinned LevelUp functions.


The equipment tracked-object stack records a four-bit entry count followed by
exactly that many optional category-0 handles (presence, then 13-bit index and
two-bit generation). Count zero has no entries; no sentinel or extra handle is
consumed. Activation and control-signal references instead use optional
category-4 handles (nine-bit index plus two-bit generation). These raw codes and
gates remain source fields; runtime interpretation is separate.


Actor-state target slots record positions on both non-object branches. The
0x10 argument to FUN_14076e494 selects precision level 16; it is not a 16-bit
quaternion. These positions use default/indexed region codes in deltas and
three raw float words in snapshots. Optional words, object handles and the
conditional eight-bit target tail retain their original gates. The actor-state
header still uses its recorded row-version threshold (8/10/11/12 bits).
Actor-control action vector mode 0 likewise uses the enclosing snapshot mode;
mode 1 remains a 19-bit direction, and modes 2/3 have no vector payload. Missing
map context stops at an indexed point, while raw/default-box points remain
self-contained. Explicit non-snapshot full-precision context remains supported.


Tacmap waypoint state, co-op tether area, display assets, areas of interest and
POI icons use the precision-level-16 point grammar: three raw words in snapshots,
or a default/indexed box followed by the selected widths in deltas. The older
traversal widths and registry-level-derived vector widths are not canonical
alternatives at these call sites. A waypoint also carries one trailing bit when
its registry row version exceeds 1. Missing map context preserves the fields
before an indexed-point gate and stops there; default-box and raw points remain
self-contained. Existing fixed prefix/tail fields retain their original order.


Non-predicted ability state reads a two-bit outer tag. Tag 3 carries an encoded
three-bit label (label = raw + 1), a category-1 optional reference, a place and
six flag bits. A present reference selects a parent-local place: two bits,
three 13-bit words and an optional 16-bit word, including in snapshots. Without
a reference, the place uses raw snapshot words or a default/indexed precision-16
point. Labels 1–6 carry their recorded handle/vector/point/optional-byte bodies;
labels 7/8 carry none. Label 6 omits the nine-bit tail used by labels 3–5. A
three-bit outer tail follows whenever the registry row version exceeds 1.

All gates and fields remain source-backed. Missing map context refuses an
indexed point after its gate, retaining prior fields; raw, parent-local and
default-box positions need no map context. Internal ability observations retain
reference values and position source ranges rather than assuming a world-space
coordinate representation. This completes the documented body branches without
claiming a semantic grappling trajectory. Source/fixture details and remaining
source disagreements are recorded in the robustness ledger.

Predicted ability tag 3 carries an optional six-bit value followed by an
optional precision-16 point, independently of registry row version. The pinned
initializer analysis clears the state bytes used by the extra-target branch;
the six-bit value does not select that branch. Separate source traces cover
the cases where libfilmshell disagrees with this analysis.

Both mobility points and all posture points also use precision 16. Raw snapshot
and default-box forms need no map table; an unavailable indexed form stops after
its gate. These biped readers retain explicit full-precision/baseline overrides
on non-snapshot reads. The control-context word is distinct: only the global
full-precision switch expands it from two to four bits, followed by one flag.


Managed-engine timers carry a 64-bit slot mask and visit set slots in increasing
index order. Tag0 has no body; tags2/3 carry R16/R16/R5; tag1 appends another R16
after that five-bit state. The state is not the final field of a tag1 body.

Multiplayer-property mode-2 fields retain their source-described typed traversal,
including variant-zero submessages and terminators0/1. The current limits are
parser resource limits, not film grammar maxima: exceeding body/count/depth
limits or exhausting the message-field cap without a terminator yields an
unsupported component at the retained prefix. No declared length/count is
clamped into a shorter successful component. A u64 LEB128 varint must terminate
within ten bytes and the last byte can carry only one value bit. Unsupported
resource/representation cases cannot be continued by an opaque fallback stub.

Damage-aftermath event0 carries fifteen independent flags after the optional
five-bit field; only the fifteenth gates the following 32-bit word. Canonical
fields retain each flag separately, including complete flags before a truncated
packet ends. This changes field granularity without changing complete-body size.

Projectile-detonation event5 selects an 8-bit direction when its tail gate is
zero and a 19-bit direction when it is one, followed by two bits. Original
LevelUp source traces establish both branches; libfilmshell's fixed eight-bit
tail only agrees with the zero-gate branch. Default-box and indexed delta points
use precision15 for detonation and precision12 for impact. These fixtures do not
establish snapshot/full-precision event admission or additional build versions.

### Managed-property enum widths

The shared class 13 property/default variant reader preserves tags 11–15 as
separate four-bit tags, each followed by a four-bit enum code in masked mode.
Scalar tags 7–15 and masked tags 0–6 carry no payload. This follows the native
switch default `FUN_141fce2f0 -> FUN_1407ef804`, independently checked against
pinned LevelUp source and captured vectors. libfilmshell's 32-bit fallback for
masked tags 12–15 disagrees; its original fixtures are retained as comparison
artifacts. See [property fixtures](../fixtures/property-v41/README.md).

This field width does not resolve all class 13 NEW-envelope/build contexts or
assign property names to scores, viewers, teams or zones. The parser retains
raw tags and codes; semantic enum/score projections remain runtime concerns.

### Exact-record DELTA schema context

Private parser tests can supply an independently established archetype for
one DELTA header through `FilmParseContext.delta_record_archetypes`. The key is
the indexed chunk, packet index, payload-relative start bit and full generation
ID. An absent or mismatched key leaves ordinary DELTA admission unchanged; an
archetype absent from the parsed registry is rejected. A reached matching DELTA
uses that schema for its own body and reports the external source in
`ProductionFrame.external_delta_schemas`. Its `EntityRecord.archetype` remains
`None`, because the DELTA did not record one. The supplied schema is not bound
for subsequent records. This is conditional decoding, not proof that the film
itself identifies the missing archetype.
