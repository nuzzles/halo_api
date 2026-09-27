# Equipment recovery source selection

Pinned native commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_equipment_recovery_source_test.go.txt`, registered
in `generate_oracles.py`. Outputs come from native `scanEquipmentRecovery` using
an actual loaded FilmContext. This is a synthetic source fixture, not a captured
positive gameplay film.

24 cases combine six packet shapes and four chunk layouts: chunk 1, chunk 3 with
missing earlier chunks, reversed 3/1 metadata, and duplicate chunk 1. Chunk-type
metadata varies independently. Packet shapes cover a valid recovery, End before
recovery, malformed tail after recovery, malformed prefix, zero-size non-End,
and an unrelated packet before recovery. The source picks the first metadata
match for a chunk number and native packet traversal stops at End or malformed
input. Exactly 12 cases recover the missing counter 6 between strict counters
5 and 7; 12 recover nothing. All records use the sparse component-mask branch.

Rust supplies the same strict-counter window through its complete equipment
change scan and compares accepted source emissions, packet ordinals, timestamps,
counters, ranks and bit offsets. Recovery attempts/windows/assembly are retained
through JSON. Existing component-walk fixtures cover dense/sparse recovery bodies;
this fixture specifically checks source admission and ordering.
