# Module organization

The public flow remains `Film::parse(chunks)` followed by `film.resolve()`.
`theater` reexports only `Film` and `ResolvedFilm`; `film`, `parser`, and `resolved`
are public modules. Reference models are declared in `film`, while decoder types and
functions are internal.

```text
theater/
  film/
    mod.rs                 Film, ordered FilmDataChunk values and model reexports
    chunks/
      mod.rs               Chunk hierarchy and reexports
      models.rs            FilmChunk and ChunkKind
      packet/
        mod.rs             Shared packet model reexports
        models.rs          Packet<T>, PacketStream<T>, read states and source ranges
      registry/
        mod.rs             RegistryChunk
        models.rs          Archetypes, owned component slots, ranges and read outcome
      replication/
        mod.rs             ReplicationStreamChunk and components
        replication_stream/
          mod.rs           ReplicationStream and ReplicationStreamPacket
          models/          Frame, view, entity, keyframe, datum and event models
        components/
          *.rs             Recorded component fields, controls, position and references
      summary/
        mod.rs             SummaryChunk, packets and ordered opaque/event segments
        models.rs          Raw recorded summary fields
  parser/
    mod.rs                 ChunkReader and registry-first version dispatch
    transport.rs           Decompression and bounded packet framing
    bits.rs                Shared bounded bit primitives
    v41/
      mod.rs               V41ChunkReader and kind-specific reader types
      chunks/              Registry, replication and summary chunk readers
      registry.rs          v41 registry layout
      config/              Private fixed and runtime decoding configuration
      components/          v41 component decoding
      diagnostics.rs       Private refusal and width traces
      observations.rs      Private decoder callback state
      packets/             Replication packet dispatch and isolated body decoders
      summary.rs           v41 summary decoding
  resolved/
    mod.rs                 ResolvedFilm construction and shared indexes
    identity.rs            Identity/player-table models and player lookup
    interpretation/        Bootstrap searches, player-slot traces and interpretation evidence
      packet/              Packet-head, pickup, damage, zoom and teleport decoders/models
    events.rs              Source references, provenance and event indexing
    summary.rs             Decoded text and semantic summary kinds
    query.rs               Filters and query indexes
    world.rs               Entity/component models and state accumulation
    playback.rs            Current snapshot, advance, seek and checkpoints
    source.rs              Internal borrowed chunk/packet navigation
  docs/                    Format, fidelity, architecture and validation
```

`ChunkReader` requires the first input to be a registry, decompresses it once,
and checks its version before decoding version-dependent fields. Its sole
current variant owns a `V41ChunkReader`, which dispatches each chunk kind to
`V41RegistryChunkReader`, `V41ReplicationStreamChunkReader`, or
`V41SummaryChunkReader`.
Source positions remain positions in the original full input: registry is zero,
the next chunk is one. Additional registry chunks are errors. Unknown later chunk kinds remain ordered
and retain their bytes without selecting a v41 body reader.

The version reader retains grammar state for subsequent records. That state is
internal decoding context, not a replay world. Resolution separately accumulates
world state and preserves references to the reference recording.

Packet framing belongs to structural decoding. Bounded player-slot reads and packet
projections run only during resolution at selected candidate boundaries. The film
parser does not use bootstrap searches to choose an unknown layout.

## Migration

The former `theater::parser` model imports move to `theater::film` (also exposed
through its section submodules). For example:

- `parser::FilmRegistry` becomes `film::chunks::registry::FilmRegistry`.
- `parser::ComponentField` becomes `film::chunks::replication::components::ComponentField`.
- `parser::SummaryEvent` becomes `film::chunks::summary::SummaryEvent`.
- Identity and player-table interpretation models live in `resolved::identity`.
- Other interpretation outputs live in `resolved::interpretation`.

`Film::parse`, `film.resolve()`, and the resolved query/playback methods keep their
signatures. No second public parsing API or compatibility parser facade is added.
Canonical packet headers retain only wire fields; locations live in `PacketSource`.
Body reads explicitly distinguish complete, partial, and opaque results. Source
retention and decoding limits are described in FORMAT.md.

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
