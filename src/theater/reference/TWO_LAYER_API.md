# Two-layer Theater API (2026-09-27)

## Design proposed before implementation

The primary types are `theater::film::Film` and
`theater::resolved::ResolvedFilm`, also re-exported by `theater`.
Supporting records, source adapters, options, filters, and borrowed views remain
strongly typed; “two types” means two aggregate entry points.

`Film::parse(&FilmSource, ParseOptions)` is the single native parsing entry.
The actual Theater transport is an ordered chunk set, often accompanied by a
manifest, rather than a documented single-file container. `FilmSource` is the
existing portable input adapter for that recording. Filesystem loading is a
separate adapter. No new container format is introduced.

Film owns the recording-only decoder's chunks, packets, nested views, records,
raw fields, summaries, and complete decompressed source. Native order, IDs,
quantized values, signed payload-relative bit coordinates, stops and synthetic
padding diagnostics survive unchanged. Unknown bytes remain in their owning
chunk. Recovery candidates and overlapping diagnostic projections remain marked
as such: they are not a second canonical record stream. Decoder configuration is
metadata, not recorded state. Packet and bit coordinates address decompressed
source. Original loader input bytes are also retained separately, including
compression and trailing transport bytes. Re-encoding modified decoded data is
not implemented. A faithful representation does not imply every unknown schema has
been decoded.

`Film::resolve(&self) -> ResolvedFilm<'_>` walks this structure,
borrows it for source references without copying source bytes, and indexes packets in timestamp order with
source chunk/packet order breaking ties. Resolution reads decoded records only,
never scans source bytes again. Entity lifetimes and accumulated raw component
state are materialized from sequential keyframe and entity records. Unproven
recovery candidates do not mutate the world. Unsupported and partial reads stay
visible, and missing state is never invented. Query and playback live in the
resolved module. Historical heuristic code is internal regression support only; it is not a
second supported parser API.

The resolved object owns the playback cursor and current world. `current()`
borrows that already-materialized world in O(1); enumeration/copy is O(world
size). Forward advancement applies intervening records. Backward/arbitrary seeks
restore periodic checkpoints then apply the remaining updates. Index lookup is
logarithmic; checkpoint restoration scales with entity count. Checkpoints share
immutable entity/component data to avoid deep copies. Native timestamps remain
exact microseconds. No interpolated state or inferred physical actions are
published as recorded facts.

## Single public API

The only aggregate flow is `Film::parse(...) -> Film` followed by
`film.resolve() -> ResolvedFilm<'_>`. The Film remains available and must outlive
the resolved model. Multiple independent resolved models can borrow one Film.

There is no public legacy module, mixed Film type, mixed constructor or adapter.
Historical implementation used by regression tests is crate-private, including
helpers whose signatures mention its aggregate. It is not a compatibility API.
Both example consumers now parse the native Film and use its resolved indexes.
Their JSON export is the native Film schema; old mixed JSON and derived CSVs are
not the new export contract. The compact mode was removed because native exports
preserve all supplied source bytes.

## Validation matrix

| Layer | Evidence / assertions |
| --- | --- |
| Native | Existing pinned-reference/captured-film tests; equality of parse entry outputs; byte retention and unchanged packet order. |
| Conversion | Hand-authored records with out-of-order/tied timestamps; stable source references; source remains unchanged; unknown packets remain queryable. |
| World | Hand-authored NEW, DEL, generation reuse, keyframe and component updates; partial reads cannot create fabricated complete state. |
| Playback | Sequential advancement equals arbitrary and backward seek, including timestamp ties and checkpoint boundaries. |
| Queries | Exact inclusive time ranges, packet/entity filters, retained records and unknowns. |
| Semantic actions | Independent in-game annotations are still required before claiming physical-action accuracy; this API migration does not invent that evidence. |
| Browser | Browser rendering/assets and UI integration are separate consumers, outside this API restructuring. |

The previous parity goal is closed. This migration does not reopen newer-film
reverse engineering or imply that legacy heuristic outputs are canonical.

