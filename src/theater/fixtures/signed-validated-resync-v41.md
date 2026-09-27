# Signed validated resynchronization

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_signed_validated_resync_test.go.txt.

126 native cases: 44 panics, 76 unsuccessful scans and six successful landings.
The matrix crosses tiny/empty buffers, optional prefixes, signed starts and
matching/nonmatching targets. Eighteen constructed cases use a bound dead-state
component and a clean End. Three successful controls begin at negative offsets
and scan to landing zero after optional-prefix reads; another returns -32 as
its successful landing. Huge positive starts have no scan iterations; this is
not evidence for multi-gigabit recordings.

Rust compares the contextual and source-only APIs, including world preservation,
restored capture hooks after native panics, success counters and signed JSON
round trips. Host checks all 126 cases per API; WASM checks 82 nonpanicking cases
per API. Final validation is tracked in reference/signed-validated-resync-validation.json.
