# Mode-guarded facts cache channels

Reference: `replay/filmfacts_gardes.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`facts_guards.rs` preserves the native section order: carrier marks and their
keyframe/record denominators, zone reads and scanned flag, flag-return gauge
reads and scanned flag, then bomb radial reads. Zone and flag coverage remain
independent even when both use managed properties. False scan/presence flags do
not erase populated values. Timestamp differences wrap in u64; native int fields
remain i64 even on WASM. Slot and bomb-time narrowing follow native casts.

The cache-specific carrier DTO preserves signed denominators and nil lists.
Decoders normalize empty slices to nil, including partial decodes, and append
partially decoded records exactly as the native loops do. They avoid native eager
allocation from untrusted counts. Negative signed slice counts produce an explicit
Rust error instead of a native panic; this is a safety divergence, not equivalent
successful decoding.

The self-contained pinned Go oracle generates 31,975 composed cases from 128
seeded channel states and every truncation boundary, all 256 boolean bytes,
ten native negative-count panics, and oversized slot/time narrowing. Assertions compare exact
writer bytes, every decoded field, nil distinctions, error strings and cursors.
This synthetic codec evidence does not replace captured positive VIP/Assault
films or independently annotated action goldens.

These are derived cache sections, not a lossless native Film representation.
Full cache assembly, remaining channels, file framing and BuildFromFacts remain
pending. Full v41 parity is incomplete; the architecture phase remains deferred.
