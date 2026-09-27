# Live single-step inference and capture restoration

Pinned native reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_context_inference_test.go.txt`, registered in
`generate_oracles.py`. Expectations come from native `decodeInferLoop` with chain
inference disabled, including its actual `inferUnboundArchetype` calls.

128 cases use unique, duplicate/ambiguous, and unsupported candidate archetypes,
known or soft-bound successors, view-table rejection, optional record prefixes,
and short payloads. There are 61 inferred records, 163 total records, 83 end/view
rejection results, 25 unknown-slot view rejections, and 1,217 actual callbacks.

Position, movement and unit-reference hooks are installed before inference.
The native speculative scope temporarily clears those callbacks, while EMP and
mobility remain active. In even cases the mobility callback installs replacement
movement/reference hooks during the scope. Later publications must see those
replacements. Scope exit restores the saved hooks; subsequent retained records
may change them again. After decoding, three explicit hook probes verify the
resulting position/reference/movement receivers. These probes are test stimuli,
not recorded film actions.

Rust compares returned inference count, stop/end, record IDs/archetypes, decoded
masks/boundaries, actual callback order/values, view rejection counters and world
slots. It also roundtrips the inference output. Supplementary Rust tests exercise
nested restoration and unwinding. The contextual API preserves diagnostic
candidates independently of which hooks are installed; actual observer delivery
is what the native callback comparison validates. Legacy no-context APIs retain
their existing filtered diagnostics behavior.

Live recursive chain inference and resync observer ownership remain separate
work. The new single-step entry point explicitly refuses the chain-inference
flag rather than running it without its observer context.