## Public use and compatibility

```rust
use halo_api::theater::{Film, FilmSource, film::ParseOptions};

# fn example(source: &FilmSource) -> Result<(), Box<dyn std::error::Error>> {
let film = Film::parse(source, ParseOptions::default())?;
let mut resolved = film.resolve();
resolved.advance_to(10_000_000);
let world = resolved.current();
# Ok(())
# }
```

Callers migrate directly to the flow above. Native Film exports contain
`original_chunks`; older native JSON without this field still loads with an
empty transport collection. Missing transport in an old export cannot be
reconstructed.

The resolved event index distinguishes structural packet entries, sequential
entity records, keyframes, controls, native event-list entries, and summaries.
Each index entry borrows its typed payload through `record(event.source)`.
An indexed partial read is not an assertion that an action completed. The event
clock is the enclosing packet's exact timestamp; summaries retain their own
recorded relative time in their payload, without silently mixing clock domains.
Recovered candidates and independent projections are accessible on their packet
but do not become duplicate gameplay events or authoritative state updates.

Current components retain the latest ordered raw update per component and its
completion flag; component-internal conditional fields require component-specific
semantic interpretation. Bootstrap identities can be queried by recorded player
index. Entity-to-player attribution and human action names are not guessed from
matching numeric indexes. Higher-level interpretations require explicit resolvers of the native data;
no legacy public API is retained. This migration establishes source-linked
query/playback storage; it does not claim independently validated human actions
or a browser renderer.

## Verification record

The migrated native parser, legacy JSON compatibility, source loading, and first
six resolver cases passed together: **31 tests, zero failures**, including the
32-film corpus (39.25 seconds of execution). Native comparison coverage remains
403,465 frame view-count/end-bit checks, 134,657 fixed-offset keyframe body checks,
and 3,667 summaries, as recorded in the parity closeout.

Final resolver guards also distinguish padded controls/keyframes from recorded
reads, reject incomplete NEW bindings, and explicitly test that recovery
candidates never create entities. Final resolver verification: **8 tests passed, zero failures** (0.01 seconds).
Final Clippy (`--lib --tests --examples -- -D warnings`), WASM library check,
formatting and whitespace checks also passed. Parser code did not change after
the successful corpus run.

An exploratory broad `resolved_` filter additionally selected
`theater::production_frame::tests::native_position_hook_resolved_profiles`.
That older fixture fails because the current reader emits two additional
`UnitReference` observations absent from its expected list. This API migration
does not modify that decoder or oracle. The mismatch remains unresolved; the
focused migration/parity results above are not a claim that the entire library
test suite is green.

All implementation is under `src/theater/`. Both example consumers now use
Film::parse followed by a borrowed resolve. Preexisting Cargo.toml and
experiments/FILM_FORMAT.md edits are retained. Nothing was committed or pushed.

Reproduce the final resolver check:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib -- theater::resolved::
```

## Borrowing API correction and query indexes

The user explicitly rejected a compatibility API and a consuming resolve.
`Film::resolve(&self) -> ResolvedFilm<'_>` now borrows the original parsed film;
there is no `From<Film>` conversion, public mixed aggregate, or legacy namespace.
Tests verify pointer identity and simultaneous resolutions from the same Film.
Public-surface compile-fail examples verify the removed namespace/constructor.

Resolution builds ordered indexes by entity, player, event kind and category.
Queries binary-search the time bounds, choose the smallest applicable index
slice, and check the remaining predicates only on those candidates. Indexes add
O(events) storage; query cost depends on the selected candidate count, not an
unconditional full-film scan. Bootstrap player lookup uses its recorded index.
Both native export/inspection examples now use this same public flow.

The query-index intersection test covers combined filters, absent keys, inverted
time ranges, exact timestamps and stable record ordering. Verification for this correction: all 9 resolver tests pass, as do the public
flow doctest and both negative API contract doctests. Clippy for library/tests/
examples, the experiments export example build, WASM, formatting and whitespace
checks pass. The last source change only closed a documentation code fence.
