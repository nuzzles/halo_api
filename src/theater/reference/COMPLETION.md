# Native v41 parser closeout — 2026-09-27

Current API migration (2026-09-27): the new two-layer goal is described in
[TWO_LAYER_API.md](TWO_LAYER_API.md). The public recording entry is now
`theater::film::Film::parse`; conversion produces `theater::resolved::ResolvedFilm`.
The historical parity closeout and requirements below remain a record of their
original scope and sequence.

Complete under the user's final acceptance: parse native Theater v41 film data
as well as the pinned LevelUp parser. Missing newer upstream features and layouts
that LevelUp itself does not decode are not completion requirements.

Baseline: JGtm/LevelUp feat/v75 commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81`. Selected native-reader improvements from
`d61443ef59268ad734355db8e9974f68db5ca6d0` are also ported and independently tested;
this is not a claim of parity with every newer recovery or replay API.

The recording-only entry is `NativeFilmData::parse_v41`, with explicit precision
and recovery options through `parse_v41_with_options`. It retains source bytes,
ordered native reads, boundaries and unsupported/partial outcomes. Bootstrap,
profiles, component and keyframe bodies, packet/frame views, native event fields,
and summaries have independent reference fixtures. Everything remains in
`src/theater` on `simbleau/theater-experiments`, uncommitted. Unrelated preexisting
Cargo.toml and experiments/FILM_FORMAT.md changes were preserved.

Final current-source verification: 22 tests passed, zero failures, 39.46 seconds:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib -- native_data_ \
  native_bounded_event_references_d61443e native_action_control_d61443e \
  native_control_closure_d61443e native_new_binding_d61443e \
  captured_and_synthetic_heads_match_native_go_outputs --include-ignored
```

This includes all 32 captured films, 403,465 baseline packet view-count/end-bit
comparisons, 134,657 fixed-offset keyframe body comparisons, native event and raw
field checks, 3,667 summaries, and source retention. All 451 sequential Go table
results at the newer pin exactly equal the previously validated original oracle.
The last source revision also passed Clippy and WASM; subsequent changes were
documentation, generators and audit artifacts only. Final diff checks pass.

All 485 original production source hashes were reverified against a fresh archive
of the pinned commit from the retained reference Git object database. The earlier
temporary checkout had been removed; no newer source was substituted.

Retained limits: unsupported tacmap/component grammars, unknown/runtime-width
bodies, externally required precision, and contradictory upstream code0 damage
layouts remain explicit. Preserving them is not a claim of complete knowledge of
every film bit or byte-for-byte re-encoding. Historical `full_native_parity: false`
records refer to that stricter full-format objective and are not rewritten.

No further reverse engineering, resolved replay model, browser visualization, or
architecture refactor is part of this closeout. The queued design remains deferred.
