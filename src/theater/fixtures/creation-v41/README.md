# Creation reader differential fixtures

Reference: OpenSpartan/libfilmshell
`18067987ebc3628ad499c819c6aee814692df43b`.

`wire_cases.py` independently authors 436 synthetic creation bodies: 96 projectile
cases, 32 vehicle cases, 48 invalid-count cases for other common-object callers,
72 biped cases, 64 manager-property cases, 52 other manager cases and 72
item/device, rigid-body, equipment and weapon cases. Every nonempty creation
reader supported by the pinned upstream has cases;
this is selected branch coverage, not exhaustive format conformance.
Cases cover optional targets and handle categories/generations, snapshot raw
words versus tick quantization, versions, common arrays, relative positions,
vehicle reference lists and illegal common counts 5–7. Biped cases include the
present post-common reference, position and player gates across nine version
forms. Manager cases cover every property tag, both scalar and masked arrays,
and interleaved tag/value fields. Remaining manager cases exercise version and
handle gates. Item-family cases include absent/present references, equipment
player/ability fields, and weapon tag lists with 0, 1 and 15 entries. Snapshot
position words include negative zero and a NaN payload; no float normalization is permitted.

`export_reference.py` archives the pinned upstream commit into a disposable
checkout, instruments `events::Bits::bits` and direct position/property-value
skips in `creation.rs`, and calls the original `creation::read` or
`creation::manager_end`. It changes no branches or return values. Both authored endpoints and completion/refusal status must
agree before an export succeeds. Traces and expectations never come from
halo_api. No real film/player data is included.

Three tests compare upstream primitive boundaries, contiguous raw fields and
serde retention; truncate every case at every bit without usable byte padding;
and exercise defaults through NEW records and keyframes. A following independent
`high-frequency` counter detects drift across the creation/component boundary.
Invalid counts must retain their prefix and stop before reading that counter.
Snapshot readers also run without position context to prove raw words do not
require map quantization widths. These checks do not prove real-film occurrence,
semantic meaning for opaque fields, or full format conformance.

Reproduce from the repository root:

```sh
python3 src/theater/fixtures/creation-v41/wire_cases.py
python3 src/theater/fixtures/creation-v41/export_reference.py /path/to/libfilmshell > /tmp/creation-reference.json
cargo test --offline --lib components::defaults::tests
```

Review the exported upstream traces before replacing `reference.json`. The
exporter requires the installed stable Rust toolchain and cached offline upstream
dependencies. Original LevelUp oracle files remain unchanged.
## Property-width evidence update

The 32 historical masked-property bodies in this fixture use libfilmshell's 32-bit
fallback for tags 12–15. These bytes/oracles remain unchanged as comparison
evidence. Canonical component/default readers now use the instruction-backed
four-bit enum branch for tags 11–15. Replacement native-body traces and following-
component/truncation tests are in [property-v41](../property-v41/README.md).
This supersedes earlier claims that upstream agreement establishes these widths.
