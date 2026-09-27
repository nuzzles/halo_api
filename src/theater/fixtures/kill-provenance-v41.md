# Kill-source provenance presentation

Pinned native revision: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`halo_rust_kill_provenance_test.go.txt` calls the actual native
`killsource.Provenance.String()` in 48 cases: both published paths, five known
origins plus empty/custom/Unicode-control text, and three multiplicities. The
native fixture is `kill-provenance-v41.json`; its expected strings are not
produced by Rust. The helper is registered in generate_oracles.py.

`KillSourceProvenance` now implements Display as native origin, ` / ` and native
path (`marche` or `scan`). Origin text is not normalized or escaped, and
multiplicity does not affect this presentation. Source records and decoding are
unchanged. `native_kill_provenance_display` runs on host and is included in the
actual WASM harness.

This checks the two paths published by the parser. Arbitrary Go Path strings
are outside the existing closed Rust enum; signed negative multiplicities are
outside its nonnegative produced-count storage. The fixture includes 2^31-1 to
exercise portable formatting independence without exceeding a wasm32 usize.
This does not claim all native struct-literal domains or whole-parser parity.
