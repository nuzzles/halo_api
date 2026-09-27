# FilmFacts delta channels, abilities and movement

Reference: `encodeCanauxDelta`/`decodeCanauxDelta`, `encodeCapacites`/
`decodeCapacites`, and `encodeEtatsDeMouvement`/`decodeEtatsDeMouvement` in
`replay/filmfacts_encode.go` and `filmfacts_decode.go`, pinned at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`facts_state_channels.rs` implements all fields of these three cache sections:

- Ability ranks, camouflage quanta, grapple reads and translocations.
- Ability impulses and eight statistics; ability charges and seven statistics.
- Movement records and eleven scalar statistics plus three map widths.

Every record list uses wrapping timestamp deltas, reset independently per list.
Native machine ints stay signed i64, including fields carried through unsigned
varints. Native uint map widths remain u64 on WASM. Slot/quantum/progress fields
narrow exactly as their native u32/u16 casts. Movement labels retain raw bytes.
Translocations carry both float32 triples regardless of HasPositions. Scanned,
Absent, Predicted and On flags never erase other stored values. Native zero
initialization, append-on-partial-read behavior, sticky errors and cursor positions
are preserved. Empty decoded record lists are allocated empty lists, not nil.
Negative outer slice capacities and string lengths are safely refused where the
native decoder panics; no untrusted count causes eager allocation.

## Rich fields omitted by the native cache

`facts-state-channels-field-inventory.json` records the native type and transported
field sets independently from Rust:

- Ability ranks omit Chunk, PacketIndex and Counter.
- Camouflage, grapple reads, ability impulses and charges omit Chunk/PacketIndex.
- Translocations and movement records transport every field.
- Ability statistics transport every field.
- Movement statistics omit DatumBindings, DatumAmbiguous, VelocityReads,
  JumpEpisodes and JumpsDerived. These native cache omissions are not evidence
  that the recording lacks the corresponding data or that richer scan output
  should discard it.

The oracle originally projected all native movement statistics, exposing these
five omitted zero fields in the first Rust comparison. Its projection was corrected
to the executable encoder's field list; the source still populates all native
statistics independently, so omitted values cannot accidentally influence bytes.

## Validation

The native generator supplies 36,545 cases across 64 source states per section,
all prefixes, extra trailing bytes, all boolean byte values, arbitrary float bits,
raw strings, wrapped timestamps and signed counters. Twelve additional scalar
controls exercise values at 2^16, 2^32, 2^63 and u64::MAX, including narrowing.
Eight native panic controls cover six negative slice capacities and two negative
movement-string lengths. Writer bytes, decoded fields/float bits, diagnostics and
cursor positions are compared. Platform results are recorded separately in
`facts-state-channels-validation.json`.

These sections do not complete either main source file. World/spawn, queue,
teams/table, full cache assembly and file framing/integration remain pending.
The Film/ResolvedFilm/playback refactor remains deferred until full v41 parity.
