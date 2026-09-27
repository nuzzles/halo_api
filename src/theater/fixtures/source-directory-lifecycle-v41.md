# Directory source lifecycle

Reference: LevelUp feat/v75, commit 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_source_directory_lifecycle_test.go.txt.
Fixture: source-directory-lifecycle-v41.json.zlib.

64 independent native cases discover 1–8 numerically named chunks containing a
mix of raw and compressed bytes. After discovery, each case reads the first file,
overwrites it, reads it again, and adds a new matching file. Selected cases then
remove an existing file or replace it with a directory. Every original source
position is read, an out-of-range read is checked, and a traced source is loaded.

Expected data comes from the pinned Go implementation: numeric identities, raw
read bytes, before/after overwrite bytes, out-of-range messages, ordered loader
calls, inflated loaded bytes, and error classes. Outcomes: 17 successful loads,
24 missing-file errors and 23 directory-read errors; 288 direct position reads
and 190 reads during loading. Count queries are recorded separately as -1.

Rust additionally verifies owned previous-read bytes, unchanged discovery count,
path-bearing read errors, and the error chain down to std::io::Error. A source
read failure preserves its original source position through FilmSourceError.
The existing directory fixture separately checks ordering, duplicate/unknown
numbers, and manifest alignment.

This fixture compares filesystem error classes rather than platform-specific
OS error text. It uses literal directories and does not establish parity for Go
filepath.Glob expressions in directory arguments, invalid glob patterns, denied
or failed enumeration, symbolic links, or concurrent filesystem mutation during
a read. It does not measure transient memory use. The library's new directory
provider is unavailable on wasm32, as are its filesystem entry points.
