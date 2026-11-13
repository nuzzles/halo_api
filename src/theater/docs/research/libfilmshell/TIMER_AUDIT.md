# Timer semantics and continuity audit

The [machine-readable report](timer-audit.json) records canonical observations,
source references, manifest/chunk SHA-256 hashes and export hashes from two
retained films. This is a local captured-data audit, not an independent native
oracle or a claim of complete timer semantics. Native field/quantizer evidence
remains in [engine-v41](../../../fixtures/engine-v41/README.md).

## Findings

| Evidence | Octagon | Recharge Oddball |
| --- | ---: | ---: |
| Round timer observations | 429 | 906 |
| Respawn timer observations | 809 | 4,408 |
| Changed, positive-time round-timer pairs | 404 | 708 |
| Pairs with no intervening scoreboard gap/reset | 0 | 0 |
| Opaque type-10 packets | 26,100 | 72,145 |
| Opaque type-6 packets | 22 | 61 |

Octagon has first round-timer word zero throughout. Its second word rises from
zero to approximately 405.133 seconds; all 404 changing positive-time pairs
increase. State 19 occurs 427 times and state 1 twice. This supports count-up
behavior and does not justify calling the second word universally "remaining".

Recharge has first word quantum 547 (approximately 300.211 seconds) throughout.
The second word has 706 decreasing pairs and two increasing pairs, with states
6 and 1. These observations support a countdown interpretation in this capture.
They do not establish all state bits, resets, round boundaries, pause semantics
or a global elapsed/remaining mapping. Exact source quanta and endpoint-derived
seconds are retained in the report examples.

Recharge respawn words frequently take the form [9,10], [8,10], etc. while the
one-bit flag is clear; set-flag [0,0] occurs 1,472 times. Octagon commonly has
clear-flag [0,1] and set-flag [0,0]. The flag alone cannot identify an ongoing
countdown. The words are raw ten-bit integers in the native reader; this audit
does not establish a time unit or a spawn/death event.

Every changing positive-time pair in both timer families crosses at least one
scoreboard gap or a reset. Even a pair classified as intact would only lack
those boundaries; the diagnostic does not prove a continuous engine clock.
Examples carry exact source references, event indices, timestamps and gap
segments. Repeated values and same-time pairs are omitted only from the *change
statistics*, not from the exported observation ledger.

## Consequence for implementation

Pinned libfilmshell `statlog.rs` calls its round-timer words limit/left and its
respawn words seconds-left/total. It detects countdown starts when a word rises,
then can reassign the owner from a nearby death. These conventions cannot be
promoted to canonical facts. Preserve the existing raw observations, exact
quantizer and recorded holder joins; do not synthesize countdown starts or
round boundaries from the observed slopes.

The immediate format prerequisite is native body evidence for packet types
10 and 6. Their sequential envelopes are already retained; they currently stop
at `UnsupportedLayout` and therefore invalidate accumulated source state.
libfilmshell's container identifies type 10 as a per-tick companion and skips
its body. Skipping it is not proof that it cannot change relevant state. The
32-film packet inventory contains 403,465 type-10 and 451 type-6 packets.
Establishing their structure/effects is a more useful next step than weakening
gap handling to obtain a smooth timer plot. Unread type-0 entity suffixes remain
an independent continuity problem.

## Reproduction

From the repository root, with the retained film corpus available:

```sh
mkdir -p /private/tmp/halo-timer-audit
HALO_TIMER_AUDIT=/private/tmp/halo-timer-audit \
  cargo test --config build.incremental=false \
  captured_scoreboard_values_keep_canonical_sources_and_expose_gaps \
  -- --ignored --nocapture
python3 src/theater/docs/research/libfilmshell/timer_audit.py \
  /private/tmp/halo-timer-audit experiments/films
```

The opt-in exporter lives beside the existing engine tests. It does not alter
production parsing/runtime state. The captured test checks each projected field
against its original packet bits and timer conversion against the exhaustive
native oracle. Both captures passed in 57.88 seconds; strict all-target Clippy,
formatting and whitespace checks also passed. The full preceding production
checkpoint remains 500 ordinary tests and ten doctests. This audit adds no
production semantics or parser admission and does not complete the broader goal.

Sources: libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`,
`src/statlog.rs`, `src/model/scoreboard.rs`, `src/container.rs`; LevelUp
`413971bcc485be53a2fa8cf229e170539a5146f8`, `vitality.go` (blob
`2caa36888f53b767db48bf16b5a391fe7fa4b205`). Logs:
`/private/tmp/halo-timer-audit-final.log`, `/private/tmp/halo-timer-audit-clippy.log`.
