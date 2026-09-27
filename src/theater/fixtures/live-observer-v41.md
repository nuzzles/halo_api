# Live observer mutation and counter visibility

Pinned native reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_live_observer_test.go.txt`, registered in
`generate_oracles.py`. Expected results come from real native execution.

256 cases, 0–64 source bytes, initial bit offsets 0–7, alternating full-precision
profiles, and five sequential component operations per case: mobility, position,
simulation state, position, mobility. The reader carries capture slot 19.

Mobility callbacks install movement and unit-reference hooks in even cases and
remove both in odd cases. The later publications within the *same* component
must observe those changes. Every position callback takes and clears the native
absolute-index histogram. Every step records status, end offset, ordered callback
values and remaining histogram. Final histogram take/reset is also compared.

The oracle contains 512 mobility, 256 movement, 145 position and 83 unit-reference
callbacks. Of the position callbacks, 143 observe a nonempty histogram. No
reference callback appears in the disabled cases. Callback values include float32
position bits, native reference fields and source ranges, kind, and capture slot.

This positively distinguishes live publication from deferred end-of-component
publication and from hooks sampled only before dispatch. It does not establish
world accumulation, frame/record observer installation, every native counter,
concurrent observer mutation, or all 30 hook families under mutation.
