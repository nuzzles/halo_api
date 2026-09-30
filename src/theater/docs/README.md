# Theater v41

The API has one parsing step and one runtime loading step:

```rust
use halo_api::theater::{Film, TheaterRuntime, film::FilmChunk};
use halo_api::theater::runtime::{SummaryFilter, SummaryKind, SummaryPayload, medals::Medal};

fn inspect(chunks: Vec<FilmChunk>) -> Result<(), Box<dyn std::error::Error>> {
    let film = Film::parse(chunks)?;
    let mut runtime = TheaterRuntime::load(film);
    for event in runtime.query_summaries(SummaryFilter {
        kind: Some(SummaryKind::Medal),
        medal: Some(Medal::Splatter),
        ..Default::default()
    }) {
        assert!(matches!(event.payload, SummaryPayload::Medal(Medal::Splatter)));
        println!("{event} [{:?}]", event.derivation);
        // For example: Nuzzles (2535472547643888) received Splatter at 0:10.000
    }
    let world = runtime.seek(10_000_000); // microseconds; checkpointed seek
    println!("{} entities", world.entities.len());
    Ok(())
}
```

`FilmChunk` combines its category/bytes with optional manifest `index` and
`start_ms`. `None` means metadata was not supplied. Input order defines source
positions. Raw and zlib-compressed chunks are accepted. The public parsing entry
point has no alternate parsing options or heuristic recovery mode.

`Film::parse` requires the first and only registry chunk, reads the version, and
selects the v41 kind-specific readers. Unsupported versions return an error.
`Film` owns `registry` and one ordered `chunks` vector containing replication,
summary, and unknown later chunk kinds. Original input, decompressed buffers,
source positions, exact known fields, stops, and unknown data remain available.

`TheaterRuntime::load` takes ownership of a `Film` (or accepts `Arc<Film>`), resolves
supported state, and builds chronological query indexes/checkpoints. Its
`ResolvedFilm` is private. There is no public `Film::resolve`, `ResolvedFilm`, or
`theater::resolved` compatibility API. Only `Film` and `TheaterRuntime` are
reexported at the Theater root. Supporting models remain public beneath `film`
and `runtime`; the parser module remains public with internal decoder modules.

`summary_events()` returns every guarded summary candidate in chronological
order. Actors contain the source-read XUID/gamertag and a separate roster-link
result. Typed payloads distinguish kills, deaths, mode events, known/unknown
medals, and unsupported categories. Unknown pairs are never assigned a nearby
medal name. These are explicitly derived summary interpretations, not canonical
record boundaries or inferred physical actions. `summary_reports()` compares
candidate counts with recorded counts without claiming matching counts prove a
complete summary grammar. `record(source)` exposes the associated raw reads.

`query_summaries()` combines kind, XUID, medal, and inclusive microsecond ranges.
Generic `query()` filters the full structural/state event stream. Both preserve
playback state and source references. `current()` is O(1) borrowed access;
advancement applies updates, and seeking restores a checkpoint and applies its
remaining updates. Copying/enumerating a world scales with its size.

- [Architecture and migration](ARCHITECTURE.md)
- [Canonical format and fidelity](FORMAT.md)
- [Runtime resolution and playback](RESOLUTION.md)
- [Validation and provenance](VALIDATION.md)
- [Completed canonical audit](CANONICAL_AUDIT.md)
- [Credits](CREDIT.md)
