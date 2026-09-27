# Caller-supplied native map catalog oracle

Reference: LevelUp feat/v75 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_map_catalog_test.go.txt, registered with the oracle generator.

328 cases call the actual file loader and Lookup. Exact input bytes are retained
as hex, because Go JSON serialization replaces malformed input UTF-8. Coverage
includes schema errors, malformed JSON, missing fields, null versus empty maps,
case folding, repeated scalar fields, repeated maps fields and entry replacement,
short and long arrays, null array entries, unsigned 64-bit maxima, coordinate
overflow even before a later overwrite, ignored unknown data, unpaired UTF-16
surrogates, a valid surrogate pair, escaped literal backslashes, every byte value
in a JSON string, and an incomplete multibyte UTF-8 sequence.

Rust compares error categories, typed native catalog values and each lookup.
All 79 pinned entries are also projected to existing FilmMapBounds and compared.
Native widths are u64 even on WASM; projection into usize is explicit and fallible.
This is catalog input support, not the deferred Film/resolved/playback redesign.

Error text includes platform-specific paths in Go and is not matched. Exhaustive
float32 decimal-rounding edge cases and broad filesystem failures remain unverified.

## Optional receiver and owned lookup

The same 328 inputs now also exercise 3,280 native lookups across explicit nil
and loaded receivers. All earlier fields are checked unchanged on regeneration.
There are 19 successful entries, 2,030 bare sentinel errors (nil receiver), and
3,261 total errors satisfying errors.Is(ErrUnknownMapBounds). The remaining
1,231 failures are missing names in present catalogs. Empty requested names
remain distinct from absent catalogs.

Rust lookup_loaded_film_map exposes this optional-receiver contract with a
MissingCatalog variant and an is_unknown_map_bounds predicate. It returns an
owned entry; native and Rust checks mutate returned strings and bounds and
verify that a repeated lookup retains the original values. Existing borrowed
lookup is retained. JSON/schema failures do not satisfy the lookup sentinel.
Error display text remains Rust-specific; these checks compare typed identity,
values and ownership, not platform/native error formatting.
