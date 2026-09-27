# Contextual lazy position-width comparisons

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

Five new fixtures extend the existing context harnesses: inference (128), chain
inference (128), validated resync (128), explicit repair (64), and production
views (128). Each sets all movement index/axis dimensions to one of 0, 65, 2^32,
2^32+1 and uint64::MAX. These dimensions are not consumed by their components.
Native output for all 576 cases is identical to the corresponding original
case after excluding the added profile metadata. Original fixtures are unchanged.
The tests compare native records, end positions, callbacks/counters, observer
restoration, calibration maps and world state as applicable to each API.

NativeFrameConfig now uses a policy-only frame adapter for all contextual APIs.
The actual raw dimensions are attached to the contextual record, repair and
chain-trial readers. Checked legacy encoding adapters remain explicit checked
conversions. Signed frame scalar settings still convert eagerly.

A reached dimension beyond the platform address domain records NativeWidthRefusal
in FilmReadDiagnostics: field, bit position, raw width and supported maximum.
This distinguishes a decoder limitation from missing bytes. Refusals remain on
failed records and speculative diagnostics, merge in encounter order, and survive
serialization. Direct/live frame readers continue returning their Width error.
Arithmetic overflow after a representable width and signed cursor continuation
remain separate unresolved cases.

The saved Go harnesses contain TestHaloRustContext{Inference,Chain,Resync,Repair,
Views}LazyWidths and are registered in generate_oracles.py. Fixture hashes and
counts are in ../reference/context-lazy-width-validation.json. These are native
parser comparisons, not independently annotated gameplay actions.

Run host comparisons with `cargo test --lib native_context_`. The wasm32 runner
`python3 src/theater/reference/verify_frame_lazy_widths_wasm.py` includes the old
frame and accumulator checks, both versions of these five context fixtures, and
existing raw-resync/live-resync cases (2,832 native cases total). It also checks
reached-width diagnostics at bit 35 through inference and production views,
merging, and serialization. For the two external-runner tests that seed the
reference histogram, the runner uses a public direct absolute-position read
before installing callbacks, equivalent to their private host-test seed.

The architectural refactor remains deferred. Broader source/cursor audits and
missing captured positives remain open; this matrix is not full parser parity.

Validation completed: 17 focused host tests, 570 full Theater tests (46 ignored),
Clippy, 2,832 actual wasm32 native cases and the reached-width diagnostic controls.
The six captured films pass all 44 input comparisons, objective-consumer controls
and complete-document checks (103.08s). Formatting, Python syntax, diff checks and
all 485 pinned source hashes pass. Logs: /private/tmp/halo-context-lazy-*.log.
