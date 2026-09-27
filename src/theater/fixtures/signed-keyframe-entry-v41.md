# Native signed full-state entry

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
24 cases cross empty/32-byte zero sources and twelve signed starts, including
negative starts and i64 extremes. Native reads TI at start+58 then skips the
header; eight cases panic and sixteen return. Both source-only and contextual
Rust entry points compare type, end, completion, panic behavior and JSON.
Native source: reference/halo_rust_context_keyframe_test.go.txt, test
TestHaloRustSignedKeyframeEntry. See reference/keyframe-context-validation.json.
Large positions use tiny sources, not multi-gigabit scans.
