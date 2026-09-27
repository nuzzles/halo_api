# Background directory and cache contracts

Pinned source: replay/map_background_index.go at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

build_map_background_index lists entries in filename order and accepts only
non-directory entries ending in lowercase .json. It loads through the native
sidecar reader, logs skipped malformed/unreadable/schema-mismatched files, and
reports ambiguous identities. Missing or unreadable directories return an error.
Symlinks are eligible entries; loading follows them, while signatures use the
link's own metadata, matching native DirEntry.Info.

map_background_index_for returns an Arc to the cached index. A process-global
mutex serializes lookup and reconstruction. The cache key preserves the supplied
path spelling; it is not canonicalized. Each call lists entries and computes the
native filename:size:mtime-nanoseconds signature. Unstatable entries contribute
an incrementing marker, forcing reconstruction. Signature generation preserves
native signed nanosecond wrapping. Cache hits reuse the same allocation.

This cache intentionally does not hash contents. Same-size rewrites retaining
mtime and changes to a symlink target can leave the cache unchanged. The fresh
builder sees such changes; tests compare both outcomes with native Go. There is
no TTL, capacity bound or explicit invalidation in the pinned native contract.

MapBackgroundKey retains original filename bytes. lookup_key exposes that key;
lookup is a UTF-8 convenience view and returns None for a non-UTF-8 key. JSON
publication substitutes each malformed UTF-8 byte just as Go does. Ambiguity
sorting and equality use raw bytes, preventing display substitution from merging
distinct keys. Native pure-helper fixtures cover invalid/truncated UTF-8 because
this macOS filesystem rejects creating such filenames.

Validation includes 19 native filesystem/cache actions, cached/fresh snapshots,
allocation reuse, malformed files, schema mismatches, ignored extensions,
directories, symlinks, collisions, additions/removals and unchanged metadata.
Additional native assertions cover vanished entries and lexical cache keys.
The Rust regression checks directory errors and vanished metadata explicitly.
See background-directory-validation.json for completed test runs and limits.

These are host external-asset APIs. WASM uses the portable index and raw-key APIs;
the browser is not given a host filesystem cache. Full v41 film parity remains
incomplete, and the Film/ResolvedFilm/playback architecture remains deferred.
