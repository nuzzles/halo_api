# Cached placement origin and disappearance evidence

ReplayEquipmentOriginIndex::from_facts indexes cached spawn and equipment-change
records without constructing source packets. Source and cache constructors share
one tuple-based reducer. It counts all supplied spawn events, but only valid
spawned references enter the index. Life generations retain u32; no conversion
through the source event's u8 generation occurs. Only the exact native taken kind
contributes written-taken evidence. Unknown kinds remain ignored for attribution,
even though the separate equipment-change publication layer normalizes them.

classify_facts uses the same pure classifier as source placements: matching spawn
event, manifest-confirmed spawned piece, then written death/taken evidence near
an owner. Contradictory death/taken evidence and absent owners retain native causes.
Caller-provided IdentityLife and owner inputs retain their existing contract.

replay_facts_equipment_ends shares the native census-bound reducer with source
placements. It keeps placement order, full life keys, lifetime reuse sorting,
wrapping film-end/frame arithmetic, and the distinction between bounded seen ends
and open ends. The last observed movement is not a disappearance timestamp.
Cache points and endpoint payload remain in FactsPlacement; these evidence
builders do not consume them.

halo_rust_facts_placement_evidence_test.go.txt extends the pinned native oracle to
1,024 cases and 32,768 origin probes. It includes u32 generations above 255,
invalid spawned references, unknown/non-UTF-8 change kinds, signed point counts,
wide timestamps and census/lifetime cases. Expectations come directly from the
native classifier and placementEnds. Source owner/publication regressions remain
separate checks. See facts-placement-evidence-validation.json for actual results.

This completes two placement evidence adapters only. Full cached placement publication now uses these adapters; see
FACTS_PLACEMENT_INPUTS_PORT.md for its own validation scope. Outer document
fallback attachment/restoration remains pending. It does not complete BuildFromFacts or v41 parity.
