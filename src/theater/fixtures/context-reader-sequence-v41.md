# Native context reader sequence oracle

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_context_reader_sequence_test.go.txt`, registered
in `generate_oracles.py`. Expectations come from native Go execution.

256 deterministic inputs, lengths 0–24 bytes, initial offsets 0–7, eight
component operations each. Seven component names rotate across first position:
EMP timer, rounds inventory, desired weapon set, mobility, crouch, simulation
state, and control context. Each case finishes with another EMP timer read.
Native profiles vary full precision, mobility body and simulation-complete flags.

Readers and scan frames are created before replacing the context profile;
existing readers retain the prior scalar profile. EMP hooks are replaced while
the reader remains alive. Both direct and frame readers are checked for status
and end bit after every operation, including zero-tail reads. Actual ordered
callbacks are recorded after every operation; frame readers inherit no observer.

This is distinct from reader-sequence-v41, which covers 1,064 per-attempt context
swap/suppression/restoration cases. It does not establish whole-Film integration,
world accumulation, every profile field, every hook, or callback-time mutation
inside a single component. Rust-only regressions separately check shared hook
reentrancy/replacement/restoration and cumulative histogram snapshot/reset.
