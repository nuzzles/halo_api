# Captured device and crate record prefix

This is a captured replication packet from
`appearance/01-cadet-blue/chunk-001-type-2.bin`, payload byte offset 567583,
timestamp 10344213721 µs. `packet.bin` contains its 8,647 payload bytes.
`reference.json` records the source chunk/registry/packet SHA-256 hashes,
registry rows and versions, initial context, recorded build/level identity,
90 sequential NEW endpoints, and every component primitive-read boundary.
Unlike the synthetic component suite, this fixture contains real film bytes.

The exporter executes libfilmshell at
`18067987ebc3628ad499c819c6aee814692df43b` in a disposable `git archive`.
It calls the original `Walker::entry` directly, sequentially, starting at bit 2.
There is no tick-marker search or recovery. Only diagnostic callbacks around
component dispatch and primitive reads are added. Initial type declarations
come from the existing independent historical context fixture; they are not
reconstructed using halo_api. The checked prefix consists entirely of NEW
records (archetypes 9, 38 and 43).

The original header/level readers select recorded level tag 88891201, engine
BuildId 269225 and branch HI_1_13_0. The primary precision-16 axes are 15/15/17
bits; region selection uses one bit. The secondary region has 18/18/18 axes.
The regression resolves the local catalogue with that recorded identity.
The product version string is a separate namespace.

The first device record spans bits 55..462. Its device-position-group component
spans 382..415. The historical d61443e dispatcher has no arm for that alias and
refuses at bit 382; its golden remains unchanged. Furthermore, the historical
comparison uses default widths 13/13/14. With those widths, original
libfilmshell ends a later device record at 4520 and then reports BadHeader.
With the recorded level, that device ends at 4827 and the verified sequence
continues to bit 64496. These observations explain this particular historical
disagreement; they do not establish that all historical differences are valid.

At bit 64496 upstream returns UnknownEntity without attempting a component.
The canonical parser retains the successor's DELTA header, refuses its body,
does not read controls, and does not infer a schema. Therefore its total end
includes that header; only the completed records and the rejection start are
compared to upstream. This test establishes a captured prefix, not a fully
decoded packet or complete device semantics.

Reproduce from the repository root with the separately retained film corpus:

```sh
python3 src/theater/fixtures/device-record-v41/export_reference.py /path/to/libfilmshell experiments/films
cargo test --offline --lib captured_device_records_match_upstream_without_inventing_the_successor
```

The exporter needs stable Rust and cached upstream dependencies.
