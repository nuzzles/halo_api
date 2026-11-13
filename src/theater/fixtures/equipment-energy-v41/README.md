# Equipment energy and delay oracle

The exporter archives OpenSpartan/libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`, adds a test harness, and executes
unchanged `components::parse` / `dequantize` and the complete `ammo::reader`.
Source blobs: `components.rs` 6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4 and
`ammo.rs` d1d7b803a5ec6eb72bc2b931a3db5b553dadb9e8..

Twenty-six updates carry 49 energy/delay components. Cases cover quantizer
endpoints, nearby interior codes, the midpoint, delay zero/1023, independent
field omission, countdown-like steps and repeated zeros. JSON retains source
bytes and endpoints, exact reference f64 bits, rounded percentages and the
original deduplicated equipment series. Local native-packet tests compare these
values and retain all source updates, including steps upstream suppresses.

Explicit adapters supply a biped generation, preassigned life and Unit parent
to the original ammo reader. They isolate its value/history behavior; they do
not prove native attachment bytes, ownership, clock units or lifetimes. Separate
local tests serialize native parent/controller/roster packets to verify current
holder policies without adopting upstream's whole-life owner lookup.

The disposable directory's Cargo.lock is removed to resolve cached dependencies
offline (verified with serde_json 1.0.154 and itoa 1.0.18). Reader source is
unchanged. Reproduce with a pinned checkout:

```sh
python3 src/theater/fixtures/equipment-energy-v41/export_reference.py /path/to/libfilmshell > /tmp/equipment-reference.json
cmp src/theater/fixtures/equipment-energy-v41/reference.json /tmp/equipment-reference.json
```
