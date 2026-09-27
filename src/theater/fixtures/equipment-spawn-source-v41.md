# Equipment-spawn source traversal

Pinned reference: JGtm/LevelUp `feat/v75`, commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The retained `reference/halo_rust_equipment_spawn_source_test.go.txt` harness
calls actual `ScanEquipmentSpawnEvents` on 128 memory sources. It retains 1,120
ordered events and 32 empty-prefix controls. Cases cover contiguous and reordered
metadata, duplicates, negative numbers, gaps, terminal type-7 packets, zero-size
nonterminal packets, oversized payload declarations and trailing partial headers.
Payloads exercise source/spawned/third-reference presence, generations, both
configuration values, absent lists, other event families, one-byte list headers,
non-delta packet kinds and nonmonotonic timestamps.

Instrumented native output must equal the production scan before the fixture is
written. Rust compares every native coverage counter, publication order, object
life, third-reference flag, wire timestamp, packet ordinal and payload range.
The supplied-head adapter is also checked with its input order reversed.

A separate 2,304-input direct-decoder probe records 394 actual native panics from
out-of-bounds raw reference reads. Rust retains bounded refusal semantics and
counts these as truncated without publishing an event. Nonpanicking reads are
compared for admission and values. This path must not use the synthetic zero-tail
policy of the zoom/pickup/translocator readers.

The fixture is zlib-compressed JSON. Regenerate with
`reference/generate_oracles.py`, which registers this harness, or copy the harness
into the pinned native grammar package and run `TestHaloRustEquipmentSpawnSource`.
Its output is `/private/tmp/halo-equipment-spawn-source.json`.

These are synthetic differential tests, not independently annotated gameplay.

Validation also compares all six native spawn events and all seven native
coverage counters across the four downloaded films in the pinned
`gameplay-levelup-v41.json.zlib` fixture. The optional
`local_equipment_spawn_source_matches_native` test performs that focused check
without running unrelated position/component scans. All three Film constructors
and full Film JSON roundtrip preserve the selected native source/spawned lives.

The standalone scanner reports missing native source chunks. Film construction
retains that failure in `equipment_spawns_error` and leaves its equipment stream
empty, preserving other decodable data in partial Film inputs. A regression test
checks the error on a source with no contiguous prefix, then its absence after
adding the missing chunk and ordering metadata. Full Film JSON retains the error.
Film's existing general packet validation still applies before this optional
scan; standalone permissive framing does not imply permissive Film construction.
