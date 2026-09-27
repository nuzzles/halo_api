# Native isolation-gap domain

Pinned reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_isolation_gap_domain_test.go.txt.

64 cases cross eight signed millisecond gaps with eight sample sequences.
Nonpositive gaps disable filtering before conversion. Positive gaps convert to
microseconds using uint64 wrapping multiplication; a wrapped-zero threshold
still filters. The 2^61 case distinguishes this from zero-as-disabled Rust
microsecond options. Inputs retain timestamps and slots; expected retained
indices come from actual native DropIsolated output. Cases include singleton,
duplicate timestamp, multiple-slot and overflow controls. Native execution passes;
Rust compares native option preservation and world-stream filtering; final validation is tracked in reference/isolation-gap-validation.json. Unsorted input
controls do not relax the native documented chronological-input precondition.
