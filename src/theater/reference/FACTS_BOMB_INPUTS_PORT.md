# Cached bomb transitions and held-object carries

`replay_facts_bomb_held_events` consumes cached timestamp, slot, family and previous
family directly. Kind bytes and signed slot indices are not classified or narrowed.
The source and cache adapters share the same filter. Timestamp conversion casts
u64 to signed i64 before dividing by 1000, then subtracts the offset with wrapping.

`build_facts_replay_held_object_carry` consumes only cached death XUID and TimeMS.
Names need no UTF-8 conversion. The shared reducer preserves stable transition
ordering and supplied death order, including the first matching death rule.
Drops close directly; deaths close a period at the next pickup or open sentinel.
Carry sums retain native signed wrapping arithmetic.

`build_facts_replay_bomb_carries` shares source assembly, including the caller carry
recognition gate, missing identity bridge coverage, temporal occupant lookup,
match clock and player-presence publication. It preserves the complete raw carry
for later statistics. Arming recognition remains a separate caller decision.

The pinned native held-object fixture was regenerated with full-u64 random change
timestamps and signed-extreme death offsets. Its 2,048 cases compare source and
cache filters and raw reducers against native expectations, and compare publication
against native carries and coverage. Cache projections inject invalid UTF-8 names
and kinds and a minimum signed slot index into fields these algorithms ignore.
The high-level shared wrapper is source-inspected, not independently covered by
these reducer fixtures. Full document wiring and native runtime logs remain open.

This work does not start the deferred Film/ResolvedFilm/playback architecture.

## Native document attachment boundaries

`attach_replay_bomb_armings_to_document` consumes already published objective
items to select and sort detonation times, preserving its independent Scanned
gate. Disabled means no mutation. Enabled replaces armings, replaces their coverage
only when the document has a coverage envelope, and returns the reducer result
including start-zero fallback counts. It logs the suppressed/accepted verdict.

`attach_replay_bomb_stats_to_document` is gated by family-wide CarryScanned and
uses the raw carry timeline rather than published frame intervals. It consumes
all identified objective events, score presence, bridge status and match kill
read status. Arming readability comes only from document coverage: missing
coverage, missing arming coverage, unscanned or suppressed all mean unread.
The film-clock cast precedes division, followed by wrapping offset subtraction.
The kills-dropped diagnostic stays signed and does not enter published coverage.

Both helpers are usable by cached composition without source reads. A disabled
pass preserves any existing document content. Separate pinned native generators
call attachBombStats (2,048 cases) and attachBombArmings (1,024 cases), with seeded
preexisting fields, absent coverage, independent mode gates, suppressed armings,
full-u64 origins and signed-extreme offsets. Tests compare whole typed documents
before/after mutation and arming fallback counts. Runtime log calls are currently
source-matched; their emitted records are not independently captured by these
fixtures. These passes still need wiring into complete cached document assembly.

## Cached carry document attachment

`attach_facts_replay_bomb_carries_to_document` shares the source/cache carry reducer,
using the temporal identity state and presence built from the current document
tracks, including deduced-track exceptions. It returns the raw timeline before
frame/presence trimming for the later statistics pass. Missing document coverage
does not disable publication; disabled carry scanning leaves the document alone.
A missing identity bridge returns empty raw carries and publishes the native
transition-count coverage. Source-matched warning/info logs retain native order.

The new 1,024-case native attachBombCarries fixture checks complete typed document
mutation and the returned raw timeline, with recycled slots, missing bridges,
deduced and unnamed tracks, full-u64 timestamps and signed-extreme offsets.
The fixture normalizes nil raw carry arrays/maps to empty containers to match the
existing raw carry DTO; it does not establish nil-versus-empty roundtrip fidelity.
Injected invalid UTF-8 kind/name bytes and signed-minimum slot indices stay unused.
Runtime log emission is source-inspected rather than independently captured.

This carry fixture also retains native null track point arrays. The test projects
those to empty arrays because the legacy ReplayTrack DTO cannot represent null;
carrier presence consumes only lifetime bounds. Document equality is therefore
conditional on this explicit projection, not evidence of nil/empty JSON fidelity.
The first host/WASM attempt exposed this deserialization gap. It remains part of
the final replay-domain audit; the native fixture is kept unchanged.
