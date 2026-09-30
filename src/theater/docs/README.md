# Theater v41

The API has two steps:

```rust
use halo_api::theater::{Film, film::{FilmChunk, ChunkKind}};

fn inspect(registry_bytes: Vec<u8>, replication_bytes: Vec<u8>, summary_bytes: Vec<u8>)
    -> Result<(), Box<dyn std::error::Error>>
{
    let film = Film::parse([
        FilmChunk { kind: ChunkKind::Registry, index: None, start_ms: None, data: registry_bytes },
        FilmChunk { kind: ChunkKind::Replication, index: None, start_ms: None, data: replication_bytes },
        FilmChunk { kind: ChunkKind::Summary, index: None, start_ms: None, data: summary_bytes },
    ])?;
    let mut resolved = film.resolve();
    println!("{} summaries", resolved.summaries().len());
    println!("{} entities", resolved.seek(10_000_000).entities.len());
    Ok(())
}
```

A `FilmChunk` combines its category and bytes with optional manifest `index` and
`start_ms`. Supply real metadata when available; `None` explicitly means it was
not supplied. Input order defines source positions, independent of manifest
numbers. Raw and zlib-compressed chunks are accepted. No separate metadata list,
source loader, or parsing options are part of the public entry point.

`Film::parse` requires a registry as the first chunk. `parser::ChunkReader`
decompresses it, reads the version before version-dependent fields, and selects
its `V41ChunkReader` variant. That reader dispatches registry, replication, and
summary chunks to their v41 kind-specific readers. Unsupported versions return
an error; callers still use the single `Film::parse` entry point. `Film` has two
fields:

- `registry`: the required registry chunk, including its source and read result.
- `chunks`: every later chunk in input order as replication, summary, or unknown.

Each section preserves original input, decompressed bytes, source positions and
unparsed data. Replication and summary chunks provide checked borrowed packet payloads.
Empty input, a non-registry first chunk, or any later registry chunk is an error.
A registry-only film is accepted. Unknown numeric chunk categories are retained
as `FilmDataChunk::Unknown` with their source bytes and transport result.

`film.resolve()` borrows this recording, builds chronological query indexes and
playback state, and performs explicitly labeled interpretation. It exposes those
results through `interpretations()` without changing the reference recording.

Only `Film` and `ResolvedFilm` are reexported at `theater`'s root. Input and reference
container and reference field models live under `film`; decoded registry, replication,
component and summary models have dedicated submodules. `parser` remains a public
module with internal implementation modules. Resolved models and interpretation
evidence live under `resolved`.

- [Module organization and migration](ARCHITECTURE.md)
- [Format and fidelity](FORMAT.md)
- [Resolution and playback](RESOLUTION.md)
- [Validation and reference provenance](VALIDATION.md)
- [Canonical audit and fidelity evidence matrix](CANONICAL_AUDIT.md)
- [Credits](CREDIT.md)
