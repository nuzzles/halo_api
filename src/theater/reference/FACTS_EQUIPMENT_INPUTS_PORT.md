# Cached equipment transitions

build_facts_replay_equipment_changes consumes exact cache records and scanner
counters. Recording and cache callers share select_equipment_changes, which
returns indices into each caller's original records. No source packet, byte or
bit coordinate is synthesized.

The cache input retains original kind bytes. Replay output retains signed
ranks/previous ranks and signed gaps. Known spawned announcements are excluded
before the origin check; every other kind except spent is normalized to taken
in the published replay, matching native behavior.
Coverage counts publication before the slot filter, and six scanner-provided
coverage counters now retain i64 on both host and WASM. Other coverage counters
are derived from actual Rust collection sizes.

The adapter returns the same ReplayEquipmentChanges type as the recording path.
ReplayEquipmentChange.gap now uses i64. Unknown and invalid-UTF-8 input kinds
remain available in FactsEquipmentChange; they do not survive the native replay
projection as kinds. This projection must not be mistaken for lossless recording
decoding. No UTF-8 normalization or rejection is applied to the cache input.

The pinned Go generator covers 1,024 cases including unknown strings, embedded
NUL, invalid UTF-8, signed extrema, negative/wide gaps, before-origin births,
missing published tracks, and zero-step behavior. Native log capture uses
json.Decoder.UseNumber so large integer counters are not rounded through float64.
See facts-equipment-inputs-validation.json for executed checks.

This completes the pure equipment-change cache adapter, not full BuildFromFacts
or the remaining equipment state/placement/episode integration. Full document
assembly, capture/fallback ordering and final parity reconciliation remain open.
