# HUD splash-text reference fixture

`cases.py` independently authors 35 complete static-component bodies, including
absent/empty text, all eight argument kinds, player absence and index limits,
signed integer limits, real-code endpoints and values surrounding zero, repeated
arguments, seven mixed arguments, and nonempty retained suffixes. No parser
output supplies fixture input or expected results.

`export_reference.py` verifies and compiles unchanged libfilmshell `splash.rs`
and `bits.rs` at commit `18067987ebc3628ad499c819c6aee814692df43b`. The appended
wrapper invokes the original private `text` and `Value::arg` functions. Minimal
container/payload types satisfy unused traversal code; no traversal or grouping
is used. `reference.json` retains both source blob hashes. Number results are
exported as exact f64 words, including signed integer interpretation.

The Rust tests run the canonical component reader on each authored body at all
eight bit alignments and compare its runtime projection with this independent
result. Every projected field retains exact source bits. All truncation points
in the mixed body must refuse a complete message. Additional tests exercise
malformed field shape/padding/order, schema mismatch, ambiguous schema names,
partial components, and actual Film baseline/DELTA observations. Repeated equal
observations remain separate; unrelated DELTAs do not repeat earlier text.

The existing ignored scoreboard corpus test also validates HUD observations in
Octagon and Recharge. Set `HALO_HUD_CAPTURE` to an existing directory to export
each complete canonical component's actual source bytes and local projection.
Pass each output to the native exporter, then compare with `verify_captured.py`.
Partial observations remain explicit errors and are excluded from native reads:
upstream's bit reader pads truncated buffers with zero.

```sh
python3 cases.py > cases.json
python3 export_reference.py /path/to/pinned/libfilmshell > reference.json
python3 export_reference.py /path/to/pinned/libfilmshell captured.json > native.json
python3 verify_captured.py captured.json native.json
```

The 24-bit real conversion is explicitly the pinned library's reference formula;
the original game quantizer is not established here. Unknown kinds retain their
tag and word locally where upstream returns an untyped missing argument. Keys
are localization keys, not variant-string-table indices. Participant identity,
holder attribution, actual viewer counts, screen duration and component suffix
semantics are outside this fixture's scope.
