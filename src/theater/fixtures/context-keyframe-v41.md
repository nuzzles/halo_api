# Native contextual full-state traversal

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
30 cases cross direct record, chain and table APIs, calibrated/stub maps and
live-hook widths -213, -181, 0, 8 and 2^32. An EMP callback at the first component
sets the following width. Record end -32 is retained by direct reads and chain
attempts; table traversal subsequently panics on its sentinel fallback. Both
panics retain the callback and map mutation. Rust compares records, component
boundaries, stops, counts, callbacks, final widths and JSON.
Native source: reference/halo_rust_context_keyframe_test.go.txt, test
TestHaloRustContextKeyframe. See reference/keyframe-context-validation.json.
