# Native reads at every recovered keyframe anchor

Reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Native generator: TestHaloRustKeyframeAnchorBodiesCorpus in
reference/halo_rust_keyframe_anchor_bodies_test.go.txt.

The harness invokes actual WalkKeyframeWorld and WalkKeyframeFullState for all
134,657 anchors in 451 keyframes from 32 local films. It uses ContexteParDefaut
and the captured corruption flag. Simulation completion is explicitly false in
the Rust comparison, matching that native context. It is not a map-calibrated
profile or a claim that recovered anchors are proven sequential boundaries.

Native results contain 95,532 complete reads, 39,125 desynchronizations, 468
endpoints past the physical payload and 43,631 non-null dead-state values.
Captured payloads include 460 round timers, 14,432 respawn timers and 43,631 each
of body vitality, shield vitality, parent state and dissolver. Native padded
reads and native desynchronizations remain expectations, not fabricated success.

The compressed JSONL fixture retains the entire native trace per anchor. Tests
stream it from disk rather than embedding hundreds of MB of JSON in the test
binary. Each row records the source file, packet ordinal, payload byte range,
anchor identity/bit offset, native component results and stopping endpoint.

Current comparison covers identities, full-state endpoints, desynchronization
indices, and ordered ported component indices/names/start offsets. The initial run covered trace boundaries only; the completed extension below
adds typed payload and source-field checks. Integrated Film retention is tested
separately by recovered-keyframe-padding-v41.md.

Rust comparison passed all 134,657 reads in 54.43s; clippy passed in 8.67s.

## Typed payload and source-retention extension

KeyframeRecord.captured_payload now projects any of the six native capture kinds
from the fields/observations already retained in a successful component span.
It does not reread source data and returns None for out-of-range spans or names
outside the native capture set. The raw fields remain present alongside values.

The expanded corpus test compares 189,416 expected captured values and 43,631
dead-state objects against the original native trace. Dissolver's extra 96 raw
bits, discarded by native Payload, are checked independently against source.
Native parent Payload.EndBit is zero without a hook; the test checks Rust's
observed end against its component range and then normalizes only that diagnostic
field for value comparison. No parent identity or relationship is inferred.

Every record is also checked for contiguous retained fields from header through
its stopping point, original source-bit equality with explicit zero-padding,
and complete JSON roundtrip. Expected padding cases remain 468. These expanded
checks passed all 134,657 anchors in 399.18s. Log:
`/private/tmp/halo-keyframe-anchor-payloads-tests.log`. The earlier 54.43s result
covers only trace boundaries.
