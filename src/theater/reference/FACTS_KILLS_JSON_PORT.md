# Killsource file JSON payload

`facts_kills_json.rs` preserves the exported native killsource.Result hierarchy,
including kills, unclaimed deaths, feed credit, damage source, provenance, assists,
damage shares, roster/index provenance, film table pinning, coverage, diagnostics,
calibration profile and optional relaxed probe. Source strings remain raw bytes
until native-compatible JSON normalization; named string/integer diagnostics are
retained. Native killsource.Kill.paquet is unexported and intentionally omitted by
its JSON writer. This cache representation is not the future canonical Film.

Existing FactsScanProfile and its complete nested profiles are reused. The new
oracle exposed a shared schema diagnostic mismatch for profile.Vec3Range; named
array schema support now preserves this native name without changing array update
semantics. Float32 encoding remains the existing pinned native formatter.

The independent Go oracle covers 163 writers (including NaN and both infinities)
and 9,685 read sequences. It visits every exported reachable field with valid and
invalid JSON types, integer limits and every single-byte string input, and checks
repeated successful pointer/slice updates. Read comparisons include raw normalized
string bytes and float32 bits. Writers compare exact bytes or errors. Errors poison
the reader; native partially mutated state after errors is outside this API.

All 25 shared facts host tests passed after the named-array correction. Combined
file-level validation and WASM results are in `facts-file-validation.json`.
