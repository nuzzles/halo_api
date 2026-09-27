# Live chain-inference budget exhaustion

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Native generator: `../reference/halo_rust_live_chain_budget_test.go.txt`, registered
in `generate_oracles.py`. Expected values come from the actual native
`decodeInferLoop` and its live observer, without lowering the 200,000-trial budget.

Eight cases combine 1/4/16/50 calibrated candidate archetypes with 16/128 repeated
32-bit records. Each record has an unbound Delta header and one component; widths
8 + 32*i create distinct aligned successors. No terminal End or hard-bound
confirmation is provided. Three cases exhaust the real budget (16/128, 50/16,
50/128); five terminate without confirmation. View-table admission is disabled
explicitly so the native chain path runs. The registry/records are synthetic v41
inputs, not a captured gameplay sequence.

Rust compares all five observer outcome counters, frame diagnostics, end offset,
End status, inferred count and returned failed Delta headers (not silently
discarded records). Empty world bindings and JSON
roundtrip are checked. This establishes positive live BudgetExhausted delivery;
it does not exhaust callback mutation or other inference/recovery contracts.
