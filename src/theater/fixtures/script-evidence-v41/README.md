# Script filename and mode evidence

`cases.py` independently authors 272 Bond variant trees: 256 combinations of
the eight mode-trigger filenames, plus empty/missing/malformed/inherited fields,
duplicates, mixed-case/repeated names, ASCII search restrictions, following bytes,
blob-end matches, and a filename outside the selected blob. Local tests use the
public Film/runtime entrypoints and check exact source spans and unchanged Film.

`export_reference.py` verifies four Git blobs at libfilmshell
`f050371a6cda83b9887eeb83a1884336fce28e53`: `src/scripts.rs`,
`src/model/variant.rs`, `src/game/options.rs`, and `src/game/options_data.rs`.
It compiles the unchanged modules with a harness. Only TagId is stubbed as the
model's unused scalar wrapper. Original script discovery and mode classification
run unchanged. The blob adapter supplies the original three-bit alignment and
two trailing zero bytes, making its exclusive search endpoint observable at the
authored blob's end. This does not insert bytes into the local canonical source.

The local result preserves exact spellings/occurrences and labels matches as
candidates, while the original API returns a lowercase set. `reference_choice`
matches upstream's precedence across all 256 combinations; all matching rules
remain available locally. Native Other maps to an absent classification, not PvP.
Local checks cover ambiguity, scope and source integrity separately from native
name-set agreement. Neither search certifies script declarations or loading.

```sh
python3 src/theater/fixtures/script-evidence-v41/cases.py
python3 src/theater/fixtures/script-evidence-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/script-reference.json
cmp src/theater/fixtures/script-evidence-v41/reference.json /tmp/script-reference.json
cargo test --offline --lib runtime::variant::scripts
```

The existing variant corpus test optionally exports script blobs, local names,
mode choices, occurrences and registry paths. The oracle then runs independently
against both each blob and the original whole-registry bytes. All 32 films match
in both scopes, with 3,336 per-film distinct names and 4,463 source occurrences.
The verifier asserts equality rather than silently dropping differences.
Captured modes cover Slayer, Strongholds and Oddball only. Canonical script
archive/bytecode grammar and actual script-loading semantics remain open.

```sh
HALO_VARIANT_EXPORT=/tmp/script-captured.json cargo test --offline --lib captured_variant_views_preserve_source_entries -- --ignored --nocapture
python3 src/theater/fixtures/script-evidence-v41/export_reference.py /path/to/pinned/libfilmshell /tmp/script-captured.json > /tmp/script-native.json
python3 src/theater/fixtures/script-evidence-v41/verify_captured.py /tmp/script-captured.json /tmp/script-native.json
```
