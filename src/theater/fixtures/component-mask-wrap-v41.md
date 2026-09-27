# Component-mask wrap

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_component_mask_wrap_test.go.txt.
36 cases cross registry lengths 0, 1, 63, 64, 65, 66, 127, 128 and 129 with
full keyframes and generic dense masks (bit 0, bit 63 or all bits). Native
component iteration uses index & 63 and continues through the registry. Each
selected EMP component carries a distinct patterned byte; the fixture retains
ordered indices, traces, callbacks and endpoint. Native generation passes.
The fixture also includes 27 direct inference-body trials and recursive chain
outcomes. Rust comparison and final validation are tracked in
reference/component-mask-wrap-validation.json.
