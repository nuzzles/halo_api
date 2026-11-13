# Hill-score reference arithmetic

`reference.json` contains 60 arithmetic results from the unchanged
`src/zones.rs::hill_score_reader` at OpenSpartan/libfilmshell commit
`8f0f8d078e513c800baa21fac416fd4c0c127d03`.
The exporter verifies the exact Git blob `4701db66d587260404006301be25b4fb2ae1931f`.

Regenerate with a checkout of that source:

```sh
python3 export_reference.py /path/to/libfilmshell > reference.json
```

The harness extracts the original Bits reader, HillScore, property constants and
hill_score_reader without changing their bodies. Minimal payload structures,
bounded MSB bit access and a matching property-width helper let it run with rustc
alone. Synthetic Create operations provide the property names and explicit row-2
block offsets used by the upstream API. These are invocation fixtures for the
original arithmetic, not independently parsed complete native packets. The
helper's historical widths for other tags are not evidence for those tags.

Five integer limits (1, 2, 99, 100, 999) cross twelve 24-bit bank values, including
both endpoints and values around the upstream zero quantum `0x7fffff`.
The upstream convention is `(q - 0x7fffff) / 0x7fffff * 100`, multiplied by the
limit, rounded and clamped at zero. This differs from engine midpoint
dequantization. The harness's fixed team callback isolates that formula; it does
not establish viewer/team identity or validate team attribution.

Runtime tests serialize the same inputs into separate authored native
keyframe/NEW/DELTA packets using the source-traced class 13 property grammar.
They compare points and source links across unchecked and both checked trailer
gates. Additional native tests cover names, limits, scopes, duplicates, lifetimes,
partial bodies and untimed gaps. Deliberately mutated models separately test
source mismatch, duplicate rows, invalid extents and partial defaults. The
underlying grammar/truncation evidence remains in property-v41; neither this
harness nor its synthetic registry expands public NEW-envelope admission.

The local runtime deliberately refuses ambiguous properties and ill-typed limits,
honors name updates, and invalidates stale data at lifetime and source boundaries.
It does not copy upstream's raw-word limit coercion, implicit retention, two-team
mapping or aggregation across viewers. Those policy differences are not arithmetic
oracle failures. See ../../docs/ZONES.md for the public API and remaining work.
