# Native fallback counter wiring inventory

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The existing [call-site audit](fallback-callsite-audit.json) remains the native
site-to-producer inventory (19 sites, 18 names). This follow-up refreshes production
references and adds measured positive/zero fixture coverage; it does not replace
that inventory or erase its remaining caller-equivalence caveats.

The pinned catalog declares 99 fallbacks: 18 with `CompteurBranche=true`,
81 without a native wired counter. Catalog coverage alone is not runtime
instrumentation. This source audit maps all 18 wired names to Rust production
sites; it does not assert complete condition/count parity from string presence.

| Native wired name | Rust production references |
| --- | --- |
| `repli_piece_engendree_sans_evenement` | `replay_document_film.rs:488` |
| `repli_geste_dernier_occupant_du_match` | `replay_usage.rs:194` |
| `repli_geste_premiere_vie_du_slot` | `replay_usage.rs:223` |
| `repli_garde_equipement_negatif_a_zero` | `replay_usage.rs:415` |
| `repli_plafond_grenade_par_defaut` | `replay_document_film.rs:476` |
| `repli_largeurs_axe_par_defaut_conservees` | `world_precision.rs:3` |
| `repli_identite_piste_meilleur_recouvrement` | `replay_document_film.rs:484` |
| `repli_vie_coupee_au_trou_de_replication` | `replay_document_film.rs:558`, `identity_lifetimes.rs:59` |
| `repli_chassis_vehicule_marqueur_neutre` | `replay_vehicle_publication.rs:96`, `replay_vehicle_coverage.rs:109` |
| `repli_position_lacher_prend_la_prise` | `replay_document_film.rs:500` |
| `repli_piste_drapeau_sans_pont_ecartee` | `replay_document_film.rs:496` |
| `repli_zone_camp_de_capture_deduit_de_l_issue` | `replay_document_film.rs:504` |
| `repli_colline_votes_periode_entiere` | `replay_document_film.rs:508` |
| `repli_colline_dernier_intervalle_ouvert` | `replay_document_film.rs:512` |
| `repli_armement_bombe_debut_a_zero` | `replay_document_film.rs:382` |
| `repli_siege_du_remplacant_par_appariement_ordinal` | `replay_document_film.rs:492` |
| `repli_manche_zero_decretee` | `replay_document_film.rs:267` |
| `repli_cadre_de_marche_par_defaut_conserve` | `replay_vehicle_publication.rs:45` |

## Publication boundaries checked

- `replay_usage.rs` triggers its three usage-specific counters on the native
  last-occupant, first-life and negative-kept-count branches, then retains
  `fb.report()` in the usage result. These projection counters are independent
  of decoder/document counters; they must not be counted twice in Film.
- `replay_vehicle_publication.rs` retains its frame-default and neutral-chassis
  counts. `replay_document_film.rs` merges the returned vehicle fallback map.
- `world_precision.rs` retains default-axis fallback observations in the
  precision result. The document maps `scan_precision.fallbacks` into its report.
- Other document sites derive counts from their corresponding returned layer
  counters. The document calls `set_fallbacks` after those layers are assembled.

## Remaining evidence gate

Check each counter against an independently executed native branch, including
negative cases and complete source/document publication. Existing component,
layer and document fixtures provide partial runtime evidence; this inventory
does not replace that audit. The 81 unwired native catalog entries still need
behavioral parity where their fallback behavior is supported, but they do not
require inventing counters that the pinned native parser does not emit.

This inventory concerns the current v41 port, not the deferred architecture.


## Existing native fixture branch coverage

These counts were computed from expected native outputs, not from Rust results.
The corresponding Rust fixture tests compare these output counters and ran in
the latest full Theater suite. A zero case is a counter-negative control; it
does not necessarily exercise every alternative branch or complete Film assembly.

| Counter | Native fixture | Cases | Positive | Zero |
| --- | --- | ---: | ---: | ---: |
| repli_geste_dernier_occupant_du_match | usage | 1036 | 611 | 425 |
| repli_geste_premiere_vie_du_slot | usage | 1036 | 558 | 478 |
| repli_garde_equipement_negatif_a_zero | usage | 1036 | 162 | 874 |
| repli_largeurs_axe_par_defaut_conservees | world-precision | 512 | 384 | 128 |
| repli_zone_camp_de_capture_deduit_de_l_issue | replay-zones | 1024 | 548 | 476 |
| repli_colline_votes_periode_entiere | replay-zone-hills | 1024 | 663 | 361 |
| repli_colline_dernier_intervalle_ouvert | replay-zone-hills | 1024 | 682 | 342 |
| repli_armement_bombe_debut_a_zero | bomb-start-fallback | 288 | 24 | 264 |

The broad replay-zones fixture has zero positive hill-fallback cases. Its
separate replay-zone-hills fixture supplies the positive cases listed above.
Do not infer missing hill coverage from the broad fixture alone, or infer
whole-document publication from the direct hill-layer comparison. The other wired counters retain their existing call-site audit evidence, including
vehicle and flag comparisons; their positive/zero counts have not all been
refreshed in this follow-up.

The bomb-start boundary fixture was added after the latest full suite; its focused
Rust test passed (0.02s). All 1,024 older replay-bomb-armings cases have zero
start fallbacks. The new fixture supplies positive branch evidence and full
direct-layer output comparison, while captured bomb/document acceptance stays open.
