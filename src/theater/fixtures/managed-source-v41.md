# Managed-property source and Film constructor oracle

Pinned LevelUp: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_managed_source_test.go.txt`, registered after
its primitive-input dependency in `generate_oracles.py`.

54 synthetic sources reuse the first 64 primitive cases except the ten with
an internal empty registry slot followed by named components. Such holes are
not encodable native registry blocks; they remain standalone context cases.
Both native and Rust parse the same retained bootstrap bytes. Each source binds
slots 512 and 514 in a keyframe, then carries a type-6 packet before four deltas.
The scanner discovers its own observed band. Wire timestamps and a deliberately
different manifest start test that managed-property output preserves wire time.

Native ScanManagedProperties supplies every accepted reading/counter. A native
observer wrapper replays that source walk and must equal the original scan
before exporting its callbacks and packet ordinals. Outcomes: 5,391 readings,
5,404 callbacks and 171 broken records. The Rust test compares both configured
Film constructors, requires positive readings in every case and callbacks from
rejected attempts, validates source packet/ordinal/time/start relationships,
and roundtrips complete Film exports.

The first fixture draft used type 7 before the deltas. Native parsing correctly
stopped at this chunk terminator and yielded no readings; the positive-output
assertion rejected that fixture. The unrelated packet is type 6 in the accepted
fixture. This correction changes no parser behavior.

This validates synthetic source-to-constructor retention. It does not establish
captured zone gameplay semantics, canonical record interpretation or source-byte
archival in Film JSON. Existing captured scanner comparisons remain separate.

Regeneration: decompress managed-property-v41.json.zlib to
/private/tmp/halo-managed-primitive.json, run TestHaloRustManagedSource in the
pinned grammar package, and zlib-compress /private/tmp/halo-managed-source.json
at level 9 after native success. The general generator wires these paths too.
