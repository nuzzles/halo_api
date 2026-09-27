# Native baseline frame-reader corpus

Pin: LevelUp d61443ef59268ad734355db8e9974f68db5ca6d0.
Generator: ../reference/halo_rust_native_baseline_d61443e_test.go.txt.
The Go harness loads the 32 original films and their metadata, takes the native
CadreDeBalayage profile, and walks sequential keyframes to seed parser bindings.
It then invokes DecodeFrameViewsCurseur at bit 2 for every delta packet, without
event-list continuation bindings or heuristic anchor recovery. This is the
baseline traversal used by the Rust native-entry integration audit, not a replay.

403,465 packet rows retain file, payload offset, completed view count and final
bit cursor. Go totals for 0/1/2/3 completed views: 58,997 / 901 / 1,558 / 342,009.
The JSONL fixture is compressed and streamed from disk. Expectations come only
from Go; Rust must match every packet, not just the aggregate.

The first pre-fix Rust mismatch was bandit/01-evo/chunk-003-type-2.bin payload
603649: Rust (3 views, bit 567), Go (3 views, bit 535). The generator also captures
that packet's Go records, incoming bindings and config at
/private/tmp/halo-native-baseline-first-diff-d61443e.json for diagnosis.
The native NEW hard-binding guard fixes the accumulated-state divergence. All
403,465 per-packet view-count/end-bit comparisons now pass on the 32-film corpus.
This does not establish that unsupported record bodies have become decoded.
