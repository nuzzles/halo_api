# v41 bootstrap fixture

Source: natural-end/01-do-nothing, chunk-000-type-1.bin. The fixture ends at the
end of the first block after the registry, so the structural terminator is readable. It retains the
registry, type-version table and identification exactly as recorded.

Expected values were emitted by the actual LevelUp parser at
43a01721e8a02c87c955e175936f0ccf8dd97a81, using ParseRegistryChunk and
ReadFilmIdentity, not computed by the Rust implementation.

Decoded byte SHA-256: `da9909cdc20200d2e5b83022957695dfbc13bfb6138a4c8d5ee7575e5dad07a0`.
