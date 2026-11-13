# Weapon charge and heat reference

The exporter archives OpenSpartan/libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`, verifies its source blob IDs and
runs unchanged `components::parse`, `dequantize` and the complete `ammo::reader`.
Source blobs are `components.rs` 6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4
and `ammo.rs` d1d7b803a5ec6eb72bc2b931a3db5b553dadb9e8.

All 4,096 twelve-bit charge codes and all 128 seven-bit heat codes are parsed
from authored native component bytes. Heat parsing additionally exercises all
four combinations of state bits for every code (512 bodies). Each code retains
its exact reference f64 bits and rounded percentage. This is complete coverage
of these scalar domains, not every writer version, layout or engine f32 result.

Nine updates carry 19 components across four independently ranked ammo and heat
slots, interleaved with type-info rows. The fixture retains native bytes, exact
component starts/end, parsed fractions and the original deduplicated history.
It covers endpoints, interior bins, separate heat/charge updates, omissions,
repeated readings, open zero words, closed gates and all heat flag combinations.
An explicit preassigned-life adapter isolates upstream's value/history behavior;
it does not establish native ownership, packet boundaries or avatar lifetimes.

Local tests feed the same native bytes through the existing component readers,
then compare source-linked weapon observations with the original conversions.
Closed charge gates deliberately keep native zero semantics; upstream represents
them as an absent fraction. The local source view also retains repeated readings
that upstream deduplicates. Heat flags remain raw bits without adopted gameplay
labels, and charge is not inverted to remaining battery by this API.

Only the disposable harness Cargo.lock is removed to resolve cached dependencies
offline. Original reference reader source is unchanged. Reproduce with:

```sh
python3 src/theater/fixtures/weapon-quantization-v41/export_reference.py /path/to/libfilmshell > /tmp/weapon-quantization.json
cmp src/theater/fixtures/weapon-quantization-v41/reference.json /tmp/weapon-quantization.json
```
