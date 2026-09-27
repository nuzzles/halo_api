# Native callout catalog port

Pinned source: replay/callouts_catalog.go at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

This is an external reference asset reader, not data embedded in a film. It fills
a previously unported native API contract without changing Film, resolved replay,
playback or browser architecture.

## Declaration mapping

- MapCalloutsSchemaVersion -> MAP_CALLOUTS_SCHEMA_VERSION (1).
- CalloutsProvenanceBrut/Decoupe/Mvar -> CALLOUTS_PROVENANCE_RAW/CLIPPED/MVAR,
  preserving the exact strings brut, decoupe and mvar.
- ErrCalloutsUnknownMap -> CalloutsCatalogError::UnknownMap and is_unknown_map.
- MapCalloutsCatalog -> MapCalloutsCatalog: all six fields, including brut and
  maps_by_id. Optional maps retain null versus allocated-empty state.
- MapCalloutsEntry -> MapCalloutsEntry: module, provenance and ordered zones.
- CalloutZone -> CalloutZone: all thirteen fields, with i64 volume indices and
  f64 coordinates. Polygon, parts and holes retain sequence order and nulls.
- CalloutBrutZone -> CalloutRawZone: original volume index and polygon. These
  original designer polygons are retained separately from clipped publication.
- LoadMapCallouts -> load_map_callouts and its portable parse_map_callouts core.
- Lookup/LookupByID -> lookup/lookup_by_id; lookup_callouts also supports the
  native nil receiver. Module and asset-ID namespaces remain distinct. Empty,
  unknown, differently cased or whitespace-modified keys are not guessed.

## JSON behavior and API boundaries

callouts_json.rs decodes directly by schema, skipping unknown fields without
forcing their numbers into f64. It shares the already-tested Go-compatible UTF
normalization and field-name folding helpers in map_catalog.rs; only their
visibility changed. Scalar null leaves an existing value intact; null maps and
slices clear them. Repeated map fields merge keys, but a repeated entry starts
from its native zero value. Repeated slices reuse their underlying element state,
including elements temporarily beyond their visible length. Fixed coordinate
pairs ignore extras and zero missing elements, while null elements preserve
previous values. The private backing state is discarded only after decoding.

parse_map_callouts/load_map_callouts are the native-input APIs. Ordinary Serde
Deserialize on the public DTOs is for canonical exports; it is not advertised as
an alternate parser for arbitrary Go JSON update behavior. Serialization follows
native omitempty rules: publication can omit empty optional maps/polygons, while
the loaded Rust value still distinguishes null from empty. Thus canonical JSON
round-tripping is tested separately from in-memory retention. Native error text
and byte-offset formatting are not emulated; JSON, schema, I/O and unknown-map
error categories remain distinct, with I/O/JSON sources retained.

## Independent reference evidence

halo_rust_callouts_catalog_test.go.txt calls production LoadMapCallouts, Lookup
and LookupByID. Its 141 cases include 53 successful loads and 88 refusals, plus
2,264 lookup results. A reflection-based native field dump retains fields hidden
by omitempty; the ordinary native JSON is also retained and compared separately.
The first Rust test run exposed the inadequacy of JSON-only retention evidence,
not a reason to discard empty maps. Both kinds of evidence now pass together.

The fixture includes the actual pinned map_callouts.json: 22 native modules,
64 Forge asset IDs, 18 original-polygon map groups and 3,352 named zones. Its
original SHA-256 is a19ab6b5fe25032d73f6642270d63d0721ee2483448bf297428336b0490057ff.
Tests compare every retained field, array ordering, publication shape, canonical
JSON round-trip and lookup outcome. Numeric type/domain errors, unknown large
numbers, duplicate fields, nulls, fixed-array rules and reused backing arrays
have explicit cases. The existing shared normalizer tests complement this matrix.

The host test exercises every input through the filesystem adapter as well as
the in-memory API. The actual WASM harness executes the public in-memory API.
Current results are recorded in callouts-catalog-validation.json. These are
native differential tests, not independently annotated golden-film actions.
Full v41 parity and the deferred architecture remain separate gates.
