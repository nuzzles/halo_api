# Captured weapon-fire continuations

Seven unchanged packet payloads from bundled Oddball match
`4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73`. Times are microseconds relative to
engine origin 15794890555. Chunk/byte positions address decompressed inputs.

| Time / filename | Player | Chunk | Payload byte | Fire end bit | Event-list end bit | SHA-256 |
|---|---|---|---|---|---|---|
| 230612473 | TEED MOBEAST | 12 | 650177 | 244 | 538 | `fbfbde34405c80b2ee829950c00569015fb11bf49ba444bc2ccdafabdf62aaee` |
| 359342177 | Vague v1 | 18 | 813376 | 186 | 274 | `1959d1a88c345512e707909be8a29bd0df2ddfd69b107a4263cdb341b1e0312f` |
| 532868633 | Nighted Zion | 27 | 712495 | 246 | 597 | `8869518bf046314309a0481ff91bdd6f29ca1bfa7730dd719ea99c369212bbcb` |
| 763286544 | TEED MOBEAST | 39 | 562325 | 241 | 329 | `c65348578a8b1e9864e457af81b517ec4604ec042d826013630d0de1de40cbe9` |
| 929442282 | Vague v1 | 47 | 681755 | 244 | 538 | `46bb294d4aff63d7144d8a9e7f2ffe46f387423a428c25207c7ecbe2bca900e4` |
| 1070150702 | Nuzzles | 54 | 672521 | 230 | 318 | `0ad407c0f10002247e68ff719414f64463889d3fb9873e2864d0c27f02aa6a09` |
| 1085031941 | Nighted Zion | 55 | 584907 | 239 | 327 | `0c58dab08cd6939192f7f97fefb02a4bd888bf5d07464726f5c779b083ec48c8` |

The grammar is ported from LevelUp commit
`413971bcc485be53a2fa8cf229e170539a5146f8`, under
`apps/go-api/internal/games/halo_infinite/film/internal/grammar/`:
`vue_a_charges_tir.go`, `bloc_action.go`, `default_state_ti41.go`,
`vue_a_versions.go`, and `vue_a_lecture.go`.

This newer port corrects the older August research: film passes param_5=1;
target references use category widths rather than raw 32-bit handles; the first
composite is FUN_140c9e4d8; its presence determines whether the 30-bit aim follows.
The final present component's three-bit field can suppress both composites/aim.
The tail includes optional scales, block-dependent vector/fields, and an optional
position. There is no fixed skip or search for following records.

The contextual resolver uses event version 3, category references, quantized
positions under the supplied map bounds and no corruption trailer.
No other version/runtime mode is inferred. The default canonical parser still
refuses event 36; unknown/default map regions stop contextual decoding too.

The four modal packets have event sequence 36,82. Sequential decoding recovers:

| Time | Full biped ID | NEW bit range | Player index |
|---|---|---|---|
| 359342177 | 1073742409 | 274..1995 | 6 |
| 763286544 | 1073742500 | 329..2018 | 7 |
| 1070150702 | 1073742571 | 318..2007 | 3 |
| 1085031941 | 1073742574 | 327..2016 | 5 |

The three nonmodal packets reach a subsequent event 0 (damage). The structural
reader's absent-on-one five-bit gate is confirmed by `chargeDegatsApres` in
`vue_a_charges.go` at the same pinned revision (D-LN-2). The separate historical
weapon-damage projection had reversed it. Correcting that projection resolves
the conflict and permits these additional sequential biped NEWs:

| Time | Full biped ID | NEW bit range | Player index |
|---|---|---|---|
| 230612473 | 1073742381 | 538..2227 | 7 |
| 532868633 | 1073742447 | 597..2286 | 5 |
| 929442282 | 1073742537 | 538..2259 | 6 |

Captured tests assert all seven event sequences/extents, reject every truncated
event-list byte prefix, and refuse disabled context/configuration=0. Additional
wire vectors exercise short fire records, timestamp blocks and optional tails.
CLI integration checks all seven recovered lives, exact NEW source ranges and
entity IDs, plus nested-shot attribution and lifetime boundaries. Historical
agreement is regression evidence, not independent annotated ground truth.
