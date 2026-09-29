# Module organization

The public flow remains `Film::parse(chunks)` followed by `film.resolve()`.
`theater` reexports only `Film` and `ResolvedFilm`; `film`, `parser`, and `resolved`
are public modules. Native models are declared in `film`, while decoder types and
functions are internal.

```text
theater/
  film/
    mod.rs                 Film, ParseError, dispatch and model reexports
    chunk.rs               FilmChunk, ChunkKind, FilmChunkRef
    registry.rs            Native component registry and read trace
    replication/           Packet, record, player-slot and native event models
    components/            Component fields, read diagnostics and value models
    summary/               Recorded summaries and medals
  parser/
    mod.rs                 ChunkReader and registry-first version dispatch
    transport.rs           Decompression and bounded packet framing
    bits.rs                Shared bounded bit primitives
    v41/
      mod.rs               V41ChunkReader and kind-specific reader types
      replication.rs       Ordered packet decoding and grammar-state updates
      registry.rs          v41 registry layout
      player_slot.rs       Bounded roster/bootstrap slot grammar
      components/          v41 component decoding
      summary.rs           v41 summary decoding
  resolved/
    mod.rs                 ResolvedFilm construction and shared indexes
    identity.rs            Identity models and player lookup
    interpretation/        Bootstrap searches and interpretation evidence
    events.rs              Source references, provenance and event indexing
    query.rs               Filters and query indexes
    world.rs               Entity/component models and state accumulation
    playback.rs            Current snapshot, advance, seek and checkpoints
  docs/                    Format, fidelity, architecture and validation
```

`ChunkReader` requires the first input to be a registry, decompresses it once,
and checks its version before decoding version-dependent fields. Its sole
current variant owns a `V41ChunkReader`, which dispatches each chunk kind to
`V41RegistryChunkReader`, `V41ReplicationStreamChunkReader`, or
`V41SummaryChunkReader`.
Source positions remain positions in the original full input: registry is zero,
the next chunk is one. Additional registry chunks are errors.

The version reader retains grammar state for subsequent records. That state is
internal decoding context, not a replay world. Resolution separately accumulates
world state and preserves references to the native recording.

Packet framing and bounded player-slot reading belong to decoding. Interpretation
may reuse those readers at a selected candidate boundary, but the native parser
does not use bootstrap candidate searches to choose an unknown layout.

## Migration

The former `theater::parser` model imports move to `theater::film` (also exposed
through its section submodules). For example:

- `parser::FilmRegistry` becomes `film::registry::FilmRegistry`.
- `parser::ComponentField` becomes `film::components::ComponentField`.
- `parser::SummaryEvent` becomes `film::summary::SummaryEvent`.
- Identity and player-table interpretation models live in `resolved::identity`.
- Other interpretation outputs live in `resolved::interpretation`.

`Film::parse`, `film.resolve()`, and the resolved query/playback methods keep their
signatures. No second public parsing API or compatibility parser facade is added.
Native model serialization and packet/source ordering are unchanged by these
module moves. Source retention and decoding limits are described in FORMAT.md.

The Halo client keeps the manifest's JSON-only chunk entry as
`clients::hi::models::FilmChunkResponse`. Downloading one or all of those entries
returns the canonical `theater::film::FilmChunk`, so the common path needs no
adapter:

```rust,ignore
let manifest = halo.match_film(match_id).await?;
let film = Film::parse(halo.film_chunks(&manifest).await?)?;
```

There is no separate downloaded `FilmChunkData` type. A second
`FilmChunkMetadata` model is unnecessary because the response DTO already owns
the service-only duration, size and file-path fields; the canonical chunk retains
only parsing metadata and bytes.
