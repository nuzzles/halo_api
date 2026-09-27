# Partial cache metadata selection

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
`halo_rust_cache_test.go.txt::TestHaloRustCachePartialMetadata` invokes actual
`filmcache.LoadFilm`, which calls `source.LoadDir`, on six temporary caches.
Generation is registered in `generate_oracles.py`.

Each cache contains `chunk_04.bin` and `chunk_unknown.bin`. Cases independently
exercise an oversized type on a missing file, a shadowed duplicate row, the
unknown-number row, and selected rows containing i64::MAX, i64::MIN and 2^32. The oracle retains aligned metadata and
all loaded chunk bytes. The first matching numbered manifest row wins; unknown
filenames never inherit a manifest row numbered -1.

Previously Rust converted every manifest type to i32 before reading the
source directory, rejecting unused and selected oversized values. The first
fix selected rows before conversion. The full-width follow-up replaces that
workaround: FilmSourceMetadata.chunk_type now retains the native i64 range,
so ordinary source alignment handles every case without lossy conversion.
All six cases compare the complete aligned metadata and loaded bytes against
the native result and roundtrip metadata through JSON.

FilmCacheMetadata and exact manifest bytes continue to preserve the original
values. Conversion into the external API's FilmChunkData remains checked because
that model has an i32 chunk_type. The same tests require exact normal conversion
and FilmSourceError::ChunkType for selected values outside that range. This is a
boundary of that adapter, not a limitation of loading or native source retention.

This is cache-input parity evidence, not a native-film field encoding change or
an implementation of the deferred architecture.
