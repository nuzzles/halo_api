# Reserve-ammunition reference sequence

`export_reference.py` archives OpenSpartan/libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03` and runs unchanged
`components::parse` and the complete `ammo::reader`. Source blobs are
`6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4` (`components.rs`) and
`d1d7b803a5ec6eb72bc2b931a3db5b553dadb9e8` (`ammo.rs`)..

Nine authored updates contain twelve reserve components. They cover four
independently ranked slots among interleaved component families, zero, the
11-bit maximum 2047, omitted components, sparse updates and two repeated values.
The original parser supplies exact starts/endpoints and each slot's reserve
fields. An explicit biped-operation and preassigned-life adapter feeds those
fields to the original ammo reader. Its output preserves unknown loaded rounds
as -1 and deduplicates equal reserve values. Local observations retain all twelve
source occurrences, agreeing with the reference when reduced for comparison.

This is native component grammar plus history-algorithm evidence. It does not
test the upstream entity walker, roster attribution or physical ammunition
semantics. Local lifetime/gap/selection tests cover those projection policies
separately. Existing components-v41 fixtures supply additional primitive traces
for this field in both snapshot and delta contexts.

The disposable harness removes only its own Cargo.lock and resolves cached
dependencies offline (verified with serde_json 1.0.154 / itoa 1.0.18). Reader
code remains unchanged.

```sh
python3 src/theater/fixtures/reserves-v41/export_reference.py /path/to/libfilmshell > /tmp/reserves-reference.json
cmp src/theater/fixtures/reserves-v41/reference.json /tmp/reserves-reference.json
```
