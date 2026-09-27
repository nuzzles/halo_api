# Native NEW binding admission

Reference pin: d61443ef59268ad734355db8e9974f68db5ca6d0.
Generator: ../reference/halo_rust_new_binding_d61443e_test.go.txt.
Run TestHaloRustNewBindingD61443e in the pinned grammar package; compress
/private/tmp/halo-new-binding-d61443e.json with zlib.

15 independent Go cases cross three generations with absent, hard same-archetype,
hard different-archetype, soft and explicitly deleted prior bindings. Only a hard
different-archetype binding rejects replacement. Every case retains the decoded
NEW record and continues through the final control view. This ports
frame_infer.go's contreditUneEntiteVivante, not an inferred player action.
The generic/raw-resync readers and unconditional bind_full setup operations
remain governed by their own reference paths.

Rust tests exercise both production and inference-view APIs, retained raw fields,
final bindings, source-referenced refusal diagnostics, shared observer counters,
merged diagnostics and JSON round trips. Counts alone are not the assertion:
each case checks its Go view count, endpoint, final archetype and refusal count.
