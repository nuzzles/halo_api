# Stat text-reference fixtures

`cases.py` independently authors 20 Bond trees. The positive example deliberately
orders its definition references differently from string-table order. Other
cases cover shared references, duplicate IDs, missing IDs, distinct indices with
equal text, conflicting fields, missing/negative/out-of-range/wrong-type indices,
inherited/duplicate fields, empty/invalid UTF-8 text, and missing/empty/ambiguous
dependent tables. Tests use public Film/runtime entrypoints and check the source
chain from definition field to string entry, text bytes and NUL terminator.

`export_reference.py` verifies the original `src/variant_stats.rs` blob
`4814607a2a27f4ad2cd4ef0a16845582f73c8172` at libfilmshell
`f050371a6cda83b9887eeb83a1884336fce28e53`. It compiles the unchanged reader with
a wrapper that exports fields 8–11 by definition row. The original numeric reader
does not enforce local signed-i32 typing or inheritance/duplicate-field policy;
those stricter behaviors are tested locally, not claimed as native behavior.
The oracle establishes values and row association, not individual text roles.

```sh
python3 src/theater/fixtures/stat-text-v41/cases.py
python3 src/theater/fixtures/stat-text-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/stat-text-reference.json
cmp src/theater/fixtures/stat-text-v41/reference.json /tmp/stat-text-reference.json
cargo test --offline --lib runtime::variant::stats::text
```

The captured view test retains 5,296 indices across all 1,324 definitions, plus
23,832 localized values. The separate comparison verifies every index against
the original reader, then every resolved language against its recorded indexed
string entry. The existing original variant-text oracle independently agrees on
all 1,323 English labels it emits. Canonical source spans must match that oracle's
input before reusing its output.

Oddball definitions 45 (`SkullHeldTime`) and 46 (`SkullScoringTicks`) both reference
entry 55, Carry Time. The original positional scan supplies that text once, leaving
the final definition without a label. This view retains both associations and
the distinct definitions. It does not infer equal units/precision from equal
labels or assign short/long/plural/display roles to fields 8–11. Consensus is
available only when all four recorded indices agree; callers can inspect fields
individually when they disagree. No absent field receives an inferred index.

```sh
HALO_VARIANT_EXPORT=/tmp/stat-text-captured.json cargo test --offline --lib captured_variant_views_preserve_source_entries -- --ignored --nocapture
python3 src/theater/fixtures/stat-text-v41/export_reference.py /path/to/pinned/libfilmshell /tmp/stat-text-captured.json > /tmp/stat-text-native.json
python3 src/theater/fixtures/variant-view-v41/export_reference.py /path/to/pinned/libfilmshell /tmp/stat-text-captured.json > /tmp/variant-native.json
python3 src/theater/fixtures/stat-text-v41/verify_captured.py /tmp/stat-text-captured.json /tmp/stat-text-native.json /tmp/variant-native.json
```
