# Dead-state result retention across live calibration

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_dead_result_retention_test.go.txt.

Thirty-two native cases cover generic and full-keyframe records containing a
dead-state component, an EMP component and another dead-state component. The
EMP hook can install a zero-width calibration before the second dead-state
reader. Four input patterns vary the native state fields. Each case compares
the final native EntityTrace.Dead, and generic cases also compare actual native
objectDeathHarvest output. The two entry points use the same production native
component loop; full-keyframe cases do not invent a timestamped harvest result.

The first Rust comparison failed on the positive death followed by calibrated
skip. Native retains the earlier non-nil value, while Rust selected the last
successful attempt and stopped even though its skipped fields produced None.
The failed comparison is retained in /private/tmp/halo-dead-result-before.log.
Native regeneration passed (0.354s); adding the full-keyframe cases preserved
every original generic expectation unchanged.

EntityRecord and KeyframeRecord now expose captured_dead_state by selecting the
last decoded result using ordered field ranges. MarchRecordFacts uses this
projection before applying its existing Mort filter. Later Mort=false readings
still replace earlier deaths, and a skipped later read does not erase a prior
value. No gameplay meaning is added to the raw fields. Keyframe exports without
attempt provenance return None rather than guessing from overlapping bit ranges.

The regression checks raw final values, generic harvested outcomes, cursor
endpoints and portable JSON reconstruction. It is registered in the WASM
harness. The captured-anchor test now uses the same public projection instead
of its previous first-component source-range lookup. Validation is tracked in
reference/dead-result-validation.json; full parity remains a separate gate.

Final validation: 32 focused cases pass (0.02s); 628 host tests pass, zero failed,
49 ignored (143.65s). Clippy and WASM pass. All 134,657 captured anchors match
(71.29s), including dead-state projection, payload/source fields and JSON checks.
Four captured march films pass (85.05s); six complete documents match (110.50s).
Fixture bytes, all 485 source hashes, formatting/diff and script syntax pass.
