# Cached ability impulses and charges

ReplayAbilityRankIndex now retains (timestamp, i64 rank) values and returns
Option<i64>. Source and cache constructors share the lookup: first supplied life
within five-second tolerance, latest prior rank at/after its start, last reading
on timestamp ties, and no extra life-end clamp. Signed wrapping at the native
lifetime boundary remains explicit. A negative recorded cache rank is a value,
not automatically an absent rank.

FactsReplayAbilityContext is the exact-cache specialization of the existing
ReplayAbilityContext. Both contexts feed the same internal attribution passes.
Caller palette, measured families, publication slots and clock retain their roles;
no synthetic source records are created. Cache charge emplacement/low fields and
impulse predicted flags stay in their input DTOs but do not change native replay
publication, which does not consume them.

build_facts_replay_ability_charges retains native i64 charge counts. It publishes
each accepted reading in original order without deriving usages from differences.
build_facts_replay_ability_impulses and fold_facts_replay_ability_impulses share
native retransmission folding and rank attribution with recording inputs. The six
scanner-provided impulse coverage counters now retain i64 on host and WASM.
Coverage computed by these builders is not a claim the scan ran: full document
composition must still honor the native stats.scanned attachment gates.

The 1,024-case pinned native oracle extends the existing ability-action fixture
with signed rank/charge/scanner extrema, negative values, wide ignored fields,
lifetime-boundary extremes, null/empty configuration and timestamp wrapping. It
compares complete charge/impulse publications, folded episodes and 32 rank queries
per case. Palettes include wide and negative rank keys so those values must
actually resolve, rather than merely surviving an input parser.

Results are tracked in facts-ability-actions-validation.json. Full cache document
integration, movement/placement/vehicle/objective adapters, capture/fallback order
and the final v41 parity audit remain pending. This is not the deferred canonical
Film/ResolvedFilm architecture.
