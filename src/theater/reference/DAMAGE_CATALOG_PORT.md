# Damage-tag catalog source mapping

Pinned source: `damagetag/damagetag.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Rust implementation: `kill_damage_tags.rs`.

| Native declaration | Rust equivalent |
| --- | --- |
| Class and seven class constants | `KillDamageLabel.class` retains the exact native strings, including undeclared external values. |
| Status and four status constants | `KillDamageLabel.status` retains the exact native strings, including undeclared external values. |
| Label / Publishable | `KillDamageLabel` / `publishable`; only VALIDE and SOUS_RESERVE publish. |
| Provenance / Source | `KillDamageProvenance` / catalog `provenance`. |
| IsDamageEffect | `KillDamageCatalog::is_damage_effect`. |
| Strong | `strong_kill_damage_tag`; independent of catalog membership. |
| IDs / Size | Catalog `ids`, a sorted set; `.clone()` gives an independent owned copy and `.len()` gives size. |
| Lookup / Labels | `lookup` and catalog `labels`, a tag-sorted map; cloning values gives independent labels. |
| rawIDs / rawLabels / init | Pinned text files and `pinned_kill_damage_catalog` using the shared parser through `OnceLock`. |
| headerDate | First date token per comment, with later nonempty comment dates taking precedence. |
| parseIDs / parseLabels | `parse_kill_damage_catalog`; IDs parsed before labels; first error aborts without a partial catalog. |

ID rows are trimmed before comment/hex processing. Label rows only use trimming
to recognize empty lines; comments must begin in column zero and all six fields
remain verbatim. Duplicate IDs collapse; duplicate labels use the last row.
Labels absent from the ID set remain present. CRLF label data retains the CR in
the last field, as native `strings.Split` does. Fixed-base hex rejects signs,
prefixes, underscores and out-of-range values.

Rust error display wording differs deliberately. The typed error retains table,
one-based physical line, input, and syntax/range/column-count failure. This is
not an assertion of byte-identical Go error formatting. Parser inputs are UTF-8
Rust strings; this catalog API does not claim arbitrary invalid-UTF-8 input support.

The native oracle invokes the pinned private parsers, not a reimplementation.
It checks edge cases and the complete embedded tables. Existing source-truth
fixtures cover label projection and category naming used by kill decoding.
Closing this source mapping does not close kill pipeline/document parity or
independent physical-action validation.
