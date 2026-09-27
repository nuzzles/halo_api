# Native damage projection and incompatible body grammars

Reference commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
The generators are `../reference/halo_rust_native_damage_test.go.txt` and
`../reference/halo_rust_damage_layout_audit_test.go.txt`.

The damage generator uses the pinned packet walker, `lot1RefDom1`, `lot1RefDom`
and `lot1DecodeDamageAftermath`, without a reconstructed world, slot-base
selection, weapon pairing or inferred actors. It records all selected type-0,
minimum-two-byte, first-byte-0xc0 attempts, including type-1 refusals.

The 32-film corpus has 2,341 selected attempts: 1,873 accepted projections and
468 refusals. None of these captured projections reads a synthetic tail. The
separate 2,048-case `weapon-hit-scan-v41.json.zlib` oracle covers padded reads.

The Rust native entry compares every captured source file/packet ordinal,
payload offset/size, wire timestamp, projection, secondary magnitude, body victim
and endpoint. Its ordered field trace is independently checked against payload
bits, including skipped opaque fields, reference generations and per-field
synthetic padding. The legacy projection shares the same implementation with
trace collection disabled.

The independent layout audit compares the direct damage reader with the pinned
generic `evStep` on the same captured bytes. Among accepted damage projections:

| Result | Count |
| --- | ---: |
| Matching body endpoints | 1,062 |
| Different body endpoints | 811 |
| Different endpoints but generic list reaches a terminator | 714 |

The native implementations disagree on the polarity of the optional five-bit
field after the source tag: `internal/facts/killsource/eventbody.go:86` reads it
when the gate is zero; `internal/grammar/weapon_hits_decode.go:81` reads it when
the gate is one. Coincident endpoints do not make their field layouts identical.
This audit does not establish which implementation matches the native recording.

Consequently, continuations through **any** code-0 body retain their generic
frame attempt with `IsolatedConflictingDamageGrammar` policy. Their binding
changes do not propagate to later packets. Other supported event lists continue
under the existing policy. Neither damage projection substitutes its endpoint
for the generic list endpoint. The raw bytes and both interpretations remain
available. This is uncertainty preservation, not a full-decoding claim.

Install the generators as extra tests in their corresponding `grammar` and
`facts/killsource` packages in the pinned film root. From `apps/go-api`, run:

```sh
GOCACHE=/private/tmp/halo-go-cache GOPATH=/private/tmp/halo-go-path \
  /private/tmp/halo-go-runtime/go/bin/go test \
  ./internal/games/halo_infinite/film/internal/grammar \
  -run '^TestHaloRustNativeDamage$' -count=1 -v
GOCACHE=/private/tmp/halo-go-cache GOPATH=/private/tmp/halo-go-path \
  /private/tmp/halo-go-runtime/go/bin/go test \
  ./internal/games/halo_infinite/film/internal/facts/killsource \
  -run '^TestHaloRustDamageLayoutAudit$' -count=1 -v
```

Compress `/private/tmp/halo-native-damage.json` with zlib level 9 to regenerate
the adjacent fixture. The second test produces
`/private/tmp/halo-native-damage-layout-audit.json`; its retained copy is
`../reference/native-damage-layout-audit.json`. Neither generator reads Rust output.

Rust validation:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_ -- --include-ignored --nocapture
```
