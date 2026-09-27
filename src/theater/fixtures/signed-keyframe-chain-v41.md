# Native signed keyframe chain boundary

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
56 direct ChainKeyframeRecords cases cross empty/16-byte sources, seven signed
starts and four targets. The return records reached/skipped/without-archetype
counts and the exact stop category. At from=i64::MAX-63 and want=i64::MAX, native
wrapping pos+64 passes the outer source check and reports invalid header. The corrected Rust guard preserves this distinction. No body is entered in this fixture.
Native source: reference/halo_rust_signed_keyframe_chain_test.go.txt, registered
in reference/generate_oracles.py. Native generation and Rust comparison pass for standalone and contextual APIs.
See reference/keyframe-context-validation.json for validation status. These are tiny-source boundary checks, not large scans.
