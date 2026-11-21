# Captured film goldens

`bazaar_idle` and `octagon_ar_kill` are small complete v41 film recordings from
local Bazaar and Octagon captures. Each `manifest.json` keeps only the
chunk kind, index, start time, and local filename. The `.bin` files contain
the captured inflated chunk bytes recompressed as zlib, matching the input
contract of `RawFilmChunk`.

These recordings exercise registry setup, multiple replication chunks, and
both empty and populated summary highlights. They also cover the initial player
table, later populated appearance blocks (including two different players),
recorded match settings, and roster playback/serialization. `Cargo.toml` excludes this
directory from the published crate. The integration tests compile only when
the directory exists in a development checkout.

An optional larger-corpus regression compares single- and multi-threaded parsing
and checks metadata and typed events. It recursively reads local `film.json`
manifests with `chunk_type`, `index`, `start_time_offset_ms`, and `file` fields;
compressed and inflated chunk files are supported. It performs no downloads:

```sh
HALO_FILM_CORPUS=/path/to/fixtures cargo test --test theater_films local_corpus -- --ignored --nocapture
```
