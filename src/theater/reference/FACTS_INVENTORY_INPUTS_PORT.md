# Cached inventory publication

build_facts_replay_inventory consumes exact FactsKeyframeInventory fields. It uses
the same borrowed internal InventoryInput reduction as build_replay_inventory;
neither path fabricates source coordinates. ReplayInventory.gs and d are now
Option<i64>, and cand is i64, preserving all native signed machine-word values on
WASM as well as the host. Negative selection/drawn-slot values remain omitted;
negative candidate counts remain present, matching native publication.

The reduction preserves every post-origin read and equal-frame ordering, counts
pre-origin drops, projects only the first two ammo slots, and distinguishes absent
read flags from measured zero. Empty reads begin as unknown; death attribution
remains a separate identity-based corroboration pass. No extra recorded action is
inferred by this adapter.

The pinned native generator extends the existing inventory publication oracle
with signed extrema, negatives and values beyond 32-bit range for all three
fields. It compares 1,024 sets of published reads, all coverage fields, death
attribution and native logs. WASM compares data/coverage/deaths; tracing output is
checked on the host. Results are in facts-inventory-inputs-validation.json.

build_facts_player_inventory now applies the same track filtering, temporal death
attribution and coverage assembly as the recording wrapper. A separate 128-case
native BuildFromFacts oracle validates inventory publication and coverage after
cache decoding and player assembly. Its platform status is tracked separately in
facts-ability-inputs-validation.json. Complete BuildFromFacts document assembly,
scan capture and the final all-data v41 parity audit remain pending.

The next composition step must preserve the native coverage gate: unavailable
recording inventory (nil) omits coverage, while a supplied empty scan publishes
zero coverage. Native decodeInventaire allocates a non-nil slice even for zero
cached records, so the decoded cache path publishes zero coverage. The cache
encoding loses the original scan's nil/empty distinction; do not claim it can
reconstruct that distinction from cache bytes. The existing source path still
retains missing scanner input separately.
