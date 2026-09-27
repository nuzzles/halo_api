# Native translocation publication

Replay publication now consumes Film.native_translocations when its scan is
available. Previously it consumed the bounded legacy scan, which can omit native
padded-header outcomes and use different source selection/equal-time ordering.
The shared publication loop borrows the selected records and preserves their
order, slot/time filtering, paired-position gating, rounding, and coverage.

Film.native_translocations_scanned distinguishes a completed empty scan from an
older export that never stored one. Nonempty native records from earlier exports
also select the native path. Only old exports without native records/scan marker
fall back to the legacy vector. An explicitly empty native result never resurrects
legacy records. Constructors set the marker and retain each native read's source,
logical extent, padding, and whether the unit reference was fully recorded.

This is compatibility with the reference replay projection, not a semantic
assertion that padded actor bits were recorded. Consumers requiring recorded actor
identities must consult the retained native read's unit_reference_recorded result.
The deferred resolved event layer still needs the requested provenance contract.

Evidence: six focused tests cover 632 synthetic decode cases, the padding oracle,
128 native source-selection/order cases, 1,024 native publication cases, and a
constructor regression where a two-byte native event survives into publication.
The constructor regression also checks explicit-empty behavior, legacy fallback,
and Film serialization. Both legacy and native adapters use the same independently
expected publication oracle.

The six-film complete-input matrix contains zero translocations in every film.
Its Translocations comparison is an empty-case check only. Positive captured
translocation decoding/publication remains a validation gap; the existing positive
oracles are synthetic native-parser outputs, not independent semantic goldens.
