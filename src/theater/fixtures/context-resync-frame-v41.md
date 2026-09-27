# Live raw resync frame loop (v41)

Pinned reference: LevelUp feat/v75 `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Native harness: `../reference/halo_rust_context_resync_frame_test.go.txt`, registered
in the generator. Native generation passed before compression.

128 cases produce 74 returned records: 62 Delta, eight New and four Delete.
There are 87 acceptance calls and 671 component/post-loop callbacks. The Rust
comparison verifies record identities/archetypes/masks, component ordering and
start boundaries, returned record ends, acceptance positions as float32 bits,
ordered callbacks, caller histograms, final bindings and JSON roundtrip.
Native components lack explicit EndBit; the test uses the next component's start
or the enclosing record's end for these fixtures without corruption bits/tails.

Cases include recovery after unknown slots, prefix fields, short sources, hard
and soft bindings, positionless records, accepted/rejected scans, New/Delete
mutations and callback replacement while recovery capture hooks are suppressed.
Recovery temporarily removes position and movement hooks, retains reference
hooks, runs the shallow-copy scan, restores saved hooks, re-reads the landing and
starts a fresh sequential reader at capture slot zero. Clean direct New/Delete
records mutate the world. No packet preamble is consumed by this native path.

NativeResyncFrame retains every speculative diagnostic separately from actual
caller callback delivery. Cumulative native counters remain on the live observer.
The older no-context ResyncFrame API retains its prior behavior.

Final-loop cursor evidence is now provided separately by resync-cursor-v41.md.
The positive
4096-record-limit case is separately covered by resync-guard-v41.md, and the
accepted-candidate re-read failure by resync-mutation-v41.md. Those checks are
not inferred from these 128 cases.
These remain distinct from the verified returned-record boundaries.
The fixture is synthetic parser evidence, not independent semantic film annotation.
