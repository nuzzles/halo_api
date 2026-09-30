# Resolution and playback

`Film::resolve(&self)` returns `ResolvedFilm<'_>`, borrowing the recording. It walks
already decoded reference records to build an ordered event stream, query indexes,
entity/component state and checkpoints. Multiple independent cursors can borrow
one film. The retained source buffers are neither copied nor decompressed again.

`interpretations()` exposes separate, source-linked evidence from bootstrap identity
and player-slot searches, event-layout inference, guarded summary candidates, bot metadata, fire-aim reads and
whole-chunk highlight scans. Those routines may inspect preserved bytes; they do
not rerun the structural packet parser or mutate Film. An inferred event gate is
reported as a selection with candidate counts, not a recorded bit, and is not used
to silently reinterpret the reference event stream or apply extra world updates.

Events use packet wire timestamps except summary candidates, whose source-read
milliseconds are converted to microseconds. Ties retain input chunk, packet, and
record order. Canonical summary packets own a count and an opaque record stream.
Guarded v41 candidate searches live in `interpretation::summary`; their owned
reads retain all 16 UTF-16 units and raw flags. `summaries()` exposes decoded text
and semantic kinds with `SummaryDerivation::GuardedV41Layout`. Summary index events
use `Provenance::DerivedSummary`; candidate association is never advertised as a
canonical decoded record. Player linkage uses a unique XUID match from bootstrap
interpretation; missing or ambiguous matches stay unresolved. This is not a
player-to-entity ownership inference.
Reference reads have source references and explicit read provenance. Accumulated state
changes retain previous/new shared values and a derivation identifier.

`EventFilter` combines kind, category, entity, player and inclusive time ranges.
Queries begin from the smallest applicable bounded index. Continuous component
updates stay available; filtering does not change playback or the stored events.

- `current()` borrows the already-materialized world in O(1).
- `advance_to(t)` applies intervening indexed updates. Backward movement seeks.
- `seek(t)` binary-searches the event/checkpoint indexes, clones a checkpoint world
  map, and applies at most 1,024 remaining events. Cloning scales with entity count;
  seeking is not O(1).
- Iterating/copying the world scales with its size. Checkpoints and historical state
  use memory even though unchanged entity states share storage.

A new resolved cursor has an empty current world until advanced or sought. Partial
reads cannot silently promote unknown component values to known state. Incomplete
NEW records do not establish entities. Keyframe data beyond a parsing stop cannot
create world entities. Seeking and sequential playback must agree at the same time.

No visual interpolation or inferred jump/shot/action semantics are added here.

The planned `TheaterRuntime` will own a private resolved representation. That
architecture and higher-level typed summary-event API are deferred until after
the canonical-model/parser phases; the existing resolved API only migrates to
consume the updated canonical hierarchy in this phase.
