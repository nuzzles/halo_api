# Reference rule evaluation fixtures

`cases.py` independently authors 518 Bond variant trees. The current catalogue
export supplies names and aliases; a separate Python Murmur3 implementation
constructs wire identifiers and checks all 98 hash/name pairs. Twelve cases
exercise all 247 rule defaults. Another 486 cases exercise aliases, including
older names and case-only aliases. Twenty additional cases cover alias order,
duplicate/equal rows, missing/malformed/unknown values, unidentified hashes,
missing/malformed containers, prefix versus script evidence, and exact float/bool
words. Tests assert that every authored alias actually matches its source row.

`export_reference.py` verifies four original Git blobs at libfilmshell
`f050371a6cda83b9887eeb83a1884336fce28e53`: `src/variant.rs`,
`src/model/variant.rs`, `src/game/options.rs`, and `src/game/options_data.rs`.
It compiles unchanged modules plus wrappers, runs the original eight-byte
decoder, and supplies its results to original `GameVariant::rules` and
`Rule::effective`. TagId is stubbed as the unused scalar wrapper. Outputs retain
the original decoder result by input row, rule order, defaults, selected values
and effective values. No local code or local numeric normalization generates
expected results. This is an original public-model oracle; malformed/unknown
wire rows cannot be represented by that model and are separately tested locally.

Local evaluation keeps source values as their exact eight bytes and f32 words.
The independent original decoded row supplies only the comparison value; it
never replaces the local value. Local negatives explicitly retain ambiguity,
unknown IDs/payloads and missing inputs instead of selecting first duplicate
rows, discarding invalid values, trying later aliases or inserting defaults.
Shared arrays keep repeated matches and unidentified-row evidence bounded.

```sh
python3 src/theater/fixtures/variant-rules-v41/cases.py
python3 src/theater/fixtures/variant-rules-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/rules-reference.json
cmp src/theater/fixtures/variant-rules-v41/reference.json /tmp/rules-reference.json
cargo test --offline --lib runtime::variant::rules
```

The captured variant test exports all reference groups, applicability evidence,
aliases/row ordinals, exact recorded values and evaluation provenance. Native
rule selection agrees across all 32 films and 465 evidenced rules: 285 recorded
overrides, 60 reference defaults and 120 rules without literal defaults. Original
decoder outputs are compared separately from local raw words. The comparison
fails on any unexpected unresolved state and checks alias row membership/order.
The reference default/applicability remains a hypothesis, not a recorded value
or proof that an Options script was loaded. Canonical Film is unchanged.

```sh
HALO_VARIANT_EXPORT=/tmp/rules-captured.json cargo test --offline --lib captured_variant_views_preserve_source_entries -- --ignored --nocapture
python3 src/theater/fixtures/variant-rules-v41/export_reference.py /path/to/pinned/libfilmshell /tmp/rules-captured.json > /tmp/rules-native.json
python3 src/theater/fixtures/variant-rules-v41/verify_captured.py /tmp/rules-captured.json /tmp/rules-native.json
```
