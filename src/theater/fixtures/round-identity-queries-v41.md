# Round identity query oracle

Pinned reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81,
internal/facts/objectives/slotidentity_rounds.go and slotidentity_elimination.go.
Generator: ../reference/halo_rust_round_identity_queries_test.go.txt.

256 constructed identity maps yield 67,584 round/slot queries, each comparing
AtRound and Origin on the original identity and FlatRoundIdentity. Resolved,
Rounds and NamedCount are called directly in native code. Empty maps, empty
identity values, negative/missing rounds and independent origin maps distinguish
lookup semantics from assumptions about successful identity resolution.

Flat construction always contains round zero, so an empty flat map is resolved
with zero named entries. Native NamedCount counts entries even when their value
is empty. Rust owns its supplied map instead of sharing a mutable Go map; it
creates no provenance or round starts. Portable flat outputs roundtrip.
These methods expose existing evidence without strengthening identity claims.
