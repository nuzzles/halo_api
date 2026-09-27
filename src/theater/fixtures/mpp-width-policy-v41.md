# Native MPP width admission

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_mpp_width_policy_test.go.txt.
Fixture: mpp-width-policy-v41.json.zlib (1,632 cases).

Native multiplayer-property reads cast signed lead/index settings to unsigned
widths only when reached. Zero-width reads are valid; widths above 64 retain the
last 64 bits numerically. Rust preserves the discarded source prefix separately.
The native contextual readers now carry the original MPP settings past the
legacy encoding adapter and admit each width at its actual read location.
The public bounded default-state helper retains its checked width contract.

The native oracle varies lead/index separately over eight bit alignments, using
structured ti36 defaults, biped defaults, disabled defaults, dedicated non-MPP
defaults, Delta, Delete, End, and invalid New archetypes. Widths are -128, -1,
i64::MIN, 0, 1, 5, 32, 33, 64, 65, 130, 4097, 2^32, and i64::MAX.
Consumed cases use only 0 through 4097: 288 consumed-width cases and 1,344 unused
cases retain 1,152 ordered MPP callbacks. Consumed negative values are deliberately
not executed in Go because their unsigned conversion requests impractical reads.

Rust checks ordered records, masks, callbacks, completion, cursor endpoints,
complete world state, original profiles, retained source fields and full view
JSON roundtrips. Additional Rust-domain controls reject consumed -1 widths at
bit 17 (lead) or bit 59 (index), preserving earlier observations and explicit
width-refusal diagnostics. Actual wasm32 execution also checks consumed 2^32
refusals. These controls are not native negative-width comparisons. The signed
cursor and platform address limits remain explicit implementation limitations.

The WASM runner includes all 1,632 native cases, bringing its total to 5,616
native comparisons plus 80 existing domain-refusal cases; the additional MPP
refusal controls are outside that fixture count. Native generation passed in
0.432s. The focused Rust comparison passed in 0.22s, and actual WASM execution
passed after an 11.09s build. No captured-film comparison was rerun here.

ID width admission, negative/overflowed cursor continuation and broader native
output reconciliation remain open. NEXT_PHASE.md remains deferred.
