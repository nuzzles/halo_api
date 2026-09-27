# Signed traversal continuation

Reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The native helper `halo_rust_signed_continuation_test.go.txt` generates 96 cases
through actual TryDeltaAt. A second call through the same native reader, header
and decodeDelta functions checks equal successful traces/endpoints and equal
panic status, while exposing the live cursor on panic. Calibrated replacements
and unsupported-component stubs are both covered. An optional final EMP read
distinguishes a cursor adjustment from a subsequent access to source bits.

The matrix crosses zero, ordinary backward skips, exact-zero boundaries,
negative boundaries and signed overflow. Its successful results contain genuine
signed CompResult.StartBit and EntityTrace.EndBit values, not unsigned bit
patterns. Repeated generation produced byte-identical output.

Native outcomes:

- 78 successful records and 18 panics.
- 34 successful records contain a negative component start.
- 22 successful records finish with a negative cursor.
- 18 successful records contain a negative component start and return to a
  nonnegative final cursor. These counts overlap rather than partitioning cases.

`native_signed_component_continuation` is a passing, nonignored acceptance test:
96 cases match, including 78 successful records and 18 native panics. It compares
status, final cursor for successful records, ID, archetype, mask and ordered
component indices/names/starts/status. The full native records and panic endpoints
remain in the fixture. The separate signed-component-cursor fixture checks the
stateful direct reader's post-panic cursor and reset/read recovery. Whole-frame
post-panic cursor, arbitrary payload types and callbacks remain separate gates.

Example: after a two-component Delta header/mask reaches bit 31, the first
calibrated skip of -32 reaches -1. The second calibrated skip of 32 reaches 31.
Native succeeds and reports the second component's StartBit as -1. Rust now preserves both skips and the intermediate signed coordinate. No source read occurs between
the two native skips; treating the first negative cursor as truncation is wrong.

Run the gate:

```sh
cargo test --lib native_signed_component_continuation
```

Host logs: `/private/tmp/halo-signed-migration-focused.log` and
`/private/tmp/halo-signed-migration-suite.log`. The full suite passed 596 tests,
with zero failures and 49 ignored. The WASM harness includes four public-frame
cases from this fixture; the private 96-case gate is host-only.
