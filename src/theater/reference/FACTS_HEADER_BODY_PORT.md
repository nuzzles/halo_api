# FilmFacts header, events and inventory

Reference: `replay/filmfacts_encode.go` and `replay/filmfacts_decode.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`. This checkpoint implements three
sections of those files, not their entire body or top-level assembly.

`facts_header.rs` preserves raw Film/MapModule bytes, all three unsigned native
axis widths, detection/refusal flags, optional signed major version, and unsigned
clock origin. Widths and native ints remain 64-bit on WASM. Catalog endpoints
retain their float bits. Header decoding checks the magic and starts the reader
at the full-blob offset. It performs typed map/layout admission before reading
the clock and before exposing any sticky transport error. Thus a successful
admission may return an errored reader, while an admission error returns no header.
Layout gate and region are zero in the decoded header result, exactly as native;
bounds come from the supplied catalog. Both directions of detection provenance
are checked by one reusable cooking-key function for future file-header use.

`facts_quote.rs` implements Go `%q` for those diagnostics, including individual
invalid UTF-8 bytes and pinned printable Unicode ranges. The Go runtime source
hashes are in `facts-quote-runtime.json`; `generate_facts_quote_tables.py --check`
verifies them and the Rust tables. The Go BSD license is retained. Tests compare
raw byte quoting and a hash of native map errors for every Unicode scalar except
NUL (which is separately covered among all 256 single bytes).

`facts_events.rs` writes/reads firing, loadout families, grenade throws and the
shared projectile-track codec in native order. Only firing uses timestamp deltas;
loadouts and throws use absolute timestamps. Absent aim omits its payload and
decodes to zero; this is native cache behavior. Family order is unchanged and an
empty decoded family list remains nil.

`facts_inventory.rs` carries all four grenade counters and ammo slots, presence
flags, signed ranks/drawn slot/candidate counts, and inventory deltas. Candidate
counts use the unsigned transport with signed native interpretation. False read
flags do not clear values. Delta timestamps wrap and an empty grenade list decodes
nil. Native negative outer capacities panic: Rust safely refuses at the same
cursor. Negative nested family/grenade counts skip their native loops instead.
No untrusted count triggers eager allocation. Partial records append at the same
points as native, including the shared ammo reader's early gauge return.

## Cache omissions

The independent field inventory is `facts-body-field-inventory.json`:

- Fire omits Chunk, PacketIndex, Variant, ShooterIndex5 and Flags.
- Loadouts omit Chunk and PacketIndex.
- Grenade throws omit Chunk, PacketIndex and BitPos.
- Inventory omits Chunk, PacketIndex and GrenadesByPosition.
- Inventory deltas omit Chunk, PacketIndex and Ammo.
- Projectile fields are fully transported by the shared track codec.

These are deliberate reference-cache omissions, not permission to discard the
richer native recording data. FilmFacts is not a lossless Film representation.
All-data v41 parity and the later Film/ResolvedFilm/playback phase remain open.

## Validation scope

The pinned native generator supplies 5,327 header cases, including every prefix
of 84 headers, trailing bytes, invalid catalogs, wrapping region widths, raw module
names and all boolean bytes. Event/inventory generators supply 15,928 cases from
48 states per section, every prefix, trailing bytes, arbitrary float bits and
signed fields, four native panic controls and four negative nested-count controls.
Writer bytes, decoded fields, float bits, cursor and errors are compared.
Platform and check results are recorded in `facts-header-body-validation.json`.
