# Theater v41

`Film::parse` consumes downloaded zlib chunks and returns a finished film. The
private parser inflates and decodes each chunk, then discards packet buffers and
source locations after projecting the supported values.

```rust
use halo_api::clients::hi::models::RawFilmChunk;
use halo_api::theater::{Film, TheaterRuntime};

fn inspect(chunks: Vec<RawFilmChunk>) -> Result<(), Box<dyn std::error::Error>> {
    let film = Film::parse(chunks)?;
    println!("{:?}", film.registry.version);
    for highlight in &film.summary.highlights {
        println!("{}: {:?}", highlight.gamertag, highlight.payload);
    }
    let mut runtime = TheaterRuntime::load(film);
    println!("{} entities", runtime.seek(10_000_000).entities.len());
    Ok(())
}
```

A `RawFilmChunk` has a manifest kind, index, start time,
and downloaded zlib bytes. The parser sorts chunks by index and requires a
registry first, at least one replication chunk, and a summary last. A complete
film is passed directly to `Film::parse(chunks)`, which requires contiguous
indices and a recorded tick marker. Damaged zlib and unsupported
versions return errors.

The resulting `Film` has three sections:

- `registry` contains the recorded version, named component definitions and
  their encoding parameters, decoded game option values, and map IDs when
  available.
- `replication_stream.observations` contains timestamped entity changes,
  component updates, controls, keyframes, and recorded events. Several
  observations can share a packet timestamp. Only completely decoded changes
  are projected; the parser does not invent values for opaque fields.
- `summary.highlights` contains guarded typed kill, death, medal, mode, and
  unknown candidates with gamertag, XUID, timestamp, and payload. The summary
  record grammar does not yet prove that this list is complete.

`TheaterRuntime` is an independent playback cursor over the typed observation
list. `current()`, `advance_to()`, `seek()`, and `rewind()` expose state at a
film-relative microsecond timestamp. `summary_highlights()` borrows the film's
highlight feed.

The detailed source and fixture notes in this directory record parser research;
source ranges and diagnostic packet structures are not part of the finished
`Film` API.

The [film world inspector](../../../examples/film_world_inspector.rs) is a
read-only egui example. Run `cargo run --example film_world_inspector` for the
bundled film, or pass a directory of compressed chunks saved by
`download_film` after `--`.

- [Architecture](ARCHITECTURE.md)
- [Format findings](FORMAT.md)
- [Coverage and limitations](COVERAGE.md)
- [Validation](VALIDATION.md)
