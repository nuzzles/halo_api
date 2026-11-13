# Property variants: instruction-backed widths and captured vectors

The native masked-property variant's tags **11–15 all carry four payload bits**.
The parser retains the original four-bit tag and enum code; it does not collapse
12–15 into 11. Scalar tags 7–15 and masked tags 0–6 consume no payload. The shared
helper applies to components and the full default-state body.

This corrects an earlier parity-driven change to 32 bits for tags 12–15. Pinned
libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03` `creation::property_bits`
uses a 32-bit fallback for these tags; its older component/creation oracles in
`../components-v41` and `../creation-v41` retain that behavior unchanged. Four
component cases assert the explicit 36-versus-8-bit disagreement. Thirty-two
historical masked default bodies use the longer layout and are excluded from
canonical boundary tests in favor of independently traced native bodies here.
They remain in the original oracle with explicit count/endpoint checks.

Evidence is pinned to LevelUp commit
[`413971bcc485be53a2fa8cf229e170539a5146f8`](https://github.com/JGtm/LevelUp/tree/413971bcc485be53a2fa8cf229e170539a5146f8):

- [components_managed_property.go](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar/components_managed_property.go), blob `8f7093171e8b48f7feb8c9f85258dd0cce24dee7`.
- [default_state_arch.go](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar/default_state_arch.go), blob `53f0146423b00f404c41791e5ca00f4eb00a6ea9`.
- [ti13_vecteurs_test.go](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar/ti13_vecteurs_test.go), blob `926944ccbe1dca6abdcc14843a774d4fa8e51e70`.
- [LOTCBIS_PHASE0.md](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/.ai/V7.5/replay2d/registre_film/LOTCBIS_PHASE0.md), blob `959bda104cff5524f8d6492463ee324ddc80fd1e`, sections2.2–2.4: switch default >=11 reaches `FUN_141fce2f0 -> FUN_1407ef804`, four bits then subtract one. The masked index comes from the component descriptor; the full-state reader loops 32 variants.

The blobs were checked against the pinned GitHub tree. `export_source.py` verifies
source Git hashes and executes the original `managedPropertyPayloadBits`,
`lireVariantDePropriete`, `consumeVersionPrefix` and `consumeDefaultStateTI13`
functions unchanged. Only bounded bit primitives are provided by the harness.
The original captured-vector struct and arrays are extracted unchanged and their
expected tag, width, payload and endpoint are independently verified by Go.

`wire_cases.py` authors 128 component bodies (both domains, all tags, zero/max
values, snapshot/DELTA) and 128 default bodies (both domains, all rotated tags,
version gate absent/present, snapshot/NEW). The oracle checks every primitive
boundary. Rust tests cover eight alignments, every truncation, exact source bits,
serde, and native NEW/DELTA/keyframe following-component endpoints. Authored
fixtures use explicit internal parser configuration, not new public build or
bootstrap admission.

The 33 original captured vectors retain film/chunk/packet/bit/slot/row provenance,
raw 64-bit windows and each original `chained` flag. Five vectors cover tags 12–15;
all five have the source's chained flag. Scalar tags2 and6 have unchained evidence
and are not presented as corpus-confirmed widths. The before-fix native test
consumed 36 rather than 8 bits for captured tag 12 (`01e1f945`, chunk 22, packet 590,
bit 435), consuming 28 bits of following source. The new test preserves those bits.

The same pinned source documents unresolved class 13 NEW *envelope* context in
`default_state_ti13_neuf.go`; this does not justify its historical tag-only fallback.
This change establishes variant widths and full-state body traversal, not every
NEW envelope, writer context, build, field meaning or hill-score attribution.

Reproduce:

```sh
python3 src/theater/fixtures/property-v41/wire_cases.py
GO_BIN=/path/to/go python3 src/theater/fixtures/property-v41/export_source.py \
  /path/to/pinned/grammar /path/to/ti13_vecteurs_test.go > /tmp/property-reference.json
cmp /tmp/property-reference.json src/theater/fixtures/property-v41/source-reference.json
cargo test --config build.incremental=false property
```

The exporter defaults to a temporary Go build cache. Captured source claims and
canonical semantics remain distinct.
