# Film cache source-contract reconciliation

Pinned reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
The two filmcache manifest entries were still pending despite their implemented
Rust adapters and native fixtures. They are now classified partial-v41 with
explicit evidence and limits; this does not promote complete parser acceptance.

Native Source/manifestJSON map to FilmCacheSource and FilmCacheMetadata. Open,
OpenChunkDir, Meta, NumChunks, Chunk, LoadFilm and LoadFilmDir map to the cache
source methods. ChunkDir, ChunksRoot, ManifestsRoot, ManifestPath and ListShortIDs
map to the film_cache path/list helpers. Index, ChunkType and StartMS are i64;
manifest order remains separate from chunk file numbers. Loading a partial cache
uses present files and joins metadata by number. Source loading and native
record decoding remain separate stages.

WriteChunk and writeManifestChunk map to FilmCacheWriteChunk, retaining Index,
ChunkType, StartMS and DurationMS as i64 and Data as raw bytes. EnsureDirs and
Write map to ensure_film_cache_dirs and write_film_cache. Both implementations
write missing chunks in input order, preserve existing paths, then create the
manifest if absent. Failure can leave partial output; neither provides atomic
transactions or byte-for-byte re-encoding of decoded native records.

Evidence already executed in component-mask-wrap-validation.json's 626-test
host suite: native_cache_paths, native_cache_manifest_contract,
native_cache_partial_metadata, cache_source_preserves_manifest_order_and_partial_file_loading,
native_cache_writer_snapshots and native_cache_creation_modes. Those fixtures
cover their represented path, manifest, duplicate/existing-file, failure and
permission cases. This reconciliation also inspected both native source files
and the current Rust read/write implementations.

Remaining API differences include Rust's unsigned Chunk position argument and
UTF-8 cache-ID handling versus native strings. Error displays are typed English
Rust errors rather than identical wrapped Go messages. External filesystem
states and arbitrary OS path byte strings are not exhaustively validated.
None of these adapter checks establishes wire-format decoding completeness.
