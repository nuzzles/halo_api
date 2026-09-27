# Sequential keyframe-table corpus

Pinned reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_keyframe_table_corpus_test.go.txt,
TestHaloRustKeyframeTableCorpus, registered in generate_oracles.py.

The native harness reads the manifests and raw chunks for all 32 local v41
films and invokes actual WalkKeyframeRecords on every type-2 keyframe packet.
It uses ContexteParDefaut with each film's native corruption flag, matching the
existing captured-keyframe control. It is not the map-calibrated replay context.

There are 451 keyframes, 902 ordered records and 5,412 component trace entries.
Every table stops at the unsupported tacmap-mapdismissallock component in its
second record. Of the component traces, 451 are the unported stopping entries;
none contains a native captured Payload value. Do not call this complete body
coverage for the 134,657 anchors recovered separately across these keyframes.

The Rust comparison checks source packet ordinals and payload ranges, record
identities/order/start/end, ported component names/start offsets, the same stop,
and that every retained raw ComponentField equals its original source bits and
stays within its record. It does not use Rust output to generate expectations.
The existing single captured-keyframe test shares these stronger checks.

This fixture establishes deterministic prefix traversal and raw-field retention
under that explicit context. It neither skips the unported component nor
interprets the rest of each payload as padding. Later anchored component reads,
map-specific contexts and whole-Film fidelity remain separate acceptance work.
