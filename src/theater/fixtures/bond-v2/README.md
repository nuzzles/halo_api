# Bond v2 and game-options fixtures

`wire_cases.py` independently authors 130 complete CBv2 structures. Coverage
includes all value types 2–18, extended/redundant field IDs and integer encodings,
duplicate/out-of-order fields, base terminators, packed and explicit list/set
counts, map key/value combinations, raw invalid UTF-8/unpaired UTF-16, floating
NaN/negative-zero words and a 4,096-byte packed array. No game identity data is
included. `cases.json` is generated independently of the Rust parser.

`export_reference.py` archives OpenSpartan/libfilmshell commit
`18067987ebc3628ad499c819c6aee814692df43b` and wraps its original `variant::value`
reader to trace wire type and start/end offsets. It changes no reader branches or
return values. The 130 original endpoints and nested-value traces are retained
in `reference.json`. These are valid-input comparisons, not evidence that the
upstream reader rejects malformed inputs or preserves all raw syntax.

`game_options_cases.py` authors eight envelopes covering both optional gates,
both Bond messages, short/maximum NUL-terminated strings and a distinctive
following-player marker. Its grammar comes from JGtm/LevelUp commit
`413971bcc485be53a2fa8cf229e170539a5146f8`, `film_variante_de_partie.go`:
game writer `FUN_1407ec560`, reader `FUN_1407ee138`, HI_1_13_0. These envelopes are
not original libfilmshell traces: that implementation searches for anchors.

Generic CBv2 rules were checked against Microsoft Bond's
[`compact_binary.h`](https://github.com/microsoft/bond/blob/e27170b0f768037087e5d3103b9e8035c0968626/cpp/inc/bond/protocol/compact_binary.h).
Its declared structure length includes the final stop byte and excludes the
length varint. Field IDs, container counts and scalar encodings are preserved
without importing the selective game projections/defaults of either reference.

Parser tests reconstruct every consumed source bit and round-trip the syntax
through serde. Small valid structures are truncated at every bit; large packed
cases use endpoints and representative cuts. Separate negatives cover malformed
lengths/types, integer overflow, enclosing bounds and depth/value-budget refusal.
Envelope tests verify shifts 0/1/3/7, field truncation and missing terminators.

The ignored `captured_game_options_boundaries` test independently compares the
sequential endpoint with the legacy runtime pattern search on all 32 retained
films. It removes the canonical header from a cloned film to force the old
search, then checks identical first-record offsets and projected player slots.
It does not prove canonical player semantics, unknown identification-block
internals or unobserved build layouts. Original LevelUp fixtures are unchanged.

Reproduction from the repository root:

```sh
python3 src/theater/fixtures/bond-v2/wire_cases.py
python3 src/theater/fixtures/bond-v2/game_options_cases.py
python3 src/theater/fixtures/bond-v2/export_reference.py /path/to/libfilmshell > /tmp/bond-reference.json
cmp src/theater/fixtures/bond-v2/reference.json /tmp/bond-reference.json
cargo test --offline --all-features --lib bootstrap::bond
cargo test --offline --all-features --lib bootstrap::game_options
cargo test --offline --all-features --lib captured_game_options_boundaries -- --ignored --nocapture
```

The exporter requires Python with `tarfile` data filters and an installed stable
Rust toolchain. `docs/fixtures.json` pins all three JSON outputs by length/hash.

## Variant schema projection and conditional events

`variant_cases.py` independently authors 77 full game-options envelopes with
selected variant instances and expected engine/playback fields. They exercise all
four optional instance lists; signed engine values; explicit/omitted booleans;
nonzero boolean encodings; missing parents; duplicate/conflicting instances and
fields; wrong types; and inherited IDs before a base terminator. Omitted defaults
apply only in a complete owning structure. No schema projection rewrites the AST.

`export_variants.py` executes the original LevelUp variant walker from commit
`413971bcc485be53a2fa8cf229e170539a5146f8`, checking Git blob
`13179ea7149628e9c31c6fe827bbaec005111ab1` before execution. It changes only
package/type wiring and supplies bounded bit/string primitives. Its walker,
branch and projection bodies remain unchanged. The 66 comparable valid results
are retained in `variants-reference.json`; negative/inheritance cases are
separate local safeguards, not claims of upstream agreement.

Writer/schema provenance is that pin's `film_variante_de_partie.go` and
[`LOT_VA_V3.md`](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/.ai/V7.5/film_re/campagne_grammaire_2026-10-01/LOT_VA_V3.md):
engine default 0 (`FUN_14014dce0`), killcam default false (`FUN_14014cc60`) and
play-of-the-game default false (`FUN_140150510`). The corresponding event guards
are mapped in `vue_a_charges_execution.go`, writers `FUN_142f18fd0` (kill tail)
and `FUN_142efa2a8` / `FUN_141f86118` (teleport orientation).

The local 32-film inventory reports engine 2, killcam false and play-of-the-game
true throughout. Consequently it cannot prove the kill tail absent: runtime
feature switches are unrecorded. Canonical parsing retains the base fields and
stops; replay labels its disabled-feature assumption. Tests exercise all 27
known/unknown engine/boolean combinations, runtime/canonical behavior, truncated
base fields, misleading following bits and actual Film continuation/refusal.
The separate ignored replay test validates source-linked diagnostics on retained
Octagon and Recharge films, without changing their canonical reads.

```sh
python3 src/theater/fixtures/bond-v2/variant_cases.py
python3 src/theater/fixtures/bond-v2/export_variants.py /path/to/pinned/film_variante_de_partie.go > /tmp/variants-reference.json
cmp src/theater/fixtures/bond-v2/variants-reference.json /tmp/variants-reference.json
cargo test --offline --all-features --lib context::events::variant
cargo test --offline --all-features --lib captured_variant_context -- --ignored --nocapture
cargo test --offline --all-features --lib captured_replay_reports_event_assumptions -- --ignored --nocapture
```

The variant exporter also requires Go; `GO_BIN` can select a local toolchain.
Both variant JSON files are additionally pinned in the fixture manifest.
