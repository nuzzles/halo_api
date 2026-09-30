# Theater architecture

The public flow is `Film::parse(chunks)` → `TheaterRuntime::load(film)`.
`theater` reexports only `Film` and `TheaterRuntime`. The canonical `film`, structural
`parser`, and higher-level `runtime` modules are public; `runtime::resolved` and
its `ResolvedFilm` type are private. Model modules are publicly reachable as
`runtime::events`, `identity`, `interpretation`, `query`, `summary`, and `world`.

```text
theater/
  film/
    mod.rs                 Film and ordered FilmDataChunk values
    chunks/
      models.rs            Downloaded FilmChunk, ChunkKind and transport containers
      packet/              Shared packet envelopes, source ranges and coverage
      registry/            RegistryChunk, ordered archetypes and component slots
      replication/
        components/        Exact source-backed fields and raw bits
        replication_stream/
          models/          Frames, default states, components, datums and keyframes
          coverage.rs      Structural coverage from owned fields
      summary/             Recorded count and complete opaque record stream
  parser/
    mod.rs                 Registry-first version dispatch
    transport.rs           Decompression and packet framing
    bits.rs                Bounded bit primitives
    v41/
      chunks/              Kind-specific structural readers
      packets/             Packet-specific body decoders
      components/          Component grammar
      summary.rs           Count/opaque-stream reader, no candidate scans
  runtime/
    mod.rs                 TheaterRuntime::load and the public query/playback API
    medals.rs              Pinned v41 typed medal identities and unknown pairs
    resolved/              Private resolution and optimized storage
      mod.rs               Shared Film ownership, event/summary indexes and checkpoints
      events.rs            Public source/provenance/event models and private indexing
      identity.rs          Public bootstrap identity models and private lookup
      summary.rs           Public typed summary payloads, actors, links and reports
      query.rs             Public filters and private query indexes
      world.rs             Public snapshots/state models and private accumulation
      playback.rs          Internal cursor/checkpoint application
      source.rs            Internal source navigation
      interpretation/      Public evidence models and internal guarded field reads
  docs/                    One format, API, fidelity and validation reference
```

`ChunkReader` requires the first registry, decompresses it once, and checks its
version before a version-dependent reader runs. Its v41 backend dispatches chunk
kinds to registry, replication, and summary readers. Unknown later kinds retain
bytes and input order; additional registry chunks are errors.

Structural parsing keeps only recorded/source-backed data and explicit stops.
Parser schema bindings are private decoding context. Resolution separately
accumulates state and gathers labeled evidence from retained bytes; it does not
repair canonical records or select speculative layouts in the native parser.

Loading moves a Film into shared ownership, avoiding self-referential borrowed
storage. Supplying an `Arc<Film>` shares the same recording across runtimes.
Runtime clones share source buffers, events, indexes, reports and checkpoints;
cloning the current world map scales with its size. The canonical film is exposed
as a borrowed immutable view. The private resolved model has no public constructor
or mutable access, so callers cannot invalidate indexes through it.

## Migration

- `film.resolve()` becomes `TheaterRuntime::load(film)`.
- `ResolvedFilm` is no longer a public type.
- `theater::resolved::*` model imports move to `theater::runtime::*` or its public
  model modules. No compatibility facade remains.
- `summaries()` becomes `summary_events()` with typed `SummaryEvent` payloads and
  actor/roster-link separation, plus `summary_reports()` for count coverage.
- Query, record, current, advance, seek, rewind and player access are runtime methods.
- `Film::parse` and the canonical chunk models keep their structural API.

The Halo client returns canonical downloaded `FilmChunk` values:

```rust,ignore
let manifest = halo.match_film(match_id).await?;
let film = Film::parse(halo.film_chunks(&manifest).await?)?;
let runtime = TheaterRuntime::load(film);
```

HTTP manifest entries remain `clients::hi::models::FilmChunkResponse`; they are
not a second downloaded chunk type. Browser visualization, map assets and visual
interpolation are future consumers of the runtime and are not implemented here.
