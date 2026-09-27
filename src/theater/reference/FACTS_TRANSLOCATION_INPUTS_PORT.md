# Cached translocation publication

build_facts_replay_translocations shares the existing recording/native-record
publication reduction. That reduction consumes time, slot and optional endpoint
pairs, rather than requiring synthetic FilmPacket or TranslocatorEvent records.
The cache has_positions flag exclusively controls whether endpoints are published;
inactive from/to payloads remain in FactsTranslocation. Source adapters retain
their existing positions() completeness rules.

Projection keeps order, drops before-origin and unpublished-slot events, retains
unpositioned events and rounds available coordinates to centimeters. Coverage
continues to distinguish positioned from merely published translocations.

The 1,024-case pinned replay-translocations fixture is reused for a separate cache
comparison consuming native input values directly. The recording and cache tests
both pass. Platform results are in facts-translocation-inputs-validation.json.
The WASM harness now supports --test-prefix for targeted incremental comparisons;
omitting it still executes the full registered suite, and unmatched prefixes fail.

This native replay projection does not establish a canonical semantic-event model.
Full cache document assembly and the final v41 parity audit remain incomplete.
