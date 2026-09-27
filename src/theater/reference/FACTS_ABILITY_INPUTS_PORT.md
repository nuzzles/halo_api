# Cached carried ability ranks

build_facts_replay_ability_reads accepts exact cache ability and keyframe ranks.
It shares build_ability_values with the recording adapter, preserving native i64
values without fabricated source records or narrowing to the wire rank domain.

Both channels keep their own source labels and equal-time ordering. Delta ranks
retain negative values; negative keyframe ranks are omitted. Reads before origin
are omitted, and native frame conversion wraps identically. The existing
publication pass rejects ranks above 27 before filtering unpublished slots, then
classifies the palette and publishes only its used labels. Carried rank identity
does not establish that the player activated that ability.

The independent 1,024-case Go oracle exercises i64 extrema, negative delta ranks,
wide positive ranks, before-origin data, u64 timestamp wrapping, source ties,
missing published tracks, overlapping palette markers and unclassified results.
It compares raw ordered readings and every field of the final publication.

This batch also validates build_facts_player_inventory using 128 native
BuildFromFacts documents. Each case encodes and decodes cache inputs before native
assembly; Rust consumes those same cache bytes, then compares inventory and its
coverage after player assembly. Empty decoded inventory yields zero coverage.

See facts-ability-inputs-validation.json for completed platform checks. Full
cache document assembly, equipment/vehicle/objective adapters, capture and
fallback ordering, and final parity reconciliation remain incomplete.
