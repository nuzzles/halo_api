# Recorded variant view fixtures

`cases.py` independently authors 62 Bond trees. They cover all four optional
variant-instance lists, 24 named numeric paths, ten flag meanings in both list
and set containers, separate loadouts/weapon entries, signed ID wrappers,
duplicate hashes, missing row members, all numeric wire representations,
negative zero/NaN/infinity/subnormal option words, unknown option discriminators,
wrong parents/lengths, duplicate fields, inherited IDs and ambiguous instances.
The Rust tests insert these trees into the previously verified sequential
game-options envelope and use public Film/runtime entrypoints.

`export_reference.py` verifies four blobs at OpenSpartan/libfilmshell
`f050371a6cda83b9887eeb83a1884336fce28e53`: `variant.rs`, `variant_stats.rs`,
`game/options.rs`, and `game/options_data.rs`. It compiles unchanged source plus
test wrappers and executes the original Bond value reader, option-value decoder,
18 named path constants and ten flag mappings. It also runs the original stat/
playback projection against an artificial anchor header surrounding the authored
BaseVariantData body. That framing is only an adapter for the original API; the
local runtime never searches for those anchors. Fixture output retains native
callback paths/values, endpoints, normalized upstream option values and selected
native stat/playback results, stat definitions, scoreboard-column matches and
the respawn-settings override tag.

Local comparisons retain exact raw values where the original API rounds floats,
casts unsigned values to signed, ignores f64s, or filters/deduplicates entries.
Native callback traces verify the source paths and eight-byte option payloads;
they do not validate all malformed-input policies or named variant semantics.
Separate local negatives check source-preserving ambiguity and absence behavior.
Missing values receive no defaults. Complete messages remain usable when later
game-options tail fields are truncated. The catalogue revision does not select
or fill variant values.

Reproduce:

```sh
python3 src/theater/fixtures/variant-view-v41/cases.py
python3 src/theater/fixtures/variant-view-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/variant-reference.json
cmp src/theater/fixtures/variant-view-v41/reference.json /tmp/variant-reference.json
cargo test --offline --lib runtime::variant
```

The ignored corpus test walks all 32 retained films. It optionally exports each
selected canonical variant span and local option rows to a scratch file. The
original reader then traverses those same spans independently; the comparison
checks exact option hash/eight-byte values in source order and whole-base endpoints.
This is separate from the authored fixture and is not proof of unrepresented builds.

Sixteen stat-specific trees additionally cover all definition properties, unsigned
IDs/attributes, repeated/unmatched/missing column IDs, missing or malformed
definition IDs, same-layer duplicates, inherited IDs, raw invalid UTF-8/NULs,
absent versus empty strings/containers, and full-width signed precision. The
local mapping retains ambiguity and unknown IDs instead of upstream's first-match
and zero-default policies. Repeated ambiguous columns share their candidate arrays.
Captured comparison now includes all definitions and column references, checks
recorded fields directly and reports omitted fields separately from upstream
defaults. It does not fill the local fields during comparison.

```sh
HALO_VARIANT_EXPORT=/tmp/captured-variants.json cargo test --offline --lib captured_variant_views_preserve_source_entries -- --ignored --nocapture
python3 src/theater/fixtures/variant-view-v41/export_reference.py /path/to/pinned/libfilmshell /tmp/captured-variants.json > /tmp/native-variants.json
python3 src/theater/fixtures/variant-view-v41/verify_captured.py /tmp/captured-variants.json /tmp/native-variants.json
```

Fifteen string-table cases cover variable/empty language lists, retained bools,
shared and suffix offsets, invalid UTF-8 and empty strings, negative/out-of-range
offsets, missing terminators, absent/empty/malformed fields and inheritance.
The reference-convention example uses the unchanged upstream text API to check
name/description/label bytes. Artificial framing only supplies that API's start
offset; the local runtime uses indexed Bond paths exclusively. Corpus comparison
checks 1,639 flags and 29,502 offsets against original value callbacks and all
32 name/description pairs against original text extraction. The local test also
verifies every text and terminator bit range against Film. Native label texts
are checked for presence without assigning table indices to stat slots.
