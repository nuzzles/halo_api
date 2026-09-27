# Theater v41

The public flow is `Film::parse(&source, options) -> Film`, then
`film.resolve() -> ResolvedFilm<'_>`. Both live under `theater`, in the `film`
and `resolved` modules. Supporting record types describe the native data;
there is no alternate legacy parser or client-side event-reporting facade.

- [Format and fidelity](FORMAT.md): native structure, coordinates, unknown data.
- [Resolution](RESOLUTION.md): source references, queries and playback costs.
- [Validation](VALIDATION.md): pinned reference and regression coverage.

`FilmSource::load(&chunks, &metadata)` loads ordered transport chunks and retains
both input and decompressed bytes. Metadata carries manifest indices, chunk types
and millisecond offsets. Supply actual manifest offsets; file names alone cannot
establish timing. Network and filesystem loading belong to callers. The client
continues to provide raw film manifests and chunk downloads.

```rust
use halo_api::theater::{Film, FilmSource, FilmSourceMetadata, film::ParseOptions};

fn inspect(chunks: &[Vec<u8>], metadata: &[FilmSourceMetadata])
    -> Result<(), Box<dyn std::error::Error>>
{
    let source = FilmSource::load(chunks, metadata)?;
    let film = Film::parse(&source, ParseOptions::default())?;
    let mut resolved = film.resolve();
    let world = resolved.seek(10_000_000); // microseconds
    println!("{} entities", world.entities.len());
    Ok(())
}
```

Input needs the v41 bootstrap/registry. Parser options expose reference grammar selection,
recovery policy and caller-provided quantization context. These are configuration,
not recorded facts. Unsupported layouts and missing context remain explicit.
