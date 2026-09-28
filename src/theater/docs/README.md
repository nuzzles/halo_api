# Theater v41

The API has two steps:

```rust
use halo_api::theater::{Film, film::{FilmChunk, ChunkKind}};

fn inspect(registry_bytes: Vec<u8>, replication_bytes: Vec<u8>, summary_bytes: Vec<u8>)
    -> Result<(), Box<dyn std::error::Error>>
{
    let film = Film::parse([
        FilmChunk::new(ChunkKind::Registry, registry_bytes),
        FilmChunk::new(ChunkKind::Replication, replication_bytes),
        FilmChunk::new(ChunkKind::Summary, summary_bytes),
    ])?;
    let mut resolved = film.resolve();
    println!("{} summaries", film.summaries.events().count());
    println!("{} entities", resolved.seek(10_000_000).entities.len());
    Ok(())
}
```

A `FilmChunk` combines its category and bytes with optional manifest `index` and
`start_ms`. Supply real metadata when available; `None` explicitly means it was
not supplied. Input order defines source positions, independent of manifest
numbers. Raw and zlib-compressed chunks are accepted. No separate metadata list,
source loader, or parsing options are part of the public entry point.

`Film::parse` locates exactly one registry chunk, reads its version, selects the
`parser::v41::V41ChunkParser` or returns an error, then parses the supplied chunks.
The version-specific decoder is internal; callers continue to use `Film::parse`. `Film` has exactly
three fields:

- `registry`: component definitions and the complete bootstrap chunk.
- `replication`: ordered replication chunks and native packet reads.
- `summaries`: ordered summary chunks and recorded summary entries.

Each section preserves original input, decompressed bytes, source positions and
unparsed data. `ParsedChunk::payload` returns a checked borrowed packet payload.
The registry can appear anywhere in the input list; zero or multiple registry
chunks are errors. Unsupported chunk categories are rejected by `ChunkKind::try_from`.

`film.resolve()` borrows this recording, builds chronological query indexes and
playback state, and performs explicitly labeled interpretation. It exposes those
results through `interpretations()` without changing the native recording.

Only `Film` and `ResolvedFilm` are reexported at `theater`'s root. Input and native
container types live under `film`; parser models remain public under `parser`;
resolved models and interpretation evidence live under `resolved`.

- [Format and fidelity](FORMAT.md)
- [Resolution and playback](RESOLUTION.md)
- [Validation and reference provenance](VALIDATION.md)
- [Credits](CREDIT.md)
