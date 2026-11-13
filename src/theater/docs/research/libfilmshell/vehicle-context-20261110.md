# Vehicle construction-context audit

The native source in `composants_vehicule_ti40.go` at LevelUp
`413971bcc485be53a2fa8cf229e170539a5146f8` says constructor
`FUN_14058c2ec` sets vehicle state byte `+0x818` when the `vehi` tag's physics
type equals 6. Both masked writers test that byte before including components
33/34. A recorded mask can therefore supply positive gate evidence, but an
omitted bit cannot prove the gate false. Unmasked snapshots require the gate
from construction context. Neither upstream revision supplies a physics-type
catalogue; the current libfilmshell snapshot walker still probes both layouts.

The new ignored audit parses all 32 films canonically, walks records in source
order and inventories vehicle NEW tags, explicit masked type rows and snapshot
stops. It never supplies a gate, scans for a successor or uses a later NEW to
label an earlier observation. Its result is
[vehicle-context-20261110.json](vehicle-context-20261110.json).

| Observation | Count |
| --- | ---: |
| Films / snapshot tables | 32 / 451 |
| Vehicle-context snapshot stops | 207 |
| Vehicle NEW records, including partial ones | 130 |
| Masked records containing vehicle-type state/physics | 7 |
| Records with all selected type rows complete | 6 |
| Gate evidence and complete tag in the same NEW | 0 |
| Masked DELTAs with an earlier complete same-full-ID vehicle NEW candidate | 2 |

All seven masked records occur late in the raid. Five have no earlier complete
same-full-ID NEW candidate. One of these five contains a complete state row and
a truncated physics row; the audit does not call that whole selection complete.
The remaining two candidates are:

- `chunk-147-type-2.bin`, packet 1180, ID `1073742663`: a candidate NEW in chunk
  135, packet 372 names tag `0xfe32c0f4` (reference catalogue: Warthog).
- `chunk-197-type-2.bin`, packet 106, ID `1073742664`: a candidate NEW in chunk
  136, packet 1000 names tag `0xaf31ab1a` (Mongoose).

Same-ID equality over these intervals does not itself prove a single lifetime:
generations are only two bits, packet bodies can be incomplete, and an unseen
reuse can invalidate an association. These are retained investigation candidates,
not an asset catalogue or a new parser admission rule. No snapshot stop was
removed. A next context investigation must verify lifetime continuity or obtain
the actual asset physics definitions before using either association.

The parser now accepts an explicitly supplied construction gate for one
snapshot record through `FilmParseContext.vehicle_snapshot_gates`, keyed by
chunk index, zero-based packet index and full ID. It records the selected
external value on the affected keyframe record. Synthetic true/false cases
cover both native branches; an unrelated packet site does not receive the gate.
None of the 207 captured stops is promoted without independently established
construction context. `TheaterRuntime::vehicle_snapshot_branches(chunk, packet,
record)` exposes both source-bounded reads on demand when the canonical record
stops at `vehicle+0x818`. It rebuilds the same recorded registry/map context as
`Film::parse` and verifies each branch against the unchanged recorded prefix.
It now also reports source-bounded next-slot plausibility for both branches,
the latest upstream's default-first layout hint, and a stricter unique hint.
An incomplete successor header is unknown, rather than zero-padded into a
choice. Neither hint selects canonical Film fields or proves `vehicle+0x818`.
The compact
19-bit type-state/physics case agrees with the original libfilmshell test at
`f050371`; the relevant component branches and test are unchanged at current
upstream `a13efb323b230c96b59685defbc376720043240f`. One retained
`appearance/01-cadet-blue` snapshot exercises the
runtime API; both alternatives preserve the canonical prefix and the Film stop.
The 207-stop audit remains the admission baseline until verified asset physics
context can select a branch.
The retained Cadet Blue film now also exercises `Film::parse_with_context` for
both externally selected branches at one exact snapshot site. Each public
record agrees in end bit, stop and component fields with its bounded diagnostic
read, and reports the supplied gate. This validates the integration path, not
which gate the original writer actually used.
For that snapshot, the no-type branch lands on a vacant-slot header and the
typed branch fails the next-slot check, yielding a unique *layout hint* of
false. The current upstream `snapshot.rs` blob at `a13efb3` is
`94c02bfe1f0c5dfa629d56e6ce9bb7f6f8eef1e3`, the same reader previously
audited. The hint remains a successor-shape heuristic, not asset physics
evidence.

The 207 stopped tags are:

| Tag | Reference label | Stops |
| --- | --- | ---: |
| `0xb857fb95` | M247 HMG Turret | 10 |
| `0x5b80c406` | Ghost | 64 |
| `0xaf31ab1a` | Mongoose | 53 |
| `0xfe32c0f4` | Warthog | 42 |
| `0xdd7f9102` | Warthog | 38 |

Labels come from the existing explicit old catalogue revision, not physics
classification. The names do not select a layout.

## Reproduction and limitations

```sh
HALO_VEHICLE_CONTEXT_AUDIT=/tmp/vehicle-all.json cargo test --offline --config build.incremental=false --all-features --lib captured_vehicle_masks_audit_prior_asset_gate_evidence -- --ignored --nocapture
python3 src/theater/docs/research/libfilmshell/seal_vehicle_context.py experiments/films /tmp/vehicle-all.json > /tmp/vehicle-sealed.json
```

`HALO_VEHICLE_FILM=raids/01-hour-long-raid` optionally selects the raid for a
focused rerun. The sealer accepts that result as a third argument to replace
only that film in a preceding full-corpus report. It independently walks byte
packet envelopes, checks all reported record extents and timestamps, and hashes
the reported source chunks plus every registry and manifest. It does not
validate semantic lifetime relationships or reinterpret source bits.

The initial all-film run took 133.56 seconds. The final raid rerun took 86.56
seconds and added full IDs, masks, encoding parameters and earlier-NEW candidates.
It also removed an unjustified audit-only `parameter == 1` filter: both native
rows record parameter 0 in the raid, and the source functions do not condition
their mask law on that parameter. The final report uses the focused result for
all seven affected records. No production parser code changed. Strict all-target
Clippy, formatting and whitespace checks pass.
