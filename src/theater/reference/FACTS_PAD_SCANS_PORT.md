# Cached pad scans and individual ground weapons

build_facts_replay_ground_pads matches native buildWeaponPads before event dating
or individual ground-item publication. ReplayGroundPads exposes the pad arrays,
pickup intervals, coverage, and retained weapon objects needed by the next stage.
It initializes native scan counters independently of each stream's scanned flag,
returns initial coverage on zero step, and gates the two chains separately.
Retained input data alone does not prove that a scan succeeded.

ReplayGroundPadCoverage slots, anchors, accepted, and powerupAccepted retain i64.
Both source and cache use append_ground_pad_chain for counts, pad accumulation and
pickup-index offsets. Weapon objects survive even if no recurrent pad is found.
Weapon objectives are excluded before family lookup; powerup families are selected
by the native powerup_ prefix from the caller's catalog. Complete scan DTOs remain
available independently of consumed publication fields.

build_facts_replay_ground_weapons shares the original item matcher. Only exact
raw taken/swapped change kinds supply candidate takes. This differs intentionally
from the separate weapon-change display adapter, which normalizes unknown kinds.
It preserves family sentinels, stable timestamp ordering, native lifetime/spatial
selection, ammo, signed frame clamping, and bounded/open end semantics. Input
positions retain their required timestamp ordering.

The native pad-scan fixture has 1,024 full two-stream comparisons spanning scan
flag combinations, signed counters and frames, caller catalogs, retained objects
without recurrent pads, and zero step. The item fixture has 1,024 input cases and
5,120 publication comparisons with raw unknown/non-UTF-8 kinds, family sentinels,
wide timestamps and signed clocks. Expectations come from the pinned native
functions; see facts-pad-scans-validation.json for completed checks.

The cache document composer must still call date_pad_pickups after pad construction
and publish the individual ground-item layer. The existing dating function does
not require recording source records; both consumers can now use these cache
builders. Full BuildFromFacts composition, vehicles/objectives, scan capture,
fallback restoration, domain reconciliation and the final source/runtime/corpus
audit remain incomplete. The deferred architecture has not begun.
