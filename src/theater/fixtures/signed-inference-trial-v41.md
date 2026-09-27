# Signed inference trials and confirmations

Pinned reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `reference/halo_rust_signed_inference_trial_test.go.txt`.

72 configurations combine calibrated/stub skips, starts 0/32, optional prefixes,
and ordinary/negative/wrapping i64 widths. Native direct trials accept 40 negative
endpoints. Single-step inference has 16 panic cases; chain inference has 32.
Two configurations confirm endpoint -32 in each inference mode.

Four native complete-frame cases enter at -8, skip the prefix to an unbound
Delta at bit 24, infer its body to endpoint -32, then use the prefix to reach
a clean bound Delta at bit zero. They finish at bit 48. The comparisons retain
record order, inferred endpoint, binding/soft state, and JSON round trips.

Rust tests compare direct trials on the host, public contextual single-step and
chain APIs, and complete frames. WASM omits panic cases (abort semantics), runs
56 single-step and 40 chain calls plus the four complete frames, and preserves
negative inference results. These finite cases do not establish every raw resync,
repair or harvest signed-domain behavior. No architectural refactor is included.
