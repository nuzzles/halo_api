# Identity registry observations

Reference: LevelUp feat/v75 at 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_identity_registry_test.go.txt.
Fixture: identity-registry-v41.json.zlib (extended existing 1,024 cases).

The harness calls actual BuildIdentityRegistry and then logRegistry, capturing
native slog JSON with only wall-clock time removed. Ordered log records retain
all message/level/attribute values. Attribute maps are compared as maps. Every
pre-existing fixture field was checked unchanged before recompression.

Two INFO summaries occur per case (2,048 total); 3,655 WARN records cover all
seven conditions with both positive and negative cases: unresolved total 728,
index out of table 627, index disagreement 616, missing creation 540, discordant
death bridge 463, divergent reads 444, death-only naming 237. Final unresolved
causes are consumed after scoreboard/elimination/exclusion; bot read counts
must not be subtracted from the final out-of-table residue.

Rust compares registry construction, section, totals, health and verdict, then
all ordered logs from IdentityRegistryOutput::log. Document assembly calls it
after player construction/origin resolution and before combat layers. This
fixture validates the registry observation boundary; it does not independently
prove all document-wide log ordering or general shared-observer context parity.
