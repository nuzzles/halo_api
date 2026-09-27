# Native reader replacement sequences

Pinned reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `reference/halo_rust_reader_sequence_test.go.txt`, already registered
in `reference/generate_oracles.py`.

The original 1,064 cases exercise 72 named components with six reads each:
profile/observer A, neutralized A, profile/observer B, restored neutralized A,
restored A captures, then B installed as a complete context. All 6,384 reads
retain endpoints, native status and both ordered receiver streams. There are
6,348 publications and 38 rejected reads with retained callbacks. Nonpublishing
component names are negative callback controls.

The extension calls actual native PoserObservation(nil) before a seventh read
of each case, verifies the returned receiver and unchanged cursor, and restores
the receiver afterward. It records native status/endpoints and both empty
receiver streams; 19 detached reads are rejected. Every field of every original
case was compared unchanged before the extended fixture was written.

The Rust test now runs a persistent NativeFilmReader beside the prior standalone
component path. It verifies independent profile/observer replacement, returned
profile and receiver identity, capture neutralization/restoration, unchanged
cursor/slot at replacement, raw component fields, every ordered callback and
detached decoding. Detached delivery does not invent observations. The separate
position sequence fixtures exercise preserved position accumulation under the
same independent setters, with native expectations left unchanged.

This is native parser API evidence, not a captured gameplay action annotation,
whole-Film acceptance, or the deferred resolved replay architecture.
