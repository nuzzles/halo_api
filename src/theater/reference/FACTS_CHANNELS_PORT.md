# Always-scanned cache channels and VehicleScan composition

Reference: `replay/filmfacts_canaux.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`facts_channels.rs` implements biped creations, held-weapon changes, pickups and
their ten counters, equipment changes and their fourteen counters (including
five walk counters), zoom events, vehicle events, occupant aim and occupancy.
Timestamp deltas wrap in u64. Native int fields stay signed i64 on WASM, unsigned
fields narrow as Go casts do, and event-kind strings retain arbitrary bytes.
Presence/validity flags never erase associated values. Partial records append
at the same point as the native loops. Decoded empty channel slices are allocated
empty lists, unlike the nil-normalizing guarded-channel codecs.

The cache projection deliberately omits scan diagnostics:

- Biped creations: Chunk, PacketIndex, BitPos, Version, Representation.
- Held-weapon changes: Chunk and cosmetic Low.
- Pickups: Chunk.
- Equipment changes, vehicle events and occupant aim: Chunk and PacketIndex.
- Zoom and occupancy: no fields omitted.

The native type/transported-field inventory is retained in
facts-channels-field-inventory.json. These omissions do not change richer native
Film records; this cache cannot serve as lossless native recording storage.

`facts_vehicles.rs` composes Scanned, keyframes, creations, creation statistics,
positions, vehicle events, aim, object deaths/statistics/configuration, then
occupancy, exactly in native order. The native source's older comment claiming
creation Stats are omitted is contradicted by its executable encoder/decoder.
False Scanned does not discard any populated section. Shared position-cache
coordinate recomputation and field omissions remain as documented separately.

## Validation

- 31,841 native channel cases from 48 source states per channel, every truncation
  point, all 256 boolean bytes and eighteen native panic controls. Negative
  slice/string lengths are safely refused in Rust at the same consumed cursor.
  Untrusted counts do not trigger native-style eager allocations.
- 8,342 composed vehicle cases across 24 populated native scans. All truncation
  points are tested for three sources; the remaining sources use complete,
  boundary and seeded sampled prefixes. Assertions compare original writer bytes,
  every projected decoded field, float bits, errors and cursors.
- All sixteen shared facts-codec tests pass. Platform/static-check details are
  recorded in facts-channels-validation.json.

This completes the shared channel source, not the entire FilmFacts codec. Main
body sections, header/admission rules, file sections/version/freshness checks,
BuildFromFacts, scan/cache conversions and final source/field/runtime/export
reconciliation remain open. Full v41 parity is incomplete; the requested
Film/ResolvedFilm/playback architecture remains deferred.
