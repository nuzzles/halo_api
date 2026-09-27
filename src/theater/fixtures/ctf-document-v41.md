# Complete captured v41 CTF document

Source: JGtm/LevelUp `feat/v75`, pinned commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81` (MIT, license in reference/).
The original eight compressed chunks and fixture metadata are under
`apps/go-api/internal/api/wire/testdata/film_e2e/c0a82e88/`.
Match: `c0a82e88-7b3b-419c-a984-13385af99259`, Husky Raid:CTF, Corpo.
The raw bootstrap declares major 41, format 27. Chunk metadata spans 119,156 ms.

`ctf-document-v41.json.zlib` retains every decompressed source byte, original
chunk metadata, each original compressed chunk's SHA-256, the API match facts,
and the complete native replay document. No packet prefixes or selected records
replace the complete recording. The Go harness removes exactly one zlib layer
and calls the pinned native scan and document assembly. All quantities travel
through JSON without conversion to float64 by the fixture wrapper.

The comparison enables the flag consumer. Player kills/deaths/assists from the
API fixture are explicitly supplied to both parsers' identity-completion and
score paths. They are external evidence, not facts claimed to be decoded from
the recording. Team scores are withheld from both builders and independently
check the three decoded captures against 3+0. Native output also has three closed
carry spans, four flag-object lifetimes, and two observed/confirmed carrier marks.
Those latter quantities and actor/timestamp assignments are reference-parser
comparisons, not independently annotated action-time ground truth.

This case uses map quantization but supplies no flag-spawn geometry; native
unknown-team/absent-home behavior is part of the comparison. It does not validate
map geometry, browser playback, or the future resolved-model contract. The companion `ctf-decoded-kill-document-v41.json.zlib` exercises decoded-kill
enrichment with the native calibrated profile inherited by subsequent scans.

Regenerate with `reference/generate_oracles.py`; its CTF harness is
`reference/halo_rust_ctf_document_test.go.txt`. Run the self-contained comparison:

```
cargo test --lib theater::replay_document_film::tests::complete_ctf_document -- --ignored --nocapture
```

The explicit invocation keeps this expensive full-film comparison outside the
fast default suite; it does not require an ignored local corpus directory.

## Decoded-kill companion

The companion fixture also retains the complete native kill-source result. Its
comparison checks all kill/assist readings, health and pass statistics, calibration
summary, publication gate, and public roster fields, then compares every replay
document field. The inherited profile object remains in the oracle; its field-level
comparison is covered by separate calibration/context tests, while this case checks
its effect on the subsequent full-film decode.

For this film the native result publishes 20 kills (16 walking, four direct scan).
The sum of API player kills is independently checked against both the retained
kill result and publication counts. Those totals are not supplied to the kill
source decoder; the later identity/score stage still receives the explicitly
noted match lines. Capture assertions remain active in the enriched case.

```
cargo test --lib theater::replay_document_film::tests::complete_ctf_decoded_kill_document -- --ignored --nocapture
```

## Catalog flag geometry companion

`ctf-geometry-document-v41.json.zlib` supplies the fixture's API map ID
`8be179f7-8940-4868-b881-44cad1ca8711` and the pinned objective catalog to the
decoded-kill path. The catalog SHA-256 is retained in `map_catalog_sha256`:
`a27eceaa880ee2529bcbeca28de5c1235da3a3d3fd37793f2fc9d65c744784eb`.
The Rust embedded catalog and native file were checked byte-for-byte equal.

The Go harness projects `PointsOfRole(flag_spawn)` using the production
`replaybuild.flagSpawns` rules, including neutral-label precedence over team index.
The Rust builder resolves its own catalog using the map ID. The comparison checks
both flag teams, all home/carried spans and coordinates, two selected spawns, four
team births, every other document field, and the complete kill comparison described
above. API kill/capture totals remain independent aggregate checks.

Home spans and flag-team attribution here are native reconstruction with external
map inputs, not independently annotated spatial ground truth. This is not browser
rendering or map-mesh validation. The earlier two fixtures preserve the native
missing-catalog behavior instead of replacing that negative case.

```
cargo test --lib theater::replay_document_film::tests::complete_ctf_geometry_document -- --ignored --nocapture
```
