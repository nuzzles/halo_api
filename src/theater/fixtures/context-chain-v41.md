# Live recursive chain inference context

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_context_chain_test.go.txt`, registered in
`generate_oracles.py`. Expected output comes from the native inference loop with
recursive chain mode enabled, including its real inferChainArchetype calls.

128 cases produce 107 inferences, 183 records and 1,129 callbacks. Chain counters
contain 81 Immediate, 26 Deep and 13 NoConfirmation outcomes. Ambiguous and
BudgetExhausted are zero in this fixture; previous chain/budget fixtures exercise
those algorithm outcomes, not positive live-counter delivery here. The separate
`live-chain-budget-v41` fixture now supplies three positive live BudgetExhausted
cases and five NoConfirmation controls at the unchanged 200,000-trial limit.

Cases include duplicate archetypes sharing an alignment, unique archetypes,
unsupported candidates, multiple unbound records before confirmation, flush End
confirmation, padded or truncated sources, soft-bound successors and view-table
rejection. Only unique inferred archetypes establish soft bindings; shared
alignments must not fabricate an arbitrary entity identity.

Unlike single-step inference, the chain scope leaves unit-reference hooks live
while suppressing position/movement. Mobility callbacks may install replacement
capture hooks during recursion. Post-decode probes verify the resulting restored
receivers; probes are explicit test stimuli rather than recorded film actions.
Every EMP callback snapshots all five cumulative chain counters, so the oracle
also checks when outcomes become visible relative to subsequent publications.

Rust compares inference results, ordered callback values and counter snapshots,
record boundaries/masks, final counters, world slots/bindings and serialization.
Recursive NEW trials now receive the same live context; this fixture does not
supply positive NEW-trial coverage (the existing chain-NEW tests cover their
non-live traversal). Explicit repair/resync context ownership, including the
native shallow observer copy, remains separate work.
