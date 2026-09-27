> Closed under the user's 2026-09-27 acceptance: native v41 parsing as capable
> as the pinned LevelUp parser. Newer upstream additions and LevelUp's own unknown
> grammars are not blockers. See [COMPLETION.md](COMPLETION.md). The historical
> checkpoints below used a stricter goal and are retained as evidence.

# LevelUp v41 Rust port

Reference: JGtm/LevelUp `feat/v75`, pinned at
`43a01721e8a02c87c955e175936f0ccf8dd97a81` (MIT; license retained here).
Changes stay on `simbleau/theater-experiments` inside the existing film implementation
(`halo_api::theater`, also exported through `clients::hi::film`).

## Acceptance criteria

The goal is the full v41 parser, with every field and observation supplied by the
reference available through Rust. A raw-byte dump, successful replay, or matching
summary counts does not establish that parity. Keep source ranges and unknown
fields explicit. Do not add older-major-version compatibility.

1. Reproduce the pinned reference on the local v41 corpus.
2. Port bootstrap, profiles, value codecs, component grammar, keyframes,
   packet/frame views, recorded events and summary data. Internal decoding state
   is allowed; resolved player timelines and replay facts are not deliverables.
3. Compare decoded values, identities, times and record/component boundaries.
4. Expose ordered native data and portable exports with source bytes/ranges and
   unknown regions; do not substitute normalized observations for raw records.
5. Run fixtures, full-corpus comparison, library checks and wasm compilation.

## Queued next phase

After these parity criteria are complete, propose the three-layer architecture
recorded in [NEXT_PHASE.md](NEXT_PHASE.md): faithful native `Film`, a separate
resolved replay model, and stateful playback/browser visualization. Propose
concrete types, migration, fidelity contract, and validation matrix before
starting that refactor. Do not redirect the current parity implementation.

## Current implementation

- Film now retains the native chunk-one clock result and its source packet,
  separately from the legacy minimum timestamp. Replay consumes that result;
  compact/old-export behavior and 1,024 native identity cases pass. Captured
  pre-publication coverage now checks 29 of 44 native inputs, including inventory,
  equipment changes and spawn events. All six input/document comparisons pass
  (82.17s). Full suite: 561 passed, 46 ignored; Clippy/WASM/fmt/diff/hashes pass.
  See CURRENT_VALIDATION_GAPS.md and film-input-retention-coverage.json.

- The complete six-film oracle now retains all 44 native scan-input fields before
  document publication. Twenty fields are compared against Film, including full
  ordered gameplay/channel records and selected diagnostics; 24 remain explicit
  follow-ups. All six input/document comparisons passed (82.34s), along with the
  schema check and Clippy. All earlier document expectations are unchanged.
  See ../fixtures/full-document-inputs-v41.md and film-input-retention-coverage.json.

- Native padded scalar reads now support wide fields and retain discarded source
  prefixes. Immediate and deferred position capture match zero/wide-width Go
  shifts, including exact NaN/infinity results. Both paths pass 4,248 native
  boundary cases, preserving all earlier oracle fields. Full suite: 560 passed,
  46 ignored (69.92s); Clippy/WASM/fmt/diff/source hashes pass. See
  ../fixtures/absolute-width-boundaries-v41.md and CURRENT_VALIDATION_GAPS.md.

- Zero delta-width overrides now use native per-axis traversal fallback, fixing a
  Rust panic while retaining the raw profile. All 1,440 actual-native cases match
  callbacks, accumulated state, endpoints and following reads. Full suite:
  559 passed, 46 ignored (69.75s); Clippy/WASM/fmt/diff/source hashes pass.
  See ../fixtures/delta-width-fallback-v41.md and CURRENT_VALIDATION_GAPS.md.

- Native source-table INFO/WARN publication is wired into Film construction.
  Twenty-one source cases now include a positive interleaved-vacancy table:
  seven occupied seats retained, direct identity mapping refused, INFO then WARN.
  All earlier fixture fields are unchanged; source and four Film paths match
  native structured observations. Full suite: 558 passed, 46 ignored (72.07s);
  Clippy/WASM/fmt/diff/source hashes pass. See CURRENT_VALIDATION_GAPS.md.

- Source player-table diagnostics now survive source reads and Film exports:
  native typed refusal, exact error text and per-call unknown-build increments.
  Twenty native source cases preserve all earlier fields; Film/JSON checks cover
  success, unknown build, absent identity and old-export absence. Full suite:
  558 passed, 46 ignored (79.14s); Clippy/WASM/fmt/diff/source hashes pass. Native
  log/global-counter delivery and full parser acceptance remain separate gates.

- Live reader profile/observer replacement now returns native prior values and
  preserves capture state. The native comparison covers 7,448 component reads,
  6,348 ordered callbacks and 32,768 position reads, including detached delivery
  and accumulator identity. Full suite: 558 passed, 46 ignored (72.85s); Clippy,
  WASM, formatting/diff and source hashes pass. See READER_CONTRACT.md and
  CURRENT_VALIDATION_GAPS.md for remaining acceptance gates.

- NativeFilmReader now exposes the native signed variable-width codec. The new
  actual-native oracle compares 3,784 sequences / 11,352 signed reads, every
  selector and alignment, truncated prefixes, negative values and following
  primitive reads. Full Theater suite: 558 passed, 46 ignored; Clippy, WASM,
  formatting/diff and reference hashes pass. Signed negative live cursors and
  full parser acceptance remain open; see CURRENT_VALIDATION_GAPS.md.

- Malformed type-1 bodies no longer discard the whole Film result. Rejected
  datum payloads retain their source and bytes in Film and replication packets;
  the direct decoder still rejects them. The native 129-length oracle and public
  constructor test retain two valid tables,127 failures and130 surrounding
  frames, including compact-export and JSON checks. See datum-lengths-v41.md.

- Standalone source-reader continuation now matches native signed overflow:
  short reads wrap their endpoints; wide reads retain the native panic cursor.
  The new 1,512 native sequences compare all 13,608 operations, including
  recovery after failure. Both source-bit tests pass (0.55s), preserving every
  pre-existing fixture case. Component-record negative endpoints remain open.

- Native source metadata retains signed 64-bit chunk types, including selected
  oversized values in partial caches. Six native cache cases match all metadata
  and bytes; conversion to the existing narrower FilmChunkData API is checked.
  The full suite passed (552 tests,46 ignored,74.45s); clippy/WASM/fmt passed.
  See `fixtures/cache-partial-metadata-v41.md` and CURRENT_VALIDATION_GAPS.md.

- NativeFilmReader wide primitive reads now consume the requested width and
  return its final 64 bits, matching the existing native source-bits oracle.
  The expanded test passes 29,376 nonnegative reads, including 7,344 wide reads
  and 11,995 padded endpoints. The regression failed before the fix and passed
  after it (0.46s). Signed negative-cursor continuation remains open.

- Fresh-reader lifecycle audit passes 1,280 seeded-world inference/resync calls
  across two new native-backed tests (0.90s). All 1,378 position deliveries,
  record results and world snapshots match; no implicit position accumulation.
  Existing fixture tests also pass; all prior native expectation fields were
  preserved exactly. Clippy/fmt/source hashes pass. No production code changed.
  See `fixtures/reader-lifecycle-v41.md` and the native-reader-lifecycle logs.

- Film now routes complete native profiles through march scanning. Kill calibration
  initializes and retains native_profile, then updates the selected handle width;
  replay installs inherited profile before map precision and film corruption.
  Existing encodings remain available; older serialized profiles preserve None.
  Native six-film oracle generation passed (96.165s), adding full march and kill
  results without changing any prior expected field. All 549 ordinary tests pass
  (46 ignored,94.32s), clippy/WASM/fmt/reference hashes pass. Four captured Film
  constructors without inherited calibration pass full march comparison (75.51s).
  All six inherited-profile cases pass (88.89s): full native march configuration,
  facts, complete kill results/calibrated profiles, and all document fields. Log:
  `/private/tmp/halo-film-native-profile-documents-retry.log`. Initial release build
  ran out of disk space; cleaning package debug outputs restored 76 GiB and the
  subsequent release build succeeded.

- NativeFilmContext.scan_march_facts consumes the full native configuration and
  preserves a detached snapshot in FilmMarchFacts.native_config. All 256 existing
  native loaded cases pass, and all four captured films pass (17.64s, release),
  including every native configuration field. Snapshot-map isolation passes.
  Full suite:548 passed,46 ignored,94.66s; clippy/WASM/fmt/source hashes pass.
  The map-aware Film constructor still uses its older projected march path;
  carrying its complete calibrated input into this path is the next integration.
  See `/private/tmp/halo-native-march-context-{suite,corpus,clippy,wasm}.log`.

- Film recovery retains native full-state outcomes alongside bounded reads;
  identical outcomes share storage, differing padded outcomes retain their own
  fields and crossing diagnostic. Older exports explicitly lack the native result.
  Captured padding regression passes (0.68s), full suite passes (547 tests,
  45 ignored,69.04s), clippy and WASM pass. See
  `fixtures/recovered-keyframe-padding-v41.md`. All six captured documents pass (1805.71s, debug) in
  `/private/tmp/halo-native-retention-documents.log`. The timing wrapper failed
  only its post-test resource query. Release also passes (75.20s); measured
  peak RSS is 2,847,883,264 bytes via child getrusage. Log:
  `/private/tmp/halo-native-retention-release-documents.log`.

- KeyframeRecord.captured_payload now exposes typed captured values from retained
  fields, without rereading bytes. The expanded full-anchor test compares six
  payload kinds, dead state, every consumed source/padded bit and JSON roundtrips;
  all 134,657 anchors passed in 399.18s in
  `/private/tmp/halo-keyframe-anchor-payloads-tests.log`.
  Regular suite:546 passed,45 ignored,70.57s. Clippy and WASM pass. Parent end-bit
  diagnostic handling and exact scope are documented in the anchor fixture note.

- Native full-state oracle now covers all 134,657 recovered anchors across 451
  keyframes: 95,532 complete reads,39,125 desynchronizations,468 padded endpoints.
  It retains six captured payload kinds and43,631 dead-state values. Native run
  passed (14.326s); Rust streaming comparison passes (54.43s), as does clippy, in
  `/private/tmp/halo-keyframe-anchor-bodies-tests.log`. Initial setup mismatches
  were corrected to the explicit native default precision/simulation context;
  no production code changed. Typed payload comparisons passed in the subsequent expanded run above.

- Sequential keyframe corpus: actual native traversal and Rust match across all
  451 keyframes / 32 films (902 records,5,412 traces). All stop at the unsupported
  component in record two; this does not decode the later recovered anchors.
  Four focused tests pass (23.70s), now checking retained raw fields against source
  bits as well as native ordering/bounds/component starts/stop. No production code
  changed. See `fixtures/keyframe-table-corpus-v41.md` and
  `/private/tmp/halo-keyframe-tables-tests.log`.

- Complete hour-long raid document and assembly both pass (530.59s):
  `/private/tmp/halo-current-raid-parity.log`. The binary precedes only the new
  preamble metadata field; parsing/assembly behavior is unchanged.
- Preamble retention passes the full suite (546 tests,89.22s) and four-film
  captured march (58.65s), with native metadata and complete result roundtrips.
  Logs: `/private/tmp/halo-march-preamble-{suite,captured}.log`.

- FilmMarchFacts now retains packet_preamble_bits, sharing the same two-bit
  setting with actual packet traversal. Native loaded-march goldens verify 201
  calibrated cases and 55 no-delta early returns; old exports retain None.
  Focused march tests pass (8.61s), including JSON retention. Clippy and WASM
  pass; full suite and captured march reruns are active in `/private/tmp/halo-march-preamble-*.log`.
  This is one retained configuration field; full native profile parity stays open.

- Hour-long raid assembly matches native: 269 player tracks, 208,865 position
  points, 86 vehicle tracks and 149 rides. The following full raid document test
  remains live in `/private/tmp/halo-current-raid-parity.log`.
- Seven captured identity/kill regressions pass (8.50s): player tables, highlights
  and deaths, Bandit/Oddball kill decoders and evidence, and complete Bazaar
  missing-identity/unknown-format/zero-axis documents.
  Log: `/private/tmp/halo-current-identity-kills.log`.

- Six captured keyframe/objective scanner tests pass (6.73s): six-film layouts
  and 346 position probes, 64 queue fixture rows across four films, four-film
  loadout sources, 32-film objective extraction and two research-directory
  precision cases. Log: `/private/tmp/halo-current-keyframe-scanners.log`.
  Coverage limits and the remaining object-death profile audit are recorded in
  `CURRENT_VALIDATION_GAPS.md`; raid comparison is still active separately.

- Eight captured combat/scanner checks pass (62.65s) after duplicate source
  correction: four-film movement and march scans, two-film calibrated kill walks,
  six-film weapon-hit raw scans/tracks/prefix tracks/aggregates and weapon patterns.
  Log: `/private/tmp/halo-current-combat-scanners.log`. Hour-long raid assembly
  and complete document checks remain running separately.

- Fixed duplicate numbered source selection in six component scanner paths.
  Later duplicate chunks previously replaced bytes underlying native anchors;
  the new regression reproduced a spurious Truncated error. Shared numbered
  bytes and packet-ordinal lookup now retain the first entry, matching native.
  Two native 512-case fixtures preserve all original source-availability results
  while adding empty/truncated/conflicting later chunks. Both pass (0.14s).
  Regular suite: 546 passed, zero failed, 43 ignored (77.30s). Clippy, WASM,
  formatting/diff, generator syntax and 485 pinned hashes pass. Captured position
  and all six integrated document goldens pass (two tests,75.45s) in
  `/private/tmp/halo-duplicate-sources-captured.log`.
  See `fixtures/duplicate-component-sources-v41.md` for positive counts and scope.

- Captured source/scanner retention rerun: six tests pass (6.10s), covering
  Bazaar raw/zlib bytes and Film JSON, 16,113 loaded statborg records in 32 films,
  16,569 managed readings in six films, 13,805 radial readings in five fixture
  rows and 204 equipment-list emissions in six films. The positive Cadet Blue
  export test now uses the constructor's context-aware scan for full diagnostic
  equality instead of the legacy scan; native record goldens, stats and JSON
  checks remain. This fixes a stale test assumption, not production decoding.
  Clippy and all 485 pinned source hashes pass. Log:
  `/private/tmp/halo-current-source-scanners-fixed.log`.

- Keyframe inventory now uses native contiguous chunk selection and tolerant
  packet traversal rather than the strict global packet index. Added a diagnostic
  API and `Film.keyframe_inventory_error`; failed scans retain counts and the
  default-cap fallback through Film/JSON. Replay consumers suppress errored input.
  The 120-case actual-native oracle covers 18 positive inventory cases, 30 source
  failures, empty catalogs, cap defaults, packet End/zero-size/malformed tails,
  metadata order and duplicates. All three focused inventory tests pass (1.39s),
  including constructor error/fallback retention. Regular suite: 544 passed,
  zero failed, 43 ignored (83.59s); clippy, WASM, formatting/diff and generator
  syntax pass. Captured position/document comparisons pass (two tests, 72.31s),
  including all six integrated document goldens. Logs:
  `/private/tmp/halo-inventory-source-{suite,clippy,wasm,captured}.log`.
- The captured position and integrated document rerun after equipment recovery
  source changes passed both tests (72.61s), including all six document goldens.
  Log: `/private/tmp/halo-recovery-source-captured.log`. This predates the latest
  inventory traversal change; its replacement run is separate.

- Fixed equipment recovery source traversal: it now matches native numbered
  ChunkAt lookup (first duplicate metadata entry, independent of chunk type),
  scans valid packet prefixes before malformed tails, and stops at End/zero-size
  non-End packets. The prior strict global packet index rejected native-accepted
  inputs and could admit packets after End. The 24-case native source oracle
  passes with 12 positive recoveries and 12 controls, including missing/reordered
  chunks and duplicate numbers. Source emissions, ordinals and retained JSON are
  compared, including final recovered assembly. All five focused recovery tests
  pass (3.01s); clippy and WASM pass. The regular suite passes 542 tests, zero
  failures, 43 ignored (70.86s), `/private/tmp/halo-recovery-source-suite.log`.
  The affected captured position/document rerun remains active in
  `/private/tmp/halo-recovery-source-captured.log`. These are synthetic source
  cases, not a captured positive recovery film.
- Captured position and equipment-state comparisons passed (two tests, 29.00s),
  including 29,852 equipment samples and full position companion/source fields.
  All native captured equipment-change cases still have zero recoveries.

- Current-code release captured checks passed: 134,657 native recovered anchors
  across all 451 keyframes (6.34s), complete CTF geometry/kill document (4.02s),
  and four-film raw inventory/gameplay (6.38s). Logs are listed in
  `CURRENT_VALIDATION_GAPS.md`; anchor parity does not imply every component body.
- Added an actual-native eight-case live chain budget fixture: three normal-budget
  BudgetExhausted outcomes and five NoConfirmation controls. Calibrated candidate
  widths induce branching without changing the 200,000-trial limit. Rust live
  observer/frame comparison passes (3.32s), including all counters, stopping
  offsets, retained failed Delta headers, no fabricated bindings and JSON.
  Generator and fixture provenance are registered. This closes the missing
  positive live-counter branch, not full inference/callback acceptance.

- Release player/combat/vehicle assembly passed all six films (157.70s), completing
  the run in `/private/tmp/halo-release-player-assembly.log`. Together with the
  six-film integrated document pass (83.51s), this supersedes the interrupted
  debug attempts. Both binaries predate static-width compaction; regular widths
  are unchanged. Current-code release comparisons for all 451 recovered keyframes
  and the complete embedded CTF geometry/kill document are now running.
  `CURRENT_VALIDATION_GAPS.md` is reconciled to these current results instead of
  repeating superseded run counts and pending-process claims.

- Static calibrated/stub width overrides now use the live reader's checked
  signed skip and bounded padding representation. Previously a large static
  override could allocate one field per 64 padded bits. The existing 18-case
  native oracle now runs through both paths, checking exact cursors, overflow
  refusals, actual source bits, bounded field counts, unused-stub controls and
  JSON retention. All three large-width tests pass (0.03s); clippy, WASM,
  formatting/diff and all 485 pinned reference hashes pass. Full Theater suite:
  540 passed, zero failed, 43 ignored (75.55s);
  `/private/tmp/halo-static-width-suite.log`.
- The release integrated document comparison passed all six native golden films
  (83.51s), including Bazaar, empty Aquarius, Bandit, Oddball, Cadet Blue and
  Cadet Brick. Log: `/private/tmp/halo-release-decoded-documents.log`. Aquarius's
  stage assertion is fixed using the native zero-frame golden. The independent
  player-assembly release run remains active. Both binaries predate the
  static-width change; ordinary corpus widths are unchanged by compaction.
  This proves the six-film complete document comparison, not all parser fields
  or the remaining positive captured VIP/bomb and reader-contract gates.

- Fresh integrated six-film document validation matched Bazaar, then exposed
  an overstrict test assertion on Aquarius (27.22s). The pinned native Aquarius
  document has frameCount=0, so only the retained-source stage runs; the test
  incorrectly demanded all six stages for every film. Expected stages now use
  the native golden frame count, independently of the Rust result. Complete
  document field comparison is unchanged. Rerun is active in
  `/private/tmp/halo-current-decoded-documents.log`; full document parity remains
  unverified until it finishes. This is a test-harness correction, not a parser
  behavior change. The separate player-assembly corpus process also remains live.

- Bomb replay now explicitly gates retained radial readings on the scan error,
  matching native `decodeFilmBombReads`. Partial scan data and its error remain
  in Film/JSON. Existing source failures occur before readings are produced;
  this fixes the adapter contract for a populated partial-result/error pair,
  not a demonstrated captured false arming. A regression injects that pair
  after constructing a Film from bootstrap and captured replication bytes,
  with successful/absent-stream controls and source-preserving JSON checks.
  All three focused bomb tests pass (0.82s). Full Theater suite: 539 passed,
  zero failed, 43 ignored (71.47s), `/private/tmp/halo-bomb-publication-suite.log`.
  Clippy lib/tests with warnings denied, WASM all-features, formatting/diff and
  generator syntax pass. The six-film corpus run remains active, with Bazaar
  and Bandit comparisons logged; complete corpus acceptance is still unverified.

- Added independently executed native bomb-start fallback boundary coverage:
  288 cases, 24 positive / 264 zero, 24 total hits. The full arming-layer result
  matches Rust, including verdict float bits, coverage and mirrored-slot
  deduplication. The focused test passed in 0.02s. The older 1,024-case bomb
  fixture had no positive start-zero cases. Generator registration, fixture
  documentation and fallback call-site evidence are updated. This closes a
  direct-layer test gap, not captured bomb or whole-document acceptance.

- Fallback source inventory now distinguishes the pinned catalog's 18 wired
  counters from 81 unwired entries. All 18 wired names have Rust production
  references; usage constants were followed to their actual trigger/report sites,
  and vehicle/precision maps were followed to document merging. See
  FALLBACK_WIRING_AUDIT.md, a follow-up to the existing 19-site
  fallback-callsite-audit.json. Seven native fixture counter distributions were
  refreshed with positive/zero counts; full caller-equivalence remains open. Do not treat the 81 unwired native
  entries as 81 missing Rust counters, or catalog presence as execution evidence.
  Six-film session 4037 remains live, with only Bazaar reported so far.

- Removed the i32 mobility-extra limitation. ComponentBodyPolicy now preserves
  signed i64 metadata and dispatch evaluates the native branch before using it.
  Active large skips use checked signed cursor arithmetic and compact synthetic
  zero padding, retaining every available source bit with a distinct
  NativeWidthPurpose::MobilityExtra diagnostic. Old override diagnostics default
  purpose to None and retain their serialized shape. Actual-native large oracle:
  208 cases / 156 callbacks, including 4096/4097, 2^31, 2^40 and i64 boundaries.
  On this 64-bit host, 204 nonnegative outcomes match and four negative overflow
  cursors are explicitly refused; source bits, bounded fields, raw profile and
  JSON retention pass. Both mobility oracle tests pass (0.03s), Go generation
  passed (0.393s). See fixtures/mobility-extra-large-v41.md. This supersedes the
  previous active-positive i32 refusal; other oversized metadata remains open.
  Full Theater: 537 passed, zero failed, 43 ignored (75.58s), log
  /private/tmp/halo-mobility-large-suite.log. Clippy lib/tests warnings denied
  passed (56.85s incl lock wait), all-features WASM passed (15.16s).
  Formatting/diff, generator syntax and all 485 pinned source hashes pass.
  The six-film assembly session 4037 remains live on the earlier binary;
  the default zero mobility-extra policy used there is unchanged.

- Corrected native profile adaptation for ignored mobility-extra metadata.
  The reference skips extra bits only when the full body is disabled and the
  signed count is positive. The adapter now applies that policy before its i32
  conversion; ignored values no longer refuse unrelated or mobility reads.
  NativeScanProfile retains the original i64 value. Actual-native oracle: 128
  cases including i64::MIN and large ignored positive widths, 96 callbacks;
  Rust dispatch/cursor/callbacks/raw-profile/JSON comparisons pass (0.02s).
  Native generation passed (0.479s). Positive out-of-i32 active widths and other
  oversized metadata remain open. The six-film corpus run uses the prior binary
  with default extra width zero, unaffected by this metadata-edge correction;
  session 4037 is still live and must not be restarted.
  Full Theater: 536 passed, zero failed, 43 ignored (73.61s), log
  /private/tmp/halo-mobility-extra-suite.log. Clippy lib/tests warnings denied
  passed (21.50s incl lock wait), all-features WASM passed (7.21s incl wait).
  Formatting/diff checks, generator syntax and all 485 source hashes passed.

- Constructor/data-contract audit while the fresh six-film comparison runs:
  all six native grammar_abilities.go declarations are now mapped to Rust fields
  and actual producers in DATA_CONTRACT_INVENTORY.md. This supersedes the stale
  four-unmapped-declarations note without claiming full captured acceptance.
  Wire-bounded counter/rank and charge narrowing preserves native values; no-rank
  emissions remain distinct from published ranks. Biped creation profile carriage
  reaches only fixed-width primitives, and held-weapon/creation setup failures
  occur before native counters accumulate; no new production fix was warranted
  for those reviewed paths. Six-film session 4037 remains live; Bazaar passed
  (one track, 71 points), and later films have not yet reported. Do not restart
  this live handle or treat the pending comparison as complete.

- Generic frame-reader accumulation is now wired through NativeFrameRuntime.
  The standard frame method uses a separately attached capture world. The new
  read_frame_records_accumulating method uses one borrowed world for capture and
  lifecycle commits, without aliasing mutable references; conflicting independent
  attachment is rejected before any mutation. Both modes preserve native defaults.
  Actual-native oracles pass: separate-world 512 cases / 2,513 records / 10,141
  callbacks; shared-world 512 cases / 2,560 records / 10,475 callbacks. Tests compare
  all callbacks, record IDs/end bits, final cursor, complete worlds using float32
  position bits and decoded-view JSON roundtrip. Focused tests pass (0.76s).
  Shared-world Go generation passed (4.799s); see both position-accumulator fixture
  markdown files. Inference/recovery scopes and further cross-call lifecycle
  cases are still open, as are the broader integration/corpus acceptance gates.
  Validation: full Theater 535 passed, zero failed, 43 ignored (67.43s),
  /private/tmp/halo-frame-accumulator-suite.log. Clippy lib/tests warnings denied
  passed (37.17s incl lock wait); all-features WASM passed (24.53s incl wait).
  Formatting/diff checks, generator syntax and all 485 pinned source hashes pass.
  A fresh six-film local_film_player_assembly run has been started separately;
  it is not yet verified. Live session 4037 was re-polled successfully.
  Log: /private/tmp/halo-frame-accumulator-film-corpus.log.

- NativeFilmReader direct component reads now support an optional borrowed
  FilmWorld accumulator. Context replacement preserves it; explicit cursor changes
  do not roll back world or observer state. Actual native existing sequence
  fixtures verify 32,768 reads, all five position publication kinds and float32
  world bits, with hooks toggled and profiles replaced. The lifecycle oracle adds
  12,288 reads with bindings/deletions/seeds, no observer and detachment. See
  READER_CONTRACT.md. Frame-reader accumulation is still open and now explicitly
  refused when attached, preventing a silent loss of the caller's requested mode.
  Full Theater: 533 passed, zero failed, 43 ignored (69.87s), log
  /private/tmp/halo-live-accumulator-suite.log. Clippy lib/tests warnings denied
  passed (23.42s incl lock wait), all-features WASM passed (10.47s incl wait).
  Formatting/diff, generator syntax and all 485 reference hashes passed.
  A new actual-native generic-frame accumulator fixture is generated and registered
  (512 cases / 2,513 records / 10,141 callbacks, Go 0.534s), but not yet consumed
  by Rust. Its fixture markdown records the next integration and ownership gates.

- Whole-Film movement retention: both map and explicit-encoding constructors now
  run the diagnostic movement scanner and retain its report even on setup failure.
  Error text and the existing error-stats field remain available. Native replay
  deliberately clears readings/stats on failure; the Rust stance adapter now
  applies that policy without deleting Film diagnostics. A new actual-native
  positive oracle supplies 32 cases / 64 crouch readings, compared through the
  public explicit constructor and Film JSON roundtrip. Existing missing-archetype
  coverage checks both constructors; a new replay regression checks failed-report
  suppression and JSON retention. See fixtures/movement-positive-v41.md.
  Validation: full Theater suite 533 passed, zero failed, 43 ignored (67.19s),
  /private/tmp/halo-movement-retention-suite.log. Clippy lib/tests with warnings
  denied passed (12.03s). All-features WASM passed (19.96s, before the final
  test-only fixture correction). Formatting/diff checks, generator syntax and
  all 485 pinned source hashes passed. The negative regression passed separately
  after its bootstrap-only setup was corrected to include replication packets.
  This closes the identified movement omission, not constructor-wide parity.

- Large live positive overrides now retain actual source bits and compact their
  synthetic zero tail instead of expanding every padded word. Widths above 4096
  bits extending beyond source use NativeWidthAdjustment.retained_bits and end_bit;
  ordinary overrides preserve their scalar layout. Work is bounded by source size.
  Signed target addition now checks the pinned reference's i64 domain before usize
  conversion; negative/wrapped targets remain explicit typed refusals. The error
  payload is boxed to prevent inflating containing frame enums; JSON is unchanged.
  Actual native large-width oracle: 18 cases, including 4096/4097, megabit, terabit
  and near-i64::MAX overrides. On this 64-bit host 16 native outcomes match and two
  signed-overflow negative cursors are explicitly refused. Tests check nonzero
  source bits and bounded field counts, plus merge/JSON retention. Go generation
  passed (0.326s), focused large test passed (0.01s). Full Theater before the final
  boxing-only change: 531 passed, zero failed, 43 ignored (67.67s), log
  /private/tmp/halo-large-width-suite.log. After boxing, both width tests passed
  (0.01s), Clippy warnings denied passed (11.42s), all-features WASM passed (40.37s,
  including lock wait). Formatting/diff, generator syntax and all 485 hashes pass.
  No process remains. Negative/wrapped-cursor continuation, legacy static usize
  override compaction and oversized/unused profile metadata remain open.
  The major remaining acceptance gates are still optional accumulation, whole-Film
  constructor/pass wiring and portable output retention, plus captured coverage.
  Prioritize auditing those integration paths next; passing reader-edge oracles
  does not establish full parser parity. Architecture remains deferred.
  See fixtures/large-width-v41.md and READER_CONTRACT.md.

- Live component width overrides now honor signed native cursor rewinds when the
  target stays nonnegative. NativeWidthAdjustment retains component, override
  kind, starting bit, original i64 width and target offset; diagnostics merge and
  serialize these adjustments. Unrepresentable targets stop entity records with
  InvalidWidthOverride instead of being mislabeled Truncated. Supported component
  readers still ignore stub overrides, including negative stubs. Calibration
  bypasses the reader. A rewind can produce a component end before its start;
  consumers must treat this as cursor movement rather than a forward byte span.
  Actual native signed-width oracle: 24 cases, 22 nonnegative native outcomes
  matched, six successful rewinds; two native negative-cursor cases are explicit
  Rust refusals, preserving width/start metadata. This is a documented boundary,
  not a claim of negative-cursor parity. Native generation passed (0.396s), focused
  Rust passed (0.01s). Clippy warnings denied passed (56.47s), all-features WASM
  passed (20.00s), with lock waits. Formatting/diff, generator syntax and all 485
  pinned source hashes pass. Full Theater: 530 passed, zero failed, 43 ignored
  (69.90s), log /private/tmp/halo-signed-width-suite.log. No process remains.
  Huge positive overrides, native signed overflow and oversized/unused profile
  metadata are still open; inspect raw_bits allocation before testing large widths.
  Optional accumulation, whole-Film wiring and other documented coverage gates
  remain incomplete. Architecture stays deferred. See fixtures/signed-width-v41.md
  and READER_CONTRACT.md.

- Live RecordMaskHook is now connected in the payload biped scanner and observed
  explicit-band loaded quantized/world scans. Publication occurs per record after
  companion decoding and saturation rejection, borrowing the complete payload and
  ordered component indices with the actual after-i0 offset. Source isolation and
  speed filtering occur afterward. Legacy APIs delegate without an observer;
  retained mask reports remain available, and generic frame loops do not emit it.
  The existing pinned record-mask fixture was reused without changing expectations:
  256 cases/6144 payloads compare 2647 live payload publications; 1280 source
  selections compare 9953 quantized and 9953 world publications. This covers
  capture-disabled and saturated negatives, duplicate selections, unreadable
  chunks, temporal filtering and dynamic orientation. Payload focused test passed
  (2.31s), loaded/world focused test passed (7.58s). Full Theater: 529 passed,
  zero failed, 43 ignored (66.32s), log /private/tmp/halo-live-record-mask-suite.log.
  Clippy warnings denied passed (20.52s), all-features WASM passed (7.22s), with lock
  waits; formatting/diff and all 485 pinned source hashes pass. No process remains.
  This is live mask publication through explicit observer APIs, not completion of
  all FilmContext/Film constructor wiring or independent semantic goldens.
  Next inspect unsupported-width classification in components/widths.rs: failed
  signed-to-usize conversions currently become None and look like truncation.
  Compare native override semantics and preserve accurate refusal/source details.
  Optional accumulation, whole-Film integration and other coverage gates remain
  open. Full parity remains incomplete; architecture remains deferred.
  See fixtures/live-record-mask-v41.md and READER_CONTRACT.md.

- NativeFrameConfig.decode_resync_frame now carries live context through the full
  raw-resync loop: sequential reads, clean New/Delete mutations, position/movement
  suppression around copied-observer scans, restoration before accepted-record
  re-read, and fresh sequential capture slot zero after recovery. NativeResyncFrame
  retains all speculative diagnostics independently of delivered callbacks, with
  source recovery offsets and typed stops. Legacy no-context APIs are unchanged.
  Actual native context-resync-frame oracle: 128 cases, 74 returned records (62
  Delta, eight New, four Delete), 87 acceptance calls and 671 observation/probe
  callbacks. It verifies returned identities/masks/component boundaries, positions
  as float32 bits, ordered callbacks, histograms, world bindings and JSON roundtrip.
  Native generation passed (0.492s), focused Rust passed (0.08s). Full Theater:
  529 passed, zero failed, 43 ignored (72.13s), log
  /private/tmp/halo-context-resync-frame-suite.log. Clippy warnings denied passed
  (57.19s), all-features WASM passed (20.43s), with lock waits. Formatting/diff,
  generator syntax and all 485 hashes pass. No process remains.
  Fixture gaps include positive record-limit exhaustion, callback-induced failed
  accepted-record re-read, and a native final-loop cursor oracle (native returns
  records only). Whole-Film integration and other existing gates remain open.
  Next inspect live RecordMaskHook publication in biped_scan against native
  offline_biped.go: it fires after direction decoding, before post-processing
  filters, and is not emitted by generic frame loops. Unsupported-width semantics,
  optional accumulation and remaining captured coverage also remain incomplete.
  Architecture remains deferred. See fixtures/context-resync-frame-v41.md and
  READER_CONTRACT.md.

- NativeFrameConfig.scan_for_target_delta now installs the shallow observer copy
  and temporary first-position collector for raw target scanning. Every candidate
  Delta uses live context; acceptance receives slot, float32 vector and presence.
  A positionless candidate can retain a previous trial vector, exactly as native,
  so the presence flag is authoritative. Caller hook replacement during copied
  callback execution persists on the caller while the scan keeps copied hook
  identities. Existing histogram maps stay shared and newly allocated maps stay
  local. Read diagnostics are retained independently; World is unchanged.
  Actual native context-raw-resync oracle: 128 cases, two scans each, 140 landings,
  172 acceptance calls and 762 component/post-scan callbacks. Native generation
  passed (0.406s), focused Rust passed (0.08s). Full Theater: 528 passed, zero
  failed, 43 ignored (66.39s), log /private/tmp/halo-context-raw-resync-suite.log.
  A test-only modulo style lint was then replaced with is_multiple_of; Clippy
  warnings denied passed (9.26s). All-features WASM passed (6.76s), formatting/diff,
  generator syntax and all 485 source hashes pass. No process remains.
  Next connect the whole DecodeFrameResync loop to live context, including scoped
  position/movement suppression around scans, accepted-record re-read, and fresh
  sequential-reader capture slots. The existing legacy loop remains unchanged.
  Biped RecordMaskHook, unsupported widths, optional accumulation, whole-Film
  integration and other documented coverage gates remain open. Full parity is
  incomplete; architecture remains deferred. See context-raw-resync-v41.md and
  READER_CONTRACT.md.

- NativeFilmObserver now represents the three native map fields independently
  from scalar counters. shallow_copy copies hook identities and scalar values,
  sharing already allocated maps while leaving nil maps independently allocated
  on first write. take_absolute_indices replaces the caller map rather than
  clearing a map still owned by a shallow copy. Ordinary Clone retains full-state
  aliasing. Existing counter snapshots remain owned FilmReadDiagnostics values.
  Actual native observer-copy oracle: 16 cases, four paired snapshots each
  (128 observer states), reset outputs and callback replacement. This verifies
  nil/empty/populated absolute maps, anticipated-map allocation, shared repair
  histograms and independent scalar counters. Native generation passed (0.326s);
  all four focused observer tests passed (0.01s). Full Theater: 527 passed, zero
  failed, 43 ignored (66.84s), log /private/tmp/halo-observer-copy-suite.log.
  Clippy warnings denied passed (26.20s), all-features WASM passed (9.03s), with
  lock wait included. Formatting/diff, generator syntax and all 485 hashes pass.
  No process remains. Raw-resync live scan integration is still open: connect
  frame_harvest::scan_for_target_delta to the new copy and its temporary first-
  position collector, retaining callback replacement semantics and map sharing.
  This primitive is not a claim of complete raw-resync parity. Biped RecordMaskHook,
  unsupported widths, optional accumulation, whole-Film integration and previously
  documented coverage gates remain open. Architecture remains deferred.
  See fixtures/observer-copy-v41.md and READER_CONTRACT.md.

- Live explicit component repair now runs through NativeFrameConfig.repair_component.
  Width sweeps, recursive confirmation and the winning-width re-read carry live
  reader context. The local fresh stub map preserves caller overrides (including
  zero-width preset refusal); shared calibration maps and the observer remain live.
  Capture receivers are scoped off/restored, with in-scope callback re-enablement.
  New repair starts at slot zero; Delta sets its record slot. Successful repairs
  increment the repair count once and every matching width; failed resolution
  increments its chain outcome. World and default traversal remain unchanged.
  Actual native context-repair oracle: 64 contexts, two attempts each, 14 New and
  14 Delta successes, 88 Ambiguous outcomes and 260496 callbacks with counter/map
  snapshots and restoration probes. Native generation passed (0.770s), focused
  Rust passed (3.10s). Full Theater: 526 passed, zero failed, 43 ignored (68.38s),
  log /private/tmp/halo-context-repair-suite.log. Clippy warnings denied passed
  (12.74s), all-features WASM passed (19.95s including lock wait); formatting/diff,
  generator syntax and all 485 source hashes pass. No process remains.
  Positive live NoConfirmation/BudgetExhausted repair cases remain absent in this
  fixture, with older no-context fixtures complementary. Raw resync shallow-copy
  ownership, biped RecordMaskHook, unsupported-width classification, optional
  accumulation and whole-Film integration remain open. Full parity is incomplete;
  architectural work remains deferred. See fixtures/context-repair-v41.md and
  READER_CONTRACT.md. Next inspect frame_harvest::scan_for_target_delta against
  native frame_harvest.go::scanForTargetDelta for live shallow-copy behavior.

- Live validated resync is connected through NativeFrameConfig.validated_resync.
  All candidate DELTAs and recursive confirmations use the shared reader context;
  capture receivers are scoped off/restored, other hooks stay live, and callbacks
  may reinstall capture receivers. Candidate DELTAs set the entity slot in native
  decodeDelta; fresh recursive body trials start at zero. Successful scans update
  the cumulative observer counter once, without mutating World or enabling
  automatic resync. Legacy no-context behavior is preserved; removed its unused
  bound-record wrapper. Existing resync fixtures now also check contextual landing,
  hook restoration and repeated success counters.
  Actual native context-resync oracle: 128 cases, two scans each, 28 successful
  landings, 682 callbacks including restoration probes. Native generation passed
  (0.356s), focused Rust passed (0.06s). Full Theater: 525 passed, zero failed,
  43 ignored (65.19s), log /private/tmp/halo-context-resync-suite.log. Clippy warnings
  denied passed (19.82s including lock wait), all-features WASM passed (6.67s),
  formatting/diff, generator syntax and all 485 pinned source hashes pass.
  No process remains. This fixture does not prove positive live budget exhaustion
  or raw-resync shallow-copy semantics. Live explicit repair is the next integration
  point; RecordMaskHook, unsupported-width classification, optional accumulation,
  whole-Film integration and other documented coverage gates remain open. Full
  parity is incomplete and the architectural refactor remains deferred.
  See fixtures/context-resync-v41.md and READER_CONTRACT.md.

- Recursive chain inference now carries the live reader context through candidate
  bodies and recursive NEW reads. It uses the native position/movement scope,
  leaving unit references live, and records each chain outcome in the shared
  observer after resolution. The configured inference entry point now honors
  chain_inference instead of refusing it; shared alignments still do not create
  arbitrary soft bindings. Old no-context and explicit repair/resync behavior is
  unchanged. Removed the now-unused non-context body-trial wrapper.
  Actual native context-chain oracle: 128 cases, 107 inferences, 183 records,
  1129 callbacks; Immediate=81, Deep=26, NoConfirmation=13. EMP callbacks snapshot
  all five counters and post-decode probes verify receivers after scope restoration.
  Native generation passed (0.408s), focused Rust passed (0.07s). Full Theater:
  524 passed, zero failed, 43 ignored (see `/private/tmp/halo-context-chain-suite.log`).
  Clippy warnings denied passed (13.08s), all-features WASM passed (19.27s), including
  lock wait. Formatting/diff, generator syntax and all 485 source hashes pass.
  No process remains. Positive live Ambiguous/BudgetExhausted and recursive NEW
  callback cases remain uncovered by this fixture; existing non-live fixtures
  remain complementary. Live repair/resync counters and shallow observer-copy
  ownership, RecordMaskHook, unsupported-width classification, optional
  accumulation and whole-Film integration remain open. Full parity is incomplete;
  architecture stays deferred. See context-chain-v41.md and READER_CONTRACT.md.

- NativeFilmObserver now supports scoped capture neutralization/restoration:
  all captures, position plus movement, or movement alone. The scopes remove
  actual shared hooks, leave mobility/other hooks and counters live, permit
  callback-driven re-enablement, and restore saved identities (including nil)
  on exit. Nested restoration and unwinding have a Rust regression.
  NativeFrameConfig.decode_inference_view now connects single-step inference
  trials and bound-successor confirmation to live context, maps and scopes.
  Existing no-context inference keeps its filtered diagnostics; contextual
  inference retains candidates independently of actual hook delivery. Single-step
  inferred records do not fabricate bindings. Recursive chain mode is explicitly
  refused until its live observer/counter path is connected.
  Actual native context-inference oracle: 128 cases, 61 inferences, 163 records,
  83 end/view-rejection outcomes, 25 unknown-slot rejections, 1217 callbacks.
  Unique/ambiguous/unsupported candidates, soft successors, short sources,
  in-scope hook replacement and post-decode restoration are compared. Native
  generation passed (0.567s); focused Rust passed (0.06s). Full Theater passed
  523 tests, zero failed, 43 ignored (65.79s), log
  `/private/tmp/halo-context-inference-suite.log`. Clippy warnings denied passed
  (26.20s), all-features WASM passed (38.20s), including lock waits. Formatting/
  diff, generator syntax and all 485 source hashes pass. No process remains.
  Recursive Context in chain_inference.rs still uses no live reader context;
  that and its outcome counters are the next integration point. Resync shallow
  observer-copy/map ownership, RecordMaskHook, invalid-width classification,
  optional accumulation and whole-Film integration remain open. Full parity is
  incomplete; architecture stays deferred. See context-inference-v41.md.

- NativeFrameConfig.decode_production_views now installs shared reader context
  in the message/entity/control path with view-table admission. Variable preamble
  offsets follow the native class dispatch, including early message failure and
  empty/truncated sources. Record attempts receive live hooks/calibration maps
  and the configured simulation-completion gate. Admission decisions increment
  cumulative observer counters before subsequent decoding, independently of the
  retained per-pass diagnostics. Existing production APIs retain their prior
  policy; disabled view classes/tables are refused by this configured entry point.
  Native context-views oracle: 128 contexts, two passes each, 906 records and
  1332 callbacks; 48 cumulative unknown-slot and 50 foreign-view rejections.
  It compares record/component boundaries, statuses/masks, callbacks, live widths,
  worlds, completed views, final cursors and observer counters across both passes.
  Native generation passed (0.457s), focused Rust passed (0.14s). Full Theater
  passed 521 tests, zero failed, 43 ignored (64.92s), log
  `/private/tmp/halo-context-views-suite.log`. Clippy warnings denied passed
  (13.68s), all-features WASM passed (20.43s, including lock wait), as did
  formatting/diff, generator syntax and all 485 pinned source hashes. No process
  remains. Positive anticipated-counter callback timing, alternative inference/
  resync contexts, biped RecordMaskHook, unsupported width semantics, optional
  accumulation and whole-Film integration remain open. Full parity is incomplete;
  the three-layer architecture stays deferred. See context-views-v41.md and
  READER_CONTRACT.md for exact scope.

- NativeFilmReader.read_frame_records now connects NativeFrameConfig to the
  existing generic native record loop. It installs the frame context, applies
  positive preambles only at cursor zero, preserves the end cursor/capture slot,
  and commits clean New/Delete records to FilmWorld. End and failed attempts
  remain in DecodedEntityView. Position accumulation is not automatically enabled,
  matching the native generic loop. Component calibration/stub maps remain shared
  and are queried at dispatch time; a callback can change the next component's
  calibrated width within the same record. The adapter retains raw frame metadata
  and rejects unsupported scalar widths before decoding. Invalid live-map width
  failure classification still needs distinction from truncation.
  Native context-frame oracle: 128 cases, 584 records, 888 ordered callbacks,
  110 complete loops, 25 padded reads and 19 nonzero initial cursors. Checks cover
  component starts/status, masks, generation rejection, callback-driven widths,
  observer replacement, final world slots/views/positions and capture slot.
  Native generation passed (0.514s). Full Theater: 520 passed, zero failed,
  43 ignored (67.46s), `/private/tmp/halo-context-frame-suite.log`. Clippy passed
  after grouping internal view context and moving tests after implementation
  (16.98s). Both new context-frame and existing record-chain tests passed again
  after that cleanup. All-features WASM also passed again after cleanup.
  All validation processes are terminal; none remain running.
  Formatting/diff, generator syntax and all 485 pinned source hashes pass.
  Generic DecodeFrameRecords does not publish RecordMaskHook; its native owner is
  the biped scan publisher. View-class/inference/resync shared observer routing,
  all remaining scan counters, optional accumulation, unsupported width semantics
  and whole-Film integration remain open. Full parity is incomplete; architecture
  remains deferred. See fixtures/context-frame-v41.md and READER_CONTRACT.md.

- NativeFilmReader observer delivery now happens at component publication sites,
  not after component completion. All shared component readers route their
  callback candidates through one publication method; legacy callers retain the
  same ordered diagnostics with no live observer installed. The reader resolves
  current hooks for every publication and records absolute-index counts at their
  actual read point, so callback histogram resets survive completion. Direct
  readers retain raw callback candidates even when no hook is installed.
  Actual native live-observer oracle: 256 sequences / 1280 reads, with mobility
  hooks enabling or disabling subsequent movement/reference hooks inside the
  same component, position callbacks taking/resetting the histogram, and final
  snapshot/reset. Exact evidence: 512 mobility, 256 movement, 145 position and
  83 reference callbacks; 143 position callbacks see nonempty histograms; disabled
  cases emit no references. Native generation passed (0.390s), focused Rust passed
  (0.04s). Full Theater passed 519 tests, zero failed, 43 ignored (64.08s), log
  `/private/tmp/halo-live-observer-suite.log`. Clippy lib/tests warnings denied
  passed (13.27s); all-features WASM passed (19.11s), including lock waits.
  Formatting/diff, generator syntax and all 485 pinned source hashes pass.
  No process remains. This closes the tested direct-read callback-time gaps;
  all 30 hooks under mutation, frame/record installation, remaining scan counters,
  world accumulation, whole-Film integration and oversized profile conversion
  remain separate acceptance work. Full parity is incomplete; architecture stays
  deferred. The prior entry's deferred-delivery caveat is superseded.

- Shared NativeFilmObserver and stateful NativeFilmReader now connect direct
  component attempts to NativeFilmContext reader/frame factories. Direct readers
  inherit shared hooks and copy scalar profiles; scan-frame readers copy profiles
  with no inherited observer. The new context-reader-sequence-v41 oracle compares
  256 actual native factory sequences, 2048 direct reads and 2048 frame reads,
  statuses, end bits, ordered callbacks and live EMP hook replacement. The earlier
  1064-case reader-sequence oracle remains separate and passes the full suite.
  Additional Rust regressions check reentrant hook replacement/restoration,
  context isolation, cumulative counters and histogram snapshot/reset. Native
  generation passed (0.494s). Full Theater: 518 passed, zero failed, 43 ignored
  (69.97s), `/private/tmp/halo-context-reader-suite.log`. All-features WASM passed
  (57.52s, including lock wait); all 485 pinned source hashes passed. Clippy lib/tests with warnings denied passed after correcting three test-only
  is_multiple_of style issues. Formatting/diff and generator syntax pass.
  Hooks currently publish after component completion and counters merge before
  delivery. Exact callback-time counter visibility and capture-hook mutation
  within a component remain gaps. Whole-profile width conversion can reject
  irrelevant oversized widths on WASM. NativeFrameConfig is a reader factory,
  not a new frame traversal. World accumulation and existing Film-pass integration
  remain open. READER_CONTRACT.md records these limits; parity is incomplete and
  NEXT_PHASE.md remains deferred.

- Native scan-profile state and eager map-context construction are implemented.
  NativeScanProfile retains all movement/keyframe/MPP/grammar fields, including
  negative raw widths and nil-versus-empty maps. NativeSharedWidths preserves
  calibration-map aliasing across scalar profile copies. Precision descriptors
  now use u64 metadata, including 2^40 widths, without WASM truncation. Context
  replacement returns previous effective settings; separately cached declared
  corruption control survives replacement, including false versus absent flags.
  Raw precision replacement leaves grammar unchanged; layout installation refuses
  zero axes, preserves index width for gates <=4, and updates region/simulation.
  NativeFilmContext.for_map resolves profiles eagerly, selects forced/valid
  catalog layouts, switches simulation on layout presence, and warns once for
  incomplete keys. It does not implicitly install movement widths. Lazy context
  construction remains silent. Source profile rejection now uses an owned typed
  NativeProfileResolveError so mutating context methods can propagate it safely.
  Actual native oracle: 96 sequences with nine state snapshots each; source
  flags absent/false/true, map/forced layout controls, map alias mutations, raw
  MPP/precision/layout changes and restoration. Native generation passed (0.443s),
  focused sequence passed (0.27s); 128-case eager/warning-once regression passed
  (0.76s). Full suite 515 passed, 0 failed, 43 ignored (69.52s), log
  `/private/tmp/halo-native-scan-profile-suite.log`. Clippy lib/tests warnings
  denied passed (53.67s), all-features WASM passed (1m14s), including lock waits.
  Formatting/diff, generator syntax and all 485 pinned source hashes pass.
  Shared native observers, reader/frame construction and integration with Film
  passes remain open. Calibration map JSON preserves values, not alias graphs.
  Full v41 parity is incomplete; the three-layer architecture stays deferred.

- Added NativeResolvedFilmProfile and loaded v41 source resolution, plus the
  lazy NativeFilmContext.profile accessor. Profiles retain optional recorded
  format/major keys, full identity, map, highlight layout, keyframe framing,
  every movement invariant, slot personalization widths, MPP declarations and
  ordered typed format/build errors. Resolved absent MPP widths remain distinct
  from scan defaults; missing maps are not native profile errors. Identity reads
  are restricted to major 41. Cached-registry mutation does not affect source
  profile keys, and callers receive deep copies, including identity type tables.
  The actual native 64-case source-profile oracle compares every identity field
  and boundary, all subprofiles, exact float32 movement values, typed error
  membership and joined error text/order. It includes short/missing headers,
  truncated identity, known/unknown formats and builds, and optional map metadata.
  A JSON integer-versus-float comparison in the test was corrected to compare
  typed map values; production profile behavior required no oracle-driven fix.
  Final native generation passed (0.398s), profile test passed (0.44s), source-cache
  regression passed (0.28s), Clippy lib/tests warnings denied passed (44.23s) and
  all-features WASM passed (58.20s), including lock waits. Formatting/diff,
  generator syntax and all 485 pinned source hashes pass. Full suite was not
  repeated for this isolated additive metadata/context API; last full remains
  511 passing before the two added context/profile tests.
  Complete eager map-context grammar/warning composition, scan-profile setters,
  shared observers, reader/frame construction and Film-pass integration remain
  parity work. The queued three-layer architecture has not begun.

- Added NativeFilmContext's source-cache portion: borrowed loaded source/packet
  buffers; lazy chunk prefix, registry success/error, biped-band and i0-result
  caches; explicit mutable cached-registry access; owned imposed-layout copies.
  Cached registry fingerprint/named-slot snapshots remain unchanged after registry
  mutation, matching native stored identity rather than recomputing changed names.
  Missing sources preserve typed no-registry/no-chunks outcomes. Invalid forced
  layouts bypass detection; failed detected candidates/measurements remain available.
  Actual native context-cache oracle has 144 source contexts, 29 invalid forced
  layout successes and seven detected successes, repeated errors/pointer identity,
  missing/valid/short/compressed registries and visible registry mutation. Its
  native generation passed (0.643s); focused Rust test passed after final API
  adjustment (0.28s), Clippy lib/tests warnings denied passed (26.61s), all-features
  WASM passed (40.75s), including lock waits. Formatting/diff, generator syntax and
  all 485 pinned source hashes pass. The previous full suite remains 511 passing;
  it predates this isolated new context module/test and was not rerun unnecessarily.
  Updated READER_CONTEXT_PORT.md and facade inventory to identify the implemented
  caches precisely. This context is not yet composed with native profile setters,
  map-profile construction/warnings, shared observers, reader/frame creation or
  existing Film passes; those remain parity work. Rust borrows source buffers
  immutably rather than exposing unsafe external byte mutation. This does not
  begin the three-layer refactor; NEXT_PHASE.md remains deferred.

- Equipment-spawn source traversal now follows the native metadata prefix and
  packet framing, rather than a sorted chunk set and the generic strict packet
  index. `scan_equipment_spawn_events` preserves source order, counts one-byte
  list headers, stops at native terminal/malformed packet boundaries, and retains
  native packet ordinals. The supplied-head adapter uses the same traversal and
  indexes heads by packet source, independent of their input order. Film calls
  the source scanner and retains its optional failure in equipment_spawns_error.
  An initial fatal propagation broke two partial-Film regressions; this was fixed
  by retaining an empty equipment stream plus the native-source error while
  preserving other decoded data. Both regressions now pass with explicit checks
  for missing-prefix versus readable-prefix outcomes; full Film JSON retains it.
  Actual native oracle: 128 source layouts, 1,120 ordered events, 32 empty-prefix
  controls, plus 2,304 direct short inputs. Native raw reference reads panic on
  394 inputs; Rust counts bounded truncation and publishes no fabricated event.
  Native generation passed (0.516s). All three Film constructors/JSON and four
  downloaded films passed (six actual spawn events and all seven counters),
  focused run 3 passed (35.05s). Final full suite 511 passed, 0 failed, 43 ignored
  (64.86s), log `/private/tmp/halo-equipment-spawn-source-suite-final.log`.
  Clippy lib/tests warnings denied passed (16.57s); all-features WASM passed
  (23.33s), including lock waits. Formatting/diff, generator syntax and all 485
  pinned source hashes pass. Standalone permissive native source framing does
  not relax the general packet validation that Film performs before this scan.
  Full parity remains incomplete: the facade inventory still identifies native
  lazy FilmContext/error caching and shared mutable observer/profile APIs as
  partial, and the captured objective/recovery evidence gates remain open.
  The three-layer architecture in NEXT_PHASE.md has not begun.

- Native pickup early-return and zero-tail behavior now has a separate retained
  path: NativePickupRead stores raw references, refusal outcome, class/catalog,
  continuation, source length, logical endpoint and synthetic padding. The direct
  reader reproduces decoder-only counters; Film.native_pickups retains every
  source attempt, publication decisions and complete native scan counters.
  Source scanning uses native contiguous-prefix framing, native biped discovery,
  source order (not timestamp sorting), and an explicit no-chunks outcome.
  Padded reads stay out of bounded gameplay pickups. All three Film constructors
  and full JSON roundtrip retain native data without promoting synthetic catalogs.
  Actual native oracle: 3,328 direct cases, 204 accepted, 131 padded; two scans
  of 1,752 candidate packets each publish 140 without a band and refuse those
  140 off-band respectively, plus an empty-prefix control. All native counters,
  direct logical endpoints, source ordinals, payload ranges and times compare.
  Native generation passed (0.407s); full suite 509 passed, 0 failed, 42 ignored
  (76.34s), log `/private/tmp/halo-native-pickup-suite.log`. Clippy lib/tests
  with warnings denied passed (42.77s); all-features WASM passed (1m06s), including
  lock wait. Formatting/diff, generator syntax and all 485 pinned source hashes pass.
  Next audit: equipment spawn source framing/counters use a set-derived chunk
  range and generic packet index in head_observations.rs; compare metadata-order
  and terminal framing to native before changing it. Native raw reference readers
  can panic on truncated input, so zero-padding is not an assumed replacement.
  Full v41 parity remains incomplete; NEXT_PHASE.md is still deferred.

- Native zoom zero-tail behavior is now exposed separately from bounded state:
  NativeZoomRead preserves raw references, source length/logical padding, slot/level
  and individual recordedness checks. Film.native_zoom_events retains native
  source output through all constructors and JSON; padded reads do not enter the
  bounded zoom_events list. Shared head decoding keeps the bounded cursor policy.
  Actual native oracle: 2,304 direct inputs, 448 admitted heads (240 padded), and
  308 source events with exact tied-time order, packet ordinals and payload ranges.
  Includes config-clear direct acceptance versus source-family rejection.
  Native generation passed (0.361s); focused tests passed (0.06s). Full suite:
  507 passed, 0 failed, 42 ignored (69.18s), log `/private/tmp/halo-native-zoom-suite.log`.
  Clippy lib/tests (41.26s), all-features WASM (1m00s), formatting/diff, generator
  syntax and all 485 source hashes passed. Vehicle event truncation was audited
  first and already has typed failure semantics; no redundant replacement added.
  Full v41 parity remains incomplete; architecture stays deferred.

- Native translocator source scanning is verified against 128 actual reference
  scans: 1,000 ordered events, including complete default-box positions and padded
  references, plus 32 empty-source controls. Exact timestamp-tie order and source
  chunk/packet/payload references match. Cases cover chunk numbering gaps,
  duplicates/reordering, packet filters, missing references, type-7 termination,
  zero-size/oversized packet stops and trailing partial headers. Native provenance
  wrapper projects identically to ScanTranslocatorTeleports. Native generation
  passed (0.493s), focused Rust passed (0.06s), Clippy lib/tests passed (8.50s), and
  generator syntax, formatting/diff and all 485 reference hashes pass. No production
  changes, so the preceding full 504-test/WASM run remains the broader baseline.
  Captured gameplay and full parser acceptance remain separate; refactor deferred.

- Fixed a native translocator data-availability gap without promoting padded
  actors into bounded events. New decode_native_translocator_head and source
  scanner preserve the reference's zero-tail admission and logical end, source
  length/padding extent, packet/ordinal and reference-completeness witness.
  Film.native_translocations retains these outputs; map assembly supplies map
  context, while bounded translocations remain unchanged. Native vector bounds
  are checked after reading all axes. The 2,048-case native oracle accepts 384
  heads, including 64 incomplete references and 120 padded reads. Existing 632
  map/profile cases and captured teleports also compare native position values.
  All three constructors/export retain a padded head without creating a bounded
  event. Native generation passed (0.454s); four focused tests passed (0.05s).
  Full Theater suite: 504 passed, 0 failed, 42 ignored (70.13s), log
  `/private/tmp/halo-translocator-padding-suite.log`; Clippy lib/tests (35.39s),
  all-features WASM (54.25s), format/diff, generator syntax and 485 source hashes
  passed. Nonfinite injected maps and broader source/publication contracts remain
  separate audits. Full v41 parity remains incomplete; architecture deferred.

- Positive managed-property retention through both Film constructors is verified:
  54 native source cases, 5,391 readings, 5,404 ordered callbacks and 171 broken
  records. Assertions include rejected-attempt callbacks, source ordinals before
  filtering, exact wire times, candidate starts and full Film JSON roundtrips.
  Uses the actual source scanner's discovered band and retained bootstrap bytes;
  unencodable internal registry holes remain standalone tests. An initial fixture
  incorrectly placed a type-7 terminator before deltas and was rejected by the
  positive-data assertion; corrected to unrelated type 6. Native generation passed
  (0.497s), constructor comparison passed (12.40s), Clippy lib/tests passed (8.59s),
  and generator syntax, formatting/diff and 485 source hashes pass. Generator
  dependency/order registered. No production changes; no redundant full-suite or
  WASM run. Captured gameplay and broader all-data/context gates remain separate;
  architecture stays deferred.

- Catalog float32 conversion now has adversarial native validation: 1,601
  file-loader cases, including exact rational midpoints and nearby decimals across
  every finite exponent band, subnormal boundaries, signed zero and overflow.
  Rust matches all 1,598 accepted bit patterns and three refusals. A float64-first
  control differs on 531 accepted cases, confirming this fixture tests rounding
  behavior that ordinary coordinate samples miss. Current direct f32 decoding
  already passes; no production change needed. Generator is registered and fixture
  provenance retained. Native test (0.559s), focused Rust (0.06s), Clippy lib/tests
  (9.00s), format/diff, generator syntax and 485 reference hashes pass. No full
  suite/WASM rerun for this test-only change. Broader parser parity stays active;
  architecture deferred. This evidence is not exhaustive decimal enumeration.

- Optional native catalog lookup is now exposed by lookup_loaded_film_map.
  MissingCatalog preserves the nil-receiver sentinel separately from a present
  catalog's UnknownMap(name); is_unknown_map_bounds recognizes both, excluding
  loader failures. The wrapper returns an owned entry while existing borrowed
  lookup stays available. Expanded the existing 328-case oracle with 3,280 native
  calls (19 successes, 2,030 bare sentinel errors, 3,261 unknown-bounds errors),
  checking ownership and empty names; all previous fixture fields are unchanged.
  Native test passed (0.494s), both Rust catalog tests passed (0.07s), Clippy
  lib/tests passed (24.37s), all-features WASM passed (38.60s), and format/diff plus
  485 reference hashes pass. No parser/loader path changed, so the preceding full
  500-test run remains the broader baseline. Platform error text, filesystem and
  exhaustive float32 rounding gates remain open, along with full v41 parity.

- Corrected stale map-catalog gap notes: the existing 328-case implementation
  already supports caller-supplied loading/schema/lookup, including native JSON
  merging, nulls, Unicode and numeric edge cases. A mistaken replacement in this
  continuation was discarded; the original source and retained harness were
  reconstructed from the task's exact creation/patch commands, and its 328-case
  native fixture regenerated successfully. Restoration verification passed: 328 native cases, both focused loader tests,
  500 Theater tests (42 ignored; 74.12s), Clippy lib/tests (13.43s), all-features
  WASM (19.46s), formatting, diff checks and 485 pinned source hashes. The remaining gaps concern optional nil-catalog context, platform error
  wording, broader filesystem failures and exhaustive float32 rounding. Do not
  replace the existing loader or count its restoration as a new parser feature.

- Native map-entry projections are now public on FilmMapBounds: effective region
  index width, absolute precision, i0 layout and quantization range. Descriptor
  construction and world precision installation share these helpers. The native
  256-entry oracle covers exact float bits (including negative zero), historical
  zero-width default, nonzero regions, wide fields and both valid/invalid layouts.
  Focused Rust comparison passed; Clippy lib/tests (12.93s), all-features WASM
  (19.03s), formatting, diff checks and 485 pinned source hashes pass. Full Theater
  regression passed: 500 passed, 0 failed, 42 ignored in 67.60s; log
  `/private/tmp/halo-map-entry-regression.log`.
  Catalog loading/schema checking and non-null lookup already exist in
  map_catalog.rs; see the correction below for remaining error/context gaps. No architecture refactor started.

- Positive navpoint source-to-Film retention is verified through both configured
  constructors: 64 native source cases, 2,313 accepted readings and 2,374 ordered
  callbacks (491 keyframe, 1,883 delta), including broken walks. Complete scans,
  source packet ordinals, candidate starts, projected timestamps and Film JSON
  roundtrips match. The native oracle parses equivalent bootstrap bytes; an
  initial injected trailing-empty-name discrepancy was corrected in the harness.
  Focused constructor regression passed (12.76s), and Clippy lib/tests with
  warnings denied passed (9.09s); all 485 pinned source hashes, formatting and
  diff checks pass. No production code changed. This closes the
  positive navpoint constructor test gap; captured bomb/VIP/recovery evidence,
  general context/shared observer APIs and constructor-wide order remain open.
  Full parity remains incomplete and the architectural refactor stays deferred.

- Incomplete profile constructor warning is implemented and its native boundary
  verified: 128 cases compare 44 WARN records and 84 silent controls, including
  format/build/missing-identity combinations, known formats without MPP widths,
  and no-registry suppression. Map-only issues do not warn. Exact joined error
  text and format/build attributes match. Map-aware Film construction invokes
  it after profile resolution; pure resolution and base decoding remain silent.
  Native passed (0.605s); focused Rust passed (0.02s). Full Theater suite passed:
  498 passed, zero failed, 42 ignored, 65.04s; Clippy passed (13.04s), WASM passed
  (19.33s with lock wait). Log /private/tmp/halo-profile-warning-regression.log.
  No checks remain live.
  All485 source hashes and format/diff passed. See fixtures/profile-warning-v41.md.
  Full constructor-wide observation ordering and general shared/lazy context
  APIs remain open, along with the other v41 acceptance gates. Refactor deferred.

- Positive vehicle death/occupancy source coverage is VERIFIED: 152 native cases
  preserve all prior 120 rows. New wire controls explicitly bind vehicle512/gen2
  and biped600/gen1; true/false deaths and attached/detached parent states have
  exact native actor/time assertions, including no false-death publication.
  New sources yield32 vehicle deaths and64 occupancy readings plus one other
  biped death from composed inputs. Rust runs its actual chronological march on
  these32 cases and compares all facts, coverage maps, event/located counts and
  calibration retention, then publishes matching source logs (255 INFO/19 WARN
  across the fixture). Native passed1.211s; focused Rust passed1.06s. No production
  parser changed. Clippy passed (8.38s), all485 source hashes and format/diff
  checks passed. No checks remain live; full suite/WASM were not repeated for
  this test-only extension. This closes positive source death/occupancy evidence;
  other source failure/observer contracts and broader captured-film gates remain.
  See fixtures/vehicle-source-logs-v41.md and VEHICLE_SOURCE_CONTEXT_AUDIT.md.
  Full parity remains incomplete; architecture remains deferred.

- Positive vehicle source aim coverage is VERIFIED: 120-case fixture preserves
  every field in the original 88 and adds 32 recognized biped aims plus 32
  out-of-band controls. Native source packet ordinals, wire times, raw yaw/pitch
  and positive summary counts all match. New cases also compare 90 creations,
  125 position projections and 64 events. All source logs match (192 INFO,
  18 WARN total), and native MPP restoration remains checked in every case.
  Native generation passed (0.991s); focused Rust passed (0.64s). No production
  code changed. Full suite/WASM were not repeated for this fixture-only extension;
  latest production validation remains the preceding 497-test/WASM run.
  Aim-only context audit confirms its vitality readers use fixed widths/constants
  and publish no callbacks; see READER_CONTEXT_PORT.md. Positive vehicle-death/
  occupancy summaries, other source failures and broader context/acceptance gates
  remain open. Full parity remains incomplete; architecture stays deferred.

- Team-walk attempts are now retained in FilmPlayerTeams before team acceptance.
  Attempts hold full optional KeyframeRecord, start/slot, raw reading and source
  packet/ordinal when available. Source packets are indexed before keyframe
  filtering; payload-only calls retain unavailable provenance. Full fields,
  stop reasons and diagnostics roundtrip through Film. Native oracle now has
  2,048 cases, 15,079 independently compared attempt boundaries/results, and
  2,914 ordered callbacks; 217 refused readings retain callbacks. Original
  1,536 rows retain all previous values. Five native cases exercise constructor
  and export retention, including nonempty callbacks in cases 1537/1545.
  Focused test passed (2.16s) before the two additional constructor checks;
  native generation passed (0.558s). Full regression passed: 497 passed, zero
  failed, 42 ignored, 67.48s; log /private/tmp/halo-team-attempts-regression.log.
  WASM passed (66s including lock wait); format/diff and all 485 hashes passed.
  Clippy requested Copy dereference instead of clone for FilmPacket; that
  semantics-preserving cleanup is applied and Clippy passed (23.60s). No checks
  remain live. Full suite/WASM ran before that Copy-only cleanup.
  Initial failures were test maintenance: accepted-only fixture comparison
  included new trace, and an added absent-registry control gained a component.
  Both were corrected without changing parser acceptance to match the fixture.
  See fixtures/player-teams-context-v41.md. General observer/context APIs and
  remaining captured objective/recovery gates stay open. Architecture deferred.

- Managed-player team context correction: Rust no longer hardcodes default start
  140 and trailing 32/64-bit guards in the native team reader. New context APIs
  use both keyframe profile widths, native padded traversal and calibrated/stub
  component spans before checking actual four-bit source values. Both configured
  Film constructors retain the effective-context scan after corruption policy.
  Native oracle now has 1,536 cases; original 1,024 rows unchanged. New rows
  compare 2,882 accepted and 1,732 refused direct reads plus all aggregate counts
  and assignments. Native cases 1025/1029/1031 prove constructor/export retention
  and differing default-profile outcomes. Native generation passed (0.592s),
  focused Rust passed (0.81s), Clippy passed (12.80s), WASM passed (19.30s with
  lock wait), all 485 hashes verified, format/diff passed. Full Theater regression
  passed: 497 passed, zero failed, 42 ignored, 65.72s; log at
  /private/tmp/halo-team-context-regression.log. No checks remain live.
  See READER_CONTEXT_PORT.md and fixtures/player-teams-context-v41.md. This
  closes team width/context handling, not shared observer API or complete
  team-walk trace retention. Full parity remains incomplete; refactor deferred.

- Shared observer declaration audit is reconciled: all 30 callbacks and all 12
  counter/map fields have Rust representations and native publication evidence.
  OBSERVER_STATUS.md contains the exact mapping; the manifest no longer implies
  that the other callback declarations remain unmapped. General mutable context
  equivalence and integrated caller retention are still open.

- Identity registry observations now include both INFO summaries and all seven
  conditional WARN records, consuming final unresolved coverage after scoreboard
  and deduction. The native 1,024-case registry fixture captures 2,048 INFO and
  3,655 WARN; every prior output field is unchanged. All warning conditions have
  positive and negative cases. Document assembly invokes the logger after player
  construction/origin resolution, before combat layers. Native generation passed
  (0.994s), focused Rust registry test passed (4.42s), Clippy passed (12.07s),
  WASM passed (17.64s including lock wait), all 485 source hashes verified.
  Fixture provenance: fixtures/identity-registry-logs-v41.md. Full Theater suite
  passed: 497 passed, zero failed, 42 ignored, 63.30s; log at
  /private/tmp/halo-identity-logs-regression.log. Format/diff checks pass. No
  checks remain live; no architecture refactor is started.
  Full parity remains incomplete: shared profile/context APIs, other integrated
  caller retention, source edge cases and positive captured objective/recovery
  coverage remain acceptance gates.


- Vehicle-local MPP restoration and creation-failure diagnostics are VERIFIED.
  Source oracle now has 88 cases: every prior field of the original 80 remains
  unchanged under varied inherited/calibrated MPP widths. Native full profile
  equality holds on 50 successes, 30 no-band exits and eight creation failures;
  Rust uses the same recorded-width resolver with immutable input precision.
  Eight truncated registries (33..40 blocks) compare exact creation WARN text,
  while Rust retains its issue and partial slot count. Native passed (0.884s),
  focused Rust passed (0.42s), full suite passed (497 passed, zero failed,
  42 ignored, 60.27s), Clippy passed (13.02s), WASM passed (4.99s), all 485
  source hashes and format/diff checks passed. Log:
  `/private/tmp/halo-vehicle-context-failure-regression.log`. No checks live.
  Detailed reachability audit: VEHICLE_SOURCE_CONTEXT_AUDIT.md. Required map/
  registry/loaded-byte inputs differ from native lazy IO; these distinctions
  are documented without claiming general source/context parity. Remaining:
  other source failure wording/lazy-source behavior, shared observer/profile API,
  nonzero source aim/vehicle-death/occupancy evidence, and broader acceptance gates.
  Full parity stays incomplete; architecture remains deferred.


- Vehicle source delta-bearing validation is VERIFIED: 80 cases preserve all old
  48 cases and add 32 matching-map creation/position inputs. Rust compares 89
  complete creations, all creation stats (103 anchors), 122 world-position
  projections, 64 complete events, aim outputs and 138 ordered logs (129 INFO,
  nine WARN). Native generation passed (0.776s), focused Rust passed (0.20s),
  Clippy passed (8.77s), all 485 source hashes and format/diff checks passed.
  Initial test-input mismatch used vehicle-filtered deaths where the diagnostic
  counts all entities; the fixture now retains full native march deaths, including
  one nonvehicle death, and verifies native march stats under reinstated MPP.
  Production code did not change this turn. Full suite/WASM were not rerun; the
  prior 497-test and WASM passes cover unchanged production code. No checks live.
  Source test injects native shared-march facts for composition, while traversal
  has separate captured/generative oracles. Nonzero source aim/vehicle-death/
  occupancy counts, other source failure logs and general context equivalence
  remain open. Full parity remains incomplete; architecture remains deferred.


- Vehicle source no-band/summary observations and missing-biped aim failure are
  VERIFIED on 48 actual native decodeFilmVehicleScan cases. Thirty early returns,
  eight successful scans without a biped band and ten with one compare success
  flags and ordered logs (66 INFO, 8 WARN). Missing biped-band aim now remains in
  issues as unavailable; the additive failure does not suppress vehicles. The
  constructor enables source observations with match ID and emits summary after
  the shared death diagnostic; standalone scanner signatures are unchanged.
  Native passed (0.352s), Rust passed (0.07s), full Theater suite passed (497
  passed, zero failed, 42 ignored, 65.30s), Clippy passed (14.71s), WASM passed
  (5.65s), all 485 hashes and format/diff checks passed. Full log:
  `/private/tmp/halo-vehicle-source-regression.log`. No checks remain live.
  Fixture uses synthetic keyframe-only data and the retained Bazaar v41 registry
  (`film-player-table-source-input-v41.zlib`, verified byte-identical). Nonzero
  delta summary counters, other source failures and shared context gates remain
  open. Native fixture generation initially used wrong bit order; corrected to
  MSB-first before consuming results and added explicit healthy biped controls.
  Full parity remains incomplete; NEXT_PHASE.md remains deferred.


- Successful vehicle death-read source observation is VERIFIED: 512 actual native
  logger cases compare 256 INFO and 256 WARN records, including ti=40-only death
  filtering, absent histogram keys, unrelated-archetype losses, absent/default
  calibration and match IDs. Native generation passed (0.739s), Rust comparison
  passed (0.13s), full Theater suite passed (496 passed, zero failed, 42 ignored,
  66.55s), Clippy passed (14.93s), WASM passed (5.76s), all 485 source hashes and
  format/diff checks passed. Full log:
  `/private/tmp/halo-vehicle-death-observation-regression.log`. No checks remain
  live. The map-aware Film constructor emits the diagnostic only after successful
  vehicle scanning and uses the existing shared march result. No second traversal
  or publication change. Remaining vehicle source gaps: absent-band/creation/
  position/event/aim/death-failure logs, final scan summary and shared context
  behavior. Native march errors originate in Registry(); the Rust map constructor
  has already resolved a registry and separately rejects invalid encodings/majors.
  Audit that distinction before modifying constructor failure propagation.
  Full parity remains incomplete and NEXT_PHASE.md stays deferred.


- Vehicle heading-source and ride-resolution observations are VERIFIED. Native
  generation passed (1.143s); existing 1024 track and 512 combined ride fixtures
  retain every previous field. Heading logs cover 24776 samples (4611 film,
  8717 velocity, 11448 absent; 6000 velocity with unpublished roll). Ride logs
  have 345 INFO, 75 WARN and 167 silent cases. Rust comparisons passed (1.39s,
  4.39s), full Theater suite passed (495 passed, zero failed, 42 ignored,
  68.04s), Clippy passed (13.95s), WASM passed (5.29s), all 485 source hashes
  and format/diff checks passed. Full log:
  `/private/tmp/halo-vehicle-provenance-regression.log`. No checks remain live.
  Document attachment now emits vehicle coverage, heading source, ride resolution
  in native order. Remaining build_vehicles gaps are source diagnostics (including
  death-mask counts and match IDs) and shared profile/observer context behavior.
  `FilmMarchFacts` already retains death-mask histograms, event/located packet
  counts and calibration/default status; vehicle publication consumes deaths by
  archetype. Audit source-stage placement before adding those diagnostics.
  Full parity remains incomplete; NEXT_PHASE.md stays deferred.


- Palette and vehicle coverage observations are VERIFIED. The 1024-case ability
  fixture preserves all prior fields and adds actual native palette-stage logs:
  1014 unclassified, ten classified. Vehicle logger has 513 native cases with
  1536 INFO, 937 WARN and nil silence; only Go's unspecified map iteration is
  normalized. Document logs the palette between translocations and impulses,
  and vehicle coverage after tracks/cycles attach. Native generation passed
  (palette 1.110s, vehicles 0.678s), Rust comparisons passed (1.51s and 0.10s),
  full Theater suite passed (495 passed, zero failed, 42 ignored, 63.23s), Clippy
  passed (13.96s), WASM passed (5.28s), all 485 source hashes and format/diff
  passed. Full log: `/private/tmp/halo-palette-vehicle-logging-regression.log`.
  No checks remain live. Next vehicle observation gaps: heading-source and
  ride-resolution logs in native attachVehicles, plus source death diagnostics.
  Existing Rust heading rules and ride tallies already retain their inputs;
  verify observations using the existing 1024 track/512 combined oracles.
  Full parity remains incomplete; NEXT_PHASE.md is still deferred.


- Ability coverage observations are VERIFIED against 1024 native cases: 2474
  INFO and 598 WARN records, including actual native impulse/charge document
  stage scan gates. All previous fixture fields are unchanged. Rust retains
  boolean attributes and raw input counts for scans that never ran; identity
  logs precede equipment/translocation logs, impulse/charge logs follow them.
  Native passed (1.284s), focused Rust passed (1.57s), full Theater suite passed
  (494 passed, zero failed, 42 ignored, 65.79s), Clippy passed after moving an
  impl before its test module (20.28s), WASM passed (20.30s), source integrity
  passed all 485 files, and format/diff checks passed. Full-suite log:
  `/private/tmp/halo-ability-logging-regression.log`. No checks remain live.
  Palette logging and broader observer/captured acceptance gaps remain open;
  the three-layer architecture stays deferred in NEXT_PHASE.md.


- Empty-inventory coverage logging is now VERIFIED: 1024 native cases compare
  complete publication/death attribution plus 887 INFO logs and 137 silent
  controls. Every prior fixture field is unchanged. Native generation passed
  (0.530s), Rust comparison passed (0.62s), Clippy lib/tests with warnings denied
  passed (9.00s), WASM all-features passed (13.17s including lock wait), source
  integrity passed all 485 files and format/diff passed. No checks remain live.
  Corrected stale inventory notes: the legacy guard and full-constructor
  independent inventory/weapon failures were already fixed and tested, as were
  later movement/navpoint errors. Remaining setup/context/captured gates are
  distinct; full parity stays incomplete and architecture remains deferred.

- Re-audited inventory ammo/grenade helper contracts and dead-reading publication
  against existing Rust consumers and native fixtures. Partial ammo retains
  optional fields, exact gauges/raw quanta, cursor and completion; grenade
  all-zero and disagreement rules are preserved. Evidence: 256 keyframes,
  2048 partial ammo parses and 1024 inventory publication/death-marking cases,
  already passed in the latest full suite. Added precise mappings without
  promoting the remaining broader error-path gates. Dead-reading source stays
  partial for its missing conditional INFO log. No parser behavior changed;
  full parity and the architectural refactor remain unfinished/deferred.

- Neutral-death filtering now emits the native conditional INFO removal log.
  Extended native fixture preserves all previous 256 input/output fields and
  adds a nonempty all-retained identity-precedence control. All 257 cases compare
  complete filtered output plus 245 actual log records and 12 silent controls.
  Initial fixture loading exposed a nil points array in the added input; the
  harness now constructs an empty array consistently with existing cases.
  Native regeneration passed (0.408s), Rust comparison passed (0.03s), Clippy
  lib/tests with warnings denied passed (8.85s), WASM all-features passed
  (12.92s including lock wait), all 485 reference hashes and format/diff passed.
  No checks remain running. Death-clock candidate/refinement and common published
  track source mappings were also reconciled against existing native oracles.
  The full 494-test suite predates this isolated log addition. Parity is incomplete.

- Reconciled closures.go, closures_respawn.go, lives.go and lives_decoupe.go
  against existing Rust algorithms and retained outputs. The 1024-case closure
  oracle checks owners/report/window/naming, and the 1024-case lifetime oracle
  checks scaffold, death-time conversion, refinement and fallbacks. Designated
  ambiguous lives, cause/provenance, refusal counters and document fallback
  publication are retained. All four source mappings now point to their existing
  implementations; no parser behavior changed. These tests passed in the latest
  494-test suite, so no redundant regression was run. Full parity remains open.

- Added native death-clock narrow-margin warning at ReplayCoverage::new's
  buildCoverage-equivalent point. bridge_health remains a pure accessor. Native
  512-case capture verifies 225 WARN records and 287 silent controls, including
  zero matches/runner-up, exact two-to-one margin and one-below margin. Level,
  message and all five attributes compare exactly. Generation passed (0.552s),
  focused comparison passed (0.02s), full Theater suite passed 494/0/42 ignored
  (68.36s), Clippy lib/tests with warnings denied passed (32.05s), WASM all-features
  passed (36.08s including lock wait), format/diff passed. No check is running.
  RosterEntry and seat assignment source mappings were reconciled against existing
  code and native tests. coverage_bridge.go is ported-v41; identity_registry_health.go
  stays partial for its separate logRegistry messages. Full parity remains open.

- Equipment-change coverage logging is now VERIFIED against 1024 actual native
  slog records. Document assembly emits the INFO message and all twelve fields
  before translocation coverage, preserving counters measured before the track
  filter. All earlier fixture fields are unchanged. Shared test-only tracing
  capture also passed the 1024-case translocation regression. Native generation
  passed (0.488s), equipment Rust comparison passed (0.18s), translocation
  regression passed (0.19s), Clippy lib/tests with warnings denied passed
  (13.07s), WASM all-features passed (19.06s including lock wait), and all 485
  reference source hashes passed. Provenance: fixtures/replay-equipment-publication-v41.md.
  Current full suite remains 493 passed before this logger addition; the focused
  checks above validate the addition. No running checks remain. Global logging,
  other retained-output contracts and captured positive gates remain open.

- Reconciled five existing publication sources against native declarations and
  actual Rust producer/document wiring: equipment changes, pickups, ground-pad
  types, vehicle types and vehicle coverage. No missing fields were found on
  these inspected paths. Evidence includes 1024 equipment cases, 512 pickup
  cases, 512 pad/cycle cases plus four captured-derived layers, 1024 vehicle
  track/coverage cases and 512 combined publications, all run by the latest
  493-test suite. Equipment/vehicle coverage entries stay partial for absent
  native logs; the three data-only source entries are now ported-v41. Details
  and limits are in DATA_CONTRACT_INVENTORY.md. No code behavior changed, no
  additional test rerun was needed, and full parser parity remains incomplete.

- Captured march validation is now TERMINAL SUCCESS: session 64835 passed all
  four films in 1534.02s. Oddball matched 182 deaths and 144 occupancy readings;
  Bazaar/Aquarius each matched 1/0 and Bandit matched 125/6. Assertions include
  complete records/counters, calibration, retained walk policy and JSON export.
  Its fixed binary predates later round/source/logging additions, which do not
  affect march. No captured march process remains running. Current full Theater
  regression also passed: 493 tests, zero failures, 42 ignored (60.30s), including
  those later additions. See CURRENT_VALIDATION_GAPS.md for exact logs and scope.
  Source reconciliation confirms existing scope and ability publication fields:
  614400 scope queries in 1024 cases, and 1024 ability cases covering charges,
  impulses, folding, palettes and labels. Scope source is now ported-v41; ability
  source entries remain partial for native logging. No parser code changed in
  this reconciliation. Remaining all-data acceptance gates are still open.

- Added scan_replay_film_player_table over FilmSource, reusing the existing
  registry/identity/slot decoders. A new 19-case native source oracle verifies
  the complete table, Lue predicate, all five refusal causes and metadata-based
  registry selection (including first duplicate wins). Three positive one-seat
  outputs and all JSON roundtrips pass. Native generation passed (0.505s), Rust
  comparison passed (0.92s), Clippy lib/tests with warnings denied passed (20.69s),
  WASM all-features passed (24.67s including lock wait). Provenance and captured
  input SHA-256: fixtures/film-player-table-source-v41.md. The source file stays
  partial for native logs/global metrics; existing Film assembly is unchanged.
  Two comment-only schema chronicles are now accurately marked reference-only.
  Captured march session 64835 is confirmed live, Oddball still outstanding.
  No refactor or commits; full v41 parity remains incomplete.

- Closed the translocation coverage logger gap: document assembly now emits
  the native INFO message and six attributes through tracing, including
  published-minus-positioned. Extended the existing native fixture to capture
  actual slog output; all previous fields in 1024 cases are unchanged. Rust
  captures and compares exactly one actual tracing event per case. Native
  generation passed (0.425s), Rust comparison passed (0.19s), Clippy lib/tests
  with warnings denied passed (13.31s), WASM all-features passed (19.65s including
  lock wait). Source integrity passed all 485 pinned files. This closes this
  logger contract, not all native logs or their global cross-layer order.
  Full Theater regression immediately before the logger change passed 492,
  failed zero, ignored 42 (67.71s), including recent round/source helpers.
  Captured march session 64835 remains running on its original binary with
  Oddball outstanding. No architecture changes or commits; parity is incomplete.

- Reconciled objective utilities and weapon/translocation publication against
  the pinned source and existing 1024-case publication fixtures. Utility and
  weapon-change entries now map to existing Rust implementations. Translocation
  stays partial: data fields and counters are retained, but native INFO logging
  is not emitted. No parser behavior changed in this audit checkpoint. Details
  are in DATA_CONTRACT_INVENTORY.md. Captured march validation remains running
  in session 64835, with Oddball outstanding; full parity remains incomplete.

- Corrected the prior source-wrapper audit: SlotIdentityResolved was already
  implemented and tested. Added only the missing SlotIdentityFromDeaths wrapper
  as statborg_source_identity_by_deaths, retaining source diagnostics/truncation.
  Native four-case fixture extension preserves all previous fields and directly
  compares death-only positive/negative results, including its intentional
  difference from totals-preferred combined resolution. Generation passed
  (0.394s), Rust comparison/JSON passed (0.01s), Clippy lib/tests passed with
  warnings denied (9.00s), WASM passed (13.16s including lock wait), formatting
  and diff checks passed. Source audit correction is in DATA_CONTRACT_INVENTORY.md.
  Captured march session 64835 remains live; Bandit has advanced with 125 deaths
  and 6 occupancy readings and no reported failure. Oddball remains outstanding.
  Do not restart the fixed-binary captured run. Full parity remains incomplete.

- Round identity query APIs VERIFIED: from_flat, origin, at_round, resolved,
  rounds and named_count. Native 256-case oracle covers 67584 lookup tuples,
  empty round maps, empty values and independently stored origins. Native
  generation passed (0.411s); Rust comparison/flat JSON roundtrip passed (0.19s);
  Clippy lib/tests with warnings denied passed (20.94s); WASM passed (24.86s
  including lock wait); formatting/diff checks passed. Existing identity and
  completion algorithms were not changed. Source inventory reconciles rounds
  and elimination implementations; death-source convenience wrappers remain
  partial. Provenance: fixtures/round-identity-queries-v41.md. Captured march
  session 64835 is still live on its original binary; do not restart it.
  Full v41 parity remains incomplete.

- Round helper APIs now VERIFIED: real_set, outliers and the native nominal
  diagnostic threshold (27). Extended the native 1024-case rounds fixture with
  actual native helper calls while preserving every original field exactly.
  Added results include 6736 excluded records in 485 positive cases. Native
  generation passed (0.569s); Rust comparison passed (0.94s); Clippy lib/tests
  with warnings denied passed (13.75s); WASM passed (20.20s including lock wait);
  formatting/diff checks passed. Both source entries now map to their existing
  full producer plus these helpers. Full parity remains incomplete.
  Captured march session 64835 is still confirmed live against its fixed binary
  compiled before these independent helper additions; it was not restarted.
  Its result will establish march-policy parity, not a new full-suite run.

- While captured validation remains live, reconciled objective source entries:
  rosterfit.go and slotidentity_residue.go already have implementations and
  native oracle coverage, including 1024 complete residue cases. Updated stale
  pending labels after checking guards, copy behavior, uniqueness and provenance.
  rounds_decision.go retains all five native decision fields and has 1024-case
  evidence; marked partial-v41 because RealSet has no dedicated Rust method
  (the public real vector supplies the same data). Details and limits are in
  DATA_CONTRACT_INVENTORY.md. No Rust edits or duplicate tests were started.
  Diff checks passed. Session 64835 remains confirmed live; retain the existing
  captured march run and log. Full parity is incomplete.

- Full Theater regression after roster/life exports and march policy retention
  VERIFIED: 491 passed, zero failed, 42 ignored, 53 non-Theater filtered in
  68.71s; session 23183 exited 0. Log:
  /private/tmp/halo-parity-after-march-policy.log. Formatting/diff checks passed.
  Extended captured march assertions to compare retained policy against native
  Config.Profil.Grammaire and roundtrip the complete march result from Film.
  The four-film integration test is still RUNNING in session 64835; do not
  restart it. Log: /private/tmp/halo-march-policy-corpus.log. Bazaar and Aquarius
  have advanced without failure; longer Bandit/Oddball assembly remains live.
  Previous equivalent captured runs took about 22 minutes. No Rust edits while
  validation is running. This is not yet a passing captured-policy gate.

- Closed a concrete march-result retention gap: FilmMarchFacts.walk_policy now
  retains the eight-view bound, generation strictness and simulation completion
  used alongside calibrated encoding. Previously the last two values existed
  only in caller arguments. Native loaded-march oracle passes all 256 cases
  (8.68s), including 201 delta cases and 55 empty controls; flags compare against
  native Config.Profil.Grammaire. Full result JSON and legacy omission checks
  pass without changing deaths, occupancy or counters. Clippy lib/tests with
  warnings denied passed (23.45s), WASM passed (36.09s including lock wait),
  formatting/diff checks passed. No process remains. Old exports retain None,
  not invented defaults. Broader profile/context parity remains incomplete.

- Native FilmFacts cache scope audit recorded in FILM_FACTS_CACHE_SCOPE.md.
  The pinned REPLAYINPUTS25 format intentionally omits nine short player fields
  and the session token; it is not an all-source fidelity oracle. Confirmed all
  eleven placement statistics and nested calibration values have retained Rust
  fields. Mapped object-death counters to Film.native_march_facts while leaving
  full profile snapshot equivalence explicitly open. Cache entries remain
  pending; no parser code or acceptance criteria changed. Diff checks passed.
  This prevents consumer-cache agreement from being used as proof of complete
  recording parity. Full parity remains incomplete; no process is running.

- External roster adapter RosterXUIDsOf is now ported as replay_roster_xuids.
  Native strict decimal/nonzero u64 conversion preserves duplicates and order;
  it must not use the separate permissive weapon-index string parser. Native
  256-list oracle passed generation (0.519s) and Rust comparison (0.01s), covering
  overflow, signs, whitespace, Unicode, NUL, leading zeros and duplicate inputs.
  Clippy lib/tests passed with warnings denied (19.93s), WASM passed (23.64s
  including lock wait), formatting/diff checks passed. No process remains.
  Provenance: fixtures/roster-strings-v41.md. This converts external evidence;
  it does not establish a recorded player identity. Full parity remains open.

- Named-life export is ported and VERIFIED: IdentityRegistryOutput::named_lives
  retains native XUID, match-clock bounds, cause and naming provenance. The
  death-naming gate, anonymous filtering, signed truncation/offset arithmetic
  and stable start/XUID/end order match 512 native cases with 10674 input lives,
  6396 outputs and 128 gate-disabled cases. Native generation passed (0.551s),
  Rust oracle/JSON roundtrip passed (0.11s), Clippy lib/tests with warnings
  denied passed (23.11s), WASM passed (36.22s including build-lock wait), and
  formatting/diff checks passed. No process remains. Provenance and limits:
  fixtures/named-life-export-v41.md. The original registry remains unchanged;
  this is the existing derived native export, not the deferred architecture.
  Full parity remains incomplete.

- Full regression after death-context and kill-position/opening ports VERIFIED:
  489 passed, zero failed, 42 ignored, 53 non-Theater filtered in 64.89s.
  Command: cargo test --lib theater::; exit 0. Log:
  /private/tmp/halo-parity-after-kill-placement.log. No process remains.
  Reconciled three stale pending entries after checking all native declarations:
  build_aim.go, document_aim.go and document_tracks.go. Existing native track
  evidence covers 1024 cases / 78336 samples / 42302 published points; complete
  document-schema serialization also passed in this run. Native health/shield
  accessors clamp before publication, so the Rust clamp is not a discrepancy.
  Mappings and evidence limits are in DATA_CONTRACT_INVENTORY.md. No production
  edits in this reconciliation. Full parity and ignored captured gates remain
  incomplete; the deferred architecture has not begun.

- Missing kill-position and opening-proxy entry points are ported. Independent
  native oracle compares all optional positions, ordering, shifts and seven
  report fields across 512 cases / 7168 kill references. Includes 512 opening
  sides rejected across replication-life bounds. Rust oracle passed (2.34s),
  shared shot-lookup regression passed (1.26s), Clippy lib/tests with warnings
  denied passed (22.54s), formatting/diff checks passed. Native generation
  passed (0.753s). WASM verification passed (14.28s); no process remains.
  Source provenance and exact counts: fixtures/kill-positions-v41.md. This
  ports native derived analytical APIs; opening estimates are not recorded
  engagement starts. Full v41 parity and the deferred architecture remain open.

- Missing native death-context analysis is now ported and VERIFIED against the
  pinned reference. build_replay_death_contexts retains all context fields and
  journal ordering, temporal named-life ownership, publication refusal, signed
  clock offsets, inclusive freshness/life bounds and strictly prior deaths.
  Independent Go-generated oracle: 512 cases / 5472 contexts, 8552 visible,
  6086 waiting, 324 out-of-sight classifications, 3958 distances and 56 empty
  cases. Native generation passed (0.683s); Rust full-output/JSON test passed
  (1.11s); WASM check passed (15.87s), Clippy lib/tests with warnings denied
  passed (40.98s), formatting/diff checks passed. An initial test-only build
  error used Default for IdentityTableLinks; corrected to its existing table
  composer. No production correction was needed after the oracle comparison.
  This exposes the existing native derived-analysis API, not recorded actions
  or the deferred architecture. Full parity remains incomplete. No process
  remains. Provenance: fixtures/death-context-v41.md.

- Identity-evidence retention audit VERIFIED: the player-layer result already
  retains all seven evidence fields; document publication is a projection, not
  an archive of that intermediate output. Added JSON roundtrip assertions to
  all 1024 native automatic-identity cases, retaining independent comparisons
  of decoded fields and scan failures. Focused test passed (0.46s); formatting
  and diff checks passed. No production behavior changed. Documented the
  caller-roster dependency in FILM_INPUT_RETENTION.md; do not cache this result
  unconditionally on Film. Full parity remains incomplete.

- Full Theater regression after provenance changes is VERIFIED: 487 passed,
  zero failed, 42 ignored, 53 non-Theater tests filtered out in 60.88s.
  Command: `cargo test --lib theater::`; terminal exit 0. Log:
  `/private/tmp/halo-parity-after-provenance-suite.log`. Ignored captured gates
  remain separate; this does not establish complete v41 parity.

- Reference integrity and scope audit: all 485 inventoried source files match
  pinned SHA-256 values. Added reference/verify_reference.py for repeatable
  verification; an isolated altered-source negative check reports exactly one
  mismatch with nonzero exit. Classified four revision/ files as reference-only
  after inspecting their source/hash/godoc tooling contracts; they do not decode
  recordings. Native parser revision values remain in scope. Other pending and
  partial entries are unchanged. Diff checks passed; no process remains.
  This confirms oracle source integrity, not completeness or full v41 parity.

- Position capture under live profile replacement is VERIFIED. Appended 512
  native sequences while preserving all original rows: 1024 cases / 16384 reads.
  Native profile installation alternates delta quantum and asserts unchanged
  private capture state. Rust compares every callback/end bit and accumulated
  position after each read while retaining world/slot/emitter state. Added cases
  include 256 accumulating sequences and 41 emitted delta samples. Native
  generation passed (0.529s), Rust sequence passed (0.54s), both other users of
  the shared test helper passed, and Clippy tests passed (17.76s). Formatting and
  diff checks passed. No process remains. Logs:
  `/private/tmp/halo-position-profile-swap-{test,clippy,calibrated,baseline}.log`.
  This verifies one concrete live profile transition; arbitrary observer aliases
  and other profile-field transitions remain separate. Full parity is incomplete.

- Full shared direct-hook name sweep added to reader-context sequence validation.
  All prior 512 rows remain byte-equivalent as JSON values. The oracle now has
  1064 cases / 6384 reads / 72 distinct component names / 6348 ordered callbacks,
  including 25 typed variants plus mobility and nonpublishing controls. All 38
  rejected reads retain their 38 callbacks. Native generation and Rust sequence
  regression passed; Clippy tests passed (7.63s), as did formatting/diff checks.
  No process remains. Logs: `/private/tmp/halo-reader-all-hooks-{test,clippy}.log`.
  READER_CONTRACT.md distinguishes direct dispatch from default creation,
  position accumulation and scan-mask paths. Source conversion audit also
  confirmed into_chunks already rejects out-of-range native chunk numbers.
  Full parity remains incomplete; positive v41 objective recordings and other
  integration/state contracts remain outstanding.

- Reader profile/observer sequence coverage expanded and VERIFIED. Added 256
  native cases for control-context, EMP timer, managed property and multiplayer
  property, preserving all original 256 rows exactly. Total: 512 cases, 3072
  reads, 3408 ordered publications; 34 rejected reads retain 34 callbacks.
  Native generation passed (0.402s); Rust sequence comparison passed (15.44s);
  Clippy tests passed (17.74s); formatting/diff checks passed. No production
  behavior changed and no process remains. Logs:
  `/private/tmp/halo-reader-sequence-expanded-{test,clippy}.log`.
  Updated READER_CONTRACT.md with exact coverage and remaining alias/state limits.
  Verified bundled reference v41 mini-film is CTF, while positive VIP/bomb tests
  require external caches. Asked user for v41 VIP/Assault local paths or match
  IDs; other parser work remains available. Full parity is incomplete.

- Weapon-shot provenance captured and Film integration checks are VERIFIED.
  Session 57588 completed with exit 0: six-film source test passed in 52.95s,
  checking 4395 native shot reads plus 105 damage reads, source packet/ordinal
  lookup, referenced byte re-decode and read serialization. Earlier RUNNING
  checkpoint is superseded. Added positive Film::retain_weapon_hits regression:
  complete Film JSON roundtrip retains shot_reads; removing that field yields
  None and preserves every other weapon-hit field. Initial synthetic packets
  used timestamp zero and hit Film's nonzero-timestamp precondition; changed
  fixture timestamps to 1000us without changing parser behavior. Rerun passed
  (0.37s), Clippy tests passed (21.22s), formatting/diff checks passed. No process
  remains. Logs: `/private/tmp/halo-shot-provenance-corpus.log` and
  `/private/tmp/halo-shot-film-retention-{test,clippy}.log`.
  Full v41 parity is still incomplete; architecture remains deferred.

- Loaded weapon-shot provenance added through WeaponShotRead and public
  scan_weapon_shot_reads / scan_weapon_shot_reads_through APIs. Existing shot
  APIs project these same reads; FilmWeaponHits retains shot_reads alongside
  compatibility shots without rescanning. None marks older exports, Some(empty)
  marks a successful empty scan. Source packet and pre-filter ordinal survive
  shots with unreadable identity pairs. Four focused weapon-hit tests passed
  (2.72s), including source lookup/redecode/JSON and ten native unpaired shots
  in the explicit-range fixture. Clippy tests and WASM passed, as did formatting
  and diff checks. Six-film source regression is RUNNING in session 57588, with
  4395 native paired shots plus 105 damage reads and empty controls. Do not
  restart it. Logs: `/private/tmp/halo-shot-provenance-{tests,clippy,wasm,corpus}.log`.
  Full parity remains incomplete.

- Extended captured weapon-damage provenance checks are VERIFIED. Session 20305
  completed with exit 0; local_weapon_hit_corpus passed all six films in 50.87s.
  All 105 positive native damage reads now pass packet/ordinal lookup, referenced
  payload re-decode, every-field comparison, enriched JSON roundtrip and legacy
  JSON omission checks. Four empty controls still match the native results.
  Explicit-range regression passed (0.01s), Clippy tests passed (15.77s), and
  formatting/diff checks passed. Earlier RUNNING checkpoint is superseded;
  no process remains. Log: `/private/tmp/halo-weapon-damage-provenance-corpus.log`.
  This closes the named damage provenance integration check, not full parity.

- Prior weapon-hit captured scan completed successfully: session 12434 exited 0,
  local_weapon_hit_corpus passed all six films (105 native damage results and
  empty controls) in 50.51s. The earlier RUNNING checkpoint is superseded.
  Extended the source-reference regression into the captured path, using the
  same packet lookup, byte re-decode and complete read comparison. Added old-JSON
  checks by removing source/packet_index and requiring None with every other
  field preserved. Focused explicit-range regression passed; Clippy --lib --tests
  passed (15.77s). Extended captured run is active in session 20305; its log is
  `/private/tmp/halo-weapon-damage-provenance-corpus.log`. Do not restart it.

- Weapon damage provenance gap fixed: loaded WeaponDamageRead now retains source
  FilmPacket and all-packet ordinal; payload-only reads and old exports keep None.
  FilmWeaponHits / Film.weapon_hits automatically preserve the enriched reads.
  Native raw damage values, attribution bases and pairing behavior are unchanged.
  Four focused weapon-hit tests passed (2.70s), including all 36 explicit-range
  cases and source lookup/redecode/JSON checks for four positive damage results.
  WASM passed; Clippy initially caught two clone_on_copy uses, then passed after
  replacing them with copies. Formatting and diff checks passed.
  Captured scan regression is RUNNING in session 12434; do not restart it.
  It compares six films (105 native damage results plus empty controls).
  Logs: `/private/tmp/halo-weapon-damage-source-{test,clippy,wasm,corpus}.log`.
  Full parity remains incomplete; this closes source attribution for this read
  type, not unknown-byte archival or all remaining producer retention.

- Full current Theater regression is VERIFIED: cargo test --lib theater::
  completed with 487 passed, zero failed, 42 ignored, 53 non-Theater tests filtered
  out, in 58.57s. Log: `/private/tmp/halo-parity-full-current.log`. No process
  remains. CURRENT_VALIDATION_GAPS.md records the exact ignored jobs and remaining
  acceptance limits; their omission is not treated as a pass. Read-only retention
  inspection confirmed equipment recovery attempts and full weapon damage reads
  already propagate through their existing Film fields; no redundant trace added.
  Local 32-film metadata contains mode categories 6/11/18 only; this inspection
  does not establish positive VIP/bomb coverage. Full parity remains incomplete.

- FilmContext and its constructors audited against the pinned implementation.
  Added resolve_imposed_i0_layout: forced layouts are copied without validity
  gating, then valid catalog layouts, otherwise detection. The 256-case native
  constructor fixture confirms this policy, including invalid forced layouts,
  simulation completion on imposed-layout presence, copied layout results,
  shared reader observers and absent frame observers. Native generation passed
  (0.496s); Rust layout test passed (0.01s); Clippy --lib --tests and WASM passed
  (21.97s / 25.90s); formatting/diff checks passed. No process remains.
  Logs: `/private/tmp/halo-context-constructor-{test,clippy,wasm}.log`.
  READER_CONTEXT_PORT.md now maps every context operation and explicitly records
  absent lazy-cache/error identity and general mutable observer/profile APIs.
  All facade declarations have now been inspected, but partial mappings remain
  partial: this does not close full v41 parity, context behavior or all-data
  retention. Next work must address those implementation/coverage gaps rather
  than treating the facade inventory as a completion certificate.

- NewWorld constructor state and ApparStats facade mappings audited. The world
  native oracle now captures nil/non-nil registry constructors and preserves all
  original fixture data exactly. Rust checks empty slots/counters, zero view and
  chunk, unknown keyframe namespace and absent anticipated table, then reruns
  the existing 128 mutation sequences and their queries. Native generation passed
  (1.598s); Rust world test passed (2.42s). Registry ownership and one-time native
  logging latch differ explicitly: registry is passed separately to Rust walkers.
  ApparStats maps all five KillMatchingStats counters; full 1024-case native
  hybrid comparison passed (1.32s). Totals: identity 476, window 301, bot_window
  785, unclaimed_window 231, pairs_without_identity 2762. The last is a separate
  diagnostic, not part of the publication sum. Clippy --lib --tests passed
  (8.02s); formatting/diff checks passed. No process remains.
  Logs: `/private/tmp/halo-world-constructor-{test,clippy}.log` and
  `/private/tmp/halo-matching-stats-test.log`.
  Remaining unreviewed facade declarations: FilmContext and its two constructors.
  Inventory progress does not close arbitrary reader/context contracts, other
  field-retention sinks or captured VIP/bomb/equipment recovery coverage.

- Native starting-profile field contract is VERIFIED. Added the pinned native
  TestHaloRustStartingProfile harness and starting-profile-v41.json containing
  both complete default and kill-starting objects. Native generation passed
  (0.348s); Rust native_kill_starting_profile_contract passed (0.00s), projecting
  existing production state into every Mouvement/Cadre/MPP/Grammaire field,
  including f32 dequantization values, fixed policy and empty override maps.
  Native default differs only by strict generation checking. This maps three
  previously unreviewed profile declarations while retaining partial status:
  Rust profile settings remain distributed and arbitrary reader replacement /
  restoration and the complete convenience-return surface remain open.
  Clippy --lib --tests initially exposed an existing field_reassign_with_default
  warning in reader_sequence_tests; converted that initialization to a struct
  literal without behavior changes. Recheck passed (26.33s), as did formatting
  and diff checks. No production behavior changed, and no process remains.
  Logs: `/private/tmp/halo-starting-profile-{test,clippy}.log`.
  Remaining unreviewed facade declarations: FilmContext, NewFilmContext,
  NewFilmContextForMap, NewWorld and ApparStats. Full parity is still incomplete.

- Kill-source facade audit advanced: BOT_SUFFIX, XUID_NAME_PREFIX and
  NO_KILL_FEED are public and used by the existing producer paths. Native
  publication values remain unchanged; typed Rust missing-feed error replaces
  native French text / Go pointer identity, as documented. Existing oracles
  contain 544 bot-bearing roster rows and 46 fallback-name feed rows. Mapped
  Assist to all five KillAssist fields and CoupleStats to all nine KillPairStats
  counters; documented unknown-assistant semantics, per-row extra scope,
  inference provenance and integer-domain limits. All 14 native_kill_ tests
  passed (2.01s), including full assist/pair/roster/feed producer comparisons.
  Clippy, WASM, formatting and diff checks passed; no process remains.
  Logs: `/private/tmp/halo-kill-facade-{test,clippy,wasm}.log`.
  Dedicated missing-feed oracle coverage remains open. Remaining unreviewed
  declarations: FilmContext/constructors, NewWorld, scan profiles, ApparStats
  and ProfilDeDepart. Broader retention/context/captured coverage gates still
  apply; neither this inventory pass nor green tests establishes full parity.

- Paquets caller-tag contract is VERIFIED against the pinned native walker.
  Added walk_tagged_film_packets and borrowed TaggedFilmPacket, preserving the
  full signed 64-bit caller tag without changing existing FilmPacket consumers.
  Ordered native packet indexes, types, timestamps and payloads are retained,
  alongside source offsets and opaque header bytes. Native fixture extension:
  512 original cases unchanged, 4608 tagged calls, 10440 packet results. Both
  signed extremes and values immediately outside i32 are covered. Native
  generation passed (0.508s); Rust source loading test passed (0.28s), including
  payload borrowing and boundary checks. Clippy and WASM completed with exit 0;
  formatting and diff checks passed. No compiler process remains for this work.
  Logs: `/private/tmp/halo-tagged-source-{test,clippy,wasm}.log`.
  This maps the facade declaration but does not establish whole-source portable
  fidelity: malformed tails remain outside the packet list. Remaining unreviewed
  facade declarations include FilmContext/constructors, NewWorld, scan profiles,
  ApparStats, Assist, CoupleStats, BotSuffix, XUIDNamePrefix, ErrNoKillFeed and
  ProfilDeDepart. Architecture work remains deferred; full parity is incomplete.

- I0 signed-axis and raw region-width facade validation is VERIFIED. The native
  oracle preserves all 512 original layouts and adds 4608 signed-axis outcomes;
  the precision oracle preserves 2048 original rows and adds 13 count boundaries.
  Focused Rust tests passed (0.05s / 0.04s). Clippy and WASM sessions completed
  with exit 0 (34.93s / 42.84s); formatting and diff checks passed. Offsets above
  axis three return None instead of a native panic. Logs:
  `/private/tmp/halo-i0-contract-{clippy,wasm}.log`.

- Captured navpoint Film integration is VERIFIED. The existing session 53917
  completed with exit 0; local_film_player_assembly passed all six films in
  2850.04s. Native accepted navpoint data (13805 positive gameplay readings),
  complete trace agreement between map and explicit-encoding constructors,
  packet/ordinal sources and complete Film JSON roundtrips all passed. Broader
  player/combat/pickup/vehicle assembly comparisons in that test also passed.
  The earlier RUNNING checkpoints below are superseded; no process remains.
  The long silent interval prompted a read-only process inspection; subsequent
  polling and the final log confirmed successful completion, not a stalled or
  restarted test. Log: `/private/tmp/halo-navpoint-film-corpus.log`.
  This closes the named constructor/captured integration gap, not full v41 parity.

- Exposed three remaining facade constants: NATIVE_FACTS_REVISION for decfilm.Rev,
  NEGATIVE_EQUIPMENT_KEPT_FALLBACK and FIRST_SLOT_LIFE_FALLBACK. Existing decoder
  coverage and usage call sites share these public constants; values/policies
  and native fixtures are unchanged. Mapped the three declarations while keeping
  broader audit status partial. Native provenance: 577 cases passed (0.01s).
  Native usage: 1036 cases passed (2.76s), including the two identifiers in
  162 and 558 cases. Clippy, WASM, formatting and diff checks passed.
  Initial test linking failed with errno=28 (filesystem full), not a Rust error.
  Inspected 78 GiB debug artifacts, then ran cargo clean -p halo_api --profile dev
  to remove rebuildable package outputs. Free space recovered to about 78 GiB;
  test retries passed. Source, fixtures and logs were not removed.
  Logs: `/private/tmp/halo-facade-constants-{provenance,usage,clippy,wasm}.log`.
  The older captured Film integration process remains LIVE under session 53917;
  last poll still reported no result after Bazaar. Do not restart it solely for
  elapsed time. Its log is `/private/tmp/halo-navpoint-film-corpus.log`.

- Captured navpoint Film integration regression now exercises both map and
  explicit-encoding constructors against the pinned six-film assembly corpus.
  It retains all existing native reading/counter checks, verifies complete
  navpoint trace agreement, actual source packet/ordinal lookup and complete
  Film JSON roundtrips. Positive native expectations include 13805 readings
  across Bandit and Oddball; empty controls remain distinct. This also repairs
  the older assembly assertion that compared new traces with a pre-trace native
  output shape, without dropping the new trace checks.
  Test is RUNNING, not yet verified: `cargo test --lib local_film_player_assembly
  -- --ignored --nocapture`, session 53917, log
  `/private/tmp/halo-navpoint-film-corpus.log`. Last authoritative poll confirmed
  the same handle live; Bazaar assembly passed, later films have not reported.
  Resume by polling this handle, not starting another test. Formatting and diff
  checks passed. Production parser code unchanged this turn.
  Lightweight next-audit lookup: decfilm.Rev maps to the facts revision currently
  used in replay_decoder_coverage.rs; both remaining named fallback strings are
  used in replay_usage.rs. No facade audit labels changed from that lookup.

- Navpoint scans now retain ordered delta components and complete optional
  keyframe attempts before publication/rollback, with raw fields, callbacks,
  stop/status, candidate start/slot and available source packet/ordinal. Projected
  match time stays separate from wire timestamps. The explicit-encoding Film
  constructor now retains the navpoint scan/result error like the map path.
  The 1024-case native fixture adds 23127 actual observer callbacks and preserves
  every prior input/output. Native generation passed (0.691s); Rust callback
  comparison passed (1.24s). Initial constructor test incorrectly expected an
  empty slot band to fail; corrected to the native successful-empty contract,
  without changing parser behavior. Focused constructor regression passed
  (0.52s). Full suite rerun: 485 passed, 42 ignored (61.86s). Six-film scan/source
  comparison passed (145.02s); Clippy, WASM, formatting and diff checks passed.
  Logs: `/private/tmp/halo-navpoint-attempts.log` and
  `/private/tmp/halo-navpoint-attempts-{film,suite,corpus,clippy,wasm}.log`.
  Positive constructor trace integration, clockless packet/source fidelity and
  remaining native API contracts are separate gates. Architecture remains deferred.

- Managed-property scans now preserve all direct component attempts before
  status/bounds filtering, with fields, references, callbacks, slot/record start,
  component index and raw dispatch/bounds status. Whole-film paths attach actual
  packet metadata and native ordinals counted before filtering; payload-only
  calls retain unavailable provenance. Film.managed_properties exports the trace
  through existing constructors. The expanded 1024-case native fixture compares
  all 54483 callbacks and keeps every prior fixture input/output unchanged.
  Native regeneration passed (0.611s); focused Rust comparison passed (2.44s).
  Full Theater suite: 485 passed, 42 ignored (62.35s). Clippy, WASM, formatting
  and diff checks passed. The six-film comparison also passed: 16569 accepted
  readings/counters, packet/ordinal provenance, bounds and JSON roundtrips.
  Logs: `/private/tmp/halo-managed-attempts.log` and
  `/private/tmp/halo-managed-attempts-{suite,clippy,wasm,corpus}.log`.
  Broader scanner/source-retention and context contracts remain open; no queued
  architectural changes were introduced.

- Positive objective Film integration is now verified through both explicit-
  encoding and map constructors. A new regression encodes the pinned oracle's
  source-valid registries into bootstrap bytes and uses unchanged native packet
  fixtures. Cases 1/2/4 cover complete and broken keyframes; case 19 covers a
  missing archetype after slot discovery. Both constructors match every native
  accepted field/counter and all 525 callbacks per API, retain packet/ordinal
  source links and roundtrip the complete Film. The 30 callbacks suppressed by
  publication in case 4 remain in Film.objective_scan.attempts.
  The initial expanded matrix incorrectly tried to encode case 7's injected
  registry hole; bootstrap parsing correctly rejects it. That arbitrary in-memory
  registry stays covered by standalone scanner tests, while constructor tests
  use source-valid registries. Focused regression passed (1.67s), formatting and
  diff checks passed. Production code unchanged; prior full suite/Clippy/WASM
  checks were not repeated. Log: `/private/tmp/halo-objective-film-positive.log`.
  Captured objective trace integration and broader v41 parity gates remain open.

- Managed-objective attempt retention and Film integration: ObjectiveScan now
  preserves ordered delta components and complete optional keyframe attempts,
  including raw fields, ranges, stop/status, source packet/ordinal, candidate
  start/slot and callbacks discarded by native publication gates. Both
  profile-aware Film constructors now retain this previously standalone scan
  and its nonfatal setup error. Old exports keep unavailable fields as None.
  The native 256-case fixture wraps the actual objectiveWalk and asserts its
  accepted output equals the public scan, then compares all 43302 callbacks
  (27750 delta, 15552 keyframe) in Rust. All prior fixture data is unchanged.
  Native generation passed (0.792s); focused comparison passed (3.29s).
  Full Theater suite: 484 passed, 42 ignored (57.40s). Clippy, WASM, formatting
  and diff checks passed. Explicit-encoding Film regression verifies missing
  slot-band retention and legacy export compatibility. Positive constructor
  and captured trace integration remain separate checks, as documented in
  FILM_RETENTION_AUDIT.md. Logs: `/private/tmp/halo-objective-attempts.log` and
  `/private/tmp/halo-objective-attempts-{suite,clippy,wasm}.log`.
  Full v41 parity remains open; the queued architecture has not started.

- Equipment-object retention now preserves every attempted ti=37 component
  before applying status/bounds gates, including successful intermediate fields
  and callbacks from rejected reads. EquipmentComponentAttempt carries packet
  source/ordinal, candidate record bit, slot/generation, component index, raw
  status/bounds and the complete decoded component. Existing accepted samples
  and counters are unchanged; Film.equipment_state exports the trace. Extended
  the 512-case native equipment fixture with actual observer emissions while
  verifying all prior fixture data unchanged. All 5339 callbacks match, including
  six emitted by directly rejected component reads. Native generation passed
  (0.491s); Rust fixture passed (0.48s), with complete stream roundtrip and legacy
  absence checks. Clippy, WASM, formatting and diff checks passed. All three equipment tests, including the eight-configuration captured
  corpus comparison, passed (see corpus log for elapsed time). Logs:
  `/private/tmp/halo-equipment-attempts.log` and
  `/private/tmp/halo-equipment-attempts-{clippy,wasm,corpus}.log`.
  Other scanner traces, source-retention and context contracts remain open;
  no deferred architecture work was started.

- Complete biped scanner attempt retention: six existing streams now preserve
  successful intermediate reads as well as failed attempts in component_attempts.
  Each entry retains component index, raw dispatch/bounds status, source and
  packet/record/slot identity, fields, references and ordered callbacks. Existing
  target publications and rejected_components remain unchanged. Native context
  fixtures verify all ordered attempts: inventory 730, unit equipment 1018,
  camo/ability 1580, charges 1857, ability states 2408 and held weapons 768.
  The fixtures contain positive accepted intermediate reads and rejected reads;
  complete stream JSON roundtrips and old-export absence are checked. Traces
  overlap across scanners and are not new canonical records or semantic events.
  Focused comparison passed (0.66s). Full Theater suite: 484 passed, 42 ignored
  (61.74s); Clippy, WASM, formatting and diff checks passed. Four-film captured
  gameplay comparison also passed (see captured log for elapsed time). Logs: `/private/tmp/halo-all-biped-attempts.log`
  and `/private/tmp/halo-all-biped-attempts-{suite,clippy,wasm,captured}.log`.
  Other caller retention, context APIs and whole-source gates remain open.

- Highlight-profile parity: added v41_highlight_profile_from_header and
  FilmHighlightProfile, preserving native major/read provenance as Option<u32>,
  layout name and gamertag offset. Short headers retain fallback metadata;
  readable non-v41 versions remain rejected. Film.native_highlights.profile is
  populated from the actual bootstrap header; explicit-version scans and old
  exports retain unavailable provenance. The 27-case native header fixture now
  includes all four profile fields with all previous data unchanged. Native
  regeneration passed (0.667s); key/profile oracle passed (1.20s), 9323-case
  highlight oracle passed (6.00s), pipeline/portable regression passed (1.78s).
  The initial pipeline assertion incorrectly expected highlights from its
  intentionally gapped manifest; corrected the test to retain that negative
  case and add a contiguous variant. Clippy, WASM, formatting and diff checks
  passed. Logs: `/private/tmp/halo-highlight-profile.log`,
  `/private/tmp/halo-highlight-profile-{pipeline,events,clippy,wasm}.log`.
  Broader source/producer retention and observer context gates remain open;
  this checkpoint does not establish full parity or begin the refactor.

- Header/player-table/highlight facade audit maps four declarations. Added
  film_major_version_from_header returning Option<u32> so short input remains
  distinct from a recorded zero; decode_v41_film_key shares it and still rejects
  readable non-v41 versions. The native key oracle adds 27 raw-header cases
  covering lengths 0..8 and words 0/41/u32::MAX; all previous fields unchanged.
  Documented all 17 player-table report fields, retained slot/error semantics,
  and the explicit-v41 plaintext/zlib highlight API. Native header regeneration
  passed (0.663s). Rust key test passed (1.20s); 289-case table test passed (1.52s);
  9323-case highlight test passed (5.99s). Clippy, WASM, formatting and diff checks
  passed. Logs: `/private/tmp/halo-header-contract.log`,
  `/private/tmp/halo-player-table-contract.log`, `/private/tmp/halo-highlight-contract.log`,
  `/private/tmp/halo-header-contract-{clippy,wasm}.log`.
  HighlightProfileFromHeader's returned profile remains an explicit next audit;
  no older-major decoder or deferred architectural changes were introduced.

- Weaponv3 identity/catalog facade audit maps KnownWeaponHigh32, PIBits,
  ResolveBest and ResolveXuidToPI. Exposed PLAYER_INDEX_BITS=5 and used it in
  the existing pattern reader. The native helper oracle now compares the full
  35-entry family catalog and adds 512 deliberate conflicting chunk assignments
  with duplicate roster XUIDs. All earlier fixture data is unchanged. Every
  case has distinct first/second assignments and verifies native first-chunk
  priority. The separate replication scanner still rejects disagreements.
  Native regeneration passed (0.373s); Rust helper test passed (0.18s), scanner
  test passed (0.47s). Clippy, WASM, formatting and diff checks passed. Logs:
  `/private/tmp/halo-player-index-contract.log`,
  `/private/tmp/halo-player-index-scan-contract.log`,
  `/private/tmp/halo-player-index-{clippy,wasm}.log`.
  No resolver behavior changed. Catalog mutation and wider source contracts
  remain separate; this does not establish complete parser parity.

- Objective-family facade parity: added ObjectiveStatName for named/identified
  stat events and public count_objective_family, matching native generic
  CountObjectiveFamily. Three internal count sites now share it; identity or
  visibility does not affect family membership. Extended the native action
  oracle with both generic instantiations over 1024 cases (9899 family events).
  All earlier action, coverage, classifier and prefix data remains unchanged.
  Mapped 15 objective-family/constants/capture-time facade declarations, keeping
  broader API status partial and recording exact canonical strings in
  DATA_CONTRACT_INVENTORY.md. Native regeneration passed (0.511s); objective
  tests: 13 passed, 1 ignored (8.62s). Clippy, WASM, formatting and diff checks
  passed. Logs: `/private/tmp/halo-objective-family-contract.log` and
  `/private/tmp/halo-objective-family-{clippy,wasm}.log`.
  Positive captured VIP/bomb and full semantic/source gates remain open; the
  deferred three-layer architecture has not started.

- Objective record/series/round-identity facade audit maps 14 formerly unreviewed
  declarations to the actual Rust fields and methods. StatValue preserves C/D
  presence separately; round identities preserve by-round assignments, origins
  and starts; derived series retain component strict/unitary policies. Added
  missing native RosterFitsStatborg helper as roster_fits_statborg and exposed
  STATBORG_PLAYER_SLOTS=8. Signed capacity semantics are intentional (n <= 8),
  not a new extraction or validation gate. Extended the native component oracle
  with eleven capacity queries, including 7/8/9 and signed extremes; all prior
  component/query/domain fixture fields are unchanged. Native regeneration
  passed (0.592s). Rust statborg tests: 21 passed, 1 ignored (9.09s). Clippy,
  WASM, formatting and diff checks passed. Logs:
  `/private/tmp/halo-statborg-contract{,-clippy,-wasm}.log`.
  Broader API/producer status remains partial; this is not semantic event-golden
  completion or a start of the deferred architecture.

- Damage catalog/category and health facade audit maps 13 formerly unreviewed
  declarations to retained Rust values and native oracle evidence. All 468 IDs,
  independent provenance dates/counts and 6608 source-label/category combinations
  are compared, including unknown and ambiguous labels. The catalog parser's
  513 cases cover complete labels and parse failures. Health retains all native
  fields and compares totals, ratios, alerts/degradations, verdict, metrics and
  publication gates over 1024 cases, including 26 exact-equality cases at each
  unexplained threshold and 30 nonpositive real-death denominators.
  Exposed KILL_SOURCE_UNEXPLAINED_WARN_RATIO, KILL_SOURCE_UNEXPLAINED_ALERT_RATIO
  and KILL_SOURCE_COVERAGE_WARN_RATIO, replacing identical inline method values.
  No health behavior changed. Focused native kill tests: 14 passed (1.99s);
  catalog parser: 1 passed (0.03s). Clippy, WASM, formatting and diff checks passed.
  Logs: `/private/tmp/halo-kill-catalog-health-contract.log`,
  `/private/tmp/halo-damage-catalog-contract.log`,
  `/private/tmp/halo-catalog-health-{clippy,wasm}.log`.
  Field mappings are in DATA_CONTRACT_INVENTORY.md; broader API/producer status
  remains partial. The current full parser and deferred architecture gates
  are unchanged.

- Added independent kill-table source-reader coverage, beyond roster fixtures
  that inject prebuilt tables. A pinned Go harness exercises 56 cases across
  first-buffer order/number variants, missing/empty sources, missing identity,
  unknown build, truncation and table-not-found. Six successful captured Bandit
  cases preserve 48 raw player slots. Every table field, read state and private
  slot field matches Rust, with JSON roundtrip. The original 49 refusal-only
  cases are unchanged. Captured bootstrap bytes are retained compressed with
  SHA-256/provenance; generation is integrated in generate_oracles.py.
  Native run passed (0.459s); Rust source reader passed (1.92s). Existing fire
  head/source tests passed (2 tests, 1.57s: 23171 heads and 48 source cases), and
  the 1024-case roster oracle passed (0.12s). Formatting, generator syntax and
  diff checks passed. Logs: `/private/tmp/halo-kill-table-source.log`,
  `/private/tmp/halo-fire-contract.log`, `/private/tmp/halo-table-roster-contract.log`.
  Mapped FireEvent and four table/refusal facade declarations; API status stays
  partial. No production behavior changed. These source/field checks strengthen
  parity evidence without establishing full parser completion.

- Biped position source-retention fix: candidate scans dropped the native
  PacketIndex even though loaded explicit-layout scans retained it. Added
  optional BipedPositionCandidate.packet_index, counted among all packet types
  within each chunk before selecting deltas. Both candidate scan entry points
  populate it; old exports and synthetic caller-created observations retain
  None. Film.biped_positions retains the stream. Updated affected test literals.
  The native record-mask oracle now includes complete accepted records (4084
  across 256 cases); every prior fixture field is unchanged. Rust checks native
  ordinals, times, slots and quanta, full stream roundtrip and absent-field
  compatibility. A prefixed-keyframe test compares automatic and explicit scans
  and verifies +1 ordinals without changing decoded positions. Focused test
  passed (2.30s). Theater suite: 483 passed, 42 ignored (57.65s). Clippy, WASM,
  formatting and diff checks passed. Logs:
  `/private/tmp/halo-biped-position-ordinal{,-suite,-clippy,-wasm}.log`.
  Mapped BipedCreation, ScanBipedCreations and BipedPosition facade declarations
  to their producers and retained fields; broader API status remains partial.
  This repairs source attribution, not the deferred native-Film architecture.

- Weapon-hit contract audit now compares the complete ordered damage arguments
  supplied to the native distance callback, rather than relying on aggregate
  equivalence. The 1024-case oracle contains 22953 callbacks (10076 healing),
  including 205 cases with no callback; every earlier fixture field is unchanged.
  Added weapon_hit_bucket_count from the fixed native default distance edges.
  Mapped five formerly unreviewed declarations and refined PairWeaponHits in
  facade-contract-inventory.json, keeping their broader API status partial.
  Documented all WeaponDamage/WeaponHitStats fields and callback ownership,
  missing-distance and mutable-edge distinctions in DATA_CONTRACT_INVENTORY.md.
  Native regeneration passed (0.804s). Rust pairing/callback and 2048-event
  reader tests passed (2 tests, 0.97s). Clippy, WASM, formatting and diff checks
  passed. Logs: `/private/tmp/halo-weapon-hit-contract.log` and
  `/private/tmp/halo-weapon-hit-contract-{clippy,wasm}.log`.
  No pairing behavior changed; this verifies previously unobserved attribution
  choices. Full source-wrapper/captured integration and broader parser gates
  remain separate. Architecture work remains deferred.

- Complete returned kill-profile comparison passed. KillCalibration.reader_policy
  now retains native traversal region and chain-inference, strict-generation,
  view-table and view-class flags, with None for older exports. These describe
  the fixed decoder policy, not mutable scan options. assert_native_result now
  reconstructs every ProfilCalibre field from retained Rust values and compares
  the whole object; only float32 JSON spelling and null/empty maps normalize.
  Captured complete result/profile comparison: Bandit 122 kills, Oddball 247
  kills (193.19s). Updated native kill-walk fixture preserves all prior values;
  Rust record-walk comparison passed both films (96.88s). Legacy JSON explicitly
  checks absent policy and observer context. Theater suite: 483 passed,
  42 ignored (67.18s); Clippy, WASM, formatting and diff checks passed.
  Logs: `/private/tmp/halo-kill-complete-profile-{captured,walk,suite,clippy,wasm}.log`.
  This closes the returned-profile field exclusion described in older entries,
  not full parser parity. Broader facade/producer retention, mutable observer
  contracts and positive captured VIP/bomb/equipment-recovery coverage remain
  open. The three-layer refactor remains deferred in NEXT_PHASE.md.

- Native calibrated observer context is now retained as optional
  KillCalibration.position_observer_context: exact float32 range/quantum bits,
  region, axis widths and region-index width. This is returned profile metadata,
  independent of hook installation and played-map geometry; old exports retain
  None. The existing v41 profile factory shares NATIVE_DELTA_QUANTUM.
  The corruption-source oracle adds the actual native context to all 216 cases,
  and both captured kill-walk calibration fixtures add the same fields. Native
  reruns passed; every previous fixture value remains unchanged. The 216-case
  Rust context/roundtrip check passed (0.58s). Theater suite: 483 passed,
  42 ignored (62.01s); Clippy, WASM, formatting and diff checks passed.
  `/private/tmp/halo-kill-observer-profile-{suite,clippy,wasm}.log`.
  Complete captured result/context rerun passed: Bandit 122 kills and Oddball
  247 kills (185.99s), including ranges, quantum, descriptors and result export.
  `/private/tmp/halo-kill-observer-profile-captured.log`; session 6213 is terminal.
  Remaining profile audit: implicit frame policy values and Traversal.Region.

- Numbered-registry corruption selection: the kill calibration previously read
  identity from chunks.first(), while native GrammaireSousFilm looks up the first
  metadata entry numbered zero and parses that buffer independently. A new
  216-case native oracle covers reorderings, no chunk zero, duplicate zeros,
  negative numbers and present/absent identities with both corruption flags.
  The Rust regression failed at case 27 (reported a read despite no chunk zero).
  Calibration now uses numbered chunk zero and its own registry. Generator
  integration and fixture are retained. Focused native comparison passed (0.57s).
  Theater suite: 483 passed, 42 ignored (57.11s); Clippy, WASM, formatting,
  generator syntax and diff checks passed. Logs:
  `/private/tmp/halo-calibration-source-{suite,clippy,wasm}.log`.
- The expanded complete kill-result comparison finished successfully: Bandit 122
  kills, Oddball 247 kills, all prior result assertions plus 24 explicit returned
  profile fields and full result JSON roundtrip (167.25s). Log:
  `/private/tmp/halo-kill-returned-profile.log`. Session 39886 is terminal.
  This binary predates the numbered-registry correction above. Implicit profile
  ranges, quantum and remaining reader policies still need integrated coverage.

- Kill-result/facade data audit maps 18 formerly unreviewed declarations to
  their actual Rust representations, without promoting whole-decoder parity.
  All native Kill/Feed/DamageShare/Coverage/Stats/Result fields are mapped in
  DATA_CONTRACT_INVENTORY.md, including distributed statistics and separate
  feed/source truth. Focused kill-source tests: 16 passed, 3 ignored (1.34s),
  `/private/tmp/halo-kill-contract-audit.log`.
  Found an integrated validation gap: assert_native_result removed ProfilCalibre.
  It now compares 24 explicit returned profile fields (rather than expected
  constants) and roundtrips the complete FilmKillSourceResult. Implicit ranges,
  quantum and remaining reader-policy values still require integrated audit.
  The captured two-film comparison is running, session 39886, log
  `/private/tmp/halo-kill-returned-profile.log`; no result claimed yet.

- Production admission retention: ProductionFrame.admission_diagnostics now
  preserves native unbound/wrong-view rejection counters and per-archetype
  anticipated-binding counts. None explicitly identifies older exports without
  these counters. The production-admission oracle now captures the real native
  Observation values for its 16 cases (6 unbound, 2 wrong-view, 2 anticipated);
  all prior fields are unchanged. MovementStateStream.admission_diagnostics
  retains nonzero per-frame reports with packet source and native ordinal;
  Some(empty) is a completed frame scan with no counted admissions, while None
  marks unavailable metadata or setup that never entered the scan. Focused
  source/export regression and complete Film constructor/JSON roundtrip passed
  (0.37s). Missing-field and setup-failure tests preserve unknown status; source
  fixtures also assert measured-empty results. Theater suite passed: 482 passed,
  42 ignored (59.35s). Clippy, WASM, format and diff checks passed. Logs:
  `/private/tmp/halo-production-admission-{suite,clippy,wasm}.log`.

- Production-frame prefix parity: decode_production_frame now accepts the native
  optional 32-bit word before each entity header, including End and rejected
  headers. The body uses inference-loop semantics (no generic NEW/DEL guards).
  ProductionFrame.record_prefixes retains raw words and payload bit ranges;
  existing padded_bits identifies synthetic tail. The default path borrows its
  encoding and does not clone it. Native movement production harness extends
  512 unchanged original cases with 512 prefixed cases; the final extension has
  2786 prefixed records, including 420 deletes, and adds final-world comparison.
  Expanded native comparison passed (1.13s), including callbacks, boundaries,
  view counts, final bindings and JSON roundtrip. Prefix count and source bits
  are also checked for End/rejected headers. Theater suite passed: 481 passed,
  42 ignored (56.83s). Clippy, WASM, format and diff checks passed; logs are
  `/private/tmp/halo-production-prefix-{suite,clippy,wasm}.log`.
  No movement-source, strict event-locator or architecture scope was changed.
  Next audit: production_frame calls world.admit_delta but has no frame-level
  native admission diagnostics (inference_frame counts unbound/other-view and
  anticipated bindings). Rejected headers and cumulative world anticipations
  already preserve some evidence; verify native publication and portable
  propagation before deciding what additional retention is needed.

- Extended the native movement resync oracle with actual cross-family callback
  offsets for scanForTargetDelta and DecodeFrameResync: 512 cases, 92 scan
  callbacks (50 after named observations), 107 frame callbacks (42 after named
  observations). All previous fixture fields are unchanged. Rust compares both
  scan and caller-visible offsets, checks detached order metadata is absent,
  and roundtrips ResyncFrame. Focused native_movement_hook_resync passed.
  The imported-order regression exposed an overflow when shifting an incoming
  malformed [usize::MAX, 0] offset vector. Saturating shifts preserve invalid
  order without wrapping or panicking; the regression covers both merge
  directions. The theater suite passed (481 passed, 42 ignored), along with
  Clippy, WASM, format and diff checks. Logs:
  `/private/tmp/halo-resync-order-{suite,clippy,wasm}.log`.
  This is not full v41 parity.
- The previously running captured-corpus preservation check completed: all 32
  films passed, including the hour-long raid (3030.77s), log
  `/private/tmp/halo-current-corpus-observations.log`. Its binary predates the
  loaded-source APIs, summary diagnostics and callback-order changes, so this
  is baseline preservation evidence, not current-change or complete-parity proof.

- Preserved cross-family callback interleaving in FilmReadDiagnostics with
  optional mobility_offsets. Emission records component-count offsets; merges
  shift only incoming offsets without recopying history, capture suppression
  remaps offsets, and detached resync routing carries order with both streams.
  ordered_publications borrows the merged stream; old/malformed diagnostics
  return unavailable order. No inferred chronology is assigned to old exports.
  Four native repair/harvest harnesses now measure 89056 actual callback offsets
  across 2048 cases; all prior fixture fields remain unchanged. The initial
  suite's four failures were expected absent-order comparisons and were resolved
  by native fixture extensions, not weakened assertions. Native reader-sequence
  aggregation, portable roundtrip, unavailable-order propagation, suppression,
  detached resync and malformed-offset tests also pass.
  Theater suite: 481 passed, 42 ignored (61.90s), log
  `/private/tmp/halo-merged-order-suite2.log`; focused tests and WASM (9.01s)
  passed. Clippy initially flagged InferenceEvidence size after the diagnostics
  addition; retained its existing inline API with a documented scoped allowance.
  Clippy retry passed: `/private/tmp/halo-merged-order-clippy2.log`.
  All change-specific processes are terminal; older corpus session 47140 remains
  live after re-polling and predates this metadata addition.
  Format, diff and generator syntax passed. The architecture refactor remains
  deferred; full parity and other publication-sink integration audits remain open.

- Added `DecodedComponent::ordered_publications`, a borrowed typed iterator over
  native hook order for an individual retained component. MobilityActionHook
  precedes its optional movement-state publication; other direct components use
  the existing ordered component vector. No serialized fields change and no
  observations are cloned. Multiple mobility reads or mobility flags under a
  different component name are rejected rather than assigned an order.
  Extended the native reader-sequence harness to record actual callback order:
  1536 reads / 2640 publications (384 mobility, 2256 component), including 256
  mixed-family reads, suppression, rejection and context restoration. All older
  fixture fields remain unchanged. Two focused tests passed (0.15s).
  Theater suite: 479 passed, 42 ignored (64.97s). Clippy (50.22s including
  build lock), WASM (23.60s), format, diff and generator syntax checks passed.
  Change-specific sessions terminal. Logs:
  `/private/tmp/halo-component-order-{test,suite,clippy,wasm}.log`.
  Older corpus preservation session 47140 remains live after re-polling.
  Single-component ordering is now verified; aggregated speculative/repair/scan
  diagnostics still require their own ordering preservation. No deferred replay
  architecture has begun and full parity remains incomplete.

- Added FILM_RETENTION_AUDIT.md mapping registry, session roster, footer and
  packet fields to current Film storage and portable behavior. Strengthened the
  116-case native registry result test to assert both physical header words,
  their registry propagation and each native archetype index (names/levels were
  already compared). All five registry tests passed (0.41s), including 256
  fingerprint edges. Native roster value/stopping/roundtrip test passed; format
  and diff checks passed. Logs: `/private/tmp/halo-{registry,roster}-retention-audit.log`.
  No production code changed in this audit. Registry/fingerprint facade entries
  now describe parsed-field evidence and the cached-versus-recomputed accessor
  limitation. Known registry constant verified against pinned source.
  Cross-family mobility/component-hook ordering remains the next concrete
  retention question. Film JSON alone is not a lossless source archive; the
  queued architecture has not begun. Full parser parity remains incomplete.

- Film assembly previously discarded the footer decoder's packet diagnostics
  and unsupported-packet counts after reducing them to one mismatch warning.
  Added optional `Film.summary_diagnostics` with `SummaryDecodeDiagnostics` to
  retain ordered packet source offsets, declared/decoded counts (including zero
  decoded events), and unsupported footer packet-type counts. It remains present
  in compact exports. Old exports deserialize to None; inspected inputs with no
  footer packets produce Some(empty), preserving unavailable versus observed-empty.
  This is additive retention in the existing Film schema, not the deferred native
  hierarchy refactor. Pipeline regression covers mismatched counts, repeated
  unknown types, no footer, portable roundtrip, old exports and compact mode.
  Focused pipeline test passed (1.53s). Theater suite: 478 passed, 42 ignored
  (63.97s). Clippy (46.92s including build lock), WASM (22.12s), format and diff
  checks passed. Change-specific sessions are terminal. Logs:
  `/private/tmp/halo-summary-retention-{test,suite,clippy,wasm}.log`.
  Corpus preservation session 47140 remains live and predates this addition.

- Closed the suspected loaded-position reader-profile dependency through a
  reachable-code audit and native experiments, without adding an unused Rust
  context parameter. Two installed profiles (zero and modified default) across
  1280 source calls produce identical complete records/errors/masks. All 30 native
  observer functions are instrumented; only RecordMaskHook fires (19906 times
  across 2560 runs). Earlier fixture fields are unchanged. Rust now compares all
  returned companion fields for both quantized and world source outputs, not just
  Q/identity/world values. Focused comparison passed (7.27s); existing mask
  regression, format, diff and generator syntax checks passed. Production Rust
  code is unchanged in this step; the prior full suite/Clippy/WASM results remain
  the latest broad checks. Logs: `/private/tmp/halo-source-context-{test,mask-test}.log`.
  General reader-pointer/context contracts outside this scanner remain separate.

- Captured portable player-assembly session 37926 completed successfully (1782.66s).
  Five films passed Film serialization/deserialization typed equality and native
  player/vehicle/ride assembly comparisons: Bazaar idle, Bandit Evo, ranked Oddball,
  Cadet Blue and Cadet Brick. Tracks/points respectively: 1/71, 128/34163,
  256/65705, 1/8, 1/8. Appearance films each expose six vehicle tracks and zero
  rides; the other three have zero vehicle tracks/rides. This binary predates the
  recent source scan work. Log: `/private/tmp/halo-current-portable-assembly.log`.
  The separate corpus preservation session 47140 remains live after re-polling.

- Connected automatic world-source scanning to shared band/layout setup.
  `scan_source_world_positions` accepts an optional band (None discovers bipeds;
  an empty supplied set errors) and optional measured layout. Native setup error
  precedence is shared with quantized scanning, with no repeated detection.
  Supplied bounds stay independent of measured wire widths. Extended all 4608
  native setup cases with world bounds, speed limits and teleport exemptions:
  18911 world records, including 1527 from measured layouts. All previous fixture
  fields remain unchanged; Rust compares ordered source identities, times, Q and
  float32 coordinate bits, and asserts positive totals. Focused comparison passed
  (14.19s). Theater suite: 478 passed, 42 ignored (67.28s). Clippy (9.45s),
  WASM (16.16s), format, diff and generator syntax passed; change-specific
  processes are terminal. Logs:
  `/private/tmp/halo-source-world-auto-{test,suite,clippy,wasm}.log`.
  Earlier captured corpus sessions 37926/47140 remain live after re-polling;
  their binaries predate the automatic source implementation.
  This closes automatic world setup; reader-context/observer installation and
  production caller composition remain open. Architectural refactor stays queued.

- Added loaded-source explicit-layout world scanning with independent world
  bounds, raw quanta/source references, direction options and direct teleport
  exemption maps. Native isolation precedes shared speed filtering; pre-filter
  mask observations survive both. The native source oracle compares 1280 world
  calls / 16914 accepted records and portable reports, with prior fields unchanged.
  Corrected shared exemption arithmetic from saturating to native wrapping u64
  additions. Native boundary oracle: 256 cases / 3072 positions; 204 cases
  distinguish the old saturating behavior. Exemption-array order is preserved
  for direct maps; event-derived maps retain their existing sorting.
  Focused comparison passed (7.14s). Added a positive world-record count guard.
  Theater suite passed: 478 passed, 42 ignored. Clippy (39.60s including build
  lock), WASM (63s including build lock), format, diff and generator syntax
  checks passed. Change-specific processes are terminal. Logs:
  `/private/tmp/halo-source-world-{test,suite,clippy,wasm}.log`.
  Earlier captured corpus sessions 37926 and 47140 were re-polled and remain
  live; their binaries predate this work and do not validate these changes.
  Automatic world setup and complete caller-context composition remain open;
  this is not full parser parity. Architectural work remains deferred.

- Added loaded-source automatic biped-band and i0-layout discovery and wired
  them into quantized scans. First-keyframe-only band selection includes the
  chunk after the last explicit selection; layout detection instead uses six
  prefix chunks plus their next keyframe. Source buffers remain borrowed.
  Inclusive band bounds avoid allocating all native slots; scan materialization
  covers only the 13-bit wire domain while retaining nonempty out-of-domain
  bands. Refusal reports and implausible candidates remain inspectable.
  Extended the pinned native fixture to 144 sources, 1008 band selections and
  4608 scan calls / 21819 records; previous 128 cases remain unchanged. Nine
  added positive layouts yield 108 successful measured-layout scan calls.
  The original fixture had no positive detection; added controlled field-flip
  patterns before accepting the automatic path. Test input now includes the
  native registry header for no-data sources (initial test failed with Empty
  when that header was omitted). Focused comparison passed (7.27s).
  Theater suite: 478 passed, 42 ignored (65.16s). Clippy (9.46s), WASM
  (16.19s), format, diff and generator syntax checks passed; change-specific
  processes are terminal. Logs: `/private/tmp/halo-source-layout-{test,suite,clippy,wasm}.log`.
  Earlier captured corpus sessions 37926/47140 were re-polled and remain live;
  their binaries predate this change and are not new-path validation.
  Updated facade inventory; full loaded world/teleport/context composition and
  the broader parity audits remain open. Architectural refactor remains queued.

- Added `scan_source_quantized_position_report_for_band`, retaining ordered native
  RecordMaskHook publications after saturation rejection and before isolation.
  Each mask retains requested chunk number, packet ordinal, source packet and
  payload-relative end-of-position offset. The existing Vec API delegates to it.
  Extended the pinned native loaded-source oracle: 9953 masks across 1280 calls,
  including 155 calls where masks outnumber returned positions. All earlier
  fixture fields are unchanged. Rust compares every mask, offset and payload,
  verifies chunk/packet references and roundtrips the report. Focused comparison
  passed (4.07s); theater suite 478 passed, 42 ignored. Clippy (26.90s),
  WASM (49.53s), format and diff checks passed. Change-specific processes
  are terminal; logs
  `/private/tmp/halo-source-mask-{test,suite,clippy,wasm}.log`.
  Earlier captured corpus sessions 37926 and 47140 remain active and predate
  this change; their results cannot validate the new report implementation.
  This closes the explicit-layout loaded quantized mask-retention gap only.
  Automatic layout/band discovery and the broader outstanding parity audits
  remain open. The three-layer architectural proposal stays deferred.

- Added `scan_source_quantized_positions_for_band` with explicit wire layout and
  sparse band. Native prefix selection, explicit ordered/duplicate chunk numbers,
  skipped missing buffers, read/empty-band errors and per-packet ordinals are
  preserved. Output distinguishes requested chunk number from source position.
  Temporal isolation shares the existing native wrapping-neighbor calculation;
  no sorting or world-speed filtering is introduced. No source bytes are copied.
  Native fixture extension compares 1280 calls, 19484 records and 429 errors,
  including metadata gaps/order, repeated selections and non-delta packet ordinals.
  Initial zero-length prefix packet stopped native indexing and made the first
  fixture vacuous; corrected to a one-byte packet and added a positive-total
  assertion. All earlier fixture fields remain unchanged. Corrected focused
  comparison passed (3.26s); full suite 478 passed, 42 ignored (66.20s). Clippy
  (26.28s), WASM (49.40s), format, diff and generator syntax passed. Current
  change's test/build sessions terminal. Logs
  `/private/tmp/halo-quantized-source-{test,suite,clippy,wasm}.log`.
  Automatic layout/band discovery and pre-filter observer publications remain
  explicit loaded-source gaps. Older captured sessions 37926 and 47140 were
  re-polled live and remain ACTIVE: portable/native assembly passed Bazaar and
  Bandit Evo (128 tracks/34163 points for the latter), observation corpus remains
  at 25 preserved clips. They use earlier binaries and must not be restarted
  on unchanged output. Full parity incomplete; architecture remains deferred.

- Added `QuantizedBipedPositionRecord` and
  `scan_quantized_biped_position_records`: payload scanning now accepts wire
  layout without any map bounds and never fabricates world coordinates. Shared
  raw record walking/companion extraction feeds both this API and the existing
  world scanner; the latter dequantizes afterward with unchanged arithmetic.
  Raw API applies optional saturation and companion capture; timestamp isolation
  and spatial speed filters are not payload-level operations. Added explicit
  `BipedScanOptions::native_defaults` (CaptureDirs=false) while keeping historical
  Rust defaults intact. Full loaded-source QuantaOnly and selected-chunk options
  remain open; this is the shared raw scan foundation.
  Native extension covers 6144 payloads/5294 accepted records with nil WorldRange;
  Q, slots, saturation and no-world serialization compare, and previous mask/
  filtering fixture fields remain unchanged. Focused native test passed (1.27s);
  full theater suite 477 passed, 42 ignored (64.23s). Clippy (46.96s), WASM
  (1m10s including lock wait), format, diff and generator syntax passed. Current
  change's test/build sessions are terminal. Logs
  `/private/tmp/halo-quantized-position-{test,suite,clippy,wasm}.log`.
  Older captured runs 37926 and 47140 remain ACTIVE and used pre-change binaries:
  Bazaar portable assembly passed, and 25/32 observation clips are preserved at
  latest log inspection. Poll existing handles; their results are not validation
  of the new raw-scanner implementation. Full parity remains incomplete and the
  architecture refactor stays deferred.

- Re-polled captured sessions 37926 and 47140 live; both still ACTIVE.
  Portable/native assembly has passed Bazaar; observation preservation has
  reached six clips (aim x2, appearance x2, Bandit Evo, controller stick circles).
  No failure or terminal result yet. Continue the same session handles and logs
  from the checkpoint below; do not restart them on unchanged output.
  Independent scan-options audit found a concrete remaining data contract:
  native QuantaOnly can emit Q with HasWorld=false without bounds, whereas the
  current Rust biped entry point requires map bounds and always stores a world
  vector. Add an explicit quantized/optional-world path without fabricated
  coordinates, then verify native filtering/publication. Also native default
  CaptureDirs=false differs from Rust's historical convenience default true.
  Updated DefaultScanFilmOptions/ScanFilmOptions facade entries with these exact
  limits; chunk selection and full loaded-source options remain open.
  Full parity remains incomplete; architecture stays deferred.

- Strengthened captured Film assembly validation: serialize and deserialize the
  complete map-aware Film, assert typed equality, then build player state from
  the restored Film before comparing with the pinned native oracle. Expected
  values remain native-produced. The shared helper also applies this check to
  the optional raid test when run. No production code changed.
  Bazaar mixed-compression source roundtrip passed (9.39s); format/diff passed.
  Two debug-profile captured runs are ACTIVE, not completed:
  - session 37926: `local_film_player_assembly --ignored --nocapture`, log
    `/private/tmp/halo-current-portable-assembly.log`; Bazaar portable/native
    assembly passed (1 track, 71 points; no vehicle tracks/rides), remaining films
    still running at the last poll.
  - session 47140: `local_v41_corpus_preserves_existing_observations --ignored
    --nocapture`, log `/private/tmp/halo-current-corpus-observations.log`; first
    four aim/appearance clips preserved at the last poll.
  Both exact session handles were re-polled live. Continue polling these runs;
  do not restart because a log or observation timeout is unchanged. Source
  roundtrip session 86526 is terminal (success). No full captured-suite success
  is claimed yet. Full parity remains incomplete; architecture stays deferred.

- Fixed native generic frame simulation-state policy: native entry points now
  honor `FrameEncoding.keyframe_simulation_complete`, using standalone false
  when unspecified, matching direct native dispatch. They previously forced
  completed bodies. The bounded convenience loop keeps its historical policy.
  A 64-case native simulation extension compares both settings, ordered records,
  trace boundaries and reader end; all 32 pairs consume different ends. Explicit
  false and unspecified policy produce identical Rust results. Initial fixture
  omitted Rust's required position-width context; supplying the existing native
  default encoding isolated the intended comparison. All earlier fields unchanged.
  Focused test passed (0.09s); full theater suite 477 passed, 42 ignored (54.31s).
  Clippy (7.82s), WASM (12.96s), format, diff and generator syntax passed; all
  processes terminal. Logs `/private/tmp/halo-frame-simulation-{test,suite,clippy,wasm}.log`.
  Observer audit cross-checked all 30 hooks against existing retained forms.
  RecordMaskHook belongs to the biped scanner, not generic DecodeFrameRecords.
  Arbitrary observer replacement/restoration and cross-family callback ordering
  remain unverified (mobility has a separate ordered vector). Updated reader
  context, fixture provenance and facade audit. Full parity remains incomplete;
  architecture stays deferred.

- Added `decode_native_frame_records`, a caller-owned `NativeFilmBits` adapter
  over the existing native generic loop. It consumes positive preamble widths
  only at reader position zero and updates the caller position on completion or
  record desynchronization. Negative reader positions are explicitly rejected
  without mutation. A 512-case native extension compares starts at zero, inside
  the stream and beyond the buffer with preambles -1/0/2/7; ordered IDs,
  header/trace boundaries, completion and reader end match (0.08s). All previous
  fixture fields remain unchanged. Core traversal logic did not change.
  Audited native capture: its private accumulator has no public installer or
  production caller. Existing 1024-case component accumulator suite passed
  (0.31s); reader-level observer/context inheritance remains a distinct audit.
  Clippy (5.80s), WASM (9.22s), format, diff and generator syntax passed; all
  processes terminal. Full suite was not repeated for this thin adapter; previous
  full run remains 477 passed/42 ignored. Logs
  `/private/tmp/halo-frame-reader-{test,accumulator,clippy,wasm}.log`.
  Updated current fixture/inventory scope. Full parity remains incomplete;
  architecture stays deferred.

- Added generic `EntityBindings::bind_wildcard` and explicit
  `EntityBinding.generation_any`. Strict generic/native record loops accept
  recorded generations against wildcard bindings; successful New records reset
  uncertainty, and Delete removes the binding. Wildcard input namespace bits
  are masked like native. Portable serialization roundtrips unknown generation;
  absent fields default false and false fields are omitted, retaining old shapes.
  Extended the existing frame oracle with 256 cases across absent, matching,
  mismatched and wildcard bindings under both generation policies. Ordered IDs,
  boundaries, completion and final binding state match native; previous fields
  remain unchanged. Focused comparison passed (0.06s); full suite 477 passed,
  42 ignored (54.11s); Clippy (8.75s), WASM (15.04s), format, diff and generator
  syntax passed. All processes terminal. Logs
  `/private/tmp/halo-frame-wildcard-{test,suite,clippy,wasm}.log`.
  Correction: held-weapon caching is NOT missing parity work. The pinned
  `world.go` explicitly removed that cache; the old `frame_records.go` comment
  is stale. Reader-owned position accumulation, observer/context inheritance,
  mutable-reader/preamble and one-time anticipation logging remain distinct
  audit items. Updated World/DecodeFrameRecords inventory and fixture notes.
  Full parity remains incomplete; architecture remains deferred.

- Added explicit `decode_native_entity_view` for reference-style zero-tail reads
  at a supplied record boundary, sharing the bounded decoder's loop. Every
  retained record, including synthetic End, reports its padding provenance;
  original bytes are never extended. Native mode uses native component-mask
  traversal, generic per-record capture slots and baseline consumption before
  missing-archetype failure in non-strict mode. Existing bounded API behavior
  is preserved. Extended the native frame oracle with 384 complete/truncated
  cases at six lengths and both generation policies; 304 consume synthetic
  bits. Ordered IDs, header/trace/end boundaries and completion match native;
  padding accounting is asserted. All earlier oracle fields remain unchanged.
  Focused comparison passed (0.03s); full theater suite 477 passed, 42 ignored
  (54.18s). Clippy (8.04s), WASM (13.64s), format, diff and generator syntax
  passed. All processes terminal. Logs
  `/private/tmp/halo-frame-native-tail-{test,suite,clippy,wasm}.log`.
  Broader mutable-reader/preamble/observer, wildcard-binding and native World
  cache contracts remain open. Full parity is incomplete; refactor deferred.

- Added explicit generic-record generation policy through
  `decode_entity_view_with_generation_policy`; existing `decode_entity_view`
  retains its strict default. Native `DecodeFrameRecords` permits slot-based
  delta decoding when `GenerationStricte` is false; the Rust generic loop
  previously rejected generation mismatches unconditionally. Relaxed decoding
  preserves wire IDs and leaves bound generations unchanged.
  Extended the existing native frame oracle with both policies on 32 chains;
  16 stale-generation chains stop strictly but complete when relaxed. Prior
  strict/production fields were verified unchanged. Initial argument-forwarding
  mistake was caught by the oracle (end bit 235 versus 319) and corrected.
  Focused comparison passed (0.02s); full theater suite 477 passed, 42 ignored
  (54.19s); Clippy, WASM, format, diff and generator syntax passed. All processes
  terminal. Logs `/private/tmp/halo-frame-generation-{test,suite,clippy,wasm}.log`.
  Updated DecodeFrameRecords audit and fixture provenance. Full native mutable
  reader/World contracts, including padding, preamble/observer inheritance and
  caches, remain broader than this bounded EntityBindings API. Parity remains
  incomplete; architecture stays deferred.

- Added native custom-roster support through
  `extract_source_objective_events_with_roster` and
  `extract_objective_events_with_roster` (`FnMut(&str) -> Option<i64>`).
  Existing map APIs delegate to the same implementation. Recorded team values
  remain authoritative; lookups occur per attributed event before final sorting.
  Extended native source oracle: 896 callback scenarios, 1252 ordered calls
  including 196 CTF calls; 392 scenarios have multiple calls. Repeated identities
  and stateful responses detect caching or ordering changes. All previous
  map/helper fixture fields remain unchanged. An initial expanded-fixture failure
  was null player-list normalization for unattributed CTF events; corrected in
  the harness consistently with existing vector normalization.
  Full theater suite passed: 477 passed, 42 ignored (54.77s). Clippy, WASM,
  format, diff and generator syntax checks passed. All processes terminal.
  Logs `/private/tmp/halo-objective-callback-{suite,clippy,wasm}.log`.
  This closes the earlier map-only roster limitation for source extraction;
  native nil-interface panic and cross-language callback exception behavior
  are not emulated. Full v41 acceptance and remaining audits are still open;
  architecture remains deferred.

- Extended loaded-source objective oracle to the full native `Extract` entry
  point: 128 source layouts, eight modes, three roster policies, 3072 calls and
  600 nonempty outputs. Ordered event fields and team controls match Rust;
  accord, contradiction and silence each occur 120 times. All prior source
  fixture fields remain unchanged. Native generator passed (0.384s); Rust
  `native_objective_source_selection` passed (0.11s); format, diff and generator
  syntax checks passed. No production logic changed. Added provenance in
  `fixtures/objective-source-v41.md` and audited Extract/MapRoster declarations.
  Map-based behavior is verified; arbitrary native Roster callbacks remain an
  explicit caller-contract gap. Full parity remains incomplete; architecture
  stays deferred. Log: `/private/tmp/halo-objective-source-extract.log`.

- Audited named-objective attribution and net flag-grab façade contracts.
  Native named fixture passed (1024 cases, 338166 events, 8.48s), including
  15072 flat-attributed events with present-but-empty identities and 175422
  round-attribution drops. Checked native stable time/XUID/stat ordering and
  exposed fields. Net flag-grab and document-adapter comparisons passed (2048
  and 512 cases, 0.13s): signed/sub-ms windows, stable span ordering, unnamed
  previous carriers, open ends, inclusive home overlap and sorted player results.
  Flag signal/carry oracle passed (2048 cases, 7.01s); inspected source signal
  counts and recognition gate. Updated eight façade declarations with evidence
  and explicit input/integration limits. No production changes were needed.
  All processes terminal; logs `/private/tmp/halo-facade-{named,net-grabs,flag-signals}.log`.
  These supplied-observation contracts do not prove complete source extraction.
  Full parity and remaining façade/caller audits remain open; refactor deferred.

- Added the missing loaded-source SlotIdentityResolved equivalent,
  statborg_source_resolved_identity, plus record-level
  resolve_statborg_identity_with_diagnostics. The existing resolver shares its
  original totals/death merge logic; source and totals-pass diagnostics are now
  accessible in native order without rescanning. Source truncation stays explicit.
  Extended pinned record/source oracles, verifying all pre-existing fields remain
  unchanged. Core 1024-case comparison passed (1.94s); three loaded-source tests
  passed (1.83s), including source/pass truncation warnings (55 across the ordinary
  source cases and five in the limit case). Added four positive/negative native
  source cases: three timed death increments resolve slot 10; no feed and a 151ms
  late feed do not; totals retain priority at equal coverage. Final source tests passed (4 tests, 2.99s), full theater suite passed
  (477 tests, 42 ignored, 54.63s), Clippy (10.33s) and WASM (15.59s) passed.
  Format/diff and generator syntax passed. All sessions terminal. Logs
  `/private/tmp/halo-resolved-identity-*`. Updated four façade entries.
  No architectural work started; full v41 all-data parity remains incomplete.

- Continued recording façade audit: keyframe-position first-packet selection,
  terminator/bounds behavior, comb margin, chunk-start timing, structural filters
  and inferred team clustering match inspected native logic. Focused fixture
  passed (3149 probes, 0.37s); six-film comparison passed (16.16s), including
  Bazaar Film construction and portable export roundtrip. The probe remains
  explicitly heuristic, without canonical player identity claims.
  Expanded weapon-pattern oracle with stateful timestamp callbacks: 3402 calls
  across 883 positive cases, 36 timestamp-tie cases and 851 reordered publications.
  All pre-existing oracle fields unchanged. Rust callback order and full published
  events match (0.39s); original formula/nibble/fire totals are 3102/3489/3402.
  Fixture has no dedup removals; that particular callback gate remains code-only
  evidence. Updated seven façade entries with this coverage and its limits.
  Clippy, format and diff checks passed. No production changes; full suite/WASM
  need not repeat for this fixture/test-only update. All sessions terminal. Logs:
  `/private/tmp/halo-weapon-callback-*`, `/private/tmp/halo-facade-keyframe-*`.
  Full parity remains open and the architectural phase remains deferred.

- Added caller-supplied native map catalog loading in map_catalog.rs:
  parse_film_map_catalog/load_film_map_catalog, normalized lookup, typed errors,
  LoadedFilmMapCatalog/Entry and explicit fallible conversion to FilmMapBounds.
  New types preserve null versus empty maps and native u64 widths on WASM without
  changing existing catalog/export deserialization. Native JSON semantics include
  repeated fields, merged maps, replaced entries, scalar/array nulls, short/long
  arrays, Unicode field matching, invalid UTF-8 and unpaired surrogate replacement,
  and coordinate overflow that cannot be hidden by a later overwrite.
  Pinned native file-loader oracle has 328 cases with exact input bytes, error
  categories, catalog fields and lookup outputs. Focused loader/file tests passed
  (0.03s); all 79 pinned map entries project identically to existing bounds.
  Harness/generator/provenance and façade inventory updated. Full suite passed
  (476 tests, 42 ignored, 58.27s). Initial Clippy flagged nonlocal visitor impls;
  moved them to module scope, then before the test module to satisfy item-order
  lint. This is placement-only cleanup. Final Clippy (19.96s), focused loader
  tests (2 passed, 0.02s) and WASM checks passed; format/diff, generator syntax
  and native fixture identity passed. All sessions terminal. Logs `/private/tmp/halo-map-catalog-*`.
  Platform error wording, broad filesystem errors and exhaustive decimal-to-f32
  rounding remain unverified. Full v41 parity remains open; no architecture work.

- Profile façade audit found NormalizeMapName used Rust whole-string lowercase,
  which differs from native per-rune lowercase (dotted I expansion and contextual
  Greek final sigma). Switched to simple per-character conversion while retaining
  whitespace folding and ordered suffix stripping. Added native map-name oracle
  with 3672 inputs (all native uppercase runes in two contexts plus targeted
  whitespace/suffix/Unicode cases). Regression failed before the fix on ISTANBUL
  with dotted capital I; all seven profile tests passed after the fix (0.26s).
  Harness/generator/provenance and façade evidence updated. Confirmed a separate
  remaining API gap: FilmMapCatalog has the pinned embedded catalog but lacks the
  native caller-supplied JSON/file loader and its schema/default/null contracts.
  This remains queued parity work, not an architectural change.
  Full suite passed: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 568 filtered out; finished in 0.00s
  Clippy (12.19s), WASM (17.62s), format/diff, generator syntax and native fixture
  identity passed. All sessions terminal. Logs
  `/private/tmp/halo-map-names-{before,after,suite,clippy,wasm}.log`.

- Audited loaded-source façade contracts against native FilmChunkAt/Numbers,
  Load/LoadDir, Inflate/Decompresser and numbered file wrappers. Existing bridge
  oracle passed: 512 cases, 5116 lookups, 4384 packets and 923 number/position
  differences. Source loading/directory and packet-file tests passed. Updated
  façade inventory with evidence and explicit limits (generic source failures,
  filesystem/path edge cases, metadata integer widths, and scanner callers).
  Native source.Film maps to Rust FilmSource, not the high-level Film type.
  Fallback library audit found discarded native unknown-name error observations
  during accumulation. FallbackCounter now retains every positive unknown-name
  increment through unknown_triggers(), including repetitions; report aggregation
  and existing return behavior are unchanged. Extended native harness captures
  actual structured slog errors: 564 diagnostics across 392 positive cases out of
  512, with all pre-existing oracle fields verified unchanged. Focused comparison
  passed (0.11s). Full suite passed: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 567 filtered out; finished in 0.00s
  Clippy (11.80s), WASM (17.74s), format/diff passed; all sessions terminal. Logs
  `/private/tmp/halo-fallback-diagnostics-{test,suite,clippy,wasm}.log`.
  These are counter-level diagnostics, not a new Film export field or proof that
  every scanner observer is wired. Architecture remains deferred; parity open.

- Façade audit found a concrete BuildBipedTracks range mismatch: Rust automatic
  selection (limit zero) included chunks beyond numbering gaps, whereas native
  FilmChunkNumbers stops at the first gap. Sorted automatic selection now uses
  only the contiguous prefix. Positive explicit limits still tolerate missing
  requested chunks. Added a pinned native 16-case oracle using one captured Bazaar
  chunk with 250 position samples copied into controlled filename layouts;
  complete ordered tracks and error presence are compared. The first candidate
  Aquarius chunk had no usable biped band and was replaced before accepting the
  fixture. The regression failed before the fix (missing chunk 1 wrongly accepted)
  and passed afterward (2.63s). Fixture provenance and negative-limit distinction
  are documented in fixtures/weapon-track-range-v41.md; generator and façade
  inventory updated. Full theater suite passed (473 tests, 42 ignored, 54.87s);
  Clippy (11.96s), WASM (17.29s), format/diff checks and existing six-film captured
  prefix comparison (5.77s) passed. All sessions terminal. Logs
  `/private/tmp/halo-weapon-track-range-*`. Generator syntax and exact native
  oracle output match also verified.
  No architecture work started; full all-data parity remains incomplete.

- Continued the explicit fallback-site audit for score and vehicle publication.
  Existing pinned score oracle verifies 1536 cases, including 186 decreed-round
  hits, 91 absent-input controls and 50 present-empty inputs. Existing vehicle
  oracle verifies 512 full publications: 110 default-frame hits and 414 unknown
  chassis hits across 252 cases; controls include 74 disabled scans, 128 zero-step
  clocks and 25 disabled scans carrying the default-frame flag. Inspected native
  gates and Rust equivalents, post-relay chassis tally order, optional score
  caller, and final document forwarding. Recorded evidence and remaining isolation
  limits in fallback-callsite-audit.json; these are not full caller parity claims.
  Focused score test passed (11.06s) and vehicle test passed (4.27s). Logs:
  `/private/tmp/halo-fallback-{score,vehicles}-audit.log`. No production changes.
  Full parity remains incomplete; the queued architecture remains unstarted.

- Continued fallback condition audit after confirming complete CTF document
  comparison passed (79.20s). Flag carrier lookup runs twice only when free flag
  lives exist; Rust has the same gate. Existing full geometry oracle always used
  nonempty free lives, leaving the single-lookup path untested. Expanded that
  pinned native harness with a second full build using no free lives for all
  1024 cases. Every pre-existing input/output field was verified unchanged.
  New native branch has 305 positive missing-bridge cases/443 hits, 551 pickup
  fallback hits, and 61 disabled-scan controls. Rust compares complete carries,
  coverage and both fallback counts for both branches (2048 full builds).
  No production behavior changed. Focused test passed (9.70s), and Clippy
  passed (7.67s); both sessions are terminal. Logs `/private/tmp/halo-flags-empty-free-{test,clippy}.log`.
  Broad suite/WASM need not repeat for a fixture/test-only change. Remaining
  fallback condition/source and overall v41 parity audits stay open; architecture
  remains deferred.

- Audited live fallback wiring rather than assuming report setters were unwired.
  Native replay contains 19 explicit trigger sites covering 18 names; most already
  have Rust layer counters and final aggregation. Recorded native locations and
  Rust counter/publication destinations in fallback-callsite-audit.json, with
  condition/caller equivalence explicitly unproven where only located.
  Found and fixed a concrete missing contribution: identity life refinement's
  IdentityOwnerOutput.replication_gap_fallbacks was retained but omitted from
  ReplayDocument.coverage.fallbacks. The report now adds both native trigger
  sites (owner refinement and unnamed-track splitting) through the shared helper.
  New regression uses 1024 pinned native owner outputs: 823 positive cases,
  4164 triggers, and 201 no-trigger controls; zero reports remain absent.
  Focused test passed (2.19s); Clippy/WASM/format/diff passed. Full suite passed
  (472 tests, 42 ignored, 59.32s). Complete captured CTF document session 35538
  passed (79.20s); it is terminal. Logs
  `/private/tmp/halo-fallback-owner-{test,suite,clippy,wasm,ctf}.log`.
  Initial test compile used the wrong coverage type name, corrected to
  ReplayCoverage with no dependency/API changes. Final code stays film-local.
  Full parser fidelity/corpus/facade verification remains incomplete. No new
  architecture work has begun.

- Wired native registry result into Film::try_from_chunks using the same first
  type-1 bootstrap source as before. Film.registry_diagnostics retains source
  chunk index, actual byte length and native TruncatedBytes without duplicating
  the registry. Option/default serialization keeps older exports explicitly
  unavailable rather than fabricating zero-tail evidence. Constructor still
  rejects incomplete headers and maps bootstrap failures through its established
  high-level contract; lower-level parse_registry_chunk retains native outcomes.
  Expanded the real Film pipeline regression with structural-end zero tail,
  nonzero bootstrap index, portable roundtrip and older-export absence. Initial
  test incorrectly expected Film construction from a truncated registry; existing
  identity decoding rejects that input. Corrected regression checks the three-byte
  tail via parse_registry_chunk and preserves the high-level rejection. Final
  focused run passed (1.41s). Full suite 82286 finished with 470 passing tests
  and only the corrected test-assumption failure. Final full suite passed:
  471 tests, 42 ignored (65.04s), `/private/tmp/halo-registry-film-suite-final.log`.
  Clippy/WASM/format/diff passed on unchanged production code. Logs `/private/tmp/halo-registry-film-{test-final,suite,clippy,wasm}.log`.
  Updated facade inventory. Full parity still requires remaining declaration,
  source/observer/fallback wiring and positive complete-film coverage audits.
  Architectural separation remains deferred.

- Added parse_registry_chunk returning FilmRegistryRead and typed
  FilmRegistryReadError::StillCompressed. Preserves native successful empty/short
  and truncated outcomes, TruncatedBytes, and explicit complete-header presence.
  Compatibility parse_registry keeps its Option contract; shared zlib detection
  avoids divergent classification. For a missing header, compatibility version
  words are zero but header=None explicitly marks them unrecorded. End byte is
  zero for an incomplete header. Returned diagnostics serialize without loss.
  Pinned oracle now has 116 cases around headers and whole-block boundaries,
  named/empty blocks, compressed signatures and 11 structural endings. Compares
  error category, truncation flag/count, fingerprint, ordered names/levels and
  serialization. Expanded final five-test run passed (116 oracle cases),
  Clippy/WASM/format/diff checks passed.
  Logs `/private/tmp/halo-registry-result-{test-final,clippy,wasm}.log`.
  Registered generator and updated facade inventory. Film constructor/portable
  propagation of the new diagnostics remains an explicit next gap; this does not
  establish full registry-source fidelity or all-data parser parity.
  Other facade/corpus gates remain open and the architecture refactor is deferred.

- Began explicit decfilm facade audit: facade-contract-inventory.json inventories
  all 166 top-level declarations with reference line numbers/hash. Twelve located
  contracts are marked partial; unreviewed entries are not claims of missing
  implementations or API parity. The audit identified missing explicit last-chunk
  control for weapon shots/damages. Added scan_weapon_shots_through and
  scan_weapon_damages_through, scanning native numbered 1..=n, erroring on missing
  requested chunks, ignoring caller slice order, and returning empty for n<=0.
  Existing contiguous-prefix convenience APIs and their validation remain intact.
  Added pinned native 36-case range oracle (six source arrangements, six limits),
  comparing success/failure, ordered shots/damages and landing base. Initial test
  failed on JSON 0 versus 0.0 representation only; changed damage comparisons to
  typed WeaponDamage values. Final focused tests passed (3 tests, 4 ignored).
  Full suite passed (470 tests, 42 ignored, 54.22s); Clippy/WASM passed on unchanged production
  code before that assertion-only correction. Logs
  `/private/tmp/halo-weapon-range-{test-final,suite,clippy,wasm}.log`.
  Full filesystem adapters/errors, stateful damage range fixtures, remaining
  facade contracts, all-source fidelity and positive corpus gaps remain open.
  No claim of full parity and no architectural refactor.

- Added cache directory/load wrappers and native lexical path composition.
  Rust Path::join previously discarded earlier cache roots for absolute-looking
  IDs; cache paths now append and clean like the pinned Unix filepath.Join.
  New 56-case native oracle covers empty/relative/absolute roots and IDs, dots,
  parent segments and root clamping. Directory wrappers resolve the native
  basename/two-parent convention and delegate to existing open/load semantics.
  Cache creation now explicitly requests Unix 0755 directories and 0644 files,
  matching Go before umask; existing paths remain untouched. Subprocess tests
  isolate permissive 000 and restrictive 077 umasks from concurrent tests.
  Tests also check wrapper loading against the same partial cache source.
  Five focused tests passed, including both umask subprocesses. Clippy, WASM,
  formatting and diff checks passed; logs
  `/private/tmp/halo-cache-paths-{test,clippy,wasm}.log`.
  Native path fixture is registered in the oracle generator. Windows path rules,
  non-UTF8 filenames and native JSON Unicode/invalid-byte behavior remain unproven.
  Full parser fidelity/facade/corpus gates remain open; architecture is deferred.

- Added raw cache writing and cache-directory preparation to film_cache.rs.
  FilmCacheWriteChunk retains native integer widths and raw data; serialization
  emits index/type/start/duration metadata in input order. Existing paths and
  historical manifest bytes are preserved; missing chunks are written before a
  new manifest. Explicitly documented non-atomic failure behavior and distinguished
  cache persistence from native Film re-encoding. Added pinned native filesystem
  snapshots for eight cases: empty input, ordinary write, duplicate chunk numbers,
  existing manifest, existing chunk, blocked chunk directory, blocked manifest
  directory after chunk writes, and an existing directory at a chunk path.
  Fixture includes exact file bytes and path kinds; registered in generator.
  Initial test build failed only because the harness decoder used an unavailable
  base64 crate; switched oracle encoding to hex without adding a dependency.
  Final focused tests passed (3 tests), Clippy and WASM passed; logs
  `/private/tmp/halo-cache-writer-{test,clippy,wasm}.log`.
  Cache API parity remains partial: directory wrappers, Go path normalization,
  Unicode/invalid-UTF8 JSON, creation modes under unusual umasks and filesystem
  edge cases remain to verify. Other v41 fidelity/corpus/facade gates remain open.
  Architectural redesign is still deferred until full parity completion.

- Added initial native disk-cache reader in film_cache.rs (native targets only).
  FilmCacheSource opens manifests with absent-versus-invalid distinction, retains
  exact manifest bytes, metadata order and native signed integer ranges, reads
  raw chunks by manifest position, and loads partial caches through FilmSource's
  existing numbered-directory path. Cache listing is sorted and absent caches
  return empty. Native 16-case JSON oracle covers nulls, duplicate keys, ASCII
  case folding, malformed shapes/types and signed overflow. Filesystem regression
  covers source order, missing/out-of-range chunks, extra files, metadata joins,
  raw JSON retention and invalid/absent manifests. Both focused tests passed;
  regenerated Go fixture is byte-identical. Harness is registered in generator.
  Clippy, WASM and formatting checks passed. Logs
  `/private/tmp/halo-cache-reader-{test,clippy,wasm}.log`. This is initial reader coverage, not all
  cache parity: directory wrappers, raw writer, Unicode/invalid-UTF8 JSON edge
  cases, Go filepath normalization and full filesystem error behavior need audit.
  Source conversion explicitly errors if native int64 chunk_type exceeds i32;
  raw manifest metadata remains intact. Cache persistence is not film re-encoding.
  Prior recovery latest captured comparison passed (99.65s); older broad position
  comparison passed (677.75s). No previous corpus runs remain active.
  Full all-data v41 parity remains incomplete and architecture stays deferred.

- Added speculative equipment-recovery read retention through the shared observed
  component walker. EquipmentChangeStream.recovery_attempts records source,
  optional native packet ordinal, slot, candidate offset, candidate outcome and
  all ordered component reads (status, bounds, full fields/references/hooks).
  Candidate=true is only component-walk success; counter/window acceptance stays
  separate. Header-only rejections have no component read and retain source bytes.
  The existing Film export carries these diagnostics. Native remaining-context
  oracle now captures the actual recovery walk hooks plus independent component
  attempts; all older fields verified unchanged and the two native callback
  sequences verified equal. 512 cases, 734 attempts, 258 rejected attempts,
  244 callbacks including 12 from rejected walks. Focused endpoint/status/source/
  candidate/serialization regression passed (0.23s), Clippy/WASM/format/diff passed.
  Full suite passed (464 tests, 42 ignored, 64.83s). Latest captured gameplay
  comparison passed (99.65s);
  logs `/private/tmp/halo-recovery-observations-{suite,captured}.log`.
  Older broad position comparison 88270 completed successfully (677.75s).
  Complete captured equipment recovery, VIP/bomb, source/portable fidelity and
  remaining integrated observer/cache/facade mapping are still acceptance gaps.
  Next cache audit source is filmcache/{filmcache.go,write.go}: Open distinguishes
  absent from malformed manifests; strict Chunk uses manifest position, LoadFilm
  instead loads existing files by native number. Write persists raw chunks before
  the manifest and preserves existing files. This is not native re-encoding.
  No cache implementation claim or architecture refactor is made here.

- Extended rejected-component retention to held-weapon and ability-state scans.
  Weapon oracle: 512 contexts, 267 rejected attempts and 11 callbacks, previous
  fixture fields unchanged. Expanded ability testing exposed and fixed a real
  disabled-anchor-body panic: successful dispatch does not imply BodyOK or
  coordinates. Grapple publication now follows the typed native hook's BodyOK,
  inner and position. Ability publication also follows actual hooks independently
  of walker bounds acceptance; rejected published reads remain explicitly marked.
  Added native grapple/impulse scanner comparisons to all 512 biped contexts:
  275 grapple reads, 24 broken tag-3 bodies (11 disabled), 551 impulse reads and
  71 impulse publications. Focused test passed; Clippy/WASM/format/diff passed.
  Initial full run had 463 pass/1 fail solely from new oracle packet timestamp
  setup (0 versus 1000); corrected native setup, regenerated fixture and verified
  no other oracle field changed. Final full suite passed: 464 tests, 42 ignored (64.70s),
  log `/private/tmp/halo-grapple-body-suite-final.log`. Latest-code four-film
  inventory/channel/ability comparison passed (104.23s),
  `/private/tmp/halo-grapple-body-captured.log`.
  Older broad biped-position run 88270 remains active, log
  `/private/tmp/halo-weapon-ability-rejected-captured.log`; it predates this fix.
  Previous gameplay rejection captured run 35127 passed (94.90s).
  READER_CONTEXT_PORT.md records the remaining offset-only equipment recovery
  diagnostic gap and its native candidate publication gate. Complete source/Film
  fidelity, integrated reader/capture, cache/facade and positive full VIP/bomb/
  equipment-recovery coverage remain open. Full parity is incomplete; the
  requested architecture remains queued in NEXT_PHASE.md, with no refactor begun.

- Extended the shared rejection-retaining walk to InventoryDeltaStream,
  AbilityChargeStream and UnitEquipmentStream. Their rejected_components fields
  preserve intermediate/target failures with source, packet ordinal when known,
  raw dispatch status, separate bounds, decoded fields/references and callbacks.
  Native successful publication gates remain unchanged. The 512-context fixture
  checks each scanner's stopping point: inventory 224 rejections/4 callbacks,
  charges 236/30, unit equipment 228/22, plus complete serialization and source
  identity. Focused regression passed (0.33s), full suite passed (464 tests,
  42 ignored, 58.84s), Clippy/WASM/format/diff checks passed. Logs:
  `/private/tmp/halo-gameplay-rejected-{test,suite,clippy,wasm}.log`.
  Previous biped rejection four-film run session 78608 is terminal and passed
  (94.68s); do not restart. Fresh latest-code four-film comparison is running on
  session 35127, log `/private/tmp/halo-gameplay-rejected-captured.log`, currently
  matched Bazaar, Aquarius and Bandit EVO; poll that handle to completion.
  Remaining standard success-only record-walk caller: weapon_changes.rs. Special
  ability_states.rs loop drops failed intermediate reads, though target partial
  bodies are retained; equipment_recovery.rs uses an offset-only walk. Audit
  those independently next. Full parity remains incomplete; refactor deferred.

- Confirmed and repaired rejected-attempt loss in the biped channel walker.
  Native callbacks occur before walkComponentsAt checks success/bounds. The Rust
  helper now offers an observed path before that gate; BipedChannels retains
  rejected_components with source, slot/record, component index, raw dispatch
  status, separate in_bounds, fields, references and callback diagnostics.
  Successful camo/ability publication gates are unchanged. Expanded existing
  biped-context native harness/fixture with attempts and actual walk callbacks;
  all old fixture fields were compared unchanged. 512 cases: 2408 attempts,
  2156 visits, 2771 callbacks; complete traversal has 252 failures/47 callbacks,
  and the channel scanner retains its 235 failures/29 callbacks before index 48.
  Focused regression and roundtrip checks passed (0.26s), Clippy/WASM and
  formatting/diff checks passed. Full suite session 47291 passed: 464 tests,
  42 ignored (59.24s). Focused four-film gameplay comparison session 78608
  remains running; poll that handle.
  Logs `/private/tmp/halo-biped-rejected-{test,suite,captured,clippy,wasm}.log`.
  Earlier captured creation-source run session 77309 completed successfully in
  644.76s; it is terminal, do not restart. Its older binary validates source
  selection, not the latest rejection fields.
  READER_CONTEXT_PORT.md records scope. Other success-only traversal consumers
  (inventory, charges, equipment) still need rejected-attempt retention auditing;
  full native fidelity and complete mode coverage remain open. Refactor deferred.

- Added independent reader replacement/restoration sequence oracle:
  halo_rust_reader_sequence_test.go.txt / reader-sequence-v41.json.zlib, registered
  in generate_oracles.py. Native mutable reader swaps two profile/observer pairs,
  suppresses captures, restores previous values and reinstalls a whole context.
  Rust explicit per-attempt contexts match all 1536 reads over 256 cases:
  endpoints, status, ordered callbacks/actions and inactive receiver outputs.
  All 34 rejected reads retain their 34 callbacks; serialization, restored-result
  equality and suppression-independent raw fields are checked. Focused test and
  Clippy passed, plus formatting/diff and generator syntax checks. Logs:
  `/private/tmp/halo-reader-sequence-{test,clippy}.log`. No production changes;
  full suite/WASM were not unnecessarily repeated for this test-only addition.
  READER_CONTRACT.md documents scope, linked from READER_CONTEXT_PORT.md.
  This is evidence for four component families, not mutable API/alias/capture-state
  parity or every integrated rejected-intermediate callback path. These remain
  open; inspect full-context scanner walk retention next. Captured creation
  source comparison session 77309 remains live and is the only running check
  from these checkpoints. Full parity remains incomplete; architecture deferred.

- Automatic creation scanning now reports native source/missing-band errors.
  Film catches this scanner independently and retains biped_creations_error,
  preserving packets and other streams in both map and explicit-encoding
  constructors. Constructor regression covers healthy/no-band cases and full
  serialization. Extended creation-source-v41 with independent valid ti35/ti37
  keyframes and native automatic results over the same 512 source cases:
  128 automatic successes (320 records), 128 source errors, 256 missing-band
  errors. Explicit sparse results remain 900 records; all 1220 ordered records,
  fields, counters, memberships and error categories match. Both focused tests
  and the Film regression passed; Clippy, WASM and format/diff checks passed.
  Full Theater suite passed; see `/private/tmp/halo-creation-auto-suite.log`.
  Other logs `/private/tmp/halo-creation-auto-{test,film,clippy,wasm}.log`.
  Captured source-selection comparison session 77309 is still live, using the
  preceding source-selection build; poll it without restart. No other sessions
  from this checkpoint remain active. NativeComponentCapture and the components
  reader are the next entry points for the remaining profile/observer retention
  audit. Full parity and positive complete mode coverage remain open; refactor
  deferred and work uncommitted.

- Added native creation source/sparse-band parity: public
  scan_biped_creations_for_slots accepts FilmSlotBand, retains exact selected
  membership in BipedCreationStream.slots, and records native packet ordinals.
  Nonempty range scans delegate to native selected-prefix/tolerant framing.
  Created and registered halo_rust_creation_source_test.go.txt and compressed
  creation-source-v41 fixture. Pinned Go produced 512 cases: 288 successes,
  128 source errors, 96 empty-band errors, 900 ordered records. All native fields,
  counters, membership exclusions and error precedence match; malformed tails,
  sparse bands, reordered/gapped/duplicate metadata and prologue rejections covered.
  Existing 2048 native prologue cases also pass. Full suite: 463 passed,
  42 ignored (58.63s). Clippy, WASM, fmt/diff, manifest and generator syntax checks
  passed. Logs `/private/tmp/halo-creation-source-{test,suite,clippy,wasm}.log`.
  Fresh full captured comparison is running on session 77309, log
  `/private/tmp/halo-creation-source-captured.log`; poll it, do not restart.
  Confirmed remaining difference: legacy automatic scan returns empty when no
  biped band is found; native automatic ScanBipedCreations returns an error.
  Explicit sparse entry point matches source-before-empty-band error precedence.
  Missing-band Film diagnostics remain open. Corrected stale biped_creation.go
  manifest status from ported to partial to reflect that gap. Architecture
  remains deferred; no commit/push.

- Both previously running captured checks are terminal and passed. Session 30600
  completed the broad equipment/all-channel comparison in 652.25s (older binary).
  Session 21016 completed the newer focused inventory/channel comparison in
  88.71s: 8636 inventory records, 4401 camo readings, 33 ability ranks, 39 charges,
  4 impulses and 26 grapple reads; all exported ordinals, values and counters
  match. Do not restart or describe these handles as running. Logs remain
  `/private/tmp/halo-equipment-ordinal-captured.log` and
  `/private/tmp/halo-channel-ordinal-captured.log`.
  Added BipedCreationStats.slots/truncated and BipedPickupStats.other_type.
  Native payload-only creation fixtures leave Slots zero; the loaded range API
  sets inclusive range cardinality. Truncated is zero behind the full-prologue
  gate; pickup OtherType is zero behind the 0xC4 gate. Tests now compare exported
  counters rather than manufacturing zeros, and check loaded creation counts,
  records and serialization across the 2048 native cases.
  Full suite passed: 462 tests, 42 ignored (56.50s); Clippy, WASM, formatting and
  diff checks passed. Logs `/private/tmp/halo-biped-counter-{suite,clippy,wasm}.log`.
  Next confirmed producer gap: native creation scanning accepts sparse SlotBand
  membership and selected-prefix tolerant framing; Rust still accepts a range
  and strict packet indexing, with source/empty-band errors also unaudited.
  Keep that scope separate from declaration-field presence. Full parity remains
  incomplete, architecture deferred, work uncommitted.

- Closed exported ordinal gaps in BipedCamoState, BipedAbilityEmission,
  AbilityCharge, AbilityImpulse and GrappleRead. BipedChannelRead carries the
  same attribution alongside raw component fields. Loaded scans and recovered
  equipment emissions derive ordinals from native framing; independent synthetic
  inputs keep None. Corpus assertions now inspect exported ordinals directly.
  Existing independent channel and 4096-case ability fixtures decode inside
  keyframe/delta envelopes and verify ordinals plus portable roundtrips, while
  retaining native value/counter comparisons. Full suite passed: 462 tests,
  42 ignored (65.12s). WASM passed. Clippy first flagged test-helper placement;
  moved it before the test module, and final Clippy passed. A later test-only
  extraction shares all channel output/counter assertions between the full corpus
  test and focused inventory/channel test. Captured run session 21016 remains
  active; log `/private/tmp/halo-channel-ordinal-captured.log`. Earlier broad
  captured comparison session 30600 remains active and uses an older binary.
  Biped creation Slots/Truncated, pickup OtherType, remaining source contracts
  and complete positive mode coverage remain acceptance gaps. Refactor deferred.

- Continued declaration audit through grammar_bipede.go and grammar_monde.go.
  Found and fixed missing inventory PacketIndex export: InventoryDeltaRead now
  carries optional packet_index from native framing, counting keyframes too;
  unframed caller-supplied component payloads keep None. The independent 16-case
  inventory fixture also runs inside keyframe/delta envelopes, checks ordinal 1
  versus byte offset 33, every published value/counter, and serialization.
  Focused fixture passed. Full Theater suite: 462 passed, 41 ignored (59.54s);
  Clippy, WASM, formatting and diff checks passed. A subsequent test-only
  extraction shares inventory assertions with a new focused four-film captured
  test (now 42 ignored tests); final Clippy and format/diff checks also passed.
  Captured inventory run session 49538 completed successfully (89.50s):
  Bazaar 9, Aquarius 0, Bandit EVO 3344, Ranked Oddball 5283; all 8636 records,
  exported ordinals and counters match. Log:
  `/private/tmp/halo-inventory-ordinal-captured.log`.
  Earlier equipment/all-channel captured run remains active as session 30600;
  its binary predates this inventory change and is not inventory validation.
  World declaration mapping distinguishes native WorldObjectTrack from legacy
  ProjectileTrack; recorded death/occupancy harvest regression passed.
  Explicit new declaration gaps: camo/grapple packet ordinals, creation
  Slots/Truncated, pickup OtherType. Reader contracts, complete positive mode
  evidence and full source-to-portable fidelity remain open. No architecture
  refactor, commit or push. NEXT_PHASE.md retains the queued requirements.

- Equipment declaration audit found a concrete output omission: native changes
  and spawn events expose PacketIndex, while Rust retained only the source byte
  offset. Added optional packet_index to both outputs. Loaded scans derive it
  from all natively framed packet types; standalone assembly leaves None when
  source bytes are unavailable. Synthetic fixture payload offsets previously
  used as ordinals remain a fixture-only convention, not a production mapping.
  Mixed keyframe/delta-envelope regression verifies ordinals 1/2 versus byte
  offset 33, both serializers and standalone unknowns. Focused test passed;
  existing equipment subset: 17 passed, 2 ignored. DATA_CONTRACT_INVENTORY.md now
  maps every grammar_equipement.go declaration and records producer limitations.
  The declaration file moves pending -> partial, not overall parity completion.
  Captured comparison now checks the exported ordinals directly instead of
  reconstructing them inside the test. Logs:
  `/private/tmp/halo-equipment-ordinal-{test,source-test,suite,captured,clippy,wasm}.log`.
  Full suite: 462 passed, 41 ignored, no failures (58.99s). Clippy with
  warnings denied, WASM, formatting/diff and manifest JSON checks passed.
  Captured comparison remains running on session 30600, verified after the
  other checks completed; poll this handle, do not restart or claim passed.
  Positive complete recovery, biped/world contracts and full parser fidelity
  remain open. Architecture deferred; no commit/push.

- Audited the broader inventory against the live pinned reference: all 485
  production Go paths are present in the manifest, no stale/missing paths, and
  every SHA-256 matches. Added DATA_CONTRACT_INVENTORY.md with explicit fields
  for types/source.go and types/killsource.go, plus source-position/number and
  payload-ownership distinctions. Corrected those two declaration-only entries
  from pending to ported-v41; this does not promote their producers or broader
  fidelity claims. Focused existing source-loading, assist and kill-pair native
  comparisons pass; logs `/private/tmp/halo-contract-{source,assists,pairs}-test.log`.
  Manifest JSON and diff checks pass. No production code changed, so the full
  suite was not repeated. Cache reader/writer and facade mappings remain pending
  and explicitly distinct from decoding: raw cache writes do not establish native
  film re-encoding. Next field-level audit targets grammar_bipede/equipement/monde
  and replay contracts, while reader replacement and complete VIP/bomb evidence
  remain open. Architecture deferred; changes remain uncommitted.

- Preserved managed-property partial failure output from native
  `zone_state_scan.go`: Slots is recorded before archetype lookup. The new
  diagnostics API shares implementation with the existing Result API, and both
  map-aware and explicit-encoding constructors retain the scan with its error.
  Zone/flag consumers read no facts from an empty failure reading list. Complete
  constructor checks verify one ti=13 slot, missing-archetype error and both Film
  serialization paths. Registered Go fixture `managed-setup-v41.json.zlib` has
  256 source/registry cases matching every output field and error category:
  64 source failures, 96 archetype failures retaining 160 slots, 96 successes.
  Focused comparisons passed. Logs:
  `/private/tmp/halo-managed-diagnostics-test.log` and
  `/private/tmp/halo-managed-setup-{go,test,suite,clippy,wasm}.log`.
  Full Theater suite: 461 passed, 41 ignored, no failures (54.22s). Clippy
  with warnings denied, WASM compilation, formatting/diff, manifest JSON and
  generator syntax checks passed. All sessions are terminal.
  Positive managed readings retain their separate existing fixture evidence.
  Broader v41 parity and complete-mode coverage remain open; no architecture
  refactor, commit or push.

- Preserved native navpoint partial results on setup errors. The diagnostics
  API returns the allocated scan and slot/keyframe census alongside failure;
  map-aware Film retains both it and `navpoint_radial_error`. Failed setup has
  no readings, so bomb replay gains no facts. The full-constructor regression
  now verifies one observed/band slot and keyframe entry survive with the error
  and through serialization. Registered pinned-Go fixture `navpoint-setup-v41`
  covers 256 cases: 64 source errors, 96 archetype errors retaining 160 total
  census entries, 96 normal scans. Focused Rust comparison passed (0.05s).
  Logs: `/private/tmp/halo-navpoint-setup-{go,test,suite,clippy,wasm}.log`.
  Full Theater suite: 460 passed, 41 ignored, no failures (53.92s). Clippy
  with warnings denied, WASM compilation, formatting/diff, manifest JSON and
  generator syntax checks passed. All validation sessions are terminal.
  Harness creation first used the wrong relative checkout path, then a copied
  test fragment had an unmatched delimiter; both were corrected before the
  successful comparisons. No native production grammar was changed.
  Positive complete bomb coverage, broader context/replacement audits and the
  full parity inventory remain open. Architecture deferred; no commit/push.

- Completed the captured movement regression after native source-selection
  repair: all 17,600 ordered movement records and full counters match across
  Bazaar idle, Aquarius, Bandit EVO and ranked Oddball (159.86s). Session 57879
  is terminal, exit 0; do not restart it. Log:
  `/private/tmp/halo-movement-source-captured.log`.

  Expanded the pinned source/setup oracle to 384 cases without changing the
  original 128 inputs/outputs (new labels and Go declaration-order normalization
  aside). Missing-biped and absent-component groups establish exact error
  categories, retained map-width counters, source-before-archetype ordering,
  and absent-channel success: 96 cases in each of four outcome categories.
  Focused Rust comparison passed (0.06s); generator/harness stay registered.
  Logs: `/private/tmp/halo-movement-setup-{go,test}.log`.
  This checkpoint changes tests/evidence only; the previous production repair
  already matched the expanded setup matrix. Formatting/diff, generator syntax
  and manifest JSON checks passed. The prior full suite/Clippy/WASM checkpoint
  remains applicable; no unnecessary repeat of the full suite. Explicit invalid
  Rust encoding inputs are separate from the valid native context matrix.
  Overall v41 parity remains incomplete and architecture deferred. No commit/push.

- Matched native movement source selection and packet framing. Movement scans
  now use the native contiguous metadata prefix and tolerant packets, with the
  new native anticipated-binding builder using the same selection. Strict Film
  admission and the older strict binding-table API remain distinct. Registered
  Go harness/fixture `movement-source-v41.json.zlib` has 128 cases across gaps,
  reordered/duplicate/negative metadata, short tails and oversized lengths.
  Rust matches all counters, 200 packets, 160 dated declarations and 32 no-prefix
  errors. These source-focused cases contain no positive movement transitions;
  captured movement regression is run separately. Go setup initially named a
  nonexistent cached-profile flag; using the actual corruption-cache flag fixed
  the harness without changing native production code.
  Logs: `/private/tmp/halo-movement-source-{go,test,suite,captured,clippy,wasm}.log`.
  Full Theater suite: 459 passed, 41 ignored, no failures (59.08s). Clippy
  with warnings denied, WASM compilation, formatting/diff, manifest JSON and
  generator syntax checks passed. Separate four-film movement comparison is
  still running on session 57879 (confirmed live after suite completion); poll
  that handle rather than restarting it. Captured regression is not yet claimed
  passed. No remaining internal production callers use the strict anticipated
  binding API; only its existing test does, plus possible external consumers.
  Remaining audits include context-derived precision, setup-error ordering, other
  strict binding callers, reader replacement and complete-film evidence.
  Architecture stays deferred; no commit or push.

- Preserved raw movement scanner failure counters separately from replay input.
  New `scan_movement_states_with_diagnostics` shares the scan with existing APIs
  and retains MapWidths before source/archetype errors, matching native
  `movement_states.go`. Film exports those counters as
  `movement_states_error_stats`; its failed movement stream remains absent so
  replay stance assembly still uses native reset-to-default behavior. Direct
  missing-source/biped tests assert exact counters and serialization; the full
  constructor checks nonzero resolved widths and Film roundtrip.
  Logs: `/private/tmp/halo-movement-diagnostics-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 458 passed, 41 ignored, no failures (53.73s). Clippy
  with warnings denied, WASM compilation, formatting/diff and manifest JSON
  checks passed. Initial direct-test compile errors were equality assertions
  against DecodeError; comparing its existing messages fixed the test only.
  This is raw-data preservation within parity, not the deferred architecture.
  Strict source admission, context-derived defaults and overall error ordering
  remain separate audits. No commit or push.

- Map-aware Film now retains independent movement-state, navpoint, combined
  biped-channel, equipment-change, ability-state and ability-charge errors instead
  of aborting construction. Equipment changes explicitly report unavailable
  channel input if their prerequisite fails. Existing replay stance/bomb
  consumers use empty streams/default stats after failure, matching native
  `film_scan_mouvement.go`, `film_scan.go` and `bomb_armings.go` behavior.
  A full-constructor regression ends the registry before ti=12 and supplies
  separate navpoint/biped keyframe chunks; all expected errors coexist with
  replication, march facts, positions and precision context, and the Film
  round-trips. Healthy and inventory/weapon-only controls verify no unrelated
  error fields. The initial fixture combined keyframes in one chunk and missed
  the biped band under first-keyframe discovery; separate chunks plus an explicit
  band assertion corrected that test assumption.
  Logs: `/private/tmp/halo-independent-scans-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 457 passed, 41 ignored, no failures (54.46s). Clippy
  with warnings denied, WASM compilation, formatting/diff and manifest JSON
  checks passed.
  Standalone native error counter preservation/setup ordering, source admission,
  reader replacement contracts and remaining complete-film evidence still need
  audit. Architecture remains deferred; no commit or push.

- Fixed equipment-state result loss in the explicit-encoding Film constructor.
  Like the map-aware path, it now retains native partial counters alongside the
  error. A full-constructor regression supplies one ti=37 keyframe slot with a
  registry ending before ti=37: Slots=1, no samples, missing-archetype error,
  continued replication and full Film roundtrip all pass. Native evidence is
  `equipment_state.go::ScanEquipmentState`, which sets Slots before lookup.
  The first test envelope was correctly rejected as truncated; adding a complete
  terminator block fixed the fixture without changing bootstrap admission.
  Logs: `/private/tmp/halo-equipment-export-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 456 passed, 41 ignored, no failures (53.84s). Clippy
  with warnings denied, WASM compilation, formatting/diff and manifest JSON
  checks passed. Changes remain uncommitted on the experiments branch.

  Continued scanner error reachability audit: fire/grenade missing-source errors
  are already caught. Equipment-change errors require absent ti=35 or failed
  indexing of the same immutable chunks already checked by the constructor.
  The next actual missing-biped blocker is earlier movement-state scanning:
  native replay treats failure as nonfatal, Rust still propagates it. Native
  bomb/navpoint replay similarly catches scanner failures. See
  `READER_CONTEXT_PORT.md` for exact callers and the remaining constructor gates.
  The full parity goal remains active; architecture remains deferred.

- Completed the legacy registry guard audit. Fixed pawn-index validation now
  gates the three legacy signature passes only, reporting an explicit limitation
  on incompatible layouts; native registry-based scans remain enabled. Bootstrap
  registry decoding is still required. The complete-constructor regression now
  covers inventory failure, held-weapon failure, both failures, and a healthy
  control, asserting independent channel retention and full Film serialization.
  A captured-record pipeline regression verifies that incompatible layouts
  suppress otherwise valid legacy signatures while preserving packets, summary,
  highlights, and the changed registry name. This closes the inventory error
  test gap described in older checkpoints below, not the broader source-fidelity
  or optional-scanner/setup-order audits. No architecture refactor has started.
  Validation logs: `/private/tmp/halo-registry-guard-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 455 passed, 41 ignored, no failures (53.99s). Clippy
  with warnings denied, WASM compilation, formatting/diff and manifest JSON
  checks passed. Changes remain uncommitted on the experiments branch.

  The previously running captured-document job (session 41028) has completed:
  all 6 ignored document tests passed, including complete raid and decoded-kill
  documents (5505.41s). Its binary predates recent scanner/helper changes, so this
  validates that older checkpoint only. Log:
  `/private/tmp/halo-document-report-captured.log`. Do not restart that job or
  mistake older "still running" status entries for current state.

- Audited replay `film_scan.go` error handling: inventory and held-weapon failures
  are explicitly nonfatal. Rust map-aware Film assembly now retains
  `inventory_deltas_error` and `weapon_changes_error` instead of propagating those
  scanner failures; healthy streams remain present and absent error fields are
  omitted from portable JSON. Other scanner error paths remain under audit.
  A complete-constructor regression uses the captured bootstrap/keyframe plus an
  explicit synthetic biped keyframe, then renames only weapon identity components
  in the bootstrap. The held-weapon failure is retained while inventory, ability,
  equipment, biped positions and replication survive; the entire Film round-trips.
  Focused test passed. Logs: `/private/tmp/halo-film-scan-errors-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 455 passed, 41 ignored, no failures (53.56s). Clippy with
  warnings denied, WASM compilation, formatting/diff and manifest JSON checks
  pass. Raid session 41028 was confirmed live; no commit, push or restart.

  The planned inventory mutation exposed an earlier gate: `decode.rs::registry`
  requires hard-coded legacy pawn component names (e.g. ammo at index 30), so it
  rejects the mutated registry before native scanner assembly. The inventory
  retention branch is added from the native error-handling contract, but its
  full-constructor failure scenario remains unverified behind that guard. Do not
  claim this closes inventory error parity. The guard remains intact because
  legacy signature readers depend on its assumptions; audit their consumers and
  retain native/unknown data without running incompatible legacy decoders before
  changing it. Initial held test also lacked a biped band; the explicit keyframe
  control now asserts that the scanner is reached. Architecture remains deferred.

- Matched inventory and held-weapon source availability: missing candidate chunks
  are skipped before counters; invalid ranges in available sources return
  `Truncated` rather than indexing/panicking. The new native public-scanner oracle
  has 512 cases across four availability configurations and separate/combined
  grenade-count, selection, weapon identity, magazine and reserve masks. Rust
  matches all 220 inventory records, 152 held changes, every native field/counter
  and the other channel checks sharing those packets. Negative tests cover source
  arithmetic overflow and out-of-bounds ranges for both APIs. Focused test passed.
  Registered harness/fixture: `inventory-source-availability-v41.json.zlib`.
  Logs: `/private/tmp/halo-inventory-source-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 454 passed, 41 ignored, no failures (58.10s). Clippy with
  warnings denied, WASM compilation, formatting/diff and generator syntax/manifest
  JSON checks pass. Raid session 41028 was confirmed live. No commit or push.
  This synthetic one-packet-per-case fixture is source-availability evidence;
  existing stateful/whole-film fixtures remain the evidence for weapon history
  and global ammo refusal (no refusal occurs in this new fixture).
  Next: shared setup-error ordering and Film retention. Native scanners resolve
  chunk numbers, slot band, layout and registry before component absence checks;
  several supplied-context Rust helpers currently enter at registry lookup.
  Do not add strict guards to helpers without auditing Film assembly's `?`
  propagation: native replay can retain other channels after scanner failure.
  Architecture work remains deferred; source-level pending entries stay open.

- Extended missing-source parity to ability charges, predicted/non-predicted
  ability state (impulses and grapple), and unit-equipment reads. Each skips an
  absent candidate chunk before counting records; malformed available-payload
  ranges retain the existing error contract. The 512-case native public-scanner
  fixture has 128 cases each for available-only, missing-after, missing-only and
  missing-before requests. Separate and combined masks exercise 114 published
  charges, 42 impulses, two grapple reads and 155 equipment emissions. Rust matches
  every native output field and counter, plus the existing combined-channel
  comparisons using the same packets. Registered harness and self-contained
  synthetic input/output fixture: `biped-source-availability-v41.json.zlib`.
  Initial comparison failed because the Rust test supplied no movement profile
  while Go used its default. The fixture now carries the explicit native profile;
  all native outputs remained unchanged, and the focused comparison passed.
  No production body-grammar change was needed. Logs:
  `/private/tmp/halo-biped-source-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 453 passed, 41 ignored, no failures (57.81s). Clippy with
  warnings denied, WASM compilation, formatting/diff and generator syntax/manifest
  JSON checks pass. Raid session 41028 was confirmed live this turn. No commit
  or push; all new changes remain within the film module.
  Inventory and held-weapon missing-source wrappers remain next, along with the
  broader setup-error ordering audit. Supplied registry/layout/slot contexts are
  explicit test inputs; these cases do not claim complete bootstrap resolution
  or independently annotated physical actions. The architecture phase is deferred.

- Matched combined biped-channel missing-chunk behavior to native
  `walkDeltaBipedRecords`: skip absent sources before counting any records or
  readings, and retain records from available chunks. Expanded the native public
  scanner fixture from 256 to 1,024 cases, preserving every original value. Added
  missing-before, missing-after and missing-only requested chunk lists, including
  camo presence/absence and readable/unreadable ability bodies. Every counter,
  error-presence flag and published value matches; 192 ability emissions survive
  missing camo across the three groups with available data. The focused comparison
  passes. Overflowing/out-of-bounds payload ranges in caller-supplied anchors now
  return `Truncated` rather than panic; separate negative assertions cover these
  invalid references, which native packet discovery cannot produce.
  Logs: `/private/tmp/halo-channel-chunks-{test,suite,clippy,wasm}.log`.
  Full Theater suite: 452 passed, 41 ignored, no failures (57.45s), including
  the malformed-reference assertions. Clippy with warnings denied, WASM
  compilation, formatting/diff and generator syntax/manifest JSON checks pass.
  Raid session 41028 was confirmed live this turn; no restart, commit or push.
  This closes missing sources only for the combined camo/ability channel API;
  ability-charge/state, inventory, held-weapon and unit-equipment supplied-anchor
  wrappers still require the same source-availability audit. Native context setup
  ordering remains a separate gate. Architecture changes stay deferred.

- Corrected combined camo/ability failure coupling in `biped_channels.rs`.
  Missing camo now leaves its counters at zero and reports `camo_error` while
  preserving independently readable ability data. Shared setup errors still
  return `Err`. Film already retains BipedChannels, so the channel error and
  surviving values propagate together; absent error fields stay omitted from JSON.
  The new native public-scanner oracle has 256 synthetic packets with explicitly
  supplied registry, slot band and layout. Four groups cover camo presence/absence
  crossed with its inclusion/exclusion from the record mask. All counters, camo
  values, ability emissions/ranks, identities and timestamps match Rust; 64 ability
  emissions survive missing camo, and 64 unknown intermediate components still
  block ability reads. Serialization retains the independent error and values.
  Harness and `channel-independence-v41.json.zlib` are generator-registered.
  Focused comparison passed. Full Theater suite: 452 passed, 41 ignored,
  no failures (57.70s). Clippy with warnings denied, WASM compilation,
  formatting/diff and generator syntax/manifest JSON checks pass. Raid session
  41028 remains live on its older binary. No commit, push or architecture change.
  Final validation logs:
  `/private/tmp/halo-channel-independence-{test,suite,clippy,wasm}.log`.
  This verifies the missing-component behavior, not occurrence of this registry
  shape in a captured film or complete context-resolution error parity. The
  ability/camo manifest entries remain partial for those remaining gates.

- Closed the remaining `offline_aim.go` helper mappings. Added native compact
  mask/overflow projection with accessors on biped records and mask callbacks;
  complete index sequences remain retained. Added the diagnostic bit reader's
  separate 32-bit accumulator and panic-on-boundary contract, rather than routing
  it through tolerant zero-padding. Pinned Go's 2,048-case helper oracle compares
  all returned values and panic/overflow states (864 panics, 1,175 overflowing
  masks), including signed index extremes, empty/duplicate masks, exact buffer
  boundaries, negative widths and reads wider than 32/64 bits. Focused tests pass.
  Full Theater suite: 451 passed, 41 ignored, no failures (60.69s). Clippy with
  warnings denied, WASM compilation, formatting/diff and generator syntax/manifest
  JSON checks pass. Raid session 41028 remains live on its older binary.
  Registered the extended harness and `offline-aim-helpers-v41.json.zlib` in the
  generator. Marked only this file's mapping ported, supported by its existing
  ordinary/dynamic companion and velocity/angle oracles; broader scanner gates
  remain open. Regression logs: `/private/tmp/halo-offline-aim-{test,suite,clippy,wasm}.log`.
  Next source-verified discrepancy: `biped_channels.rs::scan_biped_channels_impl`
  requires a camo registry component before running either channel. Native
  `ScanAbilityRanks`/`walkAbilityEmissions` resolve the biped profile independently
  of camo, whereas `ScanCamoStates` alone errors on missing camo. Add independent
  channel/error retention and a pinned native missing-component fixture before
  closing the ability/camo wrapper audit. Do not infer that this registry shape
  occurs in the current captured corpus. Architecture work remains deferred.

- Ported the remaining `components_cubemap.go` public helpers: general-width
  velocity magnitude, dynamic direction with native low-word truncation, and
  direction-times-magnitude velocity. Production movement now calls that shared
  implementation. Added checked `BipedCompanions` velocity and primary aim-angle
  query methods corresponding to `offline_aim.go`; quantized fields stay intact.
  The new pinned-Go `TestHaloRustVelocityCodec` oracle contains 8,192 cases over
  widths 0..71 and compares exact float bits, including 336 infinite magnitudes
  from native out-of-range width behavior, 5,460 high-word inputs, 2,184 accepted
  companion velocities, 6,008 absent/invalid velocities and 1,171 absent aims.
  The distinction between unchecked sentinel and checked absence is preserved.
  Harness and `velocity-codec-v41.json.zlib` are registered in the generator.
  Focused codec/orientation tests pass. Full Theater suite: 450 passed,
  41 ignored, no failures (61.34s). Clippy with warnings denied, WASM compilation,
  formatting/diff and generator syntax/manifest JSON checks pass;
  logs are `/private/tmp/halo-velocity-codec-{test,suite,clippy,wasm}.log`.
  Marked `components_cubemap.go` ported; kept `offline_aim.go` partial while
  correcting stale claims about dynamic vehicle orientation. Remaining audit:
  compact masks/overflow (full index lists already retained), and diagnostic
  `ReadBitsAtForDiag` wrapper semantics. Its native fallback panics outside the
  buffer; the existing tolerant zero-padding API is not an equivalent wrapper.
  Raid session 41028 remains live on its older binary. No inferred motion,
  position integration or queued architecture changes; no commit or push.

- Audited `aim_vector.go` against existing Rust source: the full 6..30 checked
  direction decoder was already implemented and independently tested, despite
  the stale manifest note. Added the native +Z-sentinel wrapper, validation-only
  flat projection and inverse direction quantizer in `orientation_frame.rs`.
  Extended the pinned Go orientation harness without changing its random inputs;
  all 4,096 prior rows/fields remain unchanged. Every new float32 projection and
  encoded integer matches, including illegal widths/faces, zero vectors and
  dominant-axis choices. The generator already registers this harness/fixture.
  This inverse quantizer is not Film byte-for-byte re-encoding. Marked only the
  audited `aim_vector.go` entry ported; broader parity remains incomplete.
  Focused comparison, Clippy with warnings denied, WASM compilation, formatting,
  diff and generator syntax checks pass. Full Theater suite: 449 passed,
  41 ignored, no failures (58.13s).
  Logs: `/private/tmp/halo-direction-codec-{test,suite,clippy,wasm}.log`.
  Next concrete audit target: `components_cubemap.go` general-width magnitude and
  direction/velocity wrappers, and `offline_aim.go` companion query methods.
  Production `native_velocity` currently covers 19-bit direction/10-bit scale;
  do not treat that alone as evidence for the complete public codec family.
  Raid session 41028 remains live on its older binary. The architecture proposal
  stays deferred in NEXT_PHASE.md; no changes committed or pushed.

- Added the complete CTF catalog-geometry comparison on the enriched kill path.
  The pinned API map ID selects Corpo's flag spawns in each implementation's
  catalog. Catalog bytes match exactly (SHA recorded in the fixture). Native
  production projection retains neutral-label precedence. The new oracle has two
  selected spawns, four team births, two attributed flags and explicit home spans,
  while preserving all 20 kills and three captures. It retains every source chunk,
  native kill result, complete document, independent API facts and catalog hash.
  Positive geometry assertions and full field comparisons run in
  `complete_ctf_geometry_document`; the two earlier no-catalog cases remain intact.
  Complete geometry comparison passed (105.94s), including all kill-result and
  document fields plus independent API totals. Full suite: 449 passed, 41 ignored
  (60.86s); Clippy with warnings denied, formatting/diff and generator syntax pass.
  Regenerating the two earlier CTF fixtures left their documents, kill result and
  source bytes unchanged. Logs:
  `/private/tmp/halo-ctf-geometry-{rust,suite,clippy,regression}.log`.
  This adds native geometry/reconstruction evidence, not browser or independently
  annotated spatial ground truth. Positive VIP/bomb and remaining source/profile
  audit gates remain open; the three-layer architecture remains deferred.

- Extended the complete v41 CTF fixture through decoded-kill enrichment and the
  inherited calibrated profile. The initial integrated document comparison passed
  field-for-field (85.41s), including 16 published walk kills and four scan kills.
  The companion fixture now retains the entire native kill-source result. The
  stronger comparison additionally checks every kill/assist value, health/pass
  statistic, calibration summary, publication gate and public roster field.
  The shared old kill-result comparator now checks roster data as well; inherited
  profile fields remain covered separately by calibration/context oracles.
  Independent API totals are withheld from kill decoding and assert 20 kills;
  the independent three-capture check also remains active. New full comparison,
  suite and shared-native-harness regression logs: `/private/tmp/halo-ctf-kills-*.log`.
  The stronger comparison initially found only Health.Film differed: Go received
  the fixture label before decoding, but Rust's test set it afterward. The
  comparator now supplies DecodeOptions.match_id before invoking the constructor;
  no production parser change or output normalization was needed for that mismatch.
  Final enriched comparison passed (85.46s), including all 20 native kill records,
  roster/health/pass fields and the complete document. Independent API totals
  matched 20 kills and three captures. All six existing native enriched-document
  oracles stayed unchanged. Suite: 449 passed, 40 ignored (65.79s); after the
  label-input correction, Clippy and the zero-axis document (8.74s) also passed.
  Formatting, diff checks and generator syntax pass. Only earlier raid session
  41028 remains live; its binary predates these changes.
  The map-spawn geometry path for CTF, positive complete VIP/bomb cases, and the
  remaining source/profile audit still require evidence. No architecture refactor.

- Found a complete positive v41 CTF recording in the pinned reference's API wire
  integration fixtures: `c0a82e88`, Husky Raid:CTF on Corpo, eight chunks spanning
  119,156 ms. This is distinct from the truncated objective mini-corpus.
  Added a self-contained compressed fixture containing all raw chunks, source
  metadata/hashes, independent API facts and the native complete document.
  The initial complete Rust document comparison passed field-for-field (74.84s):
  three captures, three completed carry spans, four object lifetimes, two carrier
  marks, and every other document field. External player match totals are explicit
  identity/score inputs; withheld API team scores independently check captures.
  Source, evidence boundaries and invocation: `fixtures/ctf-document-v41.md`.
  Final complete CTF comparison including the independent assertion passed
  (95.98s). Full Theater suite: 449 passed, 39 ignored (61.18s). Clippy with
  warnings denied, formatting, diff checks and generator syntax passed. The
  existing zero-axis document passed (15.26s), and re-running the native shared
  harness left all six prior complete-document oracles unchanged. Results are
  recorded in `/private/tmp/halo-ctf-*.log`.
  This covers the complete CTF recording with the baseline document path; decoded
  kill enrichment and map flag-spawn geometry are not yet exercised for this film.
  Positive complete VIP/bomb comparisons and broader parser audits remain open.
  The three-layer architecture is still deferred. Raid session 41028 remains live.

- Closed the missing independent conflict fixture for kill-source player-index
  motifs (`internal/facts/killsource/index_motif.go`). The existing Rust behavior
  matches 512 pinned-Go cases without a production-code change. Eight groups of
  64 explicitly cover unanimous names, cross-chunk disagreements, collisions
  between two and three names, absent identities, short/holey chunk prefixes,
  and feed names overriding bootstrap names. Bootstrap and final chunks contain
  deliberately contradictory patterns and must be excluded. Later occurrences
  in the same chunk must not replace the first. A single disagreement/collision
  clears the whole name map, including otherwise valid identities; native reading,
  disagreement and absence counters remain equal. Fixture and native harness
  regeneration are registered. Focused test and full Theater suite passed:
  449 passed, 38 ignored (57.06s). Clippy with warnings denied, formatting, diff
  checks and generator syntax passed. Logs:
  `/private/tmp/halo-kill-motif-{test,suite,clippy}.log`.
  The earlier raid process remains live in session 41028. No architecture change.

- Added the missing native equipment-object state scanner (`equipment_state.rs`),
  loaded/supplied-band APIs, and map-aware Film retention with a separate error.
  Six raw fields preserve Seen/Present/Val and native sample order. Mask census,
  denominators, successful reads, closed gates and broken walks are exposed.
  The scanner calls the existing full-context component readers directly and
  stops at the last wanted field; rejected walks never publish partial samples.
  A 512-case independent pinned-Go oracle matches all fields and counters:
  547/601/601/1056/1056/1017 field readings, 192/196 closed activation/creator
  gates, and 347 rejected walks. Duplicate-name first-index lookup, missing
  target fields, unknown components, truncation and simulation policy are tested.
  The independent four-film captured oracle exercises explicit-map and automatic
  native profiles: idle controls and positive Bandit/Oddball state streams.
  Current validation logs: `/private/tmp/halo-equipment-state-{test,captured,suite,clippy,wasm}.log`.
  Validation: captured comparison passed all eight film/context combinations
  (210.81s), including 2,214 Bandit and 12,712 Oddball samples per mode and zero
  idle samples; every ordered field and statistic matched. The full Theater suite
  passed 447 tests, 38 ignored (66.33s), before the final error-report wrappers.
  After those wrappers, both focused tests passed; Clippy with warnings denied,
  WASM, formatting, diff checks and generator syntax passed. The integrated
  zero-axis complete document passed (8.71s). Successful scanning grammar was
  unchanged by the wrappers; captured comparison validates that scanner binary.
  Error-report logs use `/private/tmp/halo-equipment-state-report-*.log`.
  Earlier raid document validation remains live in session 41028; it predates
  the equipment and biped-context changes and is not evidence for those changes.
  This closes the missing scanner implementation, not the full all-data goal.
  Diagnostics-preserving entry points retain counters even alongside an error
  (including discovered slots when the registry is missing); Film retains both.


- Completed context propagation through held-weapon and counter-gated equipment
  recovery scans, with new complete-context APIs and unchanged legacy entry points.
  Film assembly supplies the strict scan's resolved context to both paths. The
  independent 512-case oracle includes all four policy/mask combinations, 245
  native held-weapon emissions and 232 accepted recoveries. It compares complete
  weapon publications, counters, recovery values and public recovery acceptance.
  The map setter's native simulation-enabling side effect is explicitly respected
  in the fixture; negative coverage is asserted. Reader replacement and captured
  positive recovery remain separate gaps. See READER_CONTEXT_PORT.md.
  Validation: 446 Theater tests passed, 37 ignored (58.85s); Clippy with warnings
  denied, WASM, formatting, diff checks and generator syntax passed. The captured
  zero-axis document remains field-for-field equal to native (13.69s). Logs:
  `/private/tmp/halo-biped-remaining-{test,suite,clippy,wasm,captured}.log`.
  In the earlier document-report captured run, complete six-film documents and
  decoded-kill documents have now passed; only the raid test is still live.


- Added complete-context entry points for biped channels, inventory, ability
  charges/states and unit-equipment scans. Map-aware Film assembly now passes its
  resolved context, including simulation-completion policy; position-only APIs
  retain their previous behavior. The native direct dispatcher retains typed
  movement callbacks in target components and applies no traversal width overrides.
  A new independent 512-context oracle compares 2,156 successful component visits,
  exact endpoints and 2,724 ordered callbacks, plus public scanner gate checks.
  The harness is registered in the oracle generator. See READER_CONTEXT_PORT.md
  for scope and remaining held-weapon/recovery and replacement-API work.
  Validation: 445 Theater tests passed, 37 ignored (70.12s); Clippy with warnings
  denied, WASM, formatting, diff checks and generator syntax passed. The captured
  zero-axis document remains field-for-field equal to native (14.14s), including
  report stage order. Logs: `/private/tmp/halo-biped-context-{test,suite,clippy,wasm,captured}.log`.
  The earlier complete-document run is still live and is not counted complete.


- Fixed inherited simulation-completion policy in navpoint and managed-property
  delta scans. Both now use the existing direct-context dispatcher, preserving
  early readings while refusing later values and chaining when the native policy
  stops the record. Neither adopts traversal calibration/stub overrides.
  Expanded each native scan oracle from 512 to 1,024 cases with the original rows
  unchanged. Both regressions failed at row 512 before the fix and pass afterward.
  These supplied-registry cases test context semantics, not the prevalence of this
  layout in captured v41 films. See READER_CONTEXT_PORT.md for the source mapping.
  Validation: 444 Theater tests passed, 37 ignored (71.89s); Clippy with warnings
  denied, WASM, formatting, diff checks and generator syntax passed. Both focused
  six-film captured scanner comparisons passed, comparing every reading and all
  counters. Logs: `/private/tmp/halo-direct-context-{suite,clippy,wasm,radial-captured,managed-captured}.log`.
  The earlier long complete-document run remains separate and pending; its process
  was confirmed live after these checks. No full-parser completion is claimed.
  A compilation attempt initially exhausted disk space; `cargo clean -p halo_api
  --profile dev` removed disposable package build artifacts and subsequent checks
  passed. Source, fixtures and the running captured-document process were preserved.


- Document assembly now exposes `build_film_replay_document_with_statborg_report`.
  Its separate serializable report retains ordered stage results without changing
  the native replay document schema: retained Film source, player identity
  completion, general objective extraction, optional score, flag attachment, then
  VIP attachment. Empty stage diagnostics mean the stage returned without warnings;
  absent stages were not reached. The report remains available on assembly errors.
  This scope excludes other grammar/scanner observations, carrier-mark source
  extraction and outer application logs. Source warnings are reused, not rescanned.
  General objectives are identified once and shared with action publication and
  bomb stats, matching native `readFilmStats`; `without_track` remains computed.
  Validation: 444 Theater tests passed, 37 ignored (66.10s), including report
  retention on errors, empty-timeline gates and JSON round trips. The updated
  captured zero-axis test verifies stage order and every native document field
  (14.42s). Clippy with warnings denied, WASM, formatting and diff checks passed.
  Follow-up: five of the six captured tests have passed, including complete
  six-film documents, decoded-kill documents, missing identity, zero-axis and
  unknown format. Only the raid remains in progress. This run predates the later
  biped-context API changes; those have their own validation above. Do not count
  the pending raid result as verified. Logs use
  `/private/tmp/halo-document-report-{suite,captured,stages,clippy,wasm}.log`.


- Native loaded-film bridge APIs are available on `FilmSource`: contiguous data
  chunk numbers, number-to-position translation, numbered bytes/packet lookup,
  and registry lookup. Absent metadata uses the native positional convention;
  supplied metadata retains first-match, negative-number and truncation semantics.
  Prefix enumeration stops at the first gap but direct lookup still reaches
  later loaded chunks. The 512-case native oracle matches 5,116 lookups and 4,384
  packet payloads/ranges, including 923 number/position differences. Public
  isolated packet walking shares the existing native walker. The single-file read
  and contiguous filesystem count wrappers have an independent native fixture.
  This completes the `film_chunks.go` and `film_packets.go` inventory entries;
  it does not begin the queued architectural change.
  Validation: both dedicated native-oracle tests passed; Clippy with warnings
  denied, WASM library check, formatting, diff checks and generator syntax passed.
  Logs: `/private/tmp/halo-chunk-bridge-{complete,clippy,wasm}.log`. Existing parser
  paths were unchanged; the last full-suite checkpoint remains 395 passed,
  32 ignored. All processes at this checkpoint are terminal.


- Ground-weapon ammo now traverses preceding components with the inherited
  calibration/stub widths and simulation-completion policy. It also preserves
  the native mask-index wrapping. Added complete-encoding creation entry points
  and routed map-aware Film assembly through them, retaining resolved MPP widths
  and the film's corruption decision. Two independent 512-case native oracles
  compare ammo and complete creation records/counters; profile propagation changes
  247 creation cases and all 403 ammo records match. The direct oracle initially
  caught an actual mask-wrapping mismatch (148/618 versus native 154/1339); a later
  test-context mismatch was corrected by using native default world precision
  instead of Bazaar precision. Both harnesses are registered in the generator.
  Validation: 395 Theater tests passed, 32 ignored (48.03s); Clippy with warnings
  denied, WASM library check, formatting, diff checks and generator syntax passed.
  Complete captured missing-identity replay parity passed separately (10.98s).
  Logs: `/private/tmp/halo-ground-profile-{focused,suite,clippy,wasm,captured}.log`.
  All processes at this checkpoint are terminal.


- Shared leaf-reader inventory audit: `bit_leaf_readers.go` and `varwidth.go`
  now have a direct native oracle, independent of component-name routing. All
  4,608 calls across 768 cases match: exact variable-width values and tails,
  category-one probe switching, both gate polarities, optional word references,
  absent-reference publications and padded cursor endpoints. No parser change was
  required. Category bases remain unapplied, as in the pinned native reader.
  `component_param4.go` was also traced to its sole runtime name-table consumer:
  vehicle orientation uses measured level 2; ordinary components use registry
  levels. These three stale pending inventory entries are now backed by source
  mappings and tests. The leaf oracle is registered in the generator.
  Focused test, Clippy with warnings denied, formatting, diff checks and generator
  syntax passed (`/private/tmp/halo-leaf-audit-clippy.log`). No production reader
  code changed, so the preceding 392-pass/32-ignored suite remains the full-suite
  checkpoint. All processes from the source/leaf checkpoints are terminal.


- `source.rs` ports native strict zlib decompression, tolerant inflation, memory
  loading, directory loading and positional metadata alignment. Header failures
  stay distinct from stream failures. Tolerant loading retains decompressed
  prefixes and falls back to raw input only for invalid headers or failures with
  zero output. The source owns each inflated chunk once and reuses the native
  packet walker, with packet positions separate from manifest numbers. Directory
  sources use numeric native sorting; first duplicate metadata wins and unknown
  filenames retain index -1. `into_chunks` moves bytes into existing Film APIs
  and explicitly rejects missing metadata or numbers outside their i32 contract.
  The 512-case independent oracle compares all inflated bytes, errors, per-chunk
  counts and 1,776 packet payloads, including multi-buffer outputs; a separate
  native directory oracle covers filenames and alignment. The local Bazaar
  mixed compressed/raw source yields identical complete Film JSON to clear input.
  Both harnesses are registered in `generate_oracles.py`.
  Validation: 392 Theater tests passed, 32 ignored; Clippy with warnings denied,
  WASM library compilation, formatting, diff checks and generator syntax passed.
  Logs: `/private/tmp/halo-source-{suite,clippy,wasm}.log`. The separate source
  filter also passed the captured Bazaar round trip and the complete captured
  kill-source comparison: all eight selected tests passed in 189.91s. Session
  14670 is terminal; log `/private/tmp/halo-source-focused.log`. Final extended oracle counts include
  112 partial-output recoveries and nine inflated outputs above 32 KiB.


- `components/probes.rs` exposes the native legacy `TraverseKeyframeBipedAt`
  composition separately from full-state keyframes. Only TI35 consumes defaults;
  the component gate does not suppress the following delta mask. Missing registry
  entries retain the consumed cursor while native trace end remains zero. The loop
  preserves native mask-index wrapping above 63, failed component attempts and
  corruption guards. A 512-case independent oracle matches masks, boundaries,
  desync indices, component attempts and observer publications: 91 padded reads,
  102 missing archetypes, 85 failed attempts and 30 wrapped-mask cases. Raw fields,
  references and diagnostics survive JSON round trips. The dedicated standalone
  component probe context and typed payload projection audit remain open.
  Harness: `halo_rust_keyframe_biped_probe_test.go.txt`, registered in the generator.
  Validation: 390 Theater tests passed, 31 ignored; Clippy with warnings denied,
  WASM library compilation, formatting, diff checks and generator syntax passed.
  Logs: `/private/tmp/halo-keyframe-biped-probe-{suite,clippy,wasm}.log`.


- Native public `positions.DecodeKeyframePositions` and spatial team splitting
  are ported in `keyframe_position_probe.rs`, retained by `Film` and JSON exports.
  These remain explicitly labeled pattern probes, not entity boundaries or decoded
  player/team identities. Native packet selection, zero-length/terminator handling,
  negative bit offsets, strict float filters, structural rejection, non-overlapping
  comb scanning and final-byte guard are preserved. The 1,024-case native oracle
  matches 3,149 accepted probes plus out-of-range float bits and team inference.
  Six captured films match all 346 native observations (Bazaar 2, Bandit 117,
  Oddball 227; Aquarius and both Cadet clips 0); positive Bazaar Film construction
  and JSON round trip pass. Harnesses and fixtures are registered in the generator.
  Validation: 267 Theater tests passed, 28 ignored; Clippy with warnings denied,
  WASM compilation, formatting and diff checks passed. Logs:
  `/private/tmp/halo-keyframe-position-test.log` and
  `/private/tmp/halo-keyframe-position-suite.log`.
  The subsequently identified `grammar/weaponscan/scanner.go` gap is now ported
  and independently validated below; it remains distinct from `grammar.FireEvent`.

- `navpoint_radial_scan` now reads observed ti=12 slots from the keyframe census,
  scans native sparse delta anchors, and walks full-state keyframes with native
  padding, component-failure rollback and strong/weak boundary diagnostics.
  The 512-case oracle passes, including expanded missing-clock, pre-base-time,
  default-state and corruption-control cases. Map-aware Film exports retain the scan, and the
  arming Film builder consumes it behind an explicit caller mode gate.
  Native six-film reference passed in 76.023s with 13,805 radial readings;
  The first Rust comparison exposed a packet-type error (delta is zero), now
  corrected. The focused captured comparison passes all 13,805 readings and
  every counter in 5.93s (`/private/tmp/halo-radial-only-rust.log`). Full assembly
  passed in 255.29s at `/private/tmp/halo-radial-films-rust-fixed.log`.
  This is not positive captured bomb/arming validation: the local six-film set
  contains other modes, and radial progress alone never identifies a bomb mode.

- `navpoint_radial` and `replay_bomb_armings` port radial segments, monotonic
  rises, shape predicates, first-full-quantum truncation, pair merging, pause
  correction and whole-film fuse validation. The 1,024-case native oracle compares
  56,392 readings, 22,976 segments, 476 published armings and 619 suppressed
  layers, including exact IEEE bits for the fuse coefficient of variation.
  The pure pipeline, raw radial scanner and Film wiring are verified; positive captured
  arming parity remains pending. Native end-order and binary-search behavior after
  first-full truncation are retained, including its unsorted-input limitations.

- `replay_bomb_stats` ports all five player statistics, absent-versus-zero source
  gates, event provenance, dated events and two-pass arming attribution. All 2,048
  native cases pass: 6,134 detonations; 9,231 armings, including 1,503 attributed
  by drop, 276 by active carry, 105 without an identity bridge and 698 ambiguous;
  1,416 carrier kills and 6,215 player rows. Raw carry totals are consumed rather
  than recalculated from publication. Full Film/document stats wiring remains open.

Latest broad validation after i26 equipment-list integration: 266 active Theater tests pass,
27 opt-in tests remain ignored; library/test Clippy, WASM lib compilation,
formatting and diff checks pass. The six integrated documents and full raid
comparison passed earlier (see the dated evidence below); they were not rerun
for these explicit frame APIs. Full v41 parity remains incomplete.

- `replay_held_object` and `replay_bomb_carries` pass 2,048 native cases for
  44,803 events, 30,055 periods and 4,626 published bomb carries, covering
  transition order, time-specific occupant resolution, direct-drop versus
  death closure, open-end sentinel, cumulative carry time, bomb-family filtering,
  frame projection and carrier-presence coverage. Native death input order is
  retained; a direct drop does not consult deaths, matching executable behavior.
  Film wiring retains raw and published timelines, the no-bridge event denominator,
  and an explicit caller mode gate. Positive captured bomb parity remains pending.

- `replay_skull` passes 1,024 native cases and 15,360 presence probes. It ports
  47,865 raw trains and 4,525 published carries, including measured half-tick
  frame windows and all rejection counters,
  named-lifetime unions, anonymous-life abstention and deduced-identity handling.
  Film wiring consumes the completed player identity bridge and an explicit mode
  gate. Native six-film reference passed in 70.136s; captured Oddball produces
  39 candidate trains, 38 closed carries and one carrier-absence rejection.
  Rust six-film comparison passed in 260.62s, including all 38 captured Oddball
  carries and every coverage field (`/private/tmp/halo-skull-films-rust.log`).

- `replay_vip` and `replay_match_clock` pass 1,024 native cases: 17,091 raw
  periods, 279 published periods and forward/inverse/slack clock extremes.
  Film wiring uses the death-time round bridge and an explicit caller mode gate.
  No positive captured VIP film has been validated. The full suite after VIP
  passes 186 tests (10 opt-in ignored), Clippy and WASM compilation.

- `carrier_marks` reads the four shifted 32-bit carrier-marker views through the
  existing shared record attribution reader, emits one mark per matching biped
  record, and retains every keyframe timestamp and both record denominators.
  The native 256-case oracle passes across 768 synthetic keyframes. Film retains
  missing-scan versus empty-scan state; the observation is not treated as a
  universal carrier identity or as evidence for other modes. The expanded oracle
  has 5,552 marks over 9,024 recovered records and 20 no-mark cases. The native
  six-film reference passed in 78.109s with zero marks over 106 keyframes; positive
  carrier attribution remains generated-case evidence. Rust comparison passed
  in 247.40s (`/private/tmp/halo-carrier-films-rust.log`).

- `replay_objective_objects` assembles free objective lives by slot/generation,
  next-key reuse and the existing native motion matcher. It preserves the last
  declared identity per key before chronological sorting, creation positions,
  increasing-time samples and total content ordering. The publication gate admits
  only declared nonempty `ball` labels, retains the native flag withholding,
  rejects out-of-axis births, keeps the final sample per frame, and reports all
  counters. Raw flag lives remain separately accessible. The 256-case native
  oracle passes across five clocks; Film wiring and the production objective
  catalogs are present. The fixture covers 8,229 raw free lives and 5,165
  published lives. The native six-film reference passed in 67.909s; Rust comparison
  passed in 271.08s: 78 Oddball lives, 1,995 points and 10 motionless lives.

- `replay_score_hold` and `replay_score` complete the pure score timeline: hold
  progress uses maximum player increments at each timestamp, retaining pre-origin
  accumulation and separate same-frame emissions. The combined builder preserves
  absent versus empty input, all round/truncation/point coverage, and the explicit
  round-zero fallback count. A Film builder uses the scanned statborg/death evidence
  and the published replay origin/interval. The expanded 1,024-case Go oracle passes
  all outputs (407 timelines, 238 team hold-series). The six-film native reference
  passed in 72.083s. Its Rust comparison passed in 272.84s, including
  all 3,866 published score points and every coverage/fallback field. Durable log:
  `/private/tmp/halo-score-films-rust.log`. Full replay parity
  remains incomplete, notably objective layers and final document orchestration.

- `replay_score_teams` publishes team curves, optional team IDs, mode support and
  guarded victory targets. It prioritizes exact distinct final scores, then exact
  distinct sums of identified players' kills; unresolved teams retain their curves.
  The independent 1,024-case oracle passes (160 final-score assignments, 42 kill-sum
  assignments, 822 unresolved cases). Hold ticks, score coverage and Film wiring
  remain unfinished. Player and team code is not yet a complete score document.

- `replay_score_players` publishes all four player counters through the native
  single-round totals join and multiple-round death/elimination/residue joins.
  It retains stable slot ordering for tied names, rejects empty series, and
  recomposes each identified player's cumulative score after slot reassignment.
  The independent 1,024-case Go oracle passes: 580 automatically resolved player
  outputs and 1,455 explicitly identified player outputs match every series.
  Full team scores, hold ticks, score coverage and Film document wiring remain open.

Latest checks after carrier-marker integration: 185 active
Theater tests pass (10 opt-in tests), Clippy with warnings denied, WASM compilation,
formatting and diff checks pass. The expanded raid comparison passed in 1500.81s.

- The hour-long raid comparison passed in 1517.30s: all checked player/combat/vehicle
  fields, 269 player tracks, 208,865 points, 86 vehicle tracks and 149 rides.
  That fixture predates the new equipment/pickup layers; the expanded native
  reference passed in 286.393s. Its Rust comparison passed in 1500.81s at
  `/private/tmp/halo-raid-expanded-rust.log`, including the newer equipment and
  pickup layers. This raid snapshot still excludes ground, score, free-objective
  and carrier-marker additions; those require a later expanded raid reference. Full replay parity remains open.
- Ground weapon publication passed the expanded 256-case native object fixture
  across five clock variants, covering ammo, pickup matching and census ends.
  `Film.ground_object_tracks` retains both archetypes' motion evidence; the Film
  builder now combines weapons and power-up pads, offsets pickup indices, dates
  pickups from written events, and publishes individual moving weapon lifetimes.
  The production catalog includes explicit objective exclusions. The native
  six-film reference passed in 74.897s; Rust comparison is running at
  `/private/tmp/halo-ground-films-rust.log`. The first Rust comparison passed Bazaar and Bandit, then found only an
  Oddball objective-rejection counter difference (78 vs 0). The native harness
  incorrectly supplied empty objective labels; Go deliberately ignores empty
  labels. The harness now reads the production labels; regeneration passed in
  66.077s. The corrected full Rust comparison passed in 268.12s: all six films,
  12 pads, 430 moving weapons, 92 ended pad occupations and 32 named pickups match.
  This includes all preceding player/combat/vehicle/equipment layers.
- `replay_score_series` projects score emissions onto a millisecond frame clock,
  retaining first plateau timestamps and final same-frame values, ordered round
  publication, cumulative round offsets and chronological rejection before clipping.
  All 1,024 native generated cases pass. This is a score publication foundation;
  full player/team score assembly and replay-document integration remain open.

- `replay_weapon_changes` publishes held-weapon transitions with native restated
  rejection precedence, exact family sentinels, scan ordering and coverage. Its
  independent 1,024-case Go fixture passes. `replay_pickups_film` connects the existing
  pickup projection to temporal identity, production weapon/equipment names, observed
  dropped placements and optional explicit map spawn points. The six-film
  reference regenerated in 72.415s; Rust matched all pickup and weapon-change
  fields in 272.25s (`/private/tmp/halo-pickups-films-rust.log`).
  All 178 active tests, Clippy and WASM checks passed before the ground additions.

- `replay_equipment_ends` and `replay_equipment_placements` complete the pure
  placement publication pipeline: reused-life census bounds, display ends distinct
  from last movement, frame clipping, catalog families, measured owners/headings,
  written origin provenance, and all counters/fallback counts before stable ordering.
  The expanded 1,024-case native fixture matches the full pipeline and every display
  bound. The Film-facing builder reuses existing placement/census/spawn/identity
  scans and the production-loaded equipment family catalog. The six-film native
  reference regenerated in 73.614s with 543 placements. Its Rust comparison
  passed in 258.10s with output at `/private/tmp/halo-placements-films-rust.log`.
  All 177 active tests, Clippy and WASM checks pass.

- `replay_equipment_origin` indexes written spawn events, death-closed lives and
  taken transitions. Its origin cascade preserves spawn-event precedence, the
  explicit spawned-piece manifest fallback, death/taken contradictions and unknown
  origins. Owner selection keeps each slot's nearest-time world sample within
  250ms, chooses within 3m with lower-slot distance ties, and independently selects
  same-slot heading within 200ms. All 32,768 native probes across 1,024 cases match
  complete owner, heading, origin, provenance and fallback results. Full placement
  publication and census-based display ends remain to be connected.

- `replay_equipment_episodes` publishes active camouflage and overshield periods.
  It preserves native window sorting, maximum-overlap attribution, replication-gap
  spans stopped by recorded deaths, inclusive zero-length periods, measured versus
  life-end closure, per-life coverage, and nonbinary camouflage counts. The Film
  adapter uses raw shield quanta and the player layer's death-closed track indices.
- `replay_equipment_kills` credits every active family using temporal occupants,
  inclusive frame bounds, and the native source/origin measurement gate. Its Film
  adapter accepts explicitly resolved kill evidence; kill-source decoding is not
  implied by this API. The expanded 1,024-case native fixture matches 8,552 episodes,
  1,737 kill credits and 841 assist credits. All 176 active Theater tests, Clippy and WASM pass. The six-film episode
  reference regenerated in 74.286s. The equipment/grapple comparison passed in 257.42s. The episode-expanded
  comparison passed in 262.92s, including all 22 activation periods and the earlier
  layers. Durable output: `/private/tmp/halo-episodes-films-rust.log`.

- `replay_grapple` now pairs fires and attachments, dequantizes the attachment
  anchor, selects the published life by native attachment/fire/nearest precedence,
  and measures arrival from the closest point within 2.5 seconds. It preserves
  stable input ties, unmatched-fire counters, and life counts by slot/start frame.
  The independent 1,024-case native fixture passes every published line and counter.
  Map-free Film assembly omits the layer; scanner body failures remain separately
  available, matching the native publication counter behavior. Clippy and WASM pass.
  The six-film native harness regenerated in 67.737s and includes 12 grapple
  pulls (2 Bandit, 10 Oddball), 41 equipment changes, and the prior inventory and
  ability layers. The expanded Rust comparison passed in 257.42s, including the preceding
  inventory, ability, player, combat and vehicle layers. Durable output is
  `/private/tmp/halo-equipment-grapple-films-rust.log`.

- `replay_translocations` publishes every retained type-117 jump with timestamp
  and slot, preserving unread endpoints as absent and rounding complete paired
  endpoints to centimeters. The 1,024-case native projection oracle passes,
  including zero steps, unsigned timestamp edges, origin/slot refusal precedence,
  half-centimeter rounding and partial-position rejection. Equipment transitions
  and translocations have been added to the six-film native assembly harness;
  the native reference regenerated in 67.826s with 41 equipment transitions and
  no translocator jumps. Positive translocator publication remains verified only
  by generated cases. Rust must rerun the expanded six-film comparison after the
  active inventory comparison finishes. Clippy and WASM checks pass.

- `replay_equipment_changes` publishes taken/spent transitions, preserves recovery
  provenance and residual gaps, and excludes spawned announcements before checking
  the replay origin. It preserves native input order and the native coverage counts
  measured before filtering unpublished slots. Its 1,024-case independent Go oracle
  passes. The Film-facing builder is present; real-film equipment publication has
  not yet been added to the assembly fixture.

- `replay_inventory` projects keyframe inventories onto the replay clock with
  stable frame/slot ordering, explicit unread-versus-zero ammo/grenade fields,
  float32 consumed gauges, and the native empty-reading marker. Death corroboration
  resolves the occupant at the reading frame and applies the calibrated inclusive
  eight-second window. Film assembly retains missing-versus-empty scan coverage.
  The independent 1,024-case Go fixture verifies 25,506 decoded readings, 15,885
  published readings and 416 death corroborations, including every value and
  coverage count. All 172 active Theater tests, Clippy and WASM checks pass.
  The six-film native reference was regenerated successfully (80.207s) and includes
  704 inventory readings. Its Rust comparison passed in 254.95s, including
  ability publication and all preceding player/combat/vehicle comparisons.
  Durable output: `/private/tmp/halo-inventory-films-rust.log`.
- The previous six-film ability and raid comparison sessions disappeared after
  interruption before final results were collected. Neither run establishes a pass.
  The subsequent raid comparison passed in 1517.30s with a durable log at
  `/private/tmp/halo-raid-rust.log`; its current oracle still predates abilities
  and inventory, which are checked only when present in that snapshot.

- `replay_abilities` now projects both i48 and keyframe ranks, retains stable
  frame/slot/source ordering without deduplication, rejects ranks >27 before the
  published-slot filter, classifies palettes with native first-marker precedence
  and unanimity/90-percent thresholds, and publishes used labels with the palette
  family mapping. The expanded 1,024-case oracle compares 26,537 raw readings,
  12,293 published readings, 1,260 noise rejections and 16,384 classification probes.
- `replay_abilities_film` connects rank, charge and impulse publication to Film
  and the player registry. `ability_catalog.json` is extracted by calling LevelUp's
  production title-label loader, preserving its two palettes, English/French labels,
  HUD URLs, family mappings and measured charge/impulse families. Callers may supply
  another explicit catalog; default builds use the pinned Halo catalog. The
  underlying scanner's `scanned` flag controls optional coverage publication.
  The six-film native fixture is being extended to compare this layer. The older
  raid fixture does not yet include ability publication and cannot verify it.


- Equipment-charge and impulse publication are ported in `replay_ability_charges`
  and `replay_ability_impulses`. Shared life/rank lookup preserves supplied life
  order, five-second boundary tolerance, latest prior rank with last-read tie
  precedence, and native signed timestamp behavior. Charges retain every reading
  including zero; impulses fold from the last retransmission at gaps <=1 second.
  Both retain native refusal precedence and distinguish an unavailable resolver
  from missing identity or an unmeasured family. Impulse coverage includes optional
  raw scanner denominators. The independent 1,024-case fixture compares 17,291
  charge readings (120 published), 13,521 folded impulse episodes (91 published),
  all refusal/scan counters and 32,768 rank queries. Zero-step early returns are
  preserved. These builders still require palette classification/title metadata
  and integration into the complete Film replay document.
  Validation: all 171 active Theater tests pass (10 opt-in tests ignored, 22.26s);
  Clippy with warnings denied, WASM compilation, formatting and diff checks pass.


- `build_film_replay_combat` now includes vehicle publication and applies the
  native second shot-attachment pass using original orphans, published player
  slots, the identity index and the shared frame clock. It retains updated vehicle
  and shot coverage plus the replacement verdict when recovery adds shots. The
  existing 1,024-case vehicle-shot oracle passes; the native six-film fixture now
  independently records combat after this pass instead of comparing it only to
  the first shot pass. The updated Rust six-film comparison passes (245.51s
  release), including all final combat shot and vehicle-coverage fields.
- Added `Film::try_from_chunks_with_map_bounds` for caller-established custom-map
  geometry without borrowing a catalog map's name. A synthetic map-aware test
  checks complete output equality after changing only the supplied map name.
  Raid provenance identifies Facility Aetheria, module `fo11_blank`, in saved
  `settings/map-variant.json`; all pinned catalog entries for that module share
  the same quantization bounds. The optional native raid harness uses that entry
  solely for geometry; Rust retains the actual map name. `generate_oracles.py
  --include-raid` regenerates this expensive comparison, and the ignored Rust
  `local_film_raid_assembly` reads its fixture (or `HALO_FILM_RAID_ORACLE`). The
  native raid oracle completed (288.30s): 86 published vehicles, 149 rides
  (99 written / 50 proximity), 31,443 published samples, 71,488 aim readings,
  12 death readings / 1 matched life, and 1,302 recovered vehicle shots. The
  Rust raid comparison is running; no raid parity claim is established yet.
  Current validation: 170 active tests pass (24.14s), vehicle-shot fixture passes,
  Clippy with warnings denied, WASM, formatting and diff checks pass. A newly
  added opt-in raid comparison increases the ignored test count to ten.


- Map-aware Film decoding now includes `native_vehicles` (census, creation records
  and stats, accepted/rejected position candidates, boarding/exit events, and
  occupant aim-only reads). Older/map-free exports omit it. Native optional-scan
  failures are retained as issues, and a missing vehicle band yields an explicit
  unscanned layer. `build_film_replay_vehicles` uses the player clock/registry and
  existing march deaths/occupancy; it does not repeat the march. JSON roundtrip,
  170 active tests, Clippy and WASM checks pass with the integration.
  A census probe of all 32 local films found vehicle entities only in both High
  Ground appearance clips (6 lives each) and the raid (87). The expanded six-film
  native oracle includes both High Ground clips: 12 published tracks, 12 creation
  reads, 716 accepted position reads / 172 published samples, and 2 aim-only reads.
  These clips have no rides or written vehicle deaths; they cannot establish
  real-film ride or vehicle-death parity. The Rust six-film comparison passes
  every published vehicle field, all coverage/attribution/fallback counts, scan
  counts, and existing player/combat output (245.02s release). The raid is the
  remaining local candidate for real-film ride and vehicle-death validation.


- Complete pure vehicle-layer assembly now lives in `replay_vehicle_publication`:
  census windows and written deaths, earliest creations, drawable gating,
  combined rides, per-life clipping, relay merging before coverage, all published
  coverage fields, and named fallback counts. The 512-case native publication
  comparison covers 1,960 census lives, 1,668 published tracks, 79 relay merges,
  1,573 published rides, 1,055 death readings / 747 matches, and 213 lives with no
  publishable position. It compares each track and all coverage/attribution fields.
  `replay_vehicle_coverage` also matches the 1,024-case single-track fixture.
- Respawn cycles are ported in `replay_vehicle_cycles` and integrated after relay
  merging. Clusters choose the nearest current centroid within two meters, with
  last-cluster tie precedence; birth order is stable, missing dated destructive
  ends are counted, dominant-family ties choose alphabetically, and the shared
  ground-pad stability rule determines publication. A separate 1,024-case native
  fixture covers 2,148 birth clusters, 793 established cycles, 9,091 measured gaps
  and 2,451 missing intervals. Automatic Film vehicle scanning, actual-film layer
  comparison, and combat vehicle-shot integration are still required.
  Validation: the integrated cycle/publication comparison and all 170 active
  Theater tests pass (9 opt-in tests ignored, 22.53s); Clippy with warnings denied,
  WASM compilation, formatting and diff checks pass.


- Combined vehicle ride attribution is ported in `replay_vehicle_rides` and
  `replay_vehicle_episodes`: occupant event state machines, exits before boards
  at equal time, unpaired exits, terminal reappearance, exact and nearest event
  life selection, two-anchor geometry, position-gap fallback, written-occupancy
  precedence, inclusive gap suppression, seat enrichment and stable ordering.
  The expanded native fixture compares 512 combined cases / 1,120 rides and all
  attribution counters: 1,994 event episodes (59 direct event, 118 nearest event,
  264 geometry, 1,553 unresolved), 232 published gap fallbacks, 67 contradictory
  fallbacks rejected, and 516 seat assignments. Independent checks compare 6,306
  state-machine episodes, 10,240 overlap probes, and 20,480 seat probes (1,644
  assignments). Inputs include zero clock steps, empty and recycled lives, missing
  drawable lives, invalid event occupants, unsorted raw observations and tied times.
  Drawable-life selection also matches the existing 1,024-case track oracle.
  These pure builders are not yet integrated into complete vehicle publication;
  actual-film vehicle track parity and combat shot recovery remain outstanding.
  Validation: all 170 active Theater tests pass (9 opt-in tests ignored, 20.52s),
  Clippy with warnings denied, WASM compilation, formatting and diff checks pass.


- Map-aware decoding now populates `Film.native_march_facts`, retaining calibrated
  deaths, occupancy and all coverage counters in portable Film exports. Older and
  map-free exports default this optional field to None. The native march starts
  from its own invariant 13-bit/base-zero, MPP 9/5, no-corruption/no-strict-generation
  profile; it does not inherit the caller's separate sequential replication ID
  layout. The map-aware JSON roundtrip includes the new field. The four-film march
  test now invokes the public Film constructor and passes all four films, every
  fact and coverage field (228.60s release).
- Explicit vehicle ride attribution is now ported in `replay_vehicle_rides_film`:
  inclusive life windows, first matching life, drawable gating, next-transition /
  returning-position / life-end closure precedence, seat and time-dependent XUID,
  stable ride order, and contradiction checks against proximity fallbacks. Native
  attribution intentionally does not compare the transition's parent generation.
  Aim-only observations are grouped stably and sampled once per frame inside the
  episode, preserving native heading/pitch rounding and malformed-order searches.
  The 512-case native oracle checks 854 rides, all tally counters, 10,240
  contradiction probes and 4,096 aim-window probes (1,385 published aim samples).
  Proximity/event fallback attribution and seat enrichment are now combined in
  the pure builder described above; multi-life vehicle and replay integration remain.

- `scan_film_march_facts` now assembles chronological keyframes/deltas, seeds
  first-seen declarations, calibrates six ID widths, and harvests death/occupancy
  facts with all coverage counts. A 128-case native timeline/calibration fixture
  passes, including stable equal-time replacement, flat-profile fallback, and
  packet/event budgets. The four-film native oracle uses the production replay
  builder's separate world-object map-precision installation and contains 309
  deaths and 150 occupancy readings. An initial harness omitted that installation;
  its default-width output was not the intended production comparison.
  The comparison then exposed the missing one-bit `player-allowed-to-quit-component`
  reader (135 falsely incomplete ti5 records on Bandit); that reader is now added.
  Full-film comparison now passes every field, calibration result and coverage
  denominator across all four films: 996,240 records, 911,502 clean records,
  117,746 delta packets, 309 deaths and 150 occupancy readings (191.42s release).
  The subsequent map-aware Film integration is described above; combined vehicle
  ride assembly remains required. Validation: 169 active Theater tests passed (9 opt-in tests ignored),
  the timeline/calibration and four-film march comparisons passed, Clippy with
  warnings denied, WASM compilation, formatting and diff checks passed.

- Native march packet traversal is now separate from production view admission:
  `walk_march_records` follows eight entity views, permits per-packet NEW/DELETE
  bindings, retains records through the first failure, and leaves the caller's
  world untouched. `locate_march_event_records` implements the strict 35-bit
  slot-123 signature and free-width fallback, including the native first-strict-
  candidate generation rejection rule and configurable generation checking.
  The native oracle has 1,024 packet cases, 6,172 records and 881 localized
  boundaries across six ID widths, strict/non-strict generations, optional record
  prefixes, create/delete chains, truncation and noise. Missing-registry NEW
  records now consume the native default/gate/mask before rejection; missing-slot
  non-strict deltas consume their baseline selector before failure.
  Film-wide chronological keyframe binding and six-width calibration are now
  supplied by `scan_film_march_facts` above. Validation: 168 active Theater
  tests passed (8 opt-in tests ignored); focused native boundary comparison
  passed again after the missing-slot baseline fix. Clippy with warnings denied,
  WASM compilation, formatting and diff checks passed.

- `MarchRecordFacts` now harvests all native dead-state fields and biped parent
  transitions from existing sequential `EntityRecord` component captures. Parent
  resolution uses the native 0x200 handle base and generation bits; absent/free
  parent and absent-seat readings remain explicit. Death capture retains native
  enum/reference/source-handle sentinels, admits a valid dead state before a later
  component failure, and exposes vehicle-life evidence for ti40. Coverage counts
  every record, clean records, masks declaring dead-state and declared-but-damaged
  records by archetype. Registry component names determine the death index.
  Stable deduplication preserves the first best-quality death per time/slot/life
  and all distinct parent/seat transitions. A 1,024-case native oracle traverses
  24,576 records and compares all 2,524 deduplicated deaths, 1,680 occupancy readings
  and coverage maps, including varying component order, all five tested parent
  precision levels, light/heavy death forms, unsupported components and padded
  tails. This is record harvesting, not yet automatic film-wide march integration.
  The subsequent `scan_film_march_facts` port now supplies first-seen keyframe
  bootstrap, chronological replacement, strict/fallback slot-123 localization,
  eight-view decoding with binding rollback and six-width calibration. Automatic
  Film/replay wiring remains pending. Existing production-frame policy is a
  different traversal and must not be substituted for the native march. Validation: 167
  Theater tests passed (8 opt-in tests ignored), Clippy with warnings denied,
  WASM compilation, formatting and diff checks passed.

- Native boarding/exit event scanning now exposes all occupant, vehicle, generation,
  seat and packet-attribution fields through `decode_vehicle_event` and
  `scan_vehicle_events_for_band`. Exact band membership, wrapping base addition,
  absent references and missing-seat publication match Go. Type 53 remains
  unsupported by the native scanner. Unlike Go's out-of-range panic, Rust returns
  an explicit error for truncated reference fields. A 4,096-case native oracle
  verifies 2,648 events (936 without seats) and 436 native panic cases; packet
  attribution is tested across non-delta packets and the End boundary.
- `scan_biped_aim_records` and `scan_biped_aim_for_band` now expose aim-only biped
  observations without requiring position replication. The scanner reuses shared
  companion component traversal, preserving generation-one/ascending-mask gates,
  exact sparse band membership, and the native primary-aim overlap boundary.
  A 2,048-payload native oracle verifies all 4,005 observations, with mixed
  components, unsupported masks, noise and truncation. These are synthetic
  fixtures; actual-film aim-only parity and automatic band/document wiring remain
  pending. Ride aim publication, march deaths/occupancy and ride attribution are
  still required for complete vehicle assembly. Validation: 166 Theater tests
  passed (8 opt-in tests ignored), Clippy with warnings denied, WASM compilation,
  formatting and diff checks passed. The four-film player/combat native comparison
  also passed after the shared companion-reader refactor (29.26s release).

- `build_replay_vehicle_track` now publishes one assembled native life, combining
  earliest spawn, chassis identity/family, clamped life bounds, movement and supplied
  rides. It refuses lives with neither spawn nor samples, preserves explicit death
  time, keeps later contradictory movement, and clips rides/aim to display bounds.
  All 29 pinned chassis mappings are ported; unknown families retain ride eligibility
  while the five explicitly non-pilotable families refuse rides. Chassis formatting
  is native lowercase eight-digit hex with no prefix.
- `build_replay_vehicle_samples` preserves inclusive life windows, first sample per
  frame, heading updates even on skipped samples, and last-seen time. It prefers
  config-mode chassis orientation, then valid velocity at >=5m/s, then holds the
  previous heading. Position grouping uses stable timestamps and excludes no-world
  readings; spawn grouping keeps the first earliest re-announcement.
  The combined native oracle covers 1,024 cases: 24,776 position inputs, 6,175 grouped
  positions, 2,259 samples, 994 tracks and 1,337 clipped rides; 12,288 creation inputs
  select 6,144 earliest spawns. It also checks every chassis mapping and unknown
  behavior. Complete multi-life vehicle assembly still needs ride attribution,
  coverage/cycles and Film wiring; this stage consumes already-attributed rides.
  Validation: 163 Theater tests passed (8 opt-in tests ignored), Clippy with
  warnings denied, WASM compilation, formatting and diff checks passed.

- `decode_vehicle_creations` and `scan_vehicle_creations_for_band` now reuse the
  shared equipment/ground-weapon creation walk with ti=40 defaults and explicit
  map precision. Vehicle i0 uses its absolute four-bit spine, expected region and
  non-saturated coordinates. The native advancement excludes the biped tail;
  source chunk, packet, timestamp, generation, mask, chassis MPP fields and failure
  counters are retained. Ground-weapon and vehicle scans share the chunk loop.
  2,048 native cases across the map catalog compare 4,888 anchors and 3,803 accepted
  creations with every field and counter, including MPP width variation, optional
  media frames, dense/sparse masks, wrong regions, saturation and truncation.
  The new creation oracle and both existing creation suites pass. Full vehicle
  layer wiring and publication remain pending. Validation: 162 Theater tests
  passed (8 opt-in tests ignored), Clippy with warnings denied, WASM compilation,
  formatting and diff checks passed.

- `scan_positions_for_band` adds the native world-object position entry point with
  exact sparse-slot admission before cursor advancement, shared i0 anchoring and
  coordinate reads, dynamic companion capture, and production filtering.
  `scan_vehicle_positions` selects all generations and v41 dynamic orientation.
  Native packet-prefix traversal is used for this entry point; map/axis precision
  remains explicit. Existing biped scanning uses the same anchor implementation.
  A retained native generator compares 512 packet-stream cases across all 27 map
  catalog entries: 7,032 candidate records and 4,022 accepted positions. Every
  source timestamp, record/vector boundary, generation, mask, quantum, coordinate,
  orientation/vitality/aim companion and accepted-record index matches.
  Actual-film vehicle-layer integration remains pending; these synthetic cases do
  not establish complete vehicle parity. Next: vehicle samples/heading publication,
  march facts, rides and combined assembly (creation, event and aim scans added below).
  Validation: 161 Theater tests passed (8 opt-in tests ignored), Clippy with
  warnings denied, WASM compilation, formatting and diff checks passed.

- `orientation_frame` ports all 25 native cubemap precision entries (6..30),
  midpoint roll decoding and the asymmetric up/roll reconstruction of chassis
  forward. Fused float32 operation order was checked against the compiled pinned
  Go reference, including its different initial/final norm rounding order.
  `NativeChassisOrientation` retains direction, roll, mode, delta and default-up
  provenance; its typed projection consumes the shared dynamic component grammar.
  Film-derived replay heading remains gated to config mode 1.
- `scan_biped_companions_with_grammar` adds registry-level dynamic i2 and dynamic
  i3 traversal without duplicating their grammar. Captured vehicle orientation now
  includes roll/mode/default-up, and following vitality/aim fields remain reachable.
  The ordinary biped entry point retains its previous grammar; absent chassis data
  is omitted from serialization. A retained native oracle covers 4,096 cases of
  direction/roll/forward values, dynamic component boundaries and fields, plus
  mixed-mask companion capture with truncated inputs and body/shield/aim values.
  Every comparison passes exactly. Full vehicle band scanning and replay track
  assembly remain pending. Validation: 160 Theater tests passed (8 opt-in tests
  ignored), Clippy with warnings denied, WASM, formatting and diff checks passed.

- `build_replay_vehicle_lives` ports vehicle census windows and written-death
  attribution, including unlimited final windows, reused-slot boundaries, earliest
  death selection, tail-desync provenance, and all native unmatched-death causes
  and gap diagnostics. Life `bounds` and `end` retain later contradictory presence
  instead of truncating it, and preserve an explicit destruction at frame zero.
  1,024 native cases match 12,661 lives, 30,096 death observations (2,064 matched)
  and 50,644 bounds/end probes.
- `merge_replay_vehicle_relays` joins the first eligible pair repeatedly, preserving
  the original life identity and replacing end cause/time together. It refuses
  written destruction, requires matching chassis and distance <=0.5m, appends only
  forward samples and stably merges/clamps rides and aim. Native aim endpoint
  fast-path behavior is retained even for malformed sample order. 1,024 cases
  match 14,796 input tracks, 9,798 outputs and 4,998 merges.
  Validation: 159 Theater tests passed (8 opt-in tests ignored), Clippy with
  warnings denied, WASM compilation, formatting and diff checks passed.
- Next vehicle work: `vehicle_tracks.go` sample/track assembly requires the native
  chassis orientation path (`vehicle_heading.go`, `orientation_frame.go`), not just
  velocity. The native vector reconstruction and dynamic companion path are now ported. Native `vehicleHeadingOf` prefers roll-derived forward on config mode
  1, then falls back to velocity >=5m/s. Wire the dynamic companion path into
  vehicle band scanning before claiming vehicle movement parity. March deaths/occupancy, rides and full Film wiring remain pending; standalone
  vehicle event and aim-only scans have since been added.

- `attach_replay_vehicle_shots` ports the native second attachment pass over
  published vehicle rides: inclusive occupancy, stable seat/track/slot ordering,
  refusal across distinct track indices, clamped frames, spawn fallback and exact
  f32 fused interpolation. Published-slot gating moves the original rejection
  counters to attached/unpublished, while preserving orphan evidence. It returns
  the native replacement verdict only if a shot was added. Vehicle shot coverage
  remains optional and counts placed shots even if the player is unpublished.
  `ReplayVehicleTrack`, spawn/sample/ride/aim and every vehicle coverage field now
  preserve the native JSON contract, including explicit optional zero values.
  A retained native generator compares 1,024 cases with 30,096 orphans, 3,493
  recovered shots and 70,541 interpolation probes, including malformed sample
  order. All cases pass. This pass is not yet wired into Film combat: vehicle
  scanning, life/ride assembly and the rest of the replay document remain pending.
  Validation: 157 Theater tests passed (8 opt-in tests ignored), Clippy with
  warnings denied, WASM compilation, formatting and diff checks passed.

- `build_replay_grenades` ports projectile-first grenade location, the native
  +/-200ms candidate window, known-author nearest birth within 4m, no-author
  single-candidate refusal, biped fallback and raw-to-published projectile links.
  Births are not consumed; the known slot is retained even without world position.
  Unknown grenade ranks and unresolved locations contribute to native no-slot
  coverage. Projectile-sourced throws survive missing player-track publication.
  A shared `PositionIndex` now supplies the native unstable sorting/120ms attachment
  gate to both shots and grenades; the existing shot oracle passes after extraction.
  1,024 native cases cover 24,776 throws, 4,483 located, 3,858 published (2,643 from
  projectiles, 1,215 from bipeds) and 1,464 published projectile links.

- `build_replay_grenade_reads` and `publish_replay_grenade_reads` port the separate
  carried-grenade axis. Keyframe/delta readings remain independent, selection-only
  deltas do not erase counters, and equal-time/slot records order by source with
  stable ties. Selection zero remains explicit. Coverage is absent when no reads
  were built, retained when all were filtered, and carries the ammunition-refusal
  diagnostic. 1,024 native cases contain 19,776 keyframes, 24,776 deltas, 26,271
  built readings, 17,520 published readings and 12 absent-coverage cases.

- `build_film_replay_combat` composes shot, loadout, projectile, grenade-throw and
  carried-grenade publication from a Film and the automatic player-layer result.
  Missing projectile/grenade scan context remains explicit through optional fields.
  The native corpus harness now compares these stages after the actual player
  assembly, using the same independent map-only inputs. Vehicle-shot recovery and
  the rest of the document still remain pending.
  The full 156-test Theater suite passed (8 opt-in tests ignored), as did Clippy
  with warnings denied, WASM compilation, formatting and diff checks.
  The expanded four-film integration passed in 29.78s (release, excluding
  compilation), comparing all prior player/shot fields plus 496 loadouts,
  482 projectile trajectories and their raw-index links, 511 grenade throws and
  1,249 carried-grenade readings, including every associated coverage field.

- `build_replay_shots` ports native shot attachment using undownsampled positions,
  unstable per-slot sorting, nearest-position ties, the 120ms gate and exact
  ambiguity/no-slot/out-of-window accounting. It retains recoverable original
  events and their reasons for the future vehicle-shot pass, including the native
  early-empty-input exception that produces no orphans. Shot formatting preserves
  aim omission/360-degree handling and fixed-width 64-bit hexadecimal weapon IDs.
  Publication filtering tests slot membership rather than track time bounds.
  `build_film_replay_shots` connects this stage to the automatic player registry.
  The 1,024 native cases contain 30,096 events, 5,882 initial attachments, 3,872
  published shots, 17,925 recoverable orphans, 2,800 ambiguous refusals and 2,010
  unpublished-track refusals. Coverage balances before and after filtering.
  The expanded four-film player integration passed in 30.83s (release, excluding
  compilation), comparing 4,304 published shots from 4,395 available events and
  91 recoverable no-slot orphans, alongside all previous player-layer assertions.
  Validation: the 153-test Theater suite and the subsequently added projectile
  native test passed; Clippy with warnings denied, WASM compilation, formatting
  and diff checks passed.

- `build_replay_loadouts` projects carried-weapon keyframe snapshots onto the
  replay grid, folds aliases by canonical name while preserving the first family,
  ignores unknown families and pre-origin snapshots, and stably orders time/slot
  ties. `retain_replay_loadouts` applies the native published-slot gate. Its 1,024
  native cases cover 34,756 input snapshots, 26,161 built and 17,649 published.

- `build_replay_projectiles` publishes first-per-frame planar trajectories,
  permanently cuts jumps over 10m after coordinate rounding, counts even cuts too
  early to publish, and withholds at-rest claims on cut flights. It preserves the
  native minimum raw/grid point requirements and total publication ordering, plus
  raw-to-published indices needed by grenade links. Its 1,024 native cases cover
  19,776 raw trajectories, 10,177 publications, 65,763 points and 4,434 cuts.
  Grenade attachment is now composed with these publication stages. Vehicle-shot
  recovery and the remaining document layers are still pending.


- `build_film_replay_players` now assembles the player layer directly from a
  map-calibrated Film and its original chunks. It derives accepted position/
  companion samples, sorted replay time, creation/fire/bot evidence, bootstrap
  seats, replication indices, statborg identity (deaths, lines, elimination,
  round residue), the identity registry, scope periods and the combined player
  publication. It retains registry/evidence provenance and checked match-clock
  origin. Optional roster, participants, scoreboard checks and successions stay
  explicit inputs. This is not yet complete replay-document assembly. It requires
  decoded biped and team context; empty accepted positions produce no player layer.
  The independent corpus harness runs native `scanFilmInputs`, then `ouvrir`,
  `poserLesPistes` and `poserLesEquipesEtLeRoster`, with native statborg facts.
  The four-film integration comparison passed in 29.87s (release, excluding
  compilation): 385 tracks, 99,939 points, and the empty-position case match,
  along with roster, identity section, bounds, teams, seats and replay timing.
  Validation: 151 Theater tests passed (8 opt-in tests ignored), library/test
  Clippy, WASM compilation, formatting and diff checks passed.

- `prepare_replay_timeline` ports stable chronological order, default 100ms grid,
  origin, frame count and duration. `scan_replay_clock_origin` reads the first
  readable packet in chunk one. `resolve_replay_origin_ms` preserves native
  5-match/1,000ms contradiction refusal and signed/unsigned conversion order.
  1,024 native cases cover 996 timelines and 597 accepted origins.
  `scan_replay_identity_evidence` wires deaths, supplemented sorted nonzero roster,
  replication-index scanning, collision refusal and film clock with named nonfatal
  errors. In particular, no deaths means no replication-index scan, matching Go.
  Its 1,024 native cases include 2,226 deaths, 1,698 published index links,
  92 collision refusals, 282 death-feed failures, 26 index-scan failures and
  174 clock failures.

- `ReplayScopeLookup` reconstructs scope periods using exact explicit exits, new
  entries, raw replication life ends and the native 3.5s hold. Explicit exits
  bypass life/hold caps, and lookup keeps native inclusive ends and binary-search
  traversal even for nonmonotonic synthetic intervals. The 1,024 native cases
  contain 24,776 zoom events and 614,400 lookups, including unsorted events,
  reused lives, equal timestamps, zero hold and unsigned overflow.


- `parse_highlight_events` ports the native v41 highlight search, retaining scan
  order, all seven event fields, exclusive XUID bounds, bit-aligned marker search,
  fallback past invalid candidate blocks, UTF-16 replacement behavior and exact
  medal precedence. Zlib header failures fall back to plaintext; stream corruption
  and truncation after a valid header are errors. Empty-dictionary headers and
  trailing compressed input match Go. `scan_film_highlights` selects the final
  contiguous readable manifest chunk; `scan_film_deaths` and the stream's `deaths`
  method filter and natively sort deaths, explicitly refusing an empty death feed.
  `Film.native_highlights` exposes this data without replacing existing summaries;
  damaged streams add a diagnostic limitation. Native fixtures cover 9,323 inputs
  (314 decoded events), and all 32 recorded films match 3,667 highlights and 1,395
  deaths. The retained native generators reproduce both fixture sets.
  Validation: 148 Theater tests passed (7 opt-in tests ignored), Clippy with warnings
  denied, WASM compilation, formatting and diff checks passed. All 32 existing
  film exports passed regression with native highlight/death assertions in 67.05s
  (release, excluding compilation), including the hour-long raid.
  Highlights now supply the automatic player-layer identity inputs. Complete
  replay-document assembly remains pending.
  Wiring references: `replay/film_scan.go::balayerPont` derives replication
  indices only when deaths exist, using `rosterOf(deaths, extraRoster)` followed by
  `injectiveOrEmpty`; `replay/build.go::ouvrir` stably sorts positions and derives
  origin/step/frame count. `replay/origin.go` reads the first packet of chunk one
  and refuses a published origin when at least five death matches contradict it
  by more than 1,000ms. These stages are now ported.


- `build_replay_players` now composes identity-section publication, life-based
  tracks, robust bounds, direct/bot/succession/final naming, registry inference
  propagation, team assignment, roster construction, seat assignment and all
  related coverage from one resolved registry. The result retains death-closed
  track indices, final bridge health, name-repair provenance, overlap fallback
  counts and flag-carrier team mappings. A zero-step clock returns no publication.
  The native combined harness builds the registry from creation/death/scoreboard
  evidence before running the same stage sequence. Its 1,024 cases compare the
  registry plus 768 player publications (256 zero-step cases), including 12,764
  tracks, 19,270 points, 4,960 roster entries and 6,490 assigned track teams.
  Validation: 146 Theater tests passed (6 opt-in tests ignored), Clippy with warnings
  denied, WASM compilation, formatting and diff checks passed.
  The automatic player-layer entry point is now implemented; score/combat/objective
  layers and complete replay-document assembly remain pending.

- `assign_replay_seats` ports native written-index seat assignment, recorded seat
  reuse, original/arrival classification from the readable bootstrap player table,
  and counted ordinal replacement matching within recorded teams. It preserves
  the native sorted candidate consumption and does not rematch recorded index
  reuse. `replay_presence_envelopes` joins humans by XUID and bots by name; peak
  occupancy follows the native first-to-last presence envelope rather than counting
  separate lives. Seat provenance and every coverage counter are retained.
  The 1,024 native cases compare 19,776 roster entries, 1,269 inferred seat matches,
  153 written-reuse seats, 7,945 arrivals, 11,347 closed presence envelopes and
  4,748 entries without presence, including absent/refused/interleaved film tables.
  Validation: 145 Theater tests passed (6 opt-in tests ignored), Clippy with warnings
  denied, WASM compilation, formatting and diff checks passed. Combined player-layer
  and full replay document assembly remain pending.

- `ReplayTeamPublication` projects decoded teams through effective registry XUID
  indices and the guarded slot bridge, preferring XUID readings and withholding
  ambiguous-slot assignments. Existing track teams survive absent readings. Roster
  publication keeps humans, refuses bots on human-held indices, deduplicates bot
  names by first eligible declaration, retains declared bot IDs and preserves
  optional team/no-team distinction. External match teams only produce comparison
  counts; they never assign teams. Flag-carrier tables exclude explicit no-team
  entries. Film/death gamertag selection is shared with identity publication.
  The 1,024 native cases compare 24,576 tracks, 11,266 assigned teams, 3,464
  ambiguous-slot refusals, 5,758 roster entries, 351 explicit no-team entries,
  and 218 agreements/1,849 contradictions/1,049 missing external team readings.
  Validation: 144 Theater tests passed (6 opt-in tests ignored), Clippy with warnings
  denied, WASM compilation, formatting and diff checks passed. Seat assignment
  and full replay document integration remain pending.

- `Film.player_teams` now exposes native team designators and refusal diagnostics
  from managed-player keyframes. `read_player_team_record` checks the first named
  component against an independently replayed default-state boundary; complete
  team fields survive later component/trailer failures. Collection follows native
  anchor recovery, validates player indices and raw values, counts entity/index
  divergences and withholds every conflicting index. A recorded -1 (no team) stays
  distinct from missing data. Bootstrap identity supplies the corruption-check
  layout; absent identity leaves the field unavailable. The 1,024 synthetic cases
  compare 7,554 records, 5,438 accepted readings, 1,253 unreached records, 456 bad
  indices, 407 bad values, 196 entity divergences and 204 index divergences. Their
  2,221 published indices include 232 explicit no-team readings. A 32-film native
  oracle captures 4,569 accepted records and 68 published indices (3 no-team).
  All 32 Film outputs match that team oracle and preserve existing observations
  (66.61s release regression, including the hour-long raid). Validation: 143 Theater
  tests passed (6 opt-in tests ignored), Clippy with warnings denied, WASM
  compilation, formatting and diff checks passed. Replay team/roster publication
  and full document assembly remain pending.

- `replay_bounds` ports the viewing bounds from published track points: raw bounds
  below 200 samples, sorted per-axis 1st/99th percentile guards, a 0.5m central
  spread floor, twelve-spread rejection margins, rejection of a point if any axis
  fails and fallback to raw bounds if all points are rejected. Empty data preserves
  the native inverted X sentinel. `ReplayTrackPublication.bounds()` makes this
  available directly from the published tracks. The 1,024 native cases contain
  729,967 points and 913 rejections, including exact threshold sample counts,
  flat/constant clouds, negative coordinates and outliers on every axis.
  Validation: 142 Theater tests passed (6 opt-in tests ignored), Clippy with warnings
  denied, WASM compilation, formatting and diff checks passed. Team scanning and
  publication, and complete replay document integration remain pending.

- `publish_replay_tracks` ports native life-based frame sampling: first observed
  world position per slot/frame, registry boundary transitions, counted five-second
  replication gaps within lives, chronological stable life ordering and first-slot
  appearance output order. Absent life evidence uses the existing replication-gap
  split and counts each fallback cut. `ReplayPoint` publishes rounded coordinates,
  same-record aim and clamped vitality, preserving absent versus zero measurements,
  heading-zero avoidance and external timestamped zoom state. `ReplayTrack` retains
  native serialization fields and default team -1. Publication can apply the ordered
  identity pipeline while preserving all sampled points and coverage.
  The 1,024 native cases compare 12,694 published tracks/42,302 points, 12,572
  refused lives/16,268 points, 21,139 gap readings (331,472,500 ms) and 15,251
  fallback cuts. Validation: 141 Theater tests passed (6 opt-in tests ignored),
  Clippy with warnings denied, WASM compilation, formatting and diff checks passed.
  Replay bounds, teams, and full document integration remain pending.

- `attribute_identity_successions` ports bot replacement chains in supplied order,
  the required matched death clock, arrival/respawn windows, immediate claims and
  fire-only resolution of competing candidates. Shots covered by two tracks cast
  no vote; votes for different tracks refuse the claim. Native half-open unsigned
  containment and signed wrapping birth-window arithmetic are retained.
  `deduced_tracks` and `death_closed_tracks` map registry life evidence to tracks
  by inclusive same-slot overlap. `name_replay_identity_tracks` now composes direct
  life naming, bot seats, succession, final repair and registry deduction union in
  native replay order. The 1,024 native cases compare 8,192 candidate queries,
  including 2,470 fire-resolved choices, 1,315 replacement track assignments,
  7,098 inferred-life track links, 8,192 death-closed links and all resulting
  pipeline track names, inference sets and final repair counts. Track segmentation
  and complete replay document integration remain pending. Validation: 140 Theater
  tests passed (6 opt-in tests ignored), Clippy with warnings denied, WASM
  compilation, formatting and diff checks passed.

- `name_identity_tracks_by_lives` ports greatest-positive-overlap track naming,
  first-candidate tie handling, the expanded final frame and counted replacement
  of an earlier overlap winner. `identity_bot_names_by_seat` and
  `name_bot_identity_tracks` exclude shared seats but permit repeated identical
  declarations, ignore empty names and preserve existing track identities.
  `name_remaining_identity_tracks` ports final previous/next/slot-bridge repair,
  refuses unresolved boundaries between different occupants, does not recycle
  deductions as evidence, and retains inferred-track indices and all five health
  counters. Native signed clock wrap behavior is preserved. The 1,024 native cases
  compare 30,720 track spans: 8,885 previous-life, 4,370 next-life and 1,387 bridge
  deductions; 8,910 unresolved spans include 2,556 contested boundaries. They also
  verify 2,098 overlap-winner replacements, 2,134 shared bot seats and 6,331 bot
  track assignments. These APIs use the identity-bearing track fields; succession
  naming and registry-deduction mapping are now composed above; complete replay
  track/document assembly remains pending. Validation: 139 Theater tests passed (6 opt-in tests
  ignored), Clippy with warnings denied, WASM compilation, formatting and diff
  checks passed.

- `statborg_named_events` ports mode-specific Flag, Zone, VIP and Bomb tables,
  single-pass counter grouping, admitted-round accumulation, bounded increments,
  shared event budget, redundant-counter exclusion and stable total output order.
  `statborg_known_stats`, `statborg_named_counts` and `statborg_cross_check_named`
  expose native inventory/count diagnostics; cross-checks retain duplicate-first
  budget consumption and only compare slots present in the redundant series.
  Flat and round-aware attribution preserve their different empty-identity rules,
  stable time/XUID/stat order and explicit round-attribution drop count.
  The 1,024 native cases compare 219,120 records, 338,166 events, 230,970 flat
  attributions, 162,744 round attributions, 175,422 dropped events and 5,614
  redundant-counter discrepancies. Unsupported mode tables yield no events.
  Validation: 138 Theater tests passed (6 opt-in tests ignored), Clippy with warnings
  denied, WASM compilation, formatting and diff checks passed. Full replay assembly
  remains pending; no automatic mode guess is introduced.

- `statborg_series_by_round` and `statborg_series_total` port the published score
  series APIs, including strict/non-strict subsequence handling, unitary increment
  filtering, team/player separation and the different per-round versus cumulative
  ordering of those operations. `statborg_round_segments` derives bounded K/D/A
  segments and `statborg_round_residue` subtracts the same player's already-named
  rounds from the first matching scoreboard row. `completed_by_elimination` and
  `completed_by_round_residue` preserve native single-candidate/mutual-uniqueness
  guards, zero-segment refusal for residue, duplicate-row handling, deep-copy
  behavior, sequential round updates and provenance. The 1,024 native cases include
  9,210 K/D/A segments, 5,293 residual calculations, 380 elimination additions and
  311 residue additions, plus all strict/unitary series combinations. Full replay integration remains pending.

- `statborg_identity_by_deaths` and `statborg_identity_by_round` port the native
  death-counter timestamp joins, including the 1000-death guard, retained input
  ordering, greedy one-use 150ms coincidences, minimum-three/twofold-margin rule
  and removal of identities claimed by multiple slots. `resolve_statborg_identity`
  retains totals unless deaths name strictly more slots, then removes conflicts.
  `resolve_statborg_round_identity` supplies native round-start consensus/fallback,
  time-aware lookup and publication provenance; `completed_by_lines` fills only a
  single-round result without replacing names or duplicating a player.
  The 1,024 native cases contain 1,392 whole-film death links, 3,540 per-round links,
  148 conflicts, 726 totals-selected cases and 298 deaths-selected cases. The fixture
  also checks 46,080 timed lookups and the full maps before/after scoreboard completion.
  Broader replay wiring remains pending.

- `statborg_raw_series`, `statborg_cumulate_rounds` and `statborg_series_by_slot`
  port the native record-to-counter chain: player/team filtering, negative-value
  rejection, both-channel mode-score guards, per-round bounds, stable timestamp
  ordering, longest nondecreasing subsequence reconstruction, admitted-round
  accumulation and removal of backward timestamps. Bounded steps and series share
  the native 16-unit increment limit; `StatborgEventBudget` tracks rejected steps,
  remaining capacity and truncation across successive calls without partial emission.
  `statborg_slot_identity` joins the resulting K/D/A triplets to supplied player
  lines with one shared million-event budget, rejecting ambiguous rows and every
  identity claimed by multiple slots. The 1,024 native cases cover 142,850 records,
  5,031 accepted identity links, 3,660 rejected steps and 646 exhausted budgets.
  These foundations now support per-round death identity and named objectives; full
  replay integration remains pending.

- `Film.statborg` now publishes native binary statborg records and the truncation
  flag. The frame scanner checks sparse/dense lists and initial round headers,
  retains all A/B/C/D values with conditional-presence flags, preserves a successfully
  decoded prefix when a later component fails, and rejects whole records for A/B
  domain failures. Its 2,048 native generated frames match 717 records and 3,853
  components (1,968 C and 2,021 D channels present). Native 32-bit unsigned reads
  and truncated header reads are preserved. Film scanning follows manifest metadata
  order, ignores unknown chunk types, dates each chunk from its first FRAME, sorts
  stably by time/slot/round and stops after the frame reaching the 33,076-record cap.
  The native 32-film oracle contains 16,113 records with no cap truncations; the
  opt-in full-film regression checks every record and passed across all 32 films
  in 60.98 seconds (release), including the hour-long raid. Cap behavior is implemented
  from the native source but not exercised by the real corpus. Statborg slot identity,
  downstream score/objective extraction and final replay assembly remain pending.

- `resolve_statborg_rounds` ports native round admission, written designators,
  contradictions and the counted round-zero fallback. Admission uses stable
  time ordering and strictly increasing mode-score runs with both-channel domain
  guards, or the native player-record materiality threshold; the contiguous-round
  rule retains its distinction between a short round and an absent round.
  `resolve_statborg_round_bounds` preserves lower-median consensus, slot-majority
  admission, credible boundaries, open spans and whole-block exceptions.
  All 1,024 native cases match over 142,678 records: 2,853 admitted rounds, 773
  contradicted rounds, 182 round-zero fallbacks, 6,736 excluded records and 551
  retained blocks. Registry construction now resolves round starts from decoded
  `StatborgRecord` values, and identity publication derives its emitting-slot
  denominator from their core-kills components. The complete registry/publication
  oracle is rerun through these integrated paths. Binary statborg extraction and
  slot-to-player identity resolution remain pending.

- `build_identity_section` publishes identity players, bounded biped lives and
  statborg round/slot links, retaining native source/method strings, inclusive
  frame clipping, omitted JSON fields, per-source counts and unresolved causes.
  Human names prefer film seats, then the first nonempty death-feed name; bots
  retain declaration order and stable bids; external roster entries are deduplicated.
  No frame step produces the native empty section. Exact serialized JSON and total
  coverage match 1,024 native registry cases: 6,535 player links, 16,030 biped links,
  1,368 statborg links and 256 absent-clock cases. Statborg publication accepts the
  already-resolved per-round identity/origin maps and derives the emitting-player
  denominator from decoded records. Slot identity resolution and raw statborg
  extraction are not yet ported. Current
  statborg fixture coverage uses the native flat identity constructor, including
  unnamed emitting slots; multi-round resolution/provenance needs further coverage.
  Automatic Film/replay integration and final track-name repair remain pending.

- `build_identity_registry` connects effective film/replication table composition,
  owner construction, scoreboard resolution, roster elimination and temporal
  exclusion in the native order. Every stage consumes the same composed index
  table. The 1,024 native registry cases include 21,496 lives, 308 bot and 327 human
  scoreboard assignments, 120 roster deductions and four temporal deductions.
  `bridge_health` retains native coverage counters, optional measured clock offset
  and all bridge verdict branches, including source-accounting refusals after later
  deductions. All six verdicts occur in the native oracle (92 nominal cases).
  Registry data assembly is now available from decoded inputs; statborg round
  extraction, frame-based identity section publication, final track-name repair
  and automatic Film/replay document integration remain pending.

- `build_identity_owners` now composes initial stays, creation naming, one-time
  death-clock calibration, written-boundary refinement, repeated creation naming,
  death rejoining/verification, guarded legacy death naming, creation-only owner
  completion and firing/respawn closures in native order. Its state retains
  ambiguous slots and the native designated-life naming behavior; reports retain
  calibration, readings, refusals and gap fallbacks, including early-return cases.
  The 1,024 native pipeline cases cover 20,762 final lives, 3,505 direct and 1,880
  propagated names, 1,744 legacy death names, 196 closures and 883 ambiguous slots.
  Half include round starts extracted by the native statborg resolver; Rust accepts
  those resolved match-clock starts and converts them using the calibrated offset.
  Table/scoreboard/elimination orchestration and initial health publication are now
  connected by `build_identity_registry` above. Round extraction itself, frame-based
  identity section publication and automatic Film integration remain pending.

- `close_identity_bridge` ports the native firing-then-respawn deductions over
  supplied identity lives. It retains sample-based shot attachment, interval-based
  candidate uniqueness, corroboration by an earlier known body, calibrated respawn
  windows, overlap refusal and designated-life indices. All 1,024 native cases
  match: 251 firing deductions, 296 respawn deductions, 978 contested cases and
  457 refusals. `name_identity_closed_lives` and `extend_identity_slot_xuids` also
  match the native outputs for these cases; existing human identities and life-end
  causes survive. The fixtures include competing bodies, missing tracks/indices,
  negative offsets and empty evidence. These helpers are not yet wired into the
  full identity construction pipeline or final replay document.

- `Film.bot_metadata` now publishes native type-12 bot declarations, maximum
  declared count and packet count. The bitwise UTF-16BE/ASCII name scan retains
  slot, bot ID and source bit position; the earliest name copy per payload and
  first slot/ID declaration across packets survive. Native slot-only unstable
  sorting is preserved. The `IdentityBot` adapter supplies scoreboard inputs.
  All 512 native cases match: 1,533 payloads, 4,201 accepted payload entries and
  2,692 aggregated declarations. Tests also cover shared packet-reader integration,
  noncontiguous chunk indices and stop-at-CHUNK_END. The bot corpus oracle now includes all 32 local films and two upstream samples
  with verified v41 headers: 515 metadata packets and zero bot declarations. This
  verifies counts/empty results; positive real-film bot coverage is still absent.
  Captured packets are self-contained fixtures, and the local full-decoder regression
  checks `Film.bot_metadata` against the native corpus outputs. Complete identity/
  replay orchestration remains pending.

- `resolve_identity_scoreboard` ports native participant-row resolution for lives
  refused as `index_hors_table`. It requires an unambiguous declared bot present
  in the scoreboard, and separates a shared human/bot seat only with a known join
  timestamp and measured death clock. Inclusive arrival edges, crossing-life
  conflicts, duplicate-row behavior, existing identities and mutation provenance
  follow the reference. `IdentityBot` retains stable bot IDs and excludes missing
  or conflicting IDs from the index lookup. The 1,024 native sequences contain
  905 bot assignments, 503 human assignments, 1,176 conflicts and 2,260 no-candidate
  refusals; all reports, identity tables and lives are compared. Complete identity/replay orchestration is still pending.

- `calibrate_identity_death_clock` ports the native automatic match-to-film offset.
  Two shifted 150ms grids count distinct endpoints on both sides; the top three
  separated candidates are refined on the original 10ms grid. Plateau selection,
  nearby-candidate suppression and runner-up counts match the reference. Calibration
  retains input-order greedy counting, separate from the final distance-sorted join.
  The oracle covers 512 cases and 1,401 candidate refinements, with clustered ends,
  clustered deaths, negative offsets, ties and absent evidence. Automatic wiring
  into the complete identity/replay pipeline is still pending.

- `build_identity_life_spans` ports native per-slot timestamp grouping and the
  strict >5s initial gap split, including observations without world coordinates.
  `refine_identity_lives` merges gaps lacking written boundaries, preserving the
  first identity and final cause. Deaths, creation records (even without an index),
  round boundaries and the counted no-death fallback follow native precedence and
  endpoint rules. Death-clock conversion retains native signed integer behavior.
  All 1,024 native cases match: 24,285 stays become 19,314 lives with 534 fallbacks.

- `match_identity_deaths` ports the native greedy one-to-one 150ms match on an
  explicit offset, including Go's tied-candidate ordering. Cause marking, direct
  identity verification and the no-creation naming fallback are separate helpers.
  The same fixture matches all 5,087 pairs and resulting lives, including 3,705
  concordant and 357 discordant direct identities, plus dense equal-time ties.
  Round-boundary extraction, complete identity
  orchestration and final Film wiring remain pending.

- `name_lives_from_creations` ports direct creation identity naming over supplied
  lives. It groups only records with participant indices, preserves native unstable
  timestamp/generation sorting, assigns each creation to its first unfinished life,
  and propagates the latest known body index to other lives. Recycled slots,
  pre-creation divergence, missing records, and declared bot indices retain native
  refusal causes and evidence. `owners_from_creations` provides the separate
  creation-only bridge, including bots, for slots with one consistent index.
  Existing `FilmBipedCreation` observations convert directly to these inputs.
  The native fixture covers 2,048 sequences and 80,128 records, with 11,214 direct
  and 5,916 propagated assignments and 8,568 declared-bot readings. Construction
  of the bounded lives and integration with death/scoreboard evidence are pending.

- `compose_identity_tables` ports native film-seat priority, replication fallback,
  source methods, retained gamertags and coverage. Interleaved vacant seats refuse
  the direct table; disagreements retain film assignments; competing identities
  at an index remove every claimant. The separate native injectivity guard clears
  the whole replication table on collision. All 2,048 generated cases match,
  including duplicate seats, empty names, zero identities, refusal states and
  source-specific counter updates. `ReplayFilmPlayerTable::from_decoded` connects
  the existing bootstrap decoder to the composition input.

- `scan_player_indices` reads the first bit-aligned little-endian XUID occurrence
  and its five preceding bits from contiguous replication chunks. Prefix reads
  before bit zero are zero-padded, the final chunk is excluded, and conflicting
  readings remove only the affected identity. Empty roster/chunk/read cases are
  errors. All 1,024 native generated sequences and 4,608 constituent chunk reads
  match, including metadata gaps and unaligned/boundary patterns. These fixtures
  establish synthetic parity; real-film end-to-end identity assembly is pending.

- `ReplayIdentityState` ports the native bridge over supplied named/bounded lives:
  index collision tracking, timestamp-aware lookup with inclusive bounds, and a
  naming bridge excluding ambiguous slots. Human/bot mutations reject existing
  identities, retain life-end causes, and track deduction provenance. Native
  roster elimination and iterative temporal exclusion preserve their guards,
  simultaneous-candidate conflict refusal, and contradiction counts.
  All 1,024 native sequences compare the bridge after every stage and mutation,
  plus 65,536 identity lookups. Fixtures cover 339 roster deductions, 269 temporal
  deductions, 36 contradictions and 873 accepted mutations. This is the identity
  state machinery; construction from player tables, creation records, deaths and
  scoreboard evidence, plus final Film integration, remains incomplete.

The optimized 32-film full-decoder regression passed in 53.69 seconds after bot
corpus integration, including the hour-long raid and all native bot assertions.

Latest validation after per-round scoreboard completion and score series: 137 Theater tests passed,
6 ignored; Clippy with warnings denied, WASM compilation, formatting and diff
checks passed. Full parser parity remains incomplete.

- `PickupOriginJudge` ports native non-weapon pickup attribution. It selects the
  first nearest-in-time world position within 100ms, checks map points in input
  order with a strict 1m radius, then checks dropped placements with inclusive
  creation/disappearance bounds. Catalog state defaults to `map_absent` and does
  not override geometry. Open/unknown ends have no upper bound; deployed items
  never count as ground pickups. All 253,440 decisions across 512 native cases
  match, including timing ties, 3D distances, missing world positions, extreme
  timestamps, and competing points/placements. The placement input is currently
  a projection of published equipment lifetimes; their full assembly is pending.

- `build_replay_pickups` publishes all native pickup fields and coverage counters
  using supplied catalogs, a replay clock and timestamp-aware identity lookup.
  All 512 generated cases match native output, now including actual origin-judge
  integration and per-point-kind counts. Classes use only their relevant catalog;
  before-origin events are rejected and events beyond the frame grid retained.
  Full Film wiring, automatic clock/catalog resolution and identity registry
  construction remain pending.

- `date_pad_pickups` ports the native event-to-occupation join. It normalizes
  eight-digit weapon family IDs (optional 0x prefix), distinguishes non-weapon
  pads, uses inclusive intervals, and dates only a unique weapon event. Bounds
  remain untouched. Uncovered/ambiguous occupations preserve existing identity
  and time, and an unnamed event does not erase an earlier identity. Counters
  cover dated, named, ambiguous, uncovered and power-up occupations separately.
  All 1,024 generated native cases match, including invalid/negative pad indices,
  empty event streams, existing fields, unnamed pickups and malformed family IDs.
  `ReplayPickup` now carries all native fields; final Film wiring remains pending.

- Ground-pad rules now cover native nearest-centroid clustering (1m, same kind
  and family), assignment remapping, recurrence filtering, pickup-to-reappearance
  gaps, population deviation and cycle confidence. Pad publication accepts an
  explicit replay frame grid and preserves bounded presence, completed occupations,
  coverage counts, power-up family names and uppercase `0x` weapon IDs. Unknown
  pickup identity/time stays unset until `date_pad_pickups` is applied.
  All 512 native generated cases match, including 2,070 published pads and 11
  established cycles. Publication from the four native object fixtures matches
  all 10 pads and one cycle on the same explicit test frame grid. This verifies
  publication independently of the separately checked object assembly/scans.
  Complete Film integration and final replay assembly remain
  unfinished. Fixture generation includes both synthetic and corpus publication.

- `assemble_ground_objects` ports native `padObjects` for a supplied identity rule,
  joining creations, same-archetype tracks/census and sorted player positions.
  Objectives are excluded before family membership; missing or unknown identities
  count on their rejection paths. Native timestamp tie permutation and final object
  ordering are retained. The result carries appearance, family ID, dropper,
  movement position, pickup bounds/status, matching diagnostics and ammunition.
  The 256 generated native sequences match all 5,555 objects and rejection counts,
  including conflicting objective/catalog membership and reused generations.
  Four-film assembly matches all 519 objects and 570 identity rejections, using
  native player-position inputs and Rust creation/track/census scans. This checks
  assembly independently; position scanning has its separate production oracle.
  The corpus rule supplies no objective catalog; synthetic cases test exclusions.
  Current regular validation: 114 Theater tests pass (6 ignored), Clippy with
  warnings denied, WASM compilation and formatting pass.
  Full Film integration and weapon/power-up pad clustering remain unfinished.

- Default `Film.keyframe_ground_weapons` now exports native archetype-42 family
  signatures with slot, generation, timestamp, chunk and packet index. It shares
  the biped family scanner while preserving the full entity ID until attribution.
  It retains repeated signatures and aliases in bit order. The native fixture
  includes 512 generated/truncated keyframes and four complete production scans.
  All 720 production observations match, including families and source locations.
  Current validation: 113 Theater tests pass (6 opt-in tests ignored), production
  comparisons pass, and Clippy, WASM and formatting checks pass.
- Ground-weapon lifetime helpers now port the reference's sighting restriction,
  disappearance bounds and nearest mobile-track selection (200ms pre-birth
  tolerance, strict reuse boundary, input-order ties). Bounds distinguish final
  surviving sightings and absent later keyframes from observed disappearance.
  The helper pipeline also resolves reference pickup status, selects the first
  player passage with time/distance/slot tie breaks, splits player lifetimes,
  and infers a dropper within 200ms and 1.5m (lowest matching slot). Native
  comparisons retain exact distance bits and cover dated/unknown/never outcomes,
  missing world positions, 5s life boundaries and tied droppers. These helpers
  are prerequisites for replay ownership/drop/pickup assembly, which remains
  unfinished; they are not yet attached as assembled weapon objects.
  Durable harnesses and fixture regeneration cover both additions.

- Ground-weapon NEW payloads reuse the equipment creation walk with archetype 42
  defaults and registry-guided ammunition decoding. Magazine and reserve are read
  from component 20 after traversing preceding components, including MPP; unknown
  components stop that observation. The native corruption-check option and padded
  tail reads are preserved. Map-aware `Film.ground_weapon_creations` retains raw
  records and counters when widths and the film flag are resolved; these records
  are not yet confirmed weapon drops or assembled world objects.
  The native harness `halo_rust_ground_creations_test.go.txt` captures four films
  and 2,064 payload/truncation/check-flag cases, with 1,564 accepted observations.
  Fixture regeneration is wired into `generate_oracles.py`.
  All 1,089 full-film creation records, including 510 ammunition readings, and
  every diagnostic counter match the four-film native oracle.
  The regular suite now passes 111 Theater tests (6 opt-in tests ignored);
  Clippy with warnings denied, WASM compilation and formatting checks pass.

- Equipment NEW payload scanning is ported in `equipment_creations.rs`, sharing
  the default-state grammar and world-object position reader. All fields and
  diagnostic counters match 2,560 native cases (7,031 accepted records): sparse
  and dense masks, absent i0, optional reference/ability IDs, arbitrary bytes,
  and truncated payloads. `position_padded_bits` reports accepted zero-tail
  position reads. Reproduction is wired into `generate_oracles.py` with the
  durable `halo_rust_equipment_creations_test.go.txt` harness.
  Equipment lifetime matching, MPP calibration, placement confirmation and
  map-aware `Film.equipment_placements` integration are now ported. The scanner
  shares the census band and keeps raw creations and diagnostics. Format 27
  widths take precedence; other formats require conclusive calibration. Native
  candidate preference and early stopping are preserved. The last movement time
  remains a lower lifetime bound, not an inferred disappearance.
  All 548 placements and every placement/calibration diagnostic match the four
  production films (0 Bazaar, 0 Aquarius, 160 Bandit, 388 Oddball). Another 512
  native sequences verify lifetime grouping, repeated generations, duplicate
  creations, nearest-time matching, sorting, and calibration verdict ordering.
  Current checks: 110 Theater tests pass (6 opt-in tests ignored); the four-film
  production comparison, Clippy with warnings denied, and WASM check pass.

- Default `Film.world_object_keyframes` exports the native presence census for
  archetypes 37, 38, 41 and 42. A generic public scanner accepts other archetypes.
  All requested archetypes share one recovered-anchor pass; map-aware projectile
  scanning reuses the resulting slot band. Timestamps retain the native sorting
  and consecutive-observation deduplication rules, including repeated times after
  out-of-order input. The oracle covers all 32 films plus 64 constructed sequences,
  including changing archetypes and gaps in chunk numbering.
  All 5,736 native slot/generation entries and 34,217 presence observations match
  through the default Film export. 108 film unit tests and all four corpus checks
  pass (48.89 seconds), as do Clippy, wasm compilation, formatting and diff checks.

- Native projectile mobile tracks are available as map-aware `Film.native_projectiles`.
  The shared world-object scanner also accepts an explicit archetype or slot set.
  It fills keyframe slot ranges while excluding slots observed in other archetypes,
  rejects foreign regions/saturated axes, preserves padded accepted records, and
  splits mobile lifetimes after rest or gaps over 250ms, requiring three samples.
  All 509 production tracks / 39,283 samples match the four-film native oracle.
  2,048 generated position scans and 512 generated lifetime sequences match exactly.
  Equal position/time samples require the pinned Go sort.Slice ordering before
  lifetime splitting; native_sort.rs ports that behavior with the Go BSD license.
  Broader map/archetype coverage and world-object keyframe lifetimes remain pending.
  Validation: 107 film unit tests pass; the expanded production comparison passes
  (16.97 seconds) and all 32 existing exports pass regression (41.26 seconds).
  Clippy with warnings denied, wasm compilation, formatting and diff checks pass.

- Native grenade throws now export on `Film.grenade_throws`, with the film's
  projectile archetype selected by component names, explicit reference-profile
  fallback flags, and all scan rejection counters. The preceding sixth archetype
  bit is required; a marker at bit zero is reported as indeterminate.
  All 1,494 captured throws match native, with all 2,750 matching signatures and
  490 other-archetype rejections accounted for. Another 2,560 generated cases
  exercise all six-bit archetype values, unknown IDs, wrong high bits, boundaries,
  and truncation. Fire and grenade scans share native chunk-prefix/framing rules.
  Default Film exports match native records and counters across all 32 films; the
  separate four-film production FilmInputs comparison also passes. 105 film unit
  tests and all four corpus checks pass (42.86 seconds), as do Clippy with warnings
  denied, wasm compilation, formatting and diff checks.

- Long fire events now export on `Film.fire_events`: both shooter index widths,
  weapon ID, flags, timestamp/source ordinal, and exact float32 aim. All 19,075
  captured heads and 11,542 available aim vectors match native, alongside 4,096
  generated cases. The modal aim locator reports synthetic padding separately;
  publishing aim still requires all 30 physical bits and a valid cube face.
  The native scanner preserves metadata-prefix numbering and stops at chunk-end,
  degenerate headers, or malformed tails; 48 generated wrapper cases exercise
  those branches. Fire events do not assert hits, victims, or player identities.
  Default Film exports match all 32 films, and the scanner matches the separate
  four-film production FilmInputs oracle. 104 film unit tests and all four corpus
  regressions pass (41.97 seconds); Clippy, wasm, formatting and diff checks pass.

- Type-8 session population packets exported as `Film.roster_updates`, including
  packet source locations/timestamps, completed entries, option gates, byte-swapped
  XUIDs, all native player fields, and stopping/overflow diagnostics. All 509 captured
  packets (5,347 player entries) match native. Another 2,368 generated/truncated/random
  cases verify full UTF-16 names, gates, counters, and exact stopping positions.
  The body reader is shared with the bootstrap player table. A refused oversized
  list count is kept distinct from a read beyond the buffer, as in the reference.
  Film integration matches native packet order, payload offsets/sizes, timestamps,
  entries and reports across all 32 films. All four corpus regressions pass
  (42.50 seconds), as does the dedicated bootstrap comparison after the shared
  body refactor. 102 film unit tests, Clippy, wasm compilation, formatting and
  diff checks pass.

- Bootstrap player table exported as `Film.player_table`, independently of legacy
  roster recovery. All 67 occupied slots and every diagnostic match across all 32
  local v41 films. A captured bootstrap plus 64 generated tables and their
  truncated/unknown-build/empty variants cover vacancies, invisible scan entries,
  full-length names, raw short fields, and failed-table diagnostics. Matching the
  native prefix-only vacancy predicate does not prove the full vacant body exists.
  Replay attribution and the missing/unreadable bootstrap wrapper remain pending.
  Validation: 101 film unit tests pass; the dedicated 32-film player-table
  comparison and all four existing corpus checks pass (42.18 seconds for the
  latter). Clippy with warnings denied, wasm compilation, formatting and diff
  checks pass. The durable oracle contains 289 cases.

- Correct registry framing: two version words, 260-byte entries comprising a
  256-byte name and a precision u32, 64 slots per block, structural end detection.
  All 32 local v41 films have 50 blocks and 1,067 named slots. The previous 118
  count erroneously included later bootstrap sections.
- Bootstrap identity and per-type versions exported on Film. A captured fixture
  compares all registry names/levels and identity fields with actual Go output.
- Type-1 datum tables exported on Film, preserving all allocation flags,
  generations, view masks, component bitmaps, tail words and alignment bits.
  Sparse storage is lossless: missing slots equal the explicit default entry.
- Bounded sequential bits, signed variable integers, categorical handles,
  sparse/dense masks and prefix-coded record headers.
- Independent component readers and registry-driven delta traversal. Every read
  is retained as a named raw scalar with exact source bits. Unsupported components
  stop traversal. The opt-in sequential pipeline exports their records and stopping points;
  full gameplay parity is still incomplete.
- `parity-cases.json` records current randomized comparison coverage. Fixtures
  check cursor positions for every supported registry archetype/precision context.
  These are grammar boundary tests, not end-to-end gameplay parity.
- Dynamic-position field grammar accepts explicit precision context, including
  default/map-region axes, prediction, signed delta forms and handle tails.
  Absolute world units and baseline accumulation are separate remaining work.
- Captured multiplayer ammo records match the registry-driven walker at every
  component boundary. The existing 32-film player/projectile/clock/event streams
  passed full regression after bootstrap and datum integration.

- Actor control/state and shared action blocks retain all primitive fields.
- Message view A and control view C readers match 2,048 reference cases, including
  explicit stops on upstream message/control payload gaps.
- Default-state readers cover 47 archetypes (including verified zero-bit stubs),
  checked against 3,008 reference cases. Archetypes 23, 41 and 44 are unsupported.
  Vehicle defaults require explicit map precision; MPP widths are caller supplied.
- Full-state keyframe records compose defaults and all named components with the
  108-bit header, signed n1/n2 guards, and optional corruption checks. 256 synthetic
  reference cases check this framing; a captured real keyframe also matches the
  reference at every supported record/component start and record end.
- Dynamic orientation and simulation-state grammar preserve raw vectors, packed
  directions, roll, and delta fields. Simulation requires explicit map precision.

- Sequential entity view B handles creation, deletion, baseline selectors and
  generation-checked updates, with bindings installed only after complete creation.
  Thirty-two mixed-chain reference cases cover optional fields and generation failures.
- Sequential keyframe tables handle the no-archetype sentinel, slot ordering,
  bounded headers and explicit stop reasons; each snapshot replaces prior bindings.
- `Film::try_from_chunks_with_encoding` exports `replication` with all three views,
  keyframes, datums, untouched unknown packet bytes, and unparsed bit tails. Encoding
  context is caller-supplied; bootstrap identity overrides its corruption flag.
  This is not yet automatic profile selection or accumulated gameplay state.
- Physics state, tactical-map fields and rigid-body transforms are ported. Generic
  traversal axis widths are distinct from world-object/map-region position widths.
- Guarded keyframe recovery matches all 134,657 reference anchors in 451 cached
  keyframes. Candidate ordering, sentinel stops and first-measured repeated widths
  follow the pinned reference. Each recovered record is decoded independently;
  crossing the next anchor is reported. Bindings and later updates retain whether
  their identity came from recovery, a sequential keyframe, creation, or the caller.
- `resolve_v41_profile` resolves format 27 and the two supported builds, with explicit
  missing/unknown-context issues. The pinned 79-map catalog supplies region widths
  and coordinate bounds. All 158 map/build combinations match independent Go output.
  `Film::try_from_chunks_with_map` composes this profile and exports it alongside
  replication; ID layout and map identity remain independently supplied inputs.
- Navigation-marker flags, five filter components and formatted text are ported.
  Filters preserve all payload tags, reference gates, distances, booleans and order
  fields. Reserved tag 15 stops exactly as in Go. Randomized parity now checks
  both successful reads and explicit unsupported results; every selected component
  must have successful reference cases.
- `Film.head_events` exports native packet-head reads for boarding (8), pickup (9),
  zoom (21), vehicle exit (22), and spawned-object references (103), retaining all
  guarded references and generations with exact bit ranges. Public helpers resolve
  the reference's picker/scope/spawn bases and caller-supplied biped base for vehicles.
  Unknown types and truncated payloads remain explicit. These are head scanners;
  successful field reads do not claim an entire event-list boundary.
- Native Go outputs match 14,391 head-event cases: 13,111 captured events plus
  1,280 synthetic cases. The local corpus has no boarding events, so that family
  currently has synthetic coverage only. Other counts include 1,646 captured pickups,
  11,263 scope changes, 135 exits and 67 spawned-object events.
- `gameplay-parity.json` inventories all 44 fields of the pinned reference's
  `FilmInputs`. It separates proven parser fields from channels still requiring
  actual output comparison; existing Rust observations alone do not mark parity.
- Type-117 translocator heads retain unit/generation, effect and quantized jump
  endpoints. World coordinates use the selected map region or the engine's explicit
  default box. Missing/invalid maps, unknown regions, unsupported references and
  truncation retain the attributed event with a stop reason. `Film.translocations`
  is populated by the default API; the map-aware API resolves its positions.
  All 632 synthetic Go cases across 79 maps match (316 complete jumps, plus failure
  paths). There are no captured type-117 events in this corpus, so captured coverage
  remains pending. Position-speed exemptions now match the native Go filter cases.
- The actual production `scanFilmInputs` outputs for Bazaar, Aquarius and both Recharge matches are retained
  in `fixtures/gameplay-levelup-v41.json.zlib`, with all 44 fields. External inputs
  are map catalog entries only: no caller roster or mode/zone catalogs. Struct-keyed
  diagnostic maps are represented by sorted key/value arrays, not discarded.

Typed vitality readers now retain i4 health and flags, plus i5 shield, regeneration
words and flags. Every 8-bit health/shield quantum and 512 randomized component
payloads match native Go values and cursor ends. Truncation leaves the caller's
cursor unchanged. Native Go uses fused arithmetic here; explicit Rust `mul_add`
reproduces its rounding. The readers now populate biped position companions alongside i1 velocity, i2 direction and both i21 aiming vectors. The companion scan matches 4,096 native Go cases covering all bit alignments, truncation and unsupported components.

`Film.biped_creations` now exposes signature-guarded ti35 creation prologues without
map bounds. All 393 native production records match across four films, including
participant index, slot/generation, packet/time and bit position. 2,048 synthetic
Go cases cover rejection counters, alternate representations, truncation and scan
advancement. Rejections remain diagnostics; an absent index is never coerced to zero.
This identifies the owner of a creation but does not decode its complete default state.

`Film.pickups` and `Film.zoom_events` now expose native gameplay observations:
499 pickups, every pickup coverage counter and 1,781 zoom events match four-film
production output. 2,048 synthetic heads verify publication/rejection behavior;
3,000 timestamp/slot/hold queries verify the reference's held zoom level. Truncated
payloads do not fabricate absent fields. These remain first-event scans, as in Go.

`Film.equipment_spawns` retains the source and spawned object lives (slot plus
generation), reference-2 presence, and source packet. Six production events and
all packet/list/chunk/reference counters match across four films. The reference's
chunk counter includes the terminal summary chunk and stops at the first numbering
gap; packet-list counts are separate from accepted events. These are spawn facts,
not inferred equipment placements or deployment origins.

`Film.biped_channels` now exposes registry-driven ability and camouflage reads
under the map-aware API. The shared component walk starts after the established
position vector and the reference two-bit tail; it stops at unsupported or
truncated components. It includes movement-rejected anchors, as gameplay scans
must. Successful components retain every field and source bit; typed accessors
expose camouflage quanta and ability counter/rank emissions. All 4,401 camouflage
transmissions and 33 ability identities match four-film production output; eight
explicit no-ability emissions are retained separately. 1,536 independent Go cases
exercise intermediate failures, gated values and all bit alignments. The captured
comparison also checks every camouflage/ability scan counter, including unread and
gated counts, against independently generated Go statistics.

`Film.inventory_deltas` now exposes grenade counts, selected grenade rank, and
weapon-slot magazine/fraction/reserve changes under the map-aware API. Component
roles follow registry names and occurrence order. Publication matches the native
plausibility guards and whole-film ammunition refusal rule; raw rejected components
remain available separately. All 8,636 published updates and every inventory diagnostic counter match four-film production
output. Sixteen synthetic sequences (3,640 records) compare every counter and
exercise invalid counts/selections, envelopes, corroboration, cross-checks, the
200-sample minimum and 1-percent whole-film refusal threshold. Inventory, camouflage
and ability scans now share the same component-walk implementation.

## Verified upstream limitation

The reference's sequential walker itself stops in record two of the captured
`natural-end/01-do-nothing` keyframe at `tacmap-mapdismissallock`, bit 2279.
Rust reaches the same stop under the identical reference profile, with matching
record/component boundaries. The raw fixture and Go output are retained. This
is not a full-film parity result: guarded recovery is now ported, but high-level fact
extraction paths remain necessary to match its actual exported gameplay output.

A no-map-context diagnostic over all 32 films found 451 keyframes, all initially
blocked by physics state. After that port, all advance to tactical-map waypoint
state. The diagnostic deliberately omits position context and therefore stops there;
its result is a work-prioritization report, not a correctness assertion.

The pinned `position_capture.go` contains an optional `World` position accumulator,
but explicitly records that its installer has no callers. Production output parity
must therefore follow the actual scanner/observation assembly in
`replay/build_from_film.go` and `replay/film_scan.go`, rather than assuming that a
generic accumulated world is the reference's active export path. Its position
reader also consumes 96-bit keep-baseline payloads without emitting those bits as
coordinates. Preserve that distinction when adding typed position values.

The new `Film.biped_positions` stream matches all 423 Bazaar production delta
positions (including the 19 absent from the legacy tracks) and zero Aquarius delta
positions. Identity, packet time, raw quanta and float32 world values match exactly.
Legacy tracks remain unchanged; their Aquarius spawn is not a delta observation.
The guarded walker additionally matches 1,264 Go cases across all 79 maps; 64
sequences of 128 positions match isolation, speed reanchoring and type-117 exemptions.
All candidates retain source ranges and explicit rejection reasons. Attached aim and
vitality fields also match for the two controls; all attached fields now also match both Recharge matches. In total, all 590,677 accepted positions match across four films, including 462,732 primary aiming samples, 536,256 velocity samples, 2,770 body readings and 232,444 shield readings. Full parser parity remains incomplete.

## Remaining work

- Finish component families and per-film precision profiles, including
  world-object position, complex biped actions, navpoint filters,
  multiplayer properties, managed properties and world state.
- Finish automatic ID calibration and entity inference, baseline
  reconstruction and accumulated state; compare their actual full-film outputs.
- Port event lists, inventory/lifecycle assembly, kill/damage/assist attribution,
  objective/score data, equipment and vehicle observations.
- Extend Film exports and replace signature scans only when equivalent or better
  coverage is independently checked. Preserve old observations while migrating.
- Establish a complete field/output parity report. The reference itself contains
  explicit unsupported paths; document those separately from Rust omissions.

## Ability state observations

`Film.ability_states` exports predicted/non-predicted ability bodies, including
known prefixes of unsupported grapple bodies, thruster impulses and grapple
anchor quanta. Canonical readers now include ability energy and mobility action;
missing intermediate readers previously prevented reaching the ability state.
The mobility position leaf uses traversal widths, while ungated grapple vectors
use explicit map world-object widths. These contexts are retained separately.
All four impulses, 26 grapple reads and every associated counter match production
across the four films. 4,096 generated records compare scanner outputs/counters;
the same cases compare 8,192 direct ability bodies, mobility/energy boundaries,
and truncation. Successful heavy/light anchors and unsupported flags are covered.
Charge publication is implemented separately in `ability_charges.rs`: the map-aware
Film export retains whole/fractional values and raw mask-zero reads without
fabricating values for unarmed slots. All 39 records and every counter match the
four-film fixture; 4,096 generated cases compare publication and intermediate walks.
Movement-state assembly is exported by the map-aware Film API (see below).

`anticipated_bindings.rs` now provides the reference's dated later-keyframe table,
keyed by full ID. It preserves declarations, conflict counts and high-bit counts;
128 native tables/65,536 queries and captured keyframe construction pass. It is
integrated into the production movement frame walk. The live
reference path is `decodeFrameParRangs` (message/entity/control classes), with
`decodeInferLoop`, datum bindings and `rejetDeVue` anticipation semantics.

`world.rs` now ports binding ownership, generations, soft/wildcard/datum bindings,
positions, anticipation and owned snapshot rollback. 128 native operation sequences
of 64 steps compare complete state and 16 queries per step. Rust consumes rollback
snapshots; reusable Go oracle snapshots are refreshed after restore to avoid aliasing.
`keyframe_datums.rs` matches all candidates, longest-increasing-subsequence choices,
tables and ambiguity counts across 512 generated payloads and a captured keyframe.

`production_frame.rs` implements the default message/entity/control policy with
persistent world admission and explicit termination diagnostics. Sixteen default
frame chains and sixteen admission cases compare native records and final cursors,
including generation reuse, wrong views, unknown slots and dated anticipation.
The generic reader retains its separate strict-generation behavior. Nondefault
extra-fields profiles are explicitly unsupported by the production API; optional
inference repair remains disabled. Native record padding is now explicit, as described below.
The strict slot-123/35-bit event locator matches 256 native cases without mutating
world state. Production traversal ignores mask bits beyond the registry component
list, as native Go does; the generic decoder retains its strict validation.

`components/movement.rs` ports slide and all four posture branches. 6,144 native
cases verify component boundaries and published values, including unsupported name
aliases (Go accepts `biped-slide` but rejects `unit-crouch` and
`biped-posture-physics`), full/quantized precision, levels and truncation.
`EntityRecord.attempts` retains known fields from incomplete component bodies.
A production observer runs before binding changes and models Go's stale capture
slot on NEW records; predicted ability and mobility flags can publish before their
bodies finish. Conventional component readers reject truncation; the production
record walker uses explicit native padding and exposes its provenance.

`movement_states.rs` now walks chunks, binds anchors/datums, locates event packets,
deduplicates transitions, and derives closed jumps from held vertical velocity.
4,096 native velocity vectors match exactly; 1,024 generated multi-slot sequences
match all derived transitions and counters. Fused coordinate/norm arithmetic is
explicit to match the pinned Go build; the magnitude exponent remains unfused.
The map-aware Film API exports `movement_states`, including source packet identity,
raw progress, on/off values, diagnostics and padding provenance. All 17,600 movement
readings and every counter match actual Go output on Bazaar, Aquarius, Bandit and
Oddball: 503,980 biped records, 445,546 velocity reads and 1,633 derived jumps.
World position accumulation and broader corpus/profile coverage remain pending.

Production traversal now includes biped context/malleable fields, equipment state,
weapon ammo, scene/player metadata and mode-2 multiplayer properties. The latter
retains wire values for varints, schema skips, nested lists/maps and opaque bodies;
native limits on lengths/counts/depth are preserved. The component oracle covers
15,864 cases across 207 names and 632 contexts, including constructed nested TLV
messages and exact stopping positions on unsupported branches.

The prior Bazaar counter difference was a truncated weapon-state body: chunk 3
packet 352 has 405 payload bytes (3240 bits), but Go ends the record at 3347.
The production reader now matches native zero-tail semantics; `EntityRecord` and
`ProductionFrame` retain `padded_bits`, and `MovementStateStream.padding` identifies
affected source records. 89,505 native reads verify widths 0..64 before/across/after
payload ends. Bounded readers retain their previous truncation contract. A frame
test verifies partial-byte preservation, synthetic tail count and bounded rejection.
Native default-state fallbacks for types 23/41/44 are now explicitly marked on
records; 48 reference frame cases verify bindings, cursor and ignored mask tails.
Standalone default-state decoding still reports those unknown grammars unsupported.

The missing biped NEW tail was consequential: Bandit chunk 1 packet 338 previously
stopped at bit 1608, before the complete record at 2112 could bind its entity.
Porting those readers and the intervening scene records repaired subsequent world
bindings. Bandit and Oddball now match every traced record and movement output.

The diagnostic native harness is `halo_rust_movement_trace_test.go.txt`; it produces
`/private/tmp/halo-movement-trace.json` for Bazaar by default, or Bandit/Oddball with
`HALO_TRACE_FOLDER=bandit/01-evo` (or `ranked-arena/02-oddball`). The same environment
variable on the Rust movement corpus test compares every frame record and each
component start/support status, and stops at
the first divergence. Regenerate the native trace for the selected film first. The durable reproduction is
`cargo test --release --lib theater::corpus_tests::movement_scan_matches_production_films -- --ignored --nocapture`.


## Weapon loadouts and changes

`keyframe_loadouts.rs` ports the catalog-signature scan inside recovered biped
keyframe ranges. It preserves input order, duplicate families and aliases. The
pinned catalog is exported directly from native `weaponv3.KnownWeaponHigh32`;
callers can also supply a family set. All 501 production loadouts match across
four films; 256 constructed native payloads verify mixed-archetype attribution.
These are catalog signatures attributed by recovered bounds, not a proven full
sequential loadout decode.

`weapon_changes.rs` ports identity emissions and taken/dropped/swapped/restated
classification. The first available loadout is used even before its timestamp,
matching native `spawnSetFrom`; later input-order samples at/before the event win.
4,096 native classification cases pass. The regenerated four-film oracle adds all
weapon-change counters and preserves every existing FilmInputs field unchanged.
All 149 production weapon changes and all four counters match across four films.
Loadouts are exposed on the default Film export; weapon changes on the map-aware
export when a biped slot band exists. Missing bands leave the optional stream
unavailable. Film JSON roundtrip passes.

## Keyframe inventory

`keyframe_inventory.rs` ports all five native inference rules: unique ability anchors,
anchor-first grenade counts, exact ammo-block landing and longest-candidate selection,
positional grenade fallback, and unanimous selected-grenade reads. Unread values,
zero values, candidate counts and grenade-source counters remain distinct. The
native default grenade cap is explicit in the exported stream; callers can supply
another cap. Chunk counting excludes the registry and stops at the first numbering
gap, following `FilmChunkNumbers`.

All 701 keyframe inventories and every scan counter match the four-film production
oracle. 256 generated mixed-archetype keyframes and 2,048 partial ammo parses match
native fields and stopping positions. The oracle includes IEEE gauge bits because
the default JSON number parser can shift a float by one ULP. Rust preserves the raw
R(12) gauge quantum and reconstructs the gauge from it on JSON import; generated
inventory roundtrips retain exact values. The default Film export exposes the stream
when the native contiguous chunk prefix exists; otherwise it remains unavailable.
These are reference heuristics, not a claim that full sequential keyframe bodies
are understood. Broader corpus/output coverage and unreadable-chunk wrappers remain
pending.

## Equipment transition recovery

`equipment_changes.rs` and `equipment_recovery.rs` port strict/recovered merging,
head/interior counter windows, sparse/dense candidate walks, ambiguity rejection,
final counter-chain pruning, birth classification, previous ranks and gaps. The
map-aware Film API exports transitions, counters and candidate windows. Native
unstable sorts omit tie-breakers in final output/windows; generated comparisons
preserve complete records without requiring undefined ordering within equal keys.
All 41 transitions and every counter match four production films; those films have
no accepted recoveries, so successful captured recovery remains unverified.
512 generated assembly/window cases and 2,048 recovery walks match native Go.

## Latest validation

- Keyframe inventory: all 701 production records and counters match (15.89s release
  with prior gameplay channels). Native generated inventories, partial ammo and
  lossless gauge-quantum JSON roundtrips pass.
- Default Film integration passes all 100 film tests. The four corpus-module
  checks pass together (42.48s release), including all prior four-film gameplay
  outputs, movement, and preservation of existing observations across 32 films.
  The fourth check is a diagnostic frontier report, not a parity gate. Clippy,
  wasm compilation, formatting and diff checks pass.

- Loadout/weapon-change production comparison passes with all prior channels
  (13.07s release): 501 loadouts, 149 changes, every weapon counter. Classification
  passes 4,096 native cases; keyframe attribution passes 256 native payloads.
- Film export integration passes all 99 film tests, with five opt-in corpus tests.
  All 32 existing observation exports are preserved (33.60s release). Final
  library/test Clippy, wasm compilation, formatting and diff checks pass.

- Four-film movement comparison passes (9.26s release): all 17,600 outputs and all
  counters match Go. Bandit/Oddball record and component-start traces pass (6.80s/10.29s).
  Map-aware Film movement export and JSON roundtrip pass.
- Component oracle: 15,864 cases / 207 names / 632 contexts pass. Library/test
  Clippy and wasm compilation pass after movement export integration.
- Existing four-film positions/companions/inventory/equipment/ability regression
  passes after movement integration (10.45s release). Full film suite passes
  97 tests with five opt-in corpus tests.

Earlier validation history:

- Native padding and default fallbacks: 89,505 scalar reads and 48 default-frame cases match Go. The full movement test now matches Bazaar and Aquarius, with Bandit/Oddball still failing (8.80s release). 96 existing film tests plus the added padding-provenance frame test pass; Clippy and wasm checks pass.

- Ten traversal readers: 8,424 native component cases pass; 94 film tests, Clippy, wasm, formatting and diff checks pass. Existing four-film production regression passes (11.35 seconds, release). Movement parity remains a failing required gate.

- Existing four-film production output regression passes after the movement component additions (12.08 seconds, release).
- Movement component/derivation additions: 94 film tests pass, five opt-in corpus tests ignored; Clippy, wasm, formatting and diff checks pass. The new movement corpus parity test is explicitly failing on three films and remains a required completion gate.

- Four-film production regression passes after the shared frame-reader refactor (12.32 seconds, release); existing positions, companion fields, inventory, equipment and ability outputs remain equal to Go.
- World, keyframe-datum, production-frame admission and strict-locator additions pass: 92 film tests, four opt-in corpus tests ignored. Library/test Clippy, wasm compilation, formatting and diff checks pass.

- Charge integration passes the four-film production comparison (12.71 seconds, release), including every charge counter. Anticipation is independently checked by native generated tables and captured keyframe construction. Library/test Clippy, wasm compilation, formatting and diff checks pass.

- Ability/equipment production comparison passed in 11.44 seconds (release). Every impulse and grapple counter matches the regenerated native fixture. All 32 legacy exports passed regression during this integration. Library/test Clippy and map-aware JSON roundtrip passed.

- 88 film library tests passed after charge and anticipation work; four corpus diagnostics/regression tests are opt-in.
- Equipment production comparison passed in 13.23 seconds (release). Library/test Clippy, wasm compilation, and map-aware export roundtrip passed.
- Inventory integration passes the expanded four-film production comparison, including every inventory counter (11.73 seconds, release, excluding compilation). Map-aware JSON roundtrip, Clippy, wasm compilation, formatting and diff checks pass.
- The expanded production comparison (including camouflage/ability values and counters) passed in 11.81 seconds, release, excluding compilation. Map-aware export roundtrip, Clippy, wasm, formatting and diff checks passed.
- After pickup/zoom/equipment-spawn integration, production comparison and all 32 legacy exports passed together in 31.33 seconds (release, excluding compilation). Clippy, wasm compilation, formatting and diff checks passed.
- After biped companion/creation integration, the production comparison and all 32 legacy observation exports passed together in 28.98 seconds (release, excluding compilation).
- All 590,677 accepted position records and 393 creation prologues match the four-film native production fixture, including companion fields and source attribution.
- Component coverage is recorded in `parity-cases.json`; current totals are above.
- 2,048 view cases, 3,008 default-state cases, and 256 keyframe framing cases.
- 158 profile cases and all 134,657 recovered anchors across 451 keyframes.
- 14,391 native head-event cases, including all 13,111 supported captured heads.
- 632 translocator cases across all catalog maps, with exact float32 world values;
  Film integration checks missing-map retention and subsequent map-based resolution.
- Clippy with warnings denied, wasm library compilation, formatting and diff checks passed.
- All 32 existing film observation exports passed regression after native head-event
  integration (22.60 seconds excluding compilation). Their new supported head events
  also match the captured Go cases in chunk/packet order. This verifies those channels
  and old observation stability, not full parity of the new parser.

## Reproduction

From the repository root, with Go 1.26.5+ and the pinned reference checkout:

```sh
python3 src/theater/reference/generate_oracles.py /path/to/LevelUp \
  --go /path/to/go --corpus experiments/films
cargo test --lib theater
cargo test --release --lib theater::player_table -- --ignored --nocapture
cargo test --release --lib theater::corpus_tests -- --ignored --nocapture
```

The reference generator adds test harnesses to the reference checkout and
runs the real Go parser. Film downloads and existing decoded exports are untouched.
The full-corpus regression requires existing decoded-film.json baselines.

`levelup-port-manifest.json` inventories reference source files and their hashes.
Pending includes reference application adapters and research code until their
relevance is classified; file count is not a completion percentage.

### Objective actions and bomb assembly continuation

- Objective-action publication passes 1,024 native cases, including family-only
  coverage, roster refusal, invalid clocks, unknown identities and missing-track
  diagnostics; native variant and family classification probes also pass.
- `replay_bomb_film` assembles carry, armings and statistics, retaining raw
  millisecond carry periods and identified events for stats while using published
  detonations for fuse validation. Score presence and kill-read gates are explicit.
  This assembly still needs positive captured bomb validation and full document wiring.

### Flag carry chronology and homecoming

- `replay_flag_carries` and `replay_flag_home` pass 2,048 native cases
  comparing mode evidence, slot/time opening merge, initial bounds, handoffs,
  credited returns, object homecomings, carrier-kill closures and neutral spawn
  selection. Every intermediate stage is compared, not only the final intervals.
- Closure replacement clears stale capture/home state; carrier-kill evidence
  preserves the native exception that shortens an open carry without closing it.
- Neutrality comes from explicit spawn labels, with at least three births and
  more neutral than team births within the native 10 cm radius. Float32 subtraction,
  last-equal-distance selection, and frame quantization match the reference.
- Full flag geometry, object-drop closure, assignment, marks, coverage and timeline
  publication remain pending, as does captured flag comparison.

Flag-stage validation: 193 active Theater tests pass (11 ignored), Clippy with
warnings denied, WASM library check, formatting and diff checks pass. The flag
oracle contains 163,840 events, 51,843 openings, 18,747 object homecomings,
27,422 credited returns and 30,720 closure probes; 470 cases recognize CTF and
1,442 select neutral spawns. This is generated parity, not captured flag validation.

### Flag geometry, assignment and complete carry timeline

- `replay_flag_geometry` ports published-track identity fallback/refusals, nearest
  carrier points, position attachment and fallback counts, free-object closure,
  and resting-position correction. `replay_flag_assignment`, `replay_flag_marks`
  and `replay_flag_lives` port chronological flag assignment, independent carrier
  checks, overlap counts and all state transitions/spans.
- `build_replay_flags` assembles these with the prior chronology in native order,
  retaining raw intervals and all carry coverage/fallback counters. Return gauges
  and Film/document integration remain pending.
- The 1,024-case native oracle compares every stage and the entire carry builder:
  30,720 geometric inputs, 16,022 object closures, 8,205 attached carries, 7,504
  repositioned drops, 40,960 assignment/marker inputs, 577 recognized CTF cases
  and 5,182 end-to-end published spans. Positive captured validation is still open.
- Full assembly exposed a real equal-time return ordering mismatch in the
  ambiguity counter. Native Go sorting now applies to credited returns and object
  homecomings. Tests compare exact order, with no canonicalization hiding ties.
  The corrected end-to-end comparison passes in 12.89 seconds.

After the equal-time ordering correction: 194 active Theater tests pass, 11 ignored;
Clippy with warnings denied, WASM library compilation, formatting, generator syntax
and diff checks pass. Full parser/document parity remains incomplete.

### Return gauges and managed properties

- `replay_flag_gauges` and shared `replay_gauges` pass 1,024 native cases:
  360,509 candidate-input reads, 3,316 pairings, 16,580 gauge-bearing spans
  and 113,246 published points. Shared window/reset series, scaling through
  float32, duplicate-frame replacement and missing-source gates match.
- `managed_property` passes 512 native cases: 51,288 typed readings,
  21,808 records, 20,035 successful walks, 1,773 broken walks and 3,730 chained
  records. All 16,569 readings and scan counters match six captured films
  (2.99 seconds release, compilation excluded). This corpus validates the
  raw channel, not positive CTF gauge attribution.
- Map-aware Film retains managed-property scans or scan errors.
  `build_film_replay_flags` connects Film, published player identities/teams,
  carrier marks, objective free lives, raw gauges and independent capture bursts.
  Map objective spawn/return-rule catalogs and positive captured CTF assembly
  validation remain pending.

- `capture_bursts` ports six-tier capture signatures and the independent chunk
  clock (first FRAME skipped, wrapping timestamp subtraction). The 1,024-case
  native fixture covers duplicated/missing tiers, negative offsets and timestamp
  wrap. Film decoding retains the resulting times; flag assembly consumes them
  directly. A legacy partial-chunk decode exposed an unavailable native-prefix
  scan; integration now leaves that burst list empty instead of rejecting the
  otherwise decodable partial film. Other errors still propagate.

Final validation for this continuation: 197 active Theater tests pass, 12 ignored;
Clippy with warnings denied, WASM library compilation, formatting, generator syntax
and diff checks pass. Captured managed-property parity covers all six films. Full
CTF publication, zone layers and complete replay-document parity remain open.

### Map objectives and first zone-state stage

- The embedded schema-2 map objective catalog now resolves by map asset ID.
  `map_objectives`, `map_objective_geometry` and `replay_map_objectives` port
  role extraction, spatial ranks, missing/degenerate counters, neutral labels,
  orthonormal volumes, inclusive containment, distance, and static projection.
  The native oracle includes 128 real catalog entries plus 256 synthetic entries,
  3,481 usable zones and four projection variants per entry.
- The catalog test exposed JSON float rounding at an inclusive lower boundary:
  independently decoded center/point coordinates produced a difference slightly
  below -1 instead of exactly -1. Enabling serde_json `float_roundtrip` fixes this
  without weakening containment or its exact native assertion. This requires one
  Cargo dependency feature change outside the film module.
- `build_film_replay_flags_for_map` supplies catalog spawns to the existing Film
  flag builder. `ReplayFlagReturnZone` retains the pinned title rule and native
  publication gate; complete replay-document attachment remains pending.
- `replay_zone_series` ports speaking-slot counts, frame filtering, scalar tags,
  chained owner/designator subsets, last-read naming keys, stable chronological
  ordering and maximal nondecreasing capture ramps. Its independent native oracle
  covers 512 cases, 245,760 reads and 2,169 ramps. It does not yet attribute zones,
  publish ownership/capturer/hill intervals, or assemble the full zone layer.
- Oracle harnesses, compressed fixtures and regeneration are retained in the film
  module. Static mode-role selection belongs to the reference service; its title
  table and normalization still need an explicit integration decision.

Validation for this increment: 199 active Theater tests pass, 12 opt-in corpus
checks ignored (30.91 seconds). Clippy for library/tests with warnings denied,
WASM library compilation, formatting, generator syntax and diff checks pass. Full parser parity remains
incomplete; this run does not replace positive captured CTF/zone validation.

### Zone attribution and channel election

- `replay_zone_attribution` ports the complete pure geometric attribution stage:
  merge named/bridged player lives, stable frame ordering, nearest position with
  prior-frame ties, minimum volume distance, ambiguous-zone refusal, all counters,
  and translated negative controls. Its native oracle covers 1,024 cases and
  81,920 actions (11,632 assigned, 2,192 ambiguous); translated cases are compared
  independently and include 29,245 outside results.
- `replay_zone_pairing` ports modal gauge-slot voting, closest-peak ambiguity,
  deterministic vote ties, unpaired gauge counts, owner scoring against roster,
  the two-agreement minimum, and unique channel/zone election. The independent
  1,024-case oracle covers 81,920 capture pairs, 11,503 owner candidates, 2,643
  elected owners and 993 gauge bindings. Missing-roster scoring and constant
  neutral-channel refusal are included. Catalog reindexing and attributed-pair
  conversion are available for the upcoming full zone builder.
- This is synthetic native parity, not positive captured zone validation. Owner
  intervals, agreement coverage, capturer/gauge publication, hill states and full
  zone/document integration remain pending.

Validation: 201 active Theater tests pass, 12 opt-in corpus tests ignored
(32.66 seconds). After the owner-scoring Clippy style correction, the native
pairing test passes again; Clippy with warnings denied, WASM library compilation,
formatting, generator syntax and diff checks pass. No commits or pushes.

### Complete capture-based zone owner layer

- `replay_zone_owners` ports owner-run merging, inclusive intervals beginning at
  the first observation, unknown-run refusal without bridging the gap, peak gauge
  progress, roster validation, agreement counters, letter fallback, zone DTOs and
  complete coverage. The 1,024-case native interval oracle includes 9,451 spans
  and 9,364 rejected unknown-owner runs; it also checks roster and letter guards.
- `replay_zone_capturers` elects chained team channels using at least two complete
  ramp agreements and no disagreements. Last in-ramp reads preserve known-neutral
  versus unread/unknown values. Completed-ramp team inference remains a separately
  counted fallback.
- `build_replay_zone_owner_layer` connects election, ownership intervals, native
  live gauge thinning/reset, capture-team ramps, letter ranks and agreement
  coverage. The entire native `zoneOwnerStates` output and tally match for 1,024
  cases: 2,277 zone states, 18,969 spans, 72,864 gauge points and 8,943 counted
  outcome inferences. This exercises the complete capture-based branch, not the
  hill branch or full Film/document integration.
- Native fixtures and generator registrations are retained. Positive captured
  zone validation, hill activation/designator/ownership, top-level zone assembly
  and the complete replay document remain open.

Validation: 203 active Theater tests pass, 12 opt-in corpus tests ignored
(28.83 seconds). Clippy library/tests with warnings denied, WASM library build,
formatting, generator syntax and diff checks pass. Changes remain uncommitted.

### Complete hill branch

- `replay_zone_hills` ports designator selection and first contact, designated
  periods, ramp-window voting with full-period fallback, strict modal attribution,
  ramp-only period merging and tail closure, owner-run intersection and active
  gaps. All player points vote as in the native hill path; this path does not
  require the named-player bridge used by capture attribution.
- Hill progress is published only when an active period has a single owner span.
  No live capture gauge or letter rank is fabricated. Unknown owner values remain
  counted; known-neutral and unobserved gaps retain the reference publication.
- The 1,024-case native full-hill oracle matches 5,692 periods, 1,778 zone states,
  5,651 spans and both fallback counters. It exercises designation and position
  paths, ambiguous/overlapping geometry, sparse tracks and invalid final clocks.
  This is generated parity; positive captured hill validation remains open.
- Native harness and reproducible compressed fixture are registered. Top-level
  zone dispatch, Film wiring and full replay-document assembly remain pending.

Validation: 204 active Theater tests pass, 12 opt-in corpus tests ignored
(29.62 seconds). Clippy with warnings denied, WASM library compilation,
formatting, generator syntax and diff checks pass. Changes remain uncommitted.

### Top-level zone dispatch and Film wiring

- `build_replay_zones` now connects the complete capture and hill branches with
  native scan/catalog/clock gates, served-catalog reindexing, capture/secure-only
  attribution, explicit hill fallback, coverage and inference counters.
- The independent 1,024-case native dispatcher fixture includes 817 cases taking
  the capture method (including early refusals), 146 position-based hill cases,
  61 disabled cases and 1,504 published states. Detailed designator behavior is
  additionally covered by the preceding full-hill oracle.
- `build_film_replay_zones` uses decoded managed readings, published objective
  actions/tracks and the purified player bridge. Caller-supplied held-zone order,
  mode and external roster remain explicit, matching reference ownership.
- Inspection of `film_scan.go:463` corrected a prior flag-gauge integration
  mismatch: consumer scan flags are set from mode/catalog enablement, even if the
  shared scan fails. Film zone scanned status now follows catalog enablement;
  recognized Film flag gauges likewise report scanned with an empty read list on
  failure. Film still retains the underlying managed scan error separately.
- Static held-role service selection, positive captured zone/CTF assembly and the
  complete replay document remain open. Synthetic dispatch parity is not evidence
  of those remaining requirements.

Validation: 205 active Theater tests pass, 12 opt-in corpus tests ignored
(29.61 seconds). Clippy with warnings denied, WASM library compilation,
formatting, generator syntax and diff checks pass. Still on
`simbleau/theater-experiments`; changes remain uncommitted.

Next integration constraint verified in `internal/replaybuild/zones.go`: native
held-zone service selection admits only `strongholds_zone` and `hill`, preserving
mode-table order. Other static roles are not enabled merely because they have
volumes. Missing map IDs/catalog entries or no resulting volumes return no roles.

### Title mode selection and map-aware zone assembly

- `replay_objective_modes` embeds the pinned, native-loaded eight-entry title role
  table. Raw variant selection preserves first-role precedence and implements the
  shared Go matcher: simple lowercase, ASCII word boundaries (underscore included),
  longest UTF-8 byte-length token, first equal-length match. Overlapping matches
  remain eligible. The 876-case native oracle checks prefixes, suffixes, casing,
  Unicode boundary bytes and mixed mode labels.
- `replay_match_objectives` joins map asset ID and raw variant to static objectives,
  flag spawns, held zones and hill recognition. Only Strongholds/hill roles enable
  dynamic zones; other static volumes do not. The 896-case native catalog join
  oracle compares static projection, held-object ordering and role strings across
  all 128 catalog entries. Missing/empty IDs retain empty catalog results.
- `build_film_replay_zones_for_map` connects this selection to the existing Film
  zone assembler. Raw game-variant names are deliberate: reference replaybuild
  matches them directly. HTTP pair-name normalization is separate service behavior.
- Captured positive zone/CTF validation and complete replay-document assembly
  remain outstanding. This addition does not claim them complete.

Validation: 207 active Theater tests pass, 12 opt-in corpus tests ignored
(29.33 seconds). Clippy with warnings denied, WASM library compilation,
formatting, generator syntax and diff checks pass. Changes remain uncommitted.

### Document content schema and assembly audit

- `ReplayDocumentContent` types every top-level native document field except the
  coverage envelope. It is deliberately not advertised as a complete Film
  constructor. Required tracks retain null versus empty-array state; optional
  fields retain native omission behavior.
- `replay_document_types` adds missing surface/map geometry, weapon and vehicle
  labels, neutral deaths, static map pad/tier DTOs and stance DTOs. Native geometry
  bounds use centers only; surface bounds use the supplied rectangle endpoints,
  not polygon extent or altitude.
- The native reflection harness tests empty content, every field individually,
  all fields together and 128 geometry/structure bound cases (190 total). Rust
  reserialization preserves all keys and values. JSON integer/float spelling is
  normalized only for this small, exactly representable fixture data (0 vs 0.0).
  The generic harness uses valid `taken` kinds for typed weapon/equipment changes.
- Assembly audit found unfinished behavior beyond the final constructor: stance
  publication, coverage envelope/aggregation, and some presentation/static-pad
  builders. Their DTO availability does not establish their behavior is ported.
  These remain required alongside captured end-to-end validation.

Validation: 208 active Theater tests pass, 12 opt-in corpus tests ignored
(27.90 seconds). Clippy with warnings denied, WASM library compilation,
formatting, generator syntax and diff checks pass. Changes remain uncommitted.

### Movement stance publication

- `replay_stances` now groups by published slot and movement kind, stably sorts
  readings, folds them into native life/death-bounded intervals and publishes all
  coverage counters, including dropped/unlocated reads and derived jumps.
- It reuses the equipment episode accumulator rather than duplicating its gap and
  death semantics. Internal helpers now have unique module names to avoid glob
  import ambiguity; the accumulator accepts a borrowed family name for stances.
- `build_film_replay_stances` connects the decoded movement stream and published
  player's death-closed tracks. No stream retains the native unavailable coverage.
- The independent 1,024-case native oracle matches 100,671 readings and 5,960
  published intervals, covering pre-origin time, equal timestamps, overlapping
  windows, death closures, unknown slots, scan status and clock guards. This remains
  generated validation; captured stance output is not yet checked.
- Coverage-envelope aggregation, presentation/static-pad builders, complete Film
  document construction and captured end-to-end parity remain required.

Validation: 209 active Theater tests pass, 12 opt-in corpus tests ignored
(30.44 seconds), including shared equipment-episode regressions. Clippy with
warnings denied, WASM library compilation, formatting, generator syntax and diff
checks pass. Changes remain uncommitted.

### Complete document envelope, base coverage and kickoff dating

- `ReplayDocument` now retains the complete native schema including every coverage
  member. The native reflection oracle validates empty, individual-field and full
  documents; the schema test passes. This does not supply the full Film constructor.
- `ReplayCoverage::new` composes layer/bridge verdicts and base counters. Layer
  publication checks preserve native precedence and the exact 0.66 threshold.
  Final fallback publication combines positive counts by name, sorts names and
  replaces the report; absent document coverage stays absent.
- `detect_replay_t0` ports the contiguous movement window, teleport/replication-gap
  resets, named-player burst deduplication and explicit refusal reasons. All 1,024
  native generated cases match (6,129 tracks, 241 successful detections). Document
  integration runs only with an established origin, including a valid zero origin.
- Decoder provenance publishes the pinned reference revision strings, known build
  keys and registry status. Its 577-case native oracle covers absent identity,
  known/unknown format and build keys, zero/known/unknown fingerprints and registry
  counts. Older fingerprint entries are diagnostic classification only; decoder
  support remains scoped to v41. The Film helper reuses decoded identity/registry.
- Full coverage integration, presentation/static-pad builders, complete Film
  document construction and captured end-to-end comparison remain required.

Validation so far: 213 active Theater tests passed (12 opt-in tests ignored,
29.93 seconds) before the decoder-provenance addition; its new targeted native
parity test also passes. Clippy with warnings denied, WASM compilation, formatting,
generator syntax and diff checks pass. Changes remain uncommitted.

### Static map-pad catalog and confirmation

- `replay_map_weapon_pads` embeds the pinned schema-1 catalog (76 maps, 1,549
  spots), including raw type IDs, family, provenance and separate spawn-point
  declarations. Missing spawn-point knowledge remains distinct from an established
  empty list. Byte-based loading validates schema; map-ID lookup and overlay merge
  preserve versioned entries over runtime additions.
- `build_replay_map_weapon_pads` confirms in catalog order using full 3D distance
  strictly below one metre. Each film pad is consumed once; equal distances retain
  the first film pad. Published positions/families come from the catalog, while the
  pad index links presence and timing back to the film. No confirmation means no
  map-pad layer. `replay_map_weapon_pads` provides the map-ID join.
- All 1,328 native comparisons pass: every catalog map at four displacements plus
  1,024 generated overlap/threshold cases, 15,395 candidate spots and 9,161 published
  confirmations. Established-empty spawn knowledge is checked synthetically; the
  pinned catalog has no such entry. This is not a captured-film validation claim.
- Label catalog construction/publication, weapon-tier mode metadata, remaining
  assembly integration and captured end-to-end parity are still required.

Validation: 216 active Theater tests pass, 12 opt-in tests ignored (33.98 seconds).
Clippy with warnings denied and WASM compilation pass; only test assertions changed
since those checks. Formatting, generator syntax and diff checks pass. Changes
remain uncommitted on the requested branch.

### Injected label catalog and weapon-label publication

- `ReplayLabelCatalog` now carries all native injected title metadata. Its constructor
  joins weapon family to key to bilingual name/effect, retains unnamed family keys,
  and owns copies of key/effect maps. Empty naming catalogs remain valid.
- `replay_weapon_family` preserves native permissive `%X` parsing, lowercase-prefix
  stripping and the original UTF-8 byte-length decision for high-32 extraction.
  All 1,744 native cases pass, including overflow, whitespace, signs, trailing text,
  uppercase prefixes, underscores and generated global/family IDs.
- `build_replay_weapon_labels` consumes loadouts, shots and pads, emitting only known
  families and applying icon overrides. `ReplayDocument::complete_weapon_labels`
  adds newly known labels while preserving existing artifact labels. All 512 native
  catalog/publication/completion cases pass.
- Neutral-death filtering, layer-status publication, weapon-tier mode metadata,
  complete document construction and captured end-to-end parity remain required.

Validation: targeted native label/family tests pass; Clippy with warnings denied,
WASM compilation, formatting, generator syntax and diff checks pass. Full Theater
regression passes: 218 active tests, 12 opt-in ignored (31.85 seconds). Changes
remain uncommitted on the requested branch.

### Final layer provenance and neutral-death publication

- `replay_layers` ports the native per-field producer revision table, revision-family
  helpers and all production guards. It uses frame count, not track count, for the
  publication-only empty-film branch. Empty layer data remains distinct from a pass
  that never ran. `ReplayDocument::publish_layers` is the final publication hook.
- The native oracle exercises all 8,192 guard combinations and four zero/negative
  frame-count cases. Map context, nonnil inventory, scan status, score presence and
  resolved origin are independent inputs; they are not inferred from output lengths.
- `retain_replay_neutral_deaths` requires a known kind and a published player, using
  the shared track-XUID/slot-bridge helper. It preserves event order and existing
  image metadata. The 256-case native oracle includes named/unnamed tracks, bridge
  conflicts/zero identities, missing kinds and players without published tracks.
- Complete document construction, aggregate fallback wiring, remaining title
  presentation integration and captured end-to-end comparison remain required.

Validation: 220 active Theater tests pass, 12 opt-in tests ignored (38.68 seconds),
including all layer/neutral-death oracle cases. Clippy with warnings denied, WASM
compilation, formatting, generator syntax and diff checks pass. Changes remain
uncommitted on the requested branch.

### Film-to-document constructor (validation in progress)

- `build_film_replay_document` now connects player/identity, combat/vehicle recovery,
  objective actions/score, equipment episodes/stances, coverage/kickoff, grapple,
  placements/pickups/pads/ground items, live objectives, inventory, abilities,
  equipment changes/translocations, final fallback report and layer provenance.
- Inputs retain caller-owned mode gates, optional score/kill evidence, title labels,
  geometry, neutral deaths, spawn-point knowledge, zone teams and upstream counters.
  Empty position streams return the native publication-only document. Missing map-
  calibrated decode context stays an explicit error.
- A new native full-document harness covers the populated Bazaar capture (71 frames)
  and empty Aquarius capture. The complete-document comparison passed both (1.96
  seconds in release after compilation). Broader assembly parity remains unproven.
- Equipment/flag/free-objective helpers now accept the supplied catalog; the
  constructor uses it consistently. Both captured documents still match after this
  change (22.18 seconds in debug). The native harness is being expanded to six films.
- Required follow-up: audit all publication/decoder fallback counters; broaden
  captured checks beyond the first populated and empty examples; verify positive
  objective modes and the complete source inventory.

Constructor validation checkpoint: 220 active Theater tests pass, 13 opt-in tests
ignored (32.56 seconds). Clippy with warnings denied, WASM, formatting, generator
syntax and diff checks pass. The two captured full-document comparisons were run
explicitly and passed; the expanded six-film Go oracle has completed (67.55 seconds). The six-film Rust
comparison passed all cases in 262.10 seconds. This includes 11,947 Oddball frames,
6,138 Bandit frames, Bazaar, empty Aquarius, and two appearance clips. Mode gates
remain false and external kill evidence absent in this oracle; positive objective
mode publication and the kill-source pipeline are not covered by this result.

### Remaining kill-source pipeline identified by source audit

The inventory still marks the native `internal/facts/killsource` pipeline pending.
Rust has highlight/death streams, dead-state primitives and replay consumers of
caller-supplied kill evidence, but no counterpart of the complete native `Decode`
API or its walk/scan/hybrid credit-and-fatal-source publication was found. This is
required for the original all-data parser goal: the constructor's optional kill
inputs are not a substitute for decoding them. The next source audit/port must
cover that orchestration, roster/bijection, kill-feed couples, assists, calibration,
health and per-path provenance, reusing already-ported grammar where applicable.

### Kill-event grammar, film scan, and assist attachment

- `kill_event_chain.rs` ports native event presence configuration, supported bodies,
  mandatory code-85 fields and chain validation. The bounded shared bit cursor
  retains the native sticky overflow behavior. Unknown bodies stop the chain;
  localization requires three subsequent events and probes at most twelve.
  All 5,999 native oracle cases pass, including all event codes, truncation,
  absent references and integer damage shares above 100.
- `kill_assists.rs` ports ordered attachment with packet-identity precedence,
  inclusive 2,500ms fallback, exact resolved player couples, selected-event-only
  consumption, presence disagreement, extra assistants, rejection categories,
  raw damage shares and every native assist counter. All 1,024 native cases pass.
  These are post-resolution inputs, not a replacement for roster/bijection.
- `kill_event_scan.rs` connects readable film chunks to native timestamp ordering,
  runtime gate selection, event scanning and packet counters. The new native
  oracle exercises 128 films with tied timestamps and both gate configurations;
  all 128 Rust comparisons pass.
- These APIs expose localized evidence and attachment, not complete kill-source
  results. Walk/scan/hybrid orchestration, fatal damage tags, roster/bijection,
  feed-couple reconstruction, calibration, health and publication gates remain
  required. No completion claim is made for the complete kill-source pipeline.

- `kill_feed.rs` adds chronological kill-feed construction and the kill-source
  loader's most-kills chunk selection (first on ties), using existing v41 highlight
  parsing. This intentionally differs from the replay last-chunk helper. The
  512-case native builder fixture covers timestamp collisions, duplicate counts,
  name changes, XUID fallback names, and UTF-8 roster ordering. All 512 cases pass.

Next kill-source dependency: native `feed_couples.go` resolves same-instant,
kill-event-read, neighboring fallback and bot couples using pinned roster names
before the inferred bijection. Port this with roster pinning rather than assigning
published credit directly from raw code-85 indices. Keep full pipeline status
incomplete until preparation, scan/walk/hybrid passes and final health gates join.

Validation checkpoint: 224 active Theater tests pass, 13 opt-in tests ignored
(34.50 seconds). All four new native fixtures pass. Clippy with warnings denied,
WASM compilation, formatting, generator syntax and diff checks pass. Six captured
full-document comparisons passed separately. All changes remain uncommitted on
`simbleau/theater-experiments`; the complete all-data parser goal remains active.

### Kill-source pair, roster, bijection, and direct source-scan port

- `kill_feed_pairs.rs` reconstructs same-instant, read, neighboring fallback and
  bot pairs from directly pinned roster evidence. All 2,048 native cases pass,
  including ambiguous victims, packet reservation, death reuse and orphan outputs.
- `kill_roster.rs` preserves bot/table/motif precedence, bot succession, duplicate
  names, refusal/conflict counters, unknown placeholders and index provenance.
  All 1,024 native roster cases pass, including resolved-name and free-slot queries.
- `kill_bijection.rs` ports Hungarian initialization, pair scoring, fixed-seed
  restarts, lexicographic tie-breaking, seat-vote checks and single-swap margin.
  All 512 native solver cases and 256 consecutive native permutations pass. Go's
  RNG algorithm/constants retain its BSD license; no platform random source is used.
- `kill_damage_tags.rs` embeds all 468 native damage-effect IDs and labels with
  dates, classification, ambiguity and reserves. `kill_source_scan.rs` ports strict
  and relaxed 58-bit dead-state scans, packet-level deduplication and multiplicity.
  All 2,048 native scan cases and every catalog/unknown-tag/category case pass.
- Corrected film wrappers after auditing `source.AllPackets`: killsource consumes
  ALL supplied chunks and records source-position chunk IDs, not metadata indices.
  Only the XUID motif scan uses the contiguous numbered prefix. Previous wrapper
  documentation and implementation had incorrectly reused the replay prefix rule.
- `prepare_kill_film_evidence` links native feed loading, table/motif reads, bot
  metadata, roster pins, kill events, pair reconstruction, source candidates and
  bijection. This is preparation only, not final attributed kills. The captured
  four-film oracle passed in Rust (94.60 seconds; native 3.52 seconds). Bandit has 131 events/134 candidates/122
  pairs; Oddball has 263 events/268 candidates/248 pairs; two idle films refuse
  absent kill feeds. The comparison covers table/motif/pin values, permutations,
  scores/margins, every event/candidate/pair, and pair/packet counters. Both active
  captures are fully pinned (score and margin zero); randomized native solver
  cases separately cover inferred permutations and positive/zero margins.

Remaining: calibrated record-walk timeline and dead-state extraction, walk/scan/
hybrid source attribution, unclaimed/bot deaths, relaxed probe, health and final
publication gates, plus positive objective-mode end-to-end coverage and full
source-inventory audit. The original all-data completion criterion is unchanged.

Validation checkpoint: 228 active Theater tests pass, 14 opt-in tests ignored
(33.95 seconds). The captured kill-evidence test was explicitly run and passed.
The additional source-position regression passes (229 active tests verified in
total). Final Clippy with warnings denied, WASM compilation, formatting, generator
syntax and diff checks pass. Changes remain uncommitted; the goal is not complete.
Next native files to port/reconcile with existing march grammar: `world.go`,
`walk.go`, and `calibrate.go`, then `match.go`, `hybrid.go`, `health.go` and `decode.go`.

### Kill-source timeline, record walk and calibration

- `kill_timeline.rs` reuses guarded keyframe recovery and adds the native biped
  sweep, first-slot preload, timestamp ordering, allocation-gap wildcard windows,
  and rewind lifecycle. All 256 native cases pass. The oracle exposed native map
  aliasing after `World.Restore`: advancing can mutate the initial snapshot until
  packet rollback detaches it. The Rust timeline explicitly retains that behavior.
- The shared march decoder now supports an explicit view count and simulation-
  state completion gate. Existing callers preserve their previous eight-view,
  complete-simulation behavior. Kill-source reads the simulation body but stops
  the record when the map-dependent completion gate is false.
- `kill_walk.rs` adds chronological packet walks, clean dead-state extraction,
  credibility filtering, per-component bit offsets, deduplicated candidates and
  missing-bit counts. Native source-position identities and snapshot rollback
  are retained.
- `kill_calibration.rs` ports source-order sampling (400 eventless packets of at
  least 400 bytes), axis-width diagnosis (6..26), handle selection (1..3), map/film
  evidence, flat-score refusal and the exact calibration report. Axis diagnosis
  never overwrites map widths. All 768 native policy/width cases pass, including
  0..10 views and the simulation gate. Native delete records have no Trace.EndBit;
  tests compare the physical end on other record kinds, as existing march tests do.
- The native captured oracle completed in 13.45 seconds. Bandit: 121 deaths,
  114 credible/candidates, 5,092/5,241 event packets localized. Oddball: 108 deaths,
  86 credible/candidates, 4,332/9,888 localized. The Rust release comparison passed both captures in 176.81 seconds, matching
  every record, candidate, counter and calibration report. Log:
  `/private/tmp/halo-kill-walk-rust.log`.
- Required next stage: `match.go` and `hybrid.go` credited/fatal-source assembly,
  bot/unclaimed deaths, `health.go`, relaxed probe and the final `Decode` API.
  Full parser parity, positive objective-mode captures and inventory closure are
  still incomplete; no completion claim is made for the all-data goal.

### Kill-source attribution and final decoder (captured validation passed)

- Both Bandit and Oddball complete record-walk comparisons passed in 176.81s.
  The previous checkpoint also passed 231 active Theater tests (15 opt-in ignored),
  Clippy, WASM, formatting and generator syntax checks.
- `kill_matching.rs` and `kill_hybrid.rs` port identity-first matching, walk-first
  attribution, exact feed credit versus fatal source, self-source divergence, bot
  victims/killers, closest-candidate unclaimed deaths, ghost pairs and all pass
  counters. The expanded 1,024-case native oracle passed, including explicit
  bot-read inputs, option variations, timestamp collisions and native walk sorting.
- `kill_health.rs` adds native alerts/degradations, publication gate, metrics,
  coverage and relaxed-candidate diagnostics. `kill_decode.rs` connects evidence,
  calibration, record walking, hybrid attribution, assists, health and conditional
  relaxed probing. `scan_film_relaxed_kill_sources` preserves undeduplicated hits.
- Native complete Decode oracle finished in 15.47s: Bandit 122/122 kills covered;
  Oddball 247/248, with 378,658 relaxed candidates, 4,445 paired out-of-catalogue
  candidates and three hits for the uncovered pair. Both line gates are open.
  Rust full Decode comparison passed both captures in 182.81s
  (`/private/tmp/halo-kill-decode-rust.log`), including every relaxed-probe field.
- Event localization now receives the simulation-completion gate used by the
  record walk. An expanded native 768-case policy oracle exercises slot-123
  localization: 384 accepted with completion enabled, 258 with it disabled.
- Required next: wire decoded kill results into all-data
  Film/replay construction. Native replay construction inherits the calibrated
  kill profile (or the explicit starting profile on refusal); integration must
  retain this ordering instead of decoding kills after an unrelated Film profile. The
  current caller-supplied replay kill evidence is still not automatic integration.
  Positive objective-mode captures, extended raid assembly, fallback audit and
  final inventory closure also remain required. The goal is not complete.

Final active checks after the localization correction: 232 Theater tests pass,
16 opt-in tests ignored (33.90s). Library/test Clippy with warnings denied, WASM
compilation, formatting, generator syntax and diff checks pass. The complete
captured Decode test passed Bandit and Oddball in 182.81 seconds. This test
compares every kill/source/assist/damage field, unclaimed death, coverage/health/
pass counter, calibration report, bijection gate and relaxed probe. Roster/profile
details retain their separate captured oracles.

Integration audit pointers: `replication.rs::try_from_chunks_with_map_context`
currently constructs independent march/replication profiles. Native
`internal/replaybuild/kills.go` defines profile inheritance, `killRefs`, XUID-prefix
identity resolution and per-path coverage; `replaybuild.go::neutralDeaths` applies
the shared line gate and damage-tag icons. Preserve read-versus-empty state and
share a single kill decode for all consumers.

### Integrated kill-source Film/replay path (six-film validation passed)

- `replay_kill_inputs.rs` ports the native offline name/XUID bridge, first-name
  precedence, strict decimal `xuid:` parsing, unresolved killer/victim counters,
  equipment and match-pair references, neutral-death icons and shared publication
  gates. All 512 native reference-projection cases and all catalogued plus unknown
  neutral icons pass. Missing death reads close kill inputs without erasing the
  independent XUID source for neutral deaths.
- `Film::try_from_chunks_with_map_and_kill_sources` runs kill decoding once, retains
  the full result or refusal in portable Film fields, and passes its encoding to
  map-aware replication, component readers and march facts. The older primitive
  map constructors remain available without that additional pass. The shared
  march calibration/scan now accepts simulation completion as well as generation
  policy; old callers retain their prior policy.
- `build_film_replay_document` automatically consumes the retained kill source,
  shares the player builder's death read, and supplies equipment kills, match kill
  pairs, neutral deaths and path coverage under native gates. Explicit legacy
  inputs are used only when the Film never attempted kill decoding.
- New native integrated-document oracle covers all six prior captures with the
  inherited kill profile or explicit refusal fallback, and enables Oddball mode
  using the corpus manifest's gameplay description. The generated reference is
  complete (80.81s); it publishes 38 closed Oddball carries and one carrier-absence
  rejection. Rust integrated document comparison passed all six captures in
  460.89s, including the complete positive Oddball document. Log:
  `/private/tmp/halo-decoded-kill-document-rust.log`. This covers the six-film
  fixture, not every remaining mode or the extended raid.

### Objective source-inventory gaps found and addressed

- A function-level audit of `internal/facts/objectives` found previously missing
  `flag_grabs_net.go` and `awards.go`, rather than merely stale inventory labels.
- `flag_grabs_net.rs` now ports per-object/per-player net grabs, stable span order,
  inclusive home-window intersections, unknown/unnamed spans, open carries and
  native positive sub-millisecond windows. All 2,048 native cases pass.
- `statborg_awards.rs` ports personal-score labelling, positive quota admission
  without quota consumption, exact-unit precedence, unique two/three-part sums,
  explicit ambiguity, categories, summaries and descriptions. The 2,048-case native
  oracle passes all events, summaries and descriptions. These pure helpers retain
  caller-owned quotas/window inputs as in the native contract.
- Inventory audit remains open, including `objectives/extract.go` and the full
  fallback registry. Matching replay documents does not establish these APIs.

Validation checkpoint: 235 active Theater tests passed after integration and the
two objective helper ports (17 opt-in ignored, 34.49s). Clippy with warnings denied,
WASM compilation, formatting, generator syntax and diff checks pass. A newly added
active raw-native field-preservation audit also passed (2.91s): all keys, nulls,
array elements and nonnumeric scalar values survive across all twelve baseline/
integrated documents; numeric values remain checked by the constructor oracles.
This prevents typed fixture deserialization from hiding fields absent from Rust.
The six-film integrated constructor comparison passed all six cases in 460.89s.
Its session is terminal; no validation jobs remain live. The new schema audit
brings verified active test coverage to 236 tests (235-suite run plus that test).
Final test Clippy, WASM compilation, formatting, generator syntax and diff checks
pass. Changes remain uncommitted on `simbleau/theater-experiments`.

Remaining completion requirements: audit the full native source inventory and
close any unported APIs (`objectives/extract.go` is not established complete),
finish fallback counter wiring/audit, validate other positive objective modes
(CTF, VIP, bomb) and extended raid assembly, and verify calibrated-profile
inheritance beyond the captured invariant handle widths. The original all-data
v41 goal remains active; this checkpoint is not a full-parser completion claim.


### Objective footer and extraction parity

- `objective_extract.rs` ports unaligned footer XUID scanning, bounded first-marker
  selection, all raw footer fields, highest-index manifest footer choice, objective
  event extraction, scorer windows, team controls and stable event finalization.
  Native film team bytes remain authoritative; roster disagreements are counted.
- `Film.objective_footer` retains raw evidence with backward-compatible serde
  defaults. `Film::objective_events` uses retained evidence and capture bursts;
  the chunk-based extractor explicitly rejects non-v41 versions.
- Corrected `scan_capture_bursts` to consume all metadata-described gameplay chunks,
  including noncontiguous indices, matching native `manifestChunks` selection.
- All 1,024 synthetic oracle cases pass, including malformed blocks, shifted bits,
  scorer ties/window boundaries, missing scorers, footer selection, source order,
  wrapped clocks and complete event/team fields. The native harness is registered
  in `generate_oracles.py` with its compressed fixture.
- All 32 captured-film objective comparisons pass (17.83s together with synthetic
  cases): 159 footer interactions and four bursts. This validates this extractor,
  not complete positive CTF/VIP/bomb replay assembly. Captured tests remain opt-in.
- Full fallback registry/counter audit, remaining source inventory, positive mode
  documents, extended raid assembly and calibrated-profile inheritance remain open.


### Fallback registry audit started

- Added `fallback.rs` and the complete pinned 99-entry/seven-family catalog, with
  names, facts, mechanisms, conditions, order, source anchors, removal criteria,
  dates and reference counter-wiring metadata. Reference wiring flags are labelled
  explicitly and are not claims about instrumentation in Rust.
- Added sorted table/lookup, effective site conditions, source-package projection,
  before-read count, and per-decode mutex-protected counters, accumulation, reports
  and text output. Nonpositive hits are ignored; native signed wrapping is retained.
  Unknown names remain visible in reports, and direct trigger calls report them
  with a false return value instead of native process-global logging.
- Native catalog and 512 counter cases are captured by the registered generator.
  Structural registry validation and call-site wiring remain pending.
- Before this catalog addition, the entire active Theater suite passed: 238 tests,
  18 ignored, 30.40s. Objective corpus comparison passed all 32 films. Objective
  changes also passed Clippy, WASM compilation, format and diff checks.

Fallback catalog/counter native comparison passed (512 cases); library/test Clippy
passed. `fallback-wiring-audit.json` records 18 reference-wired entries; five lack
a literal Rust match and require semantic inspection (gesture last occupant,
gesture first slot life, negative equipment hold, default grenade ceiling and
default axis widths). This is a candidate list, not a missing-behavior verdict.

Final checkpoint: WASM compilation, library/test Clippy, formatting, generator
syntax and diff checks pass. All validation processes are terminal. Verified
active tests total 239 (238-test suite plus new fallback oracle); objective corpus
comparison passed separately. Changes remain uncommitted on the experiment branch.
The full-parser goal remains active and incomplete.


### Usage projection and scan-time fallback audit

- `replay_usage.rs` ports native `BuildUsageSummary` (revision us6): per-player
  grapple/episode durations and kills, deployments/drops, grenades, normalized
  weapon-pad families and pickups, equipment taken/spent/kept outcomes, and all
  match diagnostics. Nullable maps remain nullable; named zero-count rows remain.
- Ownership respects roster/player ordering, stable chronological lives, bots and
  unnamed life occupancy, the latest owner fallback and first-life fallback. All
  three usage-only fallback counters publish in the summary, after assembly.
- The native oracle matches 1,024 synthetic documents and all twelve captured
  baseline/integrated documents, including every summary field and fallback.
  `generate_oracles.py` runs this after its document prerequisites.
- The scan-time fallback audit found that existing native full-document harnesses
  used nil counters during scanning. They could not validate scan-time fallback
  publication. Both harnesses now construct the counter before scanning. Rust now
  publishes the existing `default_grenade_max` diagnostic under the native name.
  Corrected native reference generation and subsequent captured comparisons are
  required before claiming that added diagnostic's full-document parity.
- The fifth candidate, default world-object axis widths, remains open alongside
  calibrated-profile inheritance. Structural fallback-registry validation remains
  open. The full parser is not complete.


Validation checkpoint for usage projection: 240 active Theater tests pass (18
opt-in ignored, 37.08s). Library/test Clippy, WASM, formatting, generator syntax
and diff checks pass. All 1,036 usage summaries match native output.

Scan-counter validation is NOT closed. A focused native Bazaar probe confirms one
`repli_plafond_grenade_par_defaut` hit before building and in document coverage.
The first broad generation finished in 155.377s but its output unexpectedly matched
the old no-scan-counter fixtures. Do not treat it as verification. Both permanent
harnesses now assert exactly one grenade fallback immediately after scanning.
The asserted native rerun is live as session 45135, log
`/private/tmp/halo-document-fallback-native-asserted.log`. On completion, inspect
and compress its actual outputs into the two document fixtures. Regenerate the
usage fixture after document refresh (copy native outputs to its two input paths).

Rust integrated comparison session 83402 is live, log
`/private/tmp/halo-document-fallback-rust.log`; it compiled the currently old
counter fixtures before the asserted native rerun. Its result cannot validate the
new references. Observe it to terminal, then rerun against verified regenerated
fixtures. No inference about native source behavior should be made from the
unexplained earlier output; the focused probe is the positive evidence so far.

The five-name audit has progressed: three usage counters are fully ported/tested;
grenade default publication is implemented but captured revalidation is pending;
default world-axis widths remain unresolved. No commit or push was made.


### Asserted scan-counter references and flag bridge

- The asserted native full-document rerun passed (148.722s), proving exactly one
  default-grenade-cap fallback during each film scan. The regenerated baseline and
  integrated fixtures now carry this fallback on all five documents with coverage;
  Aquarius has no published coverage and stays unchanged. The previously unexplained
  broad output is superseded by these asserted outputs. All document changes are
  confined to coverage. Rust publication uses the already-retained inventory flag.
- `replay_flag_grab_tracks` ports the missing document bridge to net-grab inputs:
  unavailable coverage/mode/time gates, ordered spans and teams, nullable XUIDs,
  wrapping frame-to-millisecond conversion and the opening denominator. All 512
  native cases pass. FlagGrabTrack.team now retains a full native signed integer.
- The calibrated-profile audit confirms native installs the inherited profile FIRST,
  then applies map world-object precision, preserving unrelated calibrated movement
  fields. Rust still needs a targeted audit of consumers reconstructing map defaults
  and of zero-width world-object fallback; do not close this requirement on the
  basis of current captured invariant handle widths.


The structural fallback-registry validator is now ported and passes 512 native
malformed-declaration cases in addition to the existing 512 counter cases. It
checks duplicate/conforming names, required fields, syntactic dates, condition and
order domains, complete sites, condition overrides and counter-wiring metadata.
Diagnostic ordering is sorted because native required-field checks use map order.
The refreshed usage oracle still passes all 1,036 documents after scan-counter
reference regeneration. The 512-case flag bridge passes as well.

Rust integrated comparison session 83402 is still live and still uses its compiled
old reference fixture. The asserted native process 45135 and usage regeneration
45313 are terminal and passed. Fixture files on disk are now corrected; do not
restart 83402 solely because it has not returned. It must reach terminal before a
fresh comparison is launched, or its fresh per-film outputs may be reconciled to
the corrected native references with full typed field/value comparison.

Checkpoint checks: library/test Clippy and WASM compilation pass after the
validator and bridge changes, as do formatting, generator syntax and diff checks.
Verified active coverage is 241 tests (240-suite checkpoint plus flag bridge);
the expanded fallback test and refreshed usage test also pass. Only the older
integrated full-document run remains live (83402); all other sessions are terminal.


### Calibrated equipment-recovery component widths

- Found and fixed a real inheritance gap: equipment recovery rebuilt a default map
  PositionEncoding even when strict ability scanning received the inherited
  calibration. Recovery now accepts the same explicit PositionEncoding through
  `scan_equipment_changes_with_position`; the Film map constructor passes its
  component profile. The older wrapper preserves explicit default-map callers.
- Added 512 native recovery cases with nondefault delta widths and full-precision
  gates ahead of the ability component. All match. More than 100 deliberately
  differ from the default-map path, proving the oracle exercises the fixed gap.
- The native oracle is registered and its compressed fixture retained. Other map-
  reconstructed readers and default-world-axis fallback still require audit.
- The older full-document session 83402 remains live; its compiled fixture lacks
  the newly asserted native scan counters. Current on-disk native document fixtures
  are correct and include the grenade fallback. Do not restart the live run solely
  because it has not returned; observe it to terminal before a fresh comparison.

The three equipment-recovery tests pass (including 512 calibrated cases and the
existing sparse/dense/truncated and counter-window oracles). Library/test Clippy,
WASM, formatting, generator syntax and diff checks pass. Verified active coverage
is 242 tests across the suite checkpoint and added tests. Only full-document
session 83402 remains live. Changes remain uncommitted; the goal stays incomplete.


### Explicit precision through creation readers

- Added explicit-position variants for vehicle and ground-weapon creation decoding
  and scanning, and for `scan_film_vehicle_facts`. CreationWalk now uses the
  provided profile for vehicle default-state fields and component-walk ammunition.
  Existing wrappers retain map defaults; Film passes its inherited component
  precision through both creation routes.
- The native vehicle oracle has 2,048 additional cases with media-frame precision
  deliberately different from the map's creation-position widths. All records and
  diagnostics match, with more than 100 differences from the old default-map path.
  All four creation tests pass, including existing equipment and ground-weapon
  cases. This is explicit profile routing, not a claim that every profile consumer
  is now audited; missing world-axis fallback and remaining inherited flags remain.
- Clippy with warnings denied and WASM compilation pass; formatting, generator
  syntax and diff checks pass. Verified active test coverage totals 243 across
  checkpoints and additions.
- Full-document test session 83402 remains live. A process inspection identified
  its test PID as 99933, actively using about 95% CPU with over 12 minutes of CPU
  time, so it was not restarted. It still has the old compiled counter fixtures;
  the corrected on-disk fixtures await reconciliation/fresh full comparison.
  All other current validation sessions are terminal. Changes remain uncommitted.


### Event-locator speculative decoding hot path

- A one-second process sample of the long-running test located the work in march
  calibration -> event locator -> trial_delta -> multiplayer-property TLV decoding.
  The locator decoded all candidate entity slots before applying its mandatory
  slot-123 selection predicate. Trial TLV payloads could therefore be large even
  when the candidate could never be selected.
- `trial_delta` now checks the decoded header's slot before component decoding.
  Both locator passes already required that same slot after decoding; no candidate
  selection or published field changes. The native locator oracle passes.
- After identifying and fixing this work, the old process (PID 99933 / session
  83402) was explicitly interrupted and observed terminal (signal exit, not a parity
  verdict). It was NOT restarted merely on an observation timeout.
- A fresh integrated comparison is running as session 28687, with corrected native
  counter fixtures, explicit creation/recovery profiles, and the locator prefilter.
  Log: `/private/tmp/halo-document-fallback-rust-corrected.log`. It prints each film
  as it completes; Bazaar and Aquarius already match. Continue this same handle.
- Full active suite passes: 243 tests, 18 ignored, 34.35s. Library/test Clippy passes.
  The goal remains incomplete; positive modes, extended raid, full source inventory
  and remaining profile/default-axis behavior still need their required audits.

WASM compilation, formatting, generator syntax and diff checks also pass. Only
corrected integrated document session 28687 remains live; other current checks
are terminal. Changes remain uncommitted on simbleau/theater-experiments.


### Weapon-hit pairing and distance resolver

- Added public shot, damage, statistics and position-track types, native pairing,
  distance buckets, nearest-sample distance, and slot-base selection in
  `weapon_hits.rs`. Negative/missing responsible references are excluded; healing
  and zero-magnitude damage remain eligible. One damage may match multiple shots.
- Preserves native unstable timestamp ordering, earlier-sample ties, inclusive
  time windows, f32 subtraction before f64 distance, and first candidate base ties.
  Empty/nonresolving tracks select 448 in actual native code (the 512 comment is
  inaccurate).
- 1,024 native cases pass with and without distances, including repeated timestamps,
  zero and maximal windows, missing tracks, multiple weapons and invalid identities.
  Harness and compressed fixture are registered in `generate_oracles.py`.
- Film shot/damage scanners and biped-track assembly remain pending. This is not
  yet an integrated weapon-hit API on Film.
- Corrected integrated document comparison remains running in session 28687;
  Bazaar and Aquarius confirmed, remaining documents not yet reported.


### Weapon-hit event readers and scanner integration

- `weapon_hit_scan.rs` decodes native long-fire and damage-aftermath packet heads.
  It preserves raw domain-one identities, five-bit shooter identities, source,
  signed magnitude, and unreadable long-fire shots. Damage reads expose physical
  end position and synthetic zero-tail bit count.
- 2,048 native cases pass for each reader, including optional fields, truncated
  payloads, both reference widths and signed magnitude codes. The reproducible
  native harness and fixture are registered in `generate_oracles.py`.
- Added chunk-level shot scan and damage scan, including native per-chunk final
  world bindings and both-reference biped-base scores. These orchestration paths
  still require dedicated captured-film differential validation. Biped distance
  track assembly and Film-level integration remain pending.


### Captured weapon-hit scans and distance tracks

- Native and Rust event scans agree across all type-2 chunks in six captured films:
  4,395 long-fire shots and 105 damage events, including all fields and biped-base
  selection (512 for Bandit and Oddball). Ignored test `local_weapon_hit_corpus`
  passed in 48.81 seconds. The corpus harness is reproducible via the generator.
- Added `build_weapon_hit_tracks`: explicit validated map precision, unsmoothed
  positions, saturation rejection, native timestamp sort, and following-keyframe
  slot-band coverage. Native fixture includes 590,758 position samples; Rust track
  comparison is running in session 82575 (Bazaar and Aquarius confirmed so far).
- Added `scan_film_weapon_hits` and `FilmWeaponHits` to retain source shots,
  damage reads/padding, landing base, optional distance base/tracks, distance failure,
  and summary statistics. Missing distance evidence preserves hit counts.
  Aggregate wrapper validation and partial/missing-chunk boundary cases remain.
- Integrated complete-document test session 28687 has now also passed Bandit;
  Oddball and the two appearance captures are still pending. Process sample
  `/private/tmp/halo-document-current-sample.txt` showed carrier-mark keyframe
  recovery, beyond the earlier march-calibration path. The live test was preserved.


### Weapon-hit distance coverage and retained body fields

- All six captured biped-track comparisons passed (590,758 samples, 64.21s).
  First-chunk-only comparisons also passed across six films (3.91s), including
  the native following-keyframe contribution to the slot band.
- `WeaponDamageRead` now retains the second raw magnitude and optional body victim
  separately from the header victim. All 2,048 native cases match these values and
  the ending bit position, in addition to the earlier source/magnitude checks.
- Added a combined-API regression proving counted hits survive absent or invalid
  map precision. The invalid map is reported as distance_error with no invented
  distance base. The initial test passed; a smaller fixture-free equivalent is
  being checked in the latest missing-map focused test.
- Full aggregate scan comparison is running in session 33009; it validates the
  native distance base, hit statistics and distance buckets from captured films.
  Bazaar and Aquarius are confirmed so far. Complete document comparison remains
  session 28687, confirmed through Bandit.


### Weapon-hit aggregate validation and weaponv3 helpers

- `local_weapon_hit_aggregate` passed all six captured films in 109.83s: native
  distance bases, total hits and distance histograms match the public combined API.
  The missing/invalid-map regression also passed with its small synthetic input.
- Added weaponv3 string roster resolution and first-chunk-wins numeric merging in
  `player_indices.rs`. Numeric overflow wraps; original strings remain keys;
  equivalent numeric spellings use the last roster spelling, matching Go.
- Added `WeaponTimestampEstimator` and retained frame ranges. Native byte lookup
  uses the previous frame in packet gaps and after the last payload, the first
  frame before any payload, and signed wrapping timestamp differences.
- 512 independent native cases passed for roster helpers and timestamp estimation.
  Harness `halo_rust_weapon_helpers_test.go.txt` and fixture are registered in the
  generator. The shared packet walker was extracted without changing conditions.
- Latest library/test Clippy passed. Full active Theater regression suite passed:
  247 tests, 22 ignored, 36.04s. Integrated complete-document comparison remains
  session 28687, confirmed through Bandit.


### Weapon canonicalization and remaining precision audit

- Added `canonical_weapon_id` and `canonical_weapon_name` using the existing
  pinned native family catalog. Unknown high words are accepted only with common
  suffix 0x42c9679f; accepted unnamed families retain an empty label.
- The native helper oracle now includes 2,048 canonical-ID/name cases, including
  every catalog family, arbitrary suffix variants, suffix-only unknown families
  and rejected IDs. The full 512-case helper fixture passes after regeneration.
- Precision audit confirmed the reference `poserProfilPuisCarte` installs the
  inherited profile first, then replaces world-object axes/index width/region,
  enabling complete simulation grammar for a valid map. Zero-axis entries retain
  the inherited descriptor and count the named default-axis fallback.
- Rust starting kill calibration already applies valid map precision and only
  changes the traversal handle width after its diagnostic axis sweep. However,
  Rust map constructors reject zero-axis encoding before that fallback can be
  published. This remains an explicit parity gap; do not silently invent bounds.
- Latest WASM and library/test Clippy checks passed. Integrated document comparison
  session 28687 has now also passed Oddball and Cadet Blue; Cadet Brick remains.


### Corrected document gate and world precision installer

- Corrected integrated document oracle passed ALL six captured films: Bazaar,
  Aquarius, Bandit, Oddball, Cadet Blue and Cadet Brick, 1,510.73s in the unoptimized
  test build. Session 28687 is terminal. This validates the regenerated native
  scan-time fallback counters and the inherited component-profile routes present
  in that build; it does not establish complete source-inventory parity.
- Added `install_replay_map_precision` in `world_precision.rs`. Valid map axes
  replace only world axes/index/region and enable complete simulation grammar.
  Any zero map axis preserves the inherited profile and increments
  `repli_largeurs_axe_par_defaut_conservees` exactly once per call.
- 512 native synthetic installations compare full preserved profile state and
  counters, including 384 zero-axis cases and nondefault inherited descriptors.
  Harness/fixture are registered in the generator; focused Rust test passed.
- Constructor/publication wiring remains open. Native `scanFilmInputs` leaves i0
  layout detection available when map axes are absent. The Rust port currently
  lacks `internal/grammar/i0_layout.go` and rejects zero-axis map encodings; port
  that diagnostic detector before changing this path. It derives wire field widths
  from measured consecutive-record flip rates and never supplies world AABB bounds.


### Native i0 layout detector

- Added `i0_layout.rs`: layout value/formatting, guarded 72-bit sample collection,
  per-slot native unstable ordering, three-packet adjacency gate, flip-rate report,
  boundary detection (at least 10% preceding flips and strictly eightfold drop),
  and six-chunk film orchestration with following-keyframe slot coverage.
- Retains inconclusive measurements and implausible inferred widths separately from
  successful layouts. This measures wire widths, never world AABB bounds or map
  identity. The biped-header reader is shared with existing position scanning.
- 512 synthetic native cases passed for samples, pairing, every flip rate, index-bit
  counts and boundaries. All six captured detector comparisons passed in 9.09s,
  covering 66,164 paired records: valid layouts for Bazaar/Bandit/Oddball,
  inconclusive Aquarius/Cadet Blue, and implausible Cadet Brick (47/5/2).
- Native fixtures/harnesses are registered in the generator. Library/test Clippy
  passed; all 249 active Theater regressions passed (23 ignored), 30.15s.
- World-axis fallback constructor wiring remains open. The native zero-axis path
  uses detected i0 layout for biped sampling but preserves the inherited/default
  world-object descriptor. Keep those two contexts distinct when wiring; replacing
  all component widths with the detected axes would change native behavior.


### World-axis fallback constructor integration

- `ReplayPrecisionContext` retains the inherited profile, measured biped sampling
  map, world-object map, detector report and fallback report. Both derived maps
  preserve the catalog AABB; wire widths never supply invented world bounds.
- Film constructors now resolve this context and retain it as `Film.scan_precision`.
  Biped/vehicle positions use sampling widths; ground objects, equipment placements,
  projectile positions and translocations use world-object precision. Component
  readers receive the inherited profile after map installation. Replay grapple
  decoding uses the world descriptor, and replay coverage publishes scan fallbacks.
- A complete native Bazaar document with all catalog axis widths zero now matches
  Rust (12.70s), including exactly one default-axis fallback, default grenade cap
  and round-zero diagnostic. The native/Rust detector measured 17/17/16 for biped
  sampling while inherited world-object widths remained 13/13/14.
- 249 active Theater tests passed (24 ignored), 35.01s. Library/test Clippy passed.
  A fresh six-film integrated document comparison is running in session 70559,
  log `/private/tmp/halo-precision-context-documents.log`; Bazaar and Aquarius confirmed.
  WASM validation passed, `/private/tmp/halo-precision-context-wasm.log`.
- Native zero-axis full-document harness and fixture are registered in the oracle
  generator. Other zero-axis maps and inherited-profile edge combinations remain
  candidates for further coverage; do not infer full parser completion from this gate.


### Keyframe measurements and retained weapon-hit exports

- Ported native `keyframe_record_spans.go` and `vehicle_occupancy.go` in
  `keyframe_spans.rs`. All 512 native cases pass, including skipped slot gaps,
  slot-only baselines, stable state ordering, zero-tail reads and insertion ties.
  Insertion comparison uses constant memory while preserving native scores.
- Both replication-enabled Film constructors now retain `keyframe_record_spans`
  and `vehicle_keyframe_states` from their existing recovered anchors, without
  recovering the payload again. Timestamps, chunk IDs and per-chunk packet indices
  are retained. The captured map-constructor test verifies 123 spans and their
  JSON round trip. A span or extra block does not prove occupancy.
- Added opt-in `Film::retain_weapon_hits(chunks)` and portable `weapon_hits`, using
  resolved sampling precision when available. The previously native-validated
  standalone scan remains the implementation; scan failures preserve old results.
  The map-constructor regression now also checks this export against the scanner.
- Audited EMP retention: the component reader already publishes raw `timer` and
  bit bounds; 96 existing native component cases cover it. Added the native
  maximum-quantum constant (255), without inventing seconds conversion.
- Registry fingerprint audit confirms existing hash/count/coverage publication;
  the process-global native warning hook and edge audit remain pending. Its
  inventory row remains partial rather than claiming full source parity.
- Validation before weapon-hit export addition: keyframe oracle, captured Film
  export regression and Clippy passed. Latest complete Theater suite, Clippy and
  WASM checks are recorded below when finished. Integrated six-film precision
  document gate remains running in session 70559; do not restart on timeout.


### i0 value edges and standalone identity metadata

- The latest keyframe/weapon-export suite passed 250 active Theater tests (24
  ignored); Clippy and WASM passed. Logs: `/private/tmp/halo-keyframe-export-suite.log`,
  `/private/tmp/halo-keyframe-export-final-clippy.log`,
  `/private/tmp/halo-keyframe-export-wasm.log`.
- Added a reproducible 512-case native i0 value oracle, including signed and
  unsigned overflow, physically invalid widths, region display, and axis offsets.
  Go passed in 0.413s; Rust passed in 0.01s. Public header constants now match
  native profile constants; invalid positive axis indices return None safely.
- Source audit found four fields missing from standalone `FilmIdentity`, although
  available on Film.registry: format version, block count, fingerprint and named
  slot count. Identity now retains them with serde defaults for older exports.
  The captured native bootstrap oracle already contains all four expected values;
  the regression now asserts them. Latest suite passed 251 active Theater tests
  (24 ignored), 35.97s; Clippy and WASM passed. Logs:
  `/private/tmp/halo-identity-export-suite.log`,
  `/private/tmp/halo-identity-export-clippy.log`,
  `/private/tmp/halo-identity-export-wasm.log`.

- Integrated precision-context document gate (session 70559) has now matched
  Bazaar, Aquarius and Bandit EVO. Remaining documents are still running. This
  process predates the additive measurement/identity exports; it validates the
  precision routing change, while the current 251-test suite validates exports.
  Full parser parity remains incomplete; positive CTF/VIP/bomb captures, extended
  raid assembly and remaining source/API audit are still required.


### Full raid document gate and engine precision law

- Added native `TestHaloRustFullRaidDocument` to the full-document harness and
  `--include-raid` generator path. It scans the complete hour-long Facility Aetheria
  capture; the fo11_blank module's shared catalog bounds are supplied by the Corpo
  entry, as in the earlier raid harness. This is not a map-identity inference.
  Native passed in 277.209s; fixture `full-raid-document-v41.json.zlib` now retains
  the complete document. Rust ignored `local_complete_raid_document` is running
  in release mode (session 48651; `/private/tmp/halo-full-raid-rust.log`).
- Added `precision_law.rs` for `internal/profile/loi_largeurs.go`: engine float32
  quantization step, bounds-to-axis-width calculation, default build bounds and
  raw-region-count index width. An independent 2,048-case Go fixture includes
  random float bit patterns, reversed bounds, caps, extreme levels and counts.
  Go passed in 0.313s; all 2,048 Rust cases passed in 0.03s.
  The native width law also matches all 79 catalog maps. Clippy and WASM passed;
  logs `/private/tmp/halo-precision-law-rust.log`,
  `/private/tmp/halo-precision-law-catalog.log`,
  `/private/tmp/halo-precision-law-clippy.log`,
  `/private/tmp/halo-precision-law-wasm.log`.
- The initial profile-package test invocation compiled unrelated x86-only Oodle
  tests on ARM. The reproducible oracle now calls exported profile functions from
  the grammar test package, avoiding those unrelated test-only imports.
- Audited `vehicle_occupancy_march.go` against the existing implementation and
  1,024 native harvest cases; its pending row was stale. Extraction, ordering,
  deduplication, Film retention and replay consumption are already implemented.

- Full native raid document retains 86 vehicles, 5,224 shots and 1,394 projectiles,
  plus current inventory, equipment, ability, identity, score and coverage layers.
  These counts describe the oracle, not completed Rust parity. Session 48651 is
  still compiling/running the Rust comparison; session 70559 remains live with
  Bazaar, Aquarius and Bandit documents matched so far. Preserve both processes.


### Integrated document gate passed; optional chain inference

- Session 70559 completed successfully: all six integrated native documents match
  after the precision-context changes (1529.55s debug). Durable log:
  `/private/tmp/halo-precision-context-documents.log`. This supersedes its earlier
  running status. The full raid comparison (48651) remains live.
- Added `chain_inference.rs` and a shared component body trial for the optional
  native recursive inference path. It retains outcome, chosen alignment,
  archetype uniqueness and remaining trial budget. Trial walks never mutate the
  world; soft/overlay bindings cannot self-confirm as hard anchors. The native
  End-marker/zero-tail confirmation remains explicitly heuristic and opt-in.
- Initial 1,024 native cases matched (580 immediate, 69 deep, 375 unconfirmed).
  Expanded direct alignment cases cover 411 ambiguous outcomes, 137 exhausted
  budgets and successful/no-confirmation paths. All outcomes, selected ends and
  remaining budgets match Rust (0.25s), including restoration of path overlays.
  Native generation passed in 0.386s. Log: `/private/tmp/halo-chain-resolution-rust.log`.
- This source remains partial: component stub-width repair, positive mid-chain
  biped creation coverage and opt-in frame-walk integration are still required.
  The standard production policy continues to use native view admission without
  enabling inference. Current helper passed Clippy and WASM checks.

- Full raid document gate has now PASSED: session 48651 matched the complete
  Facility Aetheria native document in 202.68s release (2m06s compilation).
  `/private/tmp/halo-full-raid-rust.log` is the durable result. This validates
  current full replay assembly from decoded Film inputs, including all exported
  values and coverage metadata in the oracle. It does not run integrated kill
  source calibration on the raid; that constructor path is validated by the
  six-film integrated gate above. No long-running validation sessions remain.


### Optional component-width repair

- Added `repair_chain_component` with the native inclusive 0..640 extra-width
  sweep, distinct-end confirmation, first winning width and complete list of
  widths sharing the selected end. Preset widths for the culprit are respected;
  trials use fresh readers and never mutate the world or install global widths.
- The shared entity decoder now has an internal component-walk policy carrying
  an optional explicit stub. Existing entry points still pass no stub. Known
  fields before the failure are preserved; inferred tail bits are retained as
  `inferred_stub[n]` raw fields with exact offsets, including widths above 64.
- Added 512 native repair cases (89 successes) covering unknown components before
  and after an EMP field, raw tails, terminal/nonterminal successors, and preset
  refusal. Native passed in 0.511s. Rust focused test passed in 2.13s; all 254 active
  Theater tests passed (25 ignored) in 36.10s. Clippy and WASM passed. Logs use the prefix `/private/tmp/halo-chain-repair-`.
- Optional frame-loop integration and positive NEW/partial-prefix repair fixtures
  are still pending; this does not claim the source is complete.


### Native optional inference loop and validated resync

- Added `decode_inference_frame`, `InferenceFrameOptions` and portable frame
  results separating decoded records from inferred skips. Single-step inference
  requires exactly one archetype and never binds; recursive inference soft-binds
  only a unique archetype. View admission takes precedence over either path.
- The loop preserves the native defaults: automatic repair is a false constant
  and resync has no targets in the pinned source. Repair remains an explicit
  operation; neither behavior was silently enabled in Film constructors.
- All 1,024 native loop cases match records, type IDs, masks, component counts,
  failures, final world bindings, hit-End status and final cursor. Cases exercise
  both inference modes, view rejection, soft bindings, NEW/DELETE, extra-prefix
  words and random/truncated bytes. Portable frame round trips also pass.
  Native passed in 0.456s; Rust in 0.12s.
- Added explicit `validated_chain_resync`: target-delta candidate reads followed
  by bounded chain confirmation, with no world mutation. All 512 native cases
  match (237 accepted landings), including wrong target sets, extra prefixes,
  varying start bits and rejected garbage. Go passed in 0.334s, Rust in 0.03s.
- Reproducible harnesses/fixtures are registered in the generator. Clippy and WASM passed. Logs `/private/tmp/halo-inference-frame-rust.log`,
  `/private/tmp/halo-validated-resync-rust.log`,
  `/private/tmp/halo-inference-loop-clippy.log`,
  `/private/tmp/halo-inference-loop-wasm.log`.
- Remaining source-specific validation: positive mid-chain biped NEW overlays,
  partial-prefix/NEW repair and broader profile flags. Full v41 parity remains
  incomplete, including positive captured CTF/VIP/bomb assembly and other source rows.


### Biped NEW overlays and partial-prefix repair

- Added 128 native biped-creation chain cases: 90 deep confirmations and 38
  refusals. They cover path-local NEW replacement, deletion tombstones, hard/soft
  confirmation, MPP 9/5 and 8/3, corruption guards, and random default-state data.
  Go passed in 0.445s; Rust passed in 0.02s. Trial worlds remain unchanged.
- Added 128 native NEW repair cases with simulation-state prefixes of 1..269 bits,
  full/quantized precision, MPP variants, corruption guards and extra-prefix words.
  The inclusive width sweep is exercised at 640 and beyond it. There are 44 native
  successes, four in extra-prefix mode; at least one selected width is exactly640.
- Source inspection found and fixed an extra-prefix NEW repair mismatch: the
  generic entity reader consumes an optional NEW extra field, whereas native
  inference repair does not. Repair now re-reads the actual header with that
  generic extra-field path disabled. Its prefix and unknown tail remain separately
  preserved as raw fields with exact offsets. The expanded native regression
  checks complete ends, matching widths, prefixes, field bits and unchanged worlds.
- Native expanded repair oracle passed in 0.545s. All 258 active Theater tests
  passed (25 ignored), 33.51s; Clippy and WASM passed. Logs use prefix
  `/private/tmp/halo-chain-new-`. No live validation processes remain.


### Frame views and exhaustive target harvesting

- Added `decode_inference_views` for native A/B/C class traversal and generic
  N-view mode. Retains each entity view, control/message results, completed view
  count and exact final cursor. All 1,024 native cases pass.
- The view comparison exposed a fixture context mismatch for vehicle NEW records:
  Rust had no position encoding while native used its default profile. Both
  native frame-loop harnesses now export their complete position context; both
  1,024-case Rust comparisons pass with that context explicitly supplied.
- Added `scan_frame_targets`, with native NextBound and ChainWalk confirmations.
  All 512 cases match (656 accepted records). NextBound's short zero-tail
  acceptance is documented as a native heuristic, not a proven record boundary.
- Native fixture generation, Clippy and WASM passed. Logs:
  `/private/tmp/halo-inference-views-context-rust.log`,
  `/private/tmp/halo-frame-targets-rust.log`,
  `/private/tmp/halo-frame-harvest-clippy.log`,
  `/private/tmp/halo-frame-harvest-wasm.log`.


### Position-aware resync and forward target scanning

- Added `scan_for_target_delta`, `decode_frame_resync`, `ResyncFrame` and
  `resync_position`. The native resync algorithm scans from failed-start + 1,
  filters only clean substantive target deltas, and resumes sequential decoding
  without inferring unknown bindings. Successful NEW/DELETE records commit their
  world changes. The explicit API retains fields and recovered start bits.
- Acceptance callbacks receive complete records; `resync_position` exposes the
  first native i0 observation, with an explicit absolute/fallback/delta8/delta-axis
  kind. Absolute region filtering, absent/default/full-precision/baseline paths
  match native. No unused native position accumulator is silently installed.
- All 512 native cases match: 192 scan landings, 761 output records, 465 scanner
  callbacks, 484 frame callbacks, and 576 position-bearing callbacks. Comparisons
  include exact float32 bits, record masks/component counts, final bindings and
  JSON round trips. Inputs cover extra-prefix words, NEW/DELETE, invalid components,
  full precision, writer grammar, delta handle tails and nonzero region IDs.
- Exact-bit comparison caught absolute dequantization rounding: the pinned Go
  reader fuses q*step+min. Rust now uses explicit mul_add for identical output.
  Go oracle passed in 0.379s; Rust passed in 0.31s. Harness and fixture are
  registered in `generate_oracles.py`. Broad checks are recorded below when done.
- Full parser parity remains incomplete: broader profile context/source audit and
  positive complete captured CTF/VIP/bomb replay comparisons remain open.

- Final broad checks after resync: all 261 active Theater tests passed, 25 ignored,
  in 34.43s. Library/test Clippy passed with warnings denied; WASM compilation,
  formatting and diff checks passed. Logs use `/private/tmp/halo-frame-resync-`
  (`suite.log`, `clippy.log`, `wasm.log`). No validation processes remain running.


### Native position observations and explicit accumulation

- Added `capture_component_position`, `capture_record_positions`,
  `NativePositionSample` and `NativePositionKind` in `position_capture.rs`.
  Resync now delegates to this shared observer. Decoded fields remain the single
  wire grammar; capture never rereads bytes using a competing position layout.
- Optional caller-owned `FilmWorld` accumulation ports the native reader's seed,
  delta and baseline semantics. Unseeded deltas emit nothing with an accumulator;
  without one they emit relative values. An absolute on an unbound slot emits but
  does not create a binding. Baseline copies re-emit saved positions, never the
  96 wire bits. Other full-precision/default/uncatalogued-region paths stay absent.
- Native validation covers 1,024 sequences / 12,288 component reads with binding
  replacement, unbinding, explicit seeds, reader bit offsets, varying axis/index
  widths, nonzero regions, full precision, writer grammar, and handle tails.
  Exact float32 sample bits and final per-step world positions match. Published
  sample kinds: 450 absolute, 53 fallback, 143 delta8, 153 delta-axis, 25 baseline.
  Native passed in 0.532s; Rust in 0.26s. Portable samples round-trip exactly.
- This closes the optional accumulator omission documented above, through an
  explicit API. Native Film replay constructors still do not install an accumulator;
  the Rust constructors preserve that behavior.

- Broad validation after shared capture integration: all 262 active Theater tests
  passed (25 ignored), including the existing frame resync oracle, in 35.86s.
  Library/test Clippy with warnings denied, WASM compilation, formatting and diff
  checks passed. Logs use `/private/tmp/halo-position-capture-` (`rust.log`,
  `suite.log`, `clippy.log`, `wasm.log`). All runs are terminal.


### Native entity-reference observations

- Added `NativeUnitReference` and its three wire forms. Shared readers now retain
  native presence, raw value, generation tail, category-one probe flag, and exact
  start/end bits. `DecodedComponent`, `EntityRecord`, and `KeyframeRecord` retain
  the observations in portable exports; absent lists deserialize compatibly.
- Capture follows native call sites, not field shape alone. Inlined navpoint and
  tracked-equipment handle readers retain raw fields but do not manufacture native
  reference callbacks. Actor-control's optional 32-bit word emits only when present;
  native generic optional-word readers also emit the closed-gate observation.
- All 15,864 native component cases and 3,008 default-state cases match (1,157 and
  1,260 reference observations respectively). End bits and portable round trips
  also match. The 256-case keyframe integration oracle adds 1,227 references and
  exercises default guards, both actor slots and per-component corruption checks.
  Native component/default tests passed in 0.649s; Rust in 1.12s. Native keyframe
  oracle passed in 0.355s; its Rust result is recorded with broad validation below.
- Reproducible harnesses are registered in `generate_oracles.py`. The native hooks
  remain observations, not claims that each raw value is an entity slot.
- Source audit identified an additional real gap: native `ScanUnitEquipment`
  publishes complete i26 equipment-reference lists tied to biped slots and packet
  timestamps. The raw component is decoded, but a corresponding Rust film-level
  scanner/publication path is not yet present. This remains required work.

- Final reference-observation validation: all 265 active Theater tests passed
  (25 ignored), including keyframe reference retention, in 37.68s. Library/test
  Clippy with warnings denied, WASM compilation, formatting and diff checks pass.
  Logs use `/private/tmp/halo-unit-references-` (`rust-fixed.log`, `suite.log`,
  `clippy.log`, `wasm.log`). All validation runs are terminal. Full v41 parity is
  still incomplete; the i26 scan is a concrete next implementation item.


### i26 equipment-list scanner and Film export

- Added `UnitEquipmentRead`, `UnitEquipmentEmission`, `UnitEquipmentStream`,
  `read_unit_equipment` and `scan_unit_equipment`. The scanner walks through the
  registry-selected i26 component using existing raw position anchors, including
  movement-filter rejections. It preserves empty lists, absent entries, raw values,
  generation tails, packet metadata, record starts and complete component fields.
- Map-aware Film decoding retains `unit_equipment` or its explicit scan error.
  Portable exports default the new fields for older data. The list reader also
  supports component exports from before typed reference observations existed.
- Native synthetic oracle: 512 cases, 318 accepted reads, 40 empty lists and
  367 closed entries. Covers relocated component indices, missing/unsupported
  predecessors and truncation. Native passed in 0.421s.
- Native captured oracle: all six selected v41 films, 204 emissions (Bandit 62,
  Oddball 140, each Cadet appearance film 1, Bazaar/Aquarius 0), passed in 3.817s.
  Harnesses/fixtures are registered in the generator.
- Initial Rust validation hit disk exhaustion while writing its incremental query
  cache. Package-local generated artifacts are being cleaned with `cargo clean -p
  halo_api`; validation resumes after that terminal result. This is not a parser
  mismatch and no Rust parity result is claimed yet.


- Disk recovery completed: `cargo clean -p halo_api` removed the package's generated
  artifacts (reported logical total 151.2 GiB). Rust validation then passed all
  512 synthetic cases and all 204 native emissions across six complete captured
  films, in 60.25s. The comparison checks full lists, slot IDs and packet times.
  `/private/tmp/halo-unit-equipment-rust.log` is the terminal successful run.
  Positive Film-export and broad library checks are recorded below when terminal.


- Positive Film integration passed: the Cadet Blue captured emission matches the
  standalone/native list after `Film::try_from_chunks_with_map`, and the complete
  Film JSON round trip retains it and its diagnostics exactly (36.26s).
  `/private/tmp/halo-unit-equipment-export.log` is terminal and successful.
- The minimal keyframe fixture has no biped slot band. Its new test incorrectly
  required a successful scan; it now checks the explicit missing-band error and
  its portable retention. This preserves the native scanner's precondition rather
  than treating unavailable input as a successfully empty equipment stream.

- Final i26 checks: all 266 active Theater tests passed (27 ignored) in 33.96s;
  final library/test Clippy with warnings denied, WASM compilation, formatting
  and diff checks passed. Terminal logs are `/private/tmp/halo-unit-equipment-`
  `suite-fixed.log`, `clippy-final.log`, and `wasm.log`. No validation runs remain
  active. Full parser parity remains incomplete beyond this source entry.

### Observer audit: absolute index histogram

- Native `Observation.IndexAbsolus` increments in `absAxisWFor` on axis zero,
  before reading that axis, including default index -1. Call sites are both
  absolute i0 paths and the simulation-state handle tail in `traverse.go`.
- `FilmReadDiagnostics` now retains the histogram in component, entity and
  keyframe reads. `decode_native_component` exposes padded attempts and their
  diagnostics; `merge` and `take_absolute_indices` reproduce accumulation and
  snapshot/reset semantics. Bounded APIs preserve their existing stop policy.
- Direct `infer_chain_archetype` retains diagnostics from rejected and accepted
  component trials and NEW-record reads. Its result includes the histogram;
  speculative counters do not fabricate position/reference/movement samples.
- Independent native oracles match 4,096 component attempts (1,373 increments),
  256 keyframes (156 increments), and 512 chain-inference cases (569 increments).
  Covers padding/truncation, full precision, writer grammar, default/explicit
  regions, simulation-state tails, reset, outcomes, and JSON retention.
- Still incomplete: repair/resync accumulation and broader scanner aggregation,
  other native counters (width histograms, repair/resync, view rejections and
  anticipated bindings), and the complete hook-by-hook audit. Do not mark
  `observateur.go` complete from these low-level results.
- Fixtures/harnesses: `read-diagnostics-v41` and `chain-diagnostics-v41`, both
  registered in `generate_oracles.py`. Logs: `/private/tmp/halo-diagnostics-test.log`
  (initial direct/chain cases) and `/private/tmp/halo-diagnostics-suite.log`
  (expanded keyframe coverage). Final validation: 270 active Theater tests passed
  (29 ignored); Clippy with warnings denied, WASM, formatting and diff checks
  passed. All processes reached terminal status.

### Native legacy weapon-pattern scans

- `weapon_patterns.rs` ports all five public native operations: frame-marker
  discovery, timestamp estimation, Formula A, nibble-shifted Formula A, and
  universal-marker B5 fire scanning. The complete pinned 39-entry filmshell
  catalog preserves exact eight-byte acceptance and variant names. Family-only
  matching would incorrectly accept unknown variants and is not used here.
- Preserve the first-suffix-only Formula A rule, shifted-layer offsets and 0x26
  exclusion, native 115-bit minimum guard, optional post-weapon flags, both four-
  and five-bit indices, two-byte deduplication, and native unstable sort ordering.
  Timestamp estimates use explicit fused multiply/add, matching the pinned Go
  target; captured tests exposed and verified the rounding difference.
- `Film.weapon_patterns` retains per-chunk outputs and timing context. These
  remain labeled pattern observations, not decoded hits or player identities.
- The 1,024-case oracle matches 3,102 Formula A snapshots, 3,489 nibble-shifted
  snapshots and 3,402 fire observations. Six captured films match all 402 Formula
  A, 142 nibble-shifted and 4,965 fire observations, every timestamp and ordering.
  Positive Bazaar Film construction and complete JSON retention pass.
- Optimized equivalent single-pass nibble scanning and direct marker reads pass
  both oracles in 21.87s including Film construction/export. Native corpus takes
  3.157s. Harnesses, compressed fixtures and catalog refresh are registered in
  `generate_oracles.py`. 268 active Theater tests passed (29 ignored); optimized
  Clippy with warnings denied, WASM, formatting and diff checks passed.
- Logs: `/private/tmp/halo-weapon-patterns-optimized.log`,
  `/private/tmp/halo-weapon-patterns-native-corpus.log`,
  `/private/tmp/halo-weapon-patterns-suite.log`,
  `/private/tmp/halo-weapon-patterns-clippy-optimized.log`,
  `/private/tmp/halo-weapon-patterns-wasm-optimized.log`.

### Native inference-frame diagnostics

- `InferenceFrame.diagnostics` now accumulates absolute-index reads from committed
  records, failed reads, single-step trials and their bound successors, and chain
  trials. Each view retains its own aggregate through `InferenceViews.views`.
- `FilmReadDiagnostics` also retains chain outcome counts, unbound-datum and
  other-view rejection counts, and anticipated bindings by archetype. Direct
  chain inference records its native outcome once. Diagnostic deserialization
  defaults new counters for older exports.
- The 1,024-case native `frame-diagnostics-v41` oracle validates records, final
  cursors, world mutations, inferred counts, all diagnostic fields and JSON round
  trips. Expanded positive coverage includes 52 absolute-index increments,
  160 unbound rejections, 42 other-view rejections, 21 anticipated bindings,
  63 immediate chain resolutions and 147 unconfirmed chain outcomes.
- All 271 active Theater tests passed (29 ignored); expanded diagnostic oracle,
  Clippy with warnings denied, WASM, formatting and diff checks passed. Logs:
  `/private/tmp/halo-frame-diagnostics-suite.log`,
  `/private/tmp/halo-frame-diagnostics-expanded.log`,
  `/private/tmp/halo-frame-diagnostics-clippy.log`,
  `/private/tmp/halo-frame-diagnostics-wasm.log`.
- Remaining observer work includes repair and resync accumulation, winning-width
  histograms, repair/resync counters, and a complete hook audit. The source
  inventory deliberately keeps `observateur.go` partial.

### Repair and validated-resync diagnostics

- `FilmReadAttempt<T>` retains a result and diagnostics even when the operation
  fails. `repair_chain_component_observed` and `validated_chain_resync_observed`
  expose the native observer state; existing result-only wrappers remain available.
- Repair accumulates reads from all 641 width trials, downstream confirmation,
  and the final selected-width reread. Only successful repairs increment
  `repaired_records`; `component_widths` records every width at the winning end.
  Failed alignment resolution retains its native ambiguity/budget/failure count;
  successful repair does not increment the archetype-inference success counters.
- Validated resync attempts every delta before target filtering, as native
  `TryDeltaAt` does. Rejected and non-target attempts retain their read counters.
  Successful continuation confirmation increments `validated_resyncs` exactly once.
- Independent 512-case native repair and 512-case native resync oracles match:
  repair has 19,874 absolute-index increments, 94 successful repairs and 371
  ambiguous outcomes; resync has 500 absolute-index increments and 166 landings.
  Results, widths, diagnostics, failure retention and JSON round trips match.
  Both harnesses and fixtures are registered in `generate_oracles.py`.
- Logs: `/private/tmp/halo-recovery-diagnostics-test.log` (all six diagnostic tests
  passed). Remaining observer audit: raw resync/harvest aggregation and hook-level
  capture suppression/retention. `observateur.go` remains partial.

- Final recovery-diagnostic validation: 273 active Theater tests passed (29 ignored),
  WASM, formatting and diff checks passed; Clippy with warnings denied passed after
  a documented `large_enum_variant` exception preserving the value-based component
  API without a per-component box allocation. Logs:
  `/private/tmp/halo-recovery-diagnostics-suite.log`,
  `/private/tmp/halo-recovery-diagnostics-clippy-final.log`,
  `/private/tmp/halo-recovery-diagnostics-wasm.log`. All processes terminal.
- Next concrete observer paths to audit: `scan_frame_targets`,
  `scan_for_target_delta`, `decode_frame_resync`, and the `harvest_confirmer`
  closure. Their returned records contain per-read diagnostics, but rejected
  attempts and aggregate counts are not yet all exposed.

### Native harvest diagnostics

- `scan_frame_targets_observed` retains diagnostics from all candidate reads,
  confirmation reads and the accepted-record reread; the existing result-only
  wrapper delegates to it. NextBound checks hard binding BEFORE reading its
  successor body, matching native and avoiding spurious diagnostic increments.
  The chain confirmer returns and clears its per-call diagnostics for accumulation.
- A 512-case independent native oracle matches 633 accepted records and 532
  absolute-index increments across both confirmation modes, hard/soft successors,
  full precision, writer grammar and handle-tail variants. World immutability,
  bit fields and JSON round trips pass. Harness/fixture `harvest-diagnostics-v41`
  is registered in `generate_oracles.py`.
- Remaining raw-resync ownership detail found in `frame_harvest.go`:
  `scanForTargetDelta` creates a new `Observation`, shallow-copies the caller's
  observer, then replaces its position callback. A non-nil `IndexAbsolus` map is
  shared; if it is nil, its first allocation belongs only to the capture observer
  and the caller does not receive those counts. `DecodeFrameResync` also rereads
  accepted candidates. Test both nil and preinitialized-map cases before claiming
  exact caller-visible diagnostic parity; do not silently treat all raw scan
  trial counts as externally retained native observations.
- Logs: `/private/tmp/halo-harvest-diagnostics-test.log`,
  `/private/tmp/halo-harvest-diagnostics-clippy.log`,
  `/private/tmp/halo-harvest-diagnostics-wasm.log`.

- Final harvest validation: 274 active Theater tests passed (29 ignored), Clippy
  with warnings denied, WASM, formatting and diff checks passed. Suite log:
  `/private/tmp/halo-harvest-diagnostics-suite.log`. All processes terminal.

### Raw-resync observer ownership and complete read retention

- `scan_for_target_delta_observed` returns all reads made by the native temporary
  capture observer. `RawResyncDiagnostics` separates `caller_visible` from
  `detached_scans` and retains whether the caller histogram has been initialized.
  `absorb_direct` and `absorb_scan` implement the native map-ownership transition;
  initialized empty maps are distinct from uninitialized maps.
- `decode_frame_resync_observed` accepts explicit initial observer state. The
  existing `decode_frame_resync` starts with native uninitialized state and
  returns observations on `ResyncFrame`. Successful scanned records are reread,
  matching native, and the direct reread contributes to caller-visible counts.
- Independent 512-case oracle covers both initialization modes, callbacks,
  first-landings, final records/world, and JSON round trips. For uninitialized
  direct scans native exposes zero of 107 reads; initialized scans expose all 71.
  Full frames expose 297 of 335 reads in the uninitialized cases and all 172 reads
  in initialized cases. Rust retains the 38 detached full-frame reads separately.
- The oracle runs an additional native initialized-observer pass as independent
  evidence for complete totals. Fixture/harness `raw-resync-diagnostics-v41` is
  registered in `generate_oracles.py`. Focused comparison passed in 0.39s at
  `/private/tmp/halo-raw-resync-test.log`.
- Remaining observer work: complete hook-by-hook audit, especially which capture
  channels native deliberately keeps live during speculative reads. Existing raw
  fields and read diagnostics do not by themselves prove callback-publication parity.

- Final raw-resync validation: 275 active Theater tests passed (29 ignored),
  Clippy with warnings denied, WASM, formatting and diff checks passed. Logs:
  `/private/tmp/halo-raw-resync-suite.log`, `/private/tmp/halo-raw-resync-clippy.log`,
  `/private/tmp/halo-raw-resync-wasm.log`. All processes terminal.

### Unknown-profile publications and registry warning audit

- Added public `unknown_build_metric_pairs` and `unknown_format_metric_pairs`
  in `profile.rs`, matching native publication helpers without owning service
  counters. The caller decides when a rejection should be counted. Unknown build
  names replace each non-ASCII scalar with one underscore; empty names use
  `sans_section`. Formats preserve native signed integer formatting, including 0
  for a missing header. Native oracle covers 1,159 build names and 7 format values.
- `parse_registry` now emits a structured tracing warning once per unknown
  fingerprint across the process, matching native warning ownership. It continues
  decoding and does not change widths. Known fingerprints are never remembered
  as warnings, and repeated fingerprints deduplicate even when counts differ.
  Native oracle covers 1,024 calls. No tracing subscriber is installed by the
  library; applications retain control of log collection.
- Added 256 native registry edge cases for FNV and named-slot/block counts:
  empty/full blocks, 1/17/255/256-byte names, high precision-level words,
  nonprintable names, malformed zero tails, and partial final blocks. The Rust
  test also verifies full registry JSON round trips.
- All three native harnesses and compressed fixtures are registered in
  `generate_oracles.py`. Initial 277-test suite passed before the fingerprint edge
  addition; final checks are recorded below. The first edge-test compilation
  exposed an unavailable base64 test dependency; fixture encoding was changed to
  hex, retaining existing dependencies.
- Full parity remains incomplete: hook/publication and inherited-context audit,
  remaining source inventory, and positive complete captured CTF/VIP/bomb replay
  validation remain open. This closes specific observability gaps, not the goal.
- Final validation: 278 active Theater tests passed (29 ignored), 33.77s;
  Clippy with warnings denied and WASM compilation passed. Clippy required replacing
  `chunks_exact(2)` with `as_chunks::<2>()` in three existing test-only hex readers
  (`keyframe_position_probe`, `weapon_patterns`, `read_diagnostics`); behavior is
  unchanged. Formatting, diff checks and oracle-generator syntax checks passed.
  Logs: `/private/tmp/halo-publication-suite-final.log`,
  `/private/tmp/halo-publication-clippy-final.log`,
  `/private/tmp/halo-publication-wasm.log`. All processes terminal.
- Observer audit continuation: `RecordMaskHook` is gated by `CaptureDirs`, fires
  after saturated-position rejection but before stream isolation/speed filtering,
  and publishes `(mask, payload, I0 + layout.TotalBits())`. Rust retains masks and
  vector end offsets on `BipedPositionRecord`, plus all rejected candidates and
  source packet coordinates. Exact callback selection still needs a dedicated
  audit; do not equate every retained candidate with a native hook emission.

### Historical mobility hook through speculative reads

- Found a concrete publication gap: native `MobilityActionHook` deliberately
  stays live when position, references or movement-state captures are suppressed.
  Raw component flags alone did not retain the ordered callback stream from
  failed trials, candidate scans, confirmation reads and winning rereads.
- `FilmReadDiagnostics.mobility_actions` now retains ordered `[flag1, flag2]`
  emissions at the component reader, immediately after the two flag reads and
  before the optional mobility body. Native padded reads retain emissions even
  when the payload is exhausted. The vector is absent from JSON when empty, and
  older exports deserialize with an empty vector. Histogram snapshot/reset does
  not clear mobility emissions.
- Existing diagnostics propagation now carries the stream through component,
  keyframe, chain, harvest and repair outputs. Raw-resync shallow observer copies
  share hook closures even when their newly allocated absolute-index map is
  detached: mobility emissions therefore always join `caller_visible`, never
  `detached_scans`. Counts and callback ownership remain distinct.
- Five independent native harnesses validate exact ordered flags, read ends,
  statuses/landings, accepted records, world state and JSON round trips where
  applicable. Cases and emissions:
  - 4,096 direct reads, including all three native suppression modes: 4,096 emissions.
  - 256 keyframes: 512 emissions.
  - 512 chain cases: 1,992 emissions, including speculative failed candidates.
  - 512 raw-resync cases: 568 scan and 711 full-frame emissions, across both
    initialized and uninitialized absolute-index maps.
  - 512 target-harvest cases: 2,726 emissions, including confirmation and rereads.
  - 512 repair cases: 74,359 emissions, including width trials and winning rereads.
- Harnesses and fixtures `mobility-{reads,chain,resync,harvest,repair}-v41` are
  registered in the oracle generator. All five focused Rust tests passed in
  2.00s (`/private/tmp/halo-mobility-expanded.log`). Remaining native hooks and
  complete captured objective replay coverage still require audit/validation;
  this does not establish full parser parity.
- Final mobility validation: 283 active Theater tests passed (29 ignored),
  34.93s. Library/test Clippy with warnings denied, WASM compilation, formatting,
  diff checks and generator syntax checks passed. Logs:
  `/private/tmp/halo-mobility-suite.log`, `/private/tmp/halo-mobility-clippy.log`,
  `/private/tmp/halo-mobility-wasm.log`. All processes terminal; changes remain
  uncommitted on `simbleau/theater-experiments`.

### Inventory and ability observer publications

- Extended the audit beyond mobility: nine native component hooks remain live
  under capture suppression and publish even during speculative/padded reads.
  Their accepted component fields were present, but their ordered speculative
  publication stream was not retained.
- Added typed `FilmComponentObservation` variants for ability energy, grenade
  selection, ability selection, EMP timer, grenade counts, weapon magazine/fraction,
  reserve rounds, desired weapon selection and ground-weapon ammunition.
  `FilmReadDiagnostics.component_observations` carries the ordered stream through
  existing component/keyframe/frame/recovery results and JSON. Absent magazine or
  fraction is `None`, unarmed ability charges and absent rank retain native -1,
  and ability selection preserves consumed width. Empty streams are omitted from
  JSON; old exports default to empty.
- Readers emit at the native hook location, after the relevant fields complete;
  no field widths or gates changed. Shared optional-value reading retains the
  existing raw field names. Raw-resync shallow-copy ownership now keeps these hook
  publications caller-visible even when its absolute-index map is detached.
- Native helper `halo_rust_component_hooks_test.go.txt` installs the actual nine
  callbacks. Five new independent oracles validate:
  - 4,096 direct reads across suppression modes, 4,096 emissions;
  - 256 keyframes, 512 emissions;
  - 512 chain cases, 1,536 emissions;
  - 512 target-harvest cases, 3,521 emissions;
  - 512 repair cases, 273,136 emissions;
  - 512 raw-resync cases, 589 scan and 1,016 full-frame emissions.
  All five focused Rust tests pass, including exact sequence, statuses/landings,
  records/world changes and JSON where applicable (2.77s).
- Refreshed existing frame/repair/harvest/validated-resync and mobility repair/
  harvest native diagnostics fixtures to include these publications. Existing
  aggregate assertions remain strict; the new fields are not discarded to make
  older comparisons pass. Generator installs the shared hook helper and all five
  harnesses before running the oracles.
- Objective corpus audit: the native `minifilm_*` files explicitly retain sampled
  keyframes out of continuity; e.g. `minifilm_111fa685/PROVENANCE.txt` says it is
  not a valid film (and is v39). These cannot establish complete captured v41
  objective replay parity. The CTF/VIP/bomb full-film gate remains open, as do
  remaining native hook and inherited-context audits.
- Final inventory-hook validation: 288 active Theater tests passed (29 ignored),
  36.36s. Library/test Clippy with warnings denied, WASM compilation, formatting,
  diff checks and oracle-generator syntax checks passed. Logs:
  `/private/tmp/halo-component-hooks-suite.log`,
  `/private/tmp/halo-component-hooks-clippy.log`,
  `/private/tmp/halo-component-hooks-wasm.log`. All processes terminal. Work remains
  uncommitted on `simbleau/theater-experiments`; the full parity goal remains active.

### Camo and Spartan ability hook snapshots

- Added `NativeCamoState` and `NativeAbilityNonPredictedState`, exposed through
  three new `FilmComponentObservation` variants: camo, predicted ability and
  non-predicted ability. Compound snapshots are boxed so small hook events do not
  inherit the largest payload size. Existing raw component fields remain intact.
- Camo preserves the optional second flag, fraction and six gated sub-values.
  Predicted ability retains tag/sub/reference and native reference presence.
  Non-predicted ability retains body-walked/body-ok, inner tag (None = native -1),
  flags, position quanta, middle field, optional byte, three optional direction/
  magnitude pairs, packed tail and final nine bits. Unsupported body prefixes
  publish with `body_ok=false`; successful level>1 reads still consume the three-bit
  component tail before publication.
- Five independent native oracles cover 4,096 direct publications; 491 publications
  from 256 keyframes (some bodies stop before the second component); 1,687 mixed
  chain, 3,429 harvest, 267,369 repair and 589 scan/819 full-frame raw-resync
  publications, each recovery family using 512 cases. All five focused Rust tests
  pass in 3.57s, including exact sequence, read/record ends, unsupported outcomes,
  world changes and JSON round trips where applicable.
- Direct non-predicted coverage includes 159 complete light bodies and 87 complete
  heavy bodies, nonzero flags, every inner tag, padded tails, all three suppression
  modes and levels 0..3. This verifies snapshot semantics in supported v41 contexts;
  it does not close the remaining inherited-profile-switch audit.
- Registered `ability-hook-{reads,chain,resync,harvest,repair}-v41` fixtures and
  their native harnesses. The shared hook recorder now installs all twelve typed
  component callbacks. Full parity still requires the remaining hook families,
  context/source inventory audit and positive complete captured CTF/VIP/bomb gates.
- Final ability-hook validation: 293 active Theater tests passed (29 ignored),
  37.36s. Library/test Clippy with warnings denied, WASM compilation, formatting,
  diff checks and generator syntax checks passed. Logs:
  `/private/tmp/halo-ability-hooks-suite.log`, `/private/tmp/halo-ability-hooks-clippy.log`,
  `/private/tmp/halo-ability-hooks-wasm.log`. All processes terminal; changes remain
  uncommitted on the requested branch. Next hook families to audit include held
  weapon, object-parent and unit-equipment publications.

### Held weapons, object parents and unit equipment publications

- Added `HeldWeapon`, `ObjectParent` and `UnitEquipment` variants to the ordered
  component observation stream. Held-weapon absence publishes both native
  `0xffffffff` IDs. Parent snapshots retain archetype/parameter, exact start/end,
  all raw branch fields and optional tails; no parent/player identity is inferred.
  Unit-equipment snapshots reuse the existing typed list and preserve closed
  entries with zero value/tail. Existing raw fields and unit references remain.
- Five native oracles cover 4,096 direct and 512 keyframe emissions, 2,044 mixed
  chain, 3,560 harvest, 273,136 repair and 589 scan/946 full-frame raw-resync
  emissions. Each recovery family has 512 cases. Initial focused tests pass in
  4.58s. The expanded direct fixture independently varies all six archetypes
  (0,11,35,37,40,99) against all five parameters (0..4), including the biped tail
  exception. It includes 731 absent held-weapon publications and 2,243 closed
  equipment entries. Final suite verifies this expanded fixture.
- Added `OBSERVER_STATUS.md`, an explicit 30-hook checklist. Sixteen hook streams
  (15 typed variants plus mobility) have direct native sequence evidence;
  remaining raw-field/scanner coverage is not mislabeled as publication parity.
  Profile switches and complete captured objective replay gates remain open.
- Registered all `object-hook-*` fixtures and native harnesses in the generator.
  The shared native test observer now installs all fifteen typed callbacks.
- Final object-hook validation: 298 active Theater tests passed (29 ignored),
  38.67s, including the expanded parent parameter/archetype fixture. Library/test
  Clippy with warnings denied, WASM compilation, formatting, diff checks and oracle
  generator syntax checks passed. Logs: `/private/tmp/halo-object-hooks-suite.log`,
  `/private/tmp/halo-object-hooks-clippy.log`, `/private/tmp/halo-object-hooks-wasm.log`.
  All processes terminal; changes remain uncommitted on the requested branch.

### Managed object, navpoint, objective and property publications

- Added four typed ordered hook variants covering 14 named fields. Boundary
  visibility preserves native bit order; RTPC and property gates preserve their
  variable value lists. Navpoint timers publish through ObjectiveHook, matching
  the native reader alias. Other fields do not fabricate managed-hook callbacks; separate hooks remain
  covered by their own audit.
- Native fixtures verify 4,096 direct reads (2,787 emissions), 512 emissions from
  256 actual archetype 10-13 keyframes, 1,223 chain, 2,754 harvest, 250,701 repair,
  and 395 scan/648 frame resync emissions. Recovery families each use 512 cases.
  Correction: these fixtures originally left the default-state guard closed.
  The default-hook audit below opens that guard and validates the real prefixes.
- Registered five harnesses and fixtures in generate_oracles.py. The observer
  checklist records 20 verified streams (19 typed plus mobility).
- Validation: 303 active Theater tests passed, 29 ignored, in 41.28s. Library/test
  Clippy with warnings denied, WASM compilation, formatting, diff checks and
  generator syntax checks passed. Logs: /private/tmp/halo-managed-hooks-suite.log,
  /private/tmp/halo-managed-hooks-clippy.log, /private/tmp/halo-managed-hooks-wasm.log.
  Full parser parity remains incomplete; remaining callbacks, profile/context and
  source inventory audits and complete captured objective replays remain open.

### Game-engine publications

- Added NativeGameEngineField and the ordered GameEngine observation with raw
  values and native presence. All five fields publish; current-round's inverted
  gate emits an empty, absent callback when closed. The separately captured round
  timer produces no duplicate GameEngine publication.
- Five independent native fixtures cover 4,096 direct reads (3,414 publications,
  including 301 absent rounds), 428 publications from 256 archetype-0 keyframes,
  1,376 chain, 2,875 harvest, 273,136 repair and 394 scan/802 frame resync
  publications. Each recovery family has 512 cases; exact ordered streams pass.
- Registered engine-hook harnesses and fixtures in generate_oracles.py, checking
  each new managed/engine install and run occurs exactly once. Twenty-one of the
  thirty observer streams now have native publication evidence (20 typed plus
  mobility). Existing strict aggregate diagnostic fixtures continue to pass.
- Final validation: 308 active Theater tests passed, 29 ignored, in 43.06s.
  Library/test Clippy with warnings denied, WASM compilation, formatting, diff
  checks and generator syntax checks passed. Logs:
  /private/tmp/halo-engine-hooks-suite.log,
  /private/tmp/halo-engine-hooks-clippy.log,
  /private/tmp/halo-engine-hooks-wasm.log.
- Next publication work: PlayerStateHook (11 fields, especially fixed 24-value
  malleable-property snapshots and desired-respawn-location's absent/default
  vector branches), ProbeHook (including property-name's separate publication),
  then movement, default-state and equipment hook routing. RecordMask, PosCapture
  and UnitRef still require their full suppression/selection audits. Inherited
  profile/custom-context and fallback integration, source inventory, and positive
  complete captured v41 CTF/VIP/bomb replay gates remain open.
- Changes remain uncommitted on simbleau/theater-experiments. The parser-parity
  goal is active and incomplete.

### Player-state, probe and equipment-state publications

- Added PlayerState with all eleven native fields. Malleable properties retain
  the fixed 24-value snapshot, including a zero placeholder after every closed
  gate. Desired respawn location distinguishes no location, default vector with
  an ID only, and transmitted XYZ/ID/level. Quantized vector reads are shared with
  existing consumers; raw field names and bit consumption remain unchanged.
- Player oracles: 4,096 direct reads, 3,220 publications, all levels 0-4; respawn
  location includes 158 closed gates, 67 default vectors and 68 transmitted
  vectors. Actual archetype-5 keyframes yield 404 publications; chain 1,423,
  harvest 3,028, repair 262,238, resync 442 scan/704 full frame. The five focused
  comparisons pass in 3.35s.
- Added Probe with four native component labels and the caller's archetype.
  Static splash messages publish only their unconditional R24 after consuming
  the entire body. Property-name publishes separately from managed-property
  hooks. Refreshed managed fixtures to retain that now-observed Probe stream.
- Probe oracles: 4,096 direct reads, 3,277 publications, all 20 combinations of
  four components and archetypes 0,4,13,47,99. Actual 4/13/47 keyframes yield 341
  publications; chain 1,706, harvest 3,042, repair 258,393, resync 471 scan/824
  full frame. The five focused comparisons pass in 3.14s.
- Added EquipmentState with all six native fields. Activated consumes its
  alternate reference body before publishing an absent value; creator also
  preserves absence. Four unconditional fields always publish as present.
- Equipment oracles: 4,096 direct reads, 3,512 publications including 541 absent
  values; actual archetype-37 keyframes yield 440 publications; chain 1,324,
  harvest 3,126, repair 262,880, resync 511 scan/837 full frame. Recovery families
  each use 512 cases. All fifteen new tests pass in the first full run.
- Registered all fifteen harnesses/fixtures in generate_oracles.py. The shared
  native observer installs 23 typed callbacks; with mobility, 24/30 streams have
  native publication evidence. Remaining six hooks and broader parity gates
  remain explicitly open in OBSERVER_STATUS.md.
- Initial full suite: 323 passed, 29 ignored, in 42.32s. Clippy identified four
  unnecessary Some/unwrap wrappers on unconditional equipment fields; removed
  them without changing publication semantics. Final validation follows below.
- Next default-state audit: MppHook's four publications originate in the fixed
  block in components/defaults.rs::multiplayer, not the TLV delta component in
  components/tlv.rs. EquipmentCreationHook covers both archetype 37 (reference
  and ability ID) and archetype 42 (reference only). Capture exact publication
  timing within default reads, including speculative keyframes and recovery.

- Final validation after cleanup: 323 active Theater tests passed, 29 ignored,
  in 41.70s. Library/test Clippy with warnings denied, WASM compilation,
  formatting, diff checks, generator syntax, manifest JSON and unique callback/
  oracle registration checks passed. Logs:
  /private/tmp/halo-player-probe-equipment-suite-final.log,
  /private/tmp/halo-player-probe-equipment-clippy-final.log,
  /private/tmp/halo-player-probe-equipment-wasm-final.log.
  All processes terminal; changes remain uncommitted on the requested branch.
  Full parser parity remains incomplete and the goal remains active.

### MPP and equipment-creation default-state publications

- Added Mpp (four native fields) and EquipmentCreation (reference/ability ID)
  observations. Publications occur at their native read points: MPP tail-name
  publishes before the remaining tail reference/fraction. Closed gates publish
  value zero with presence false. Creation hooks cover equipment default 37 and
  ground-weapon default 42. The TLV delta component remains a different grammar.
- Direct oracle covers 3,008 default reads across 47 archetypes with varied MPP
  widths: 2,048 MPP and 192 creation publications. The New-chain oracle covers
  all eight MPP-bearing archetypes (35-40,42,43): 512 cases, 4,556 total callbacks
  including 4,096 MPP and 384 creation callbacks. The 512 repair cases preserve
  1,788,954 callbacks, including 1,329,664 MPP and 124,510 creation callbacks.
- New-frame fixture compares 512 cases in each of NextBound, ChainWalk and raw
  resync. NextBound has 1,296 ordinary callbacks and no default callbacks;
  ChainWalk has 3,545 callbacks (1,964 MPP/185 creation); raw resync has 2,569
  (1,620 MPP/153 creation). Exact records and ordered publications are compared,
  including truncated payloads and leading noise. Harvest world state stays
  unchanged; resync callback streams remain caller-visible.
- Coverage correction: earlier managed/player/probe/equipment keyframe harnesses
  selected actual archetypes but set default_guard=0. Their component-loop
  evidence was valid; prior claims about exercising default prefixes were not.
  Corrected all four harnesses plus the new default fixture to set default_guard=1
  and write components_guard at the native-measured default end. Expanded
  equipment keyframes now contain 1,024 MPP and 512 creation callbacks in
  addition to 440 equipment-state callbacks. Default keyframes add 736 MPP and
  69 creation callbacks (1,147 total observations across 256 cases).
- The native keyframe oracle must use decode_native_keyframe_record for padded
  payloads. A random TLV length initially compared an 8,389,866-bit native end
  against the bounded reader's 8,162-bit truncated end. Native-compatible reading
  matched; the bounded public API is unchanged. Structured TLV direct inputs
  replace repeated random maximum-length bodies; padded keyframe cases remain.
- Registered five default-hook harnesses and fixtures. Refreshed 57 observer
  fixtures after new callbacks exposed speculative default reads in older chain,
  repair, harvest, frame and resync scenarios. Strict assertions were retained.
  Initial five focused tests passed in 85.26s before the guard-expansion audit;
  final full validation of the corrected guards and refreshed snapshots follows.

- Final validation: 328 active Theater tests passed, 29 ignored, in 40.63s,
  including corrected default guards in managed/player/probe/equipment keyframes
  and all refreshed speculative observer snapshots. Library/test Clippy with
  warnings denied, WASM compilation, formatting, diff checks, generator syntax,
  manifest JSON and unique native callback checks passed. Logs:
  /private/tmp/halo-default-hooks-suite-final.log,
  /private/tmp/halo-default-hooks-clippy.log,
  /private/tmp/halo-default-hooks-wasm.log.
  All processes terminal. Changes remain uncommitted on the requested branch.
- Observer coverage is now 26/30 streams (25 typed plus mobility). Remaining:
  EtatMouvementHook slot attribution/suppression, RecordMaskHook selection and
  CaptureDirs gate, and the full PosCaptureHook/UnitRefHook routing/suppression
  audits. Native movement publishes the reader's accumSlot and is disabled by
  all three suppression helpers; this differs from ordinary component hooks.
  Profile/custom-context and fallback integration, source inventory and positive
  complete captured v41 CTF/VIP/bomb replay gates still prevent full-goal completion.

### Movement-state snapshots and default production routing

- Added MovementState with seven native component identifiers, the explicit
  reader accumulator slot and native fixed-length values. Active ability emits
  immediately after its tag, and unit control before its reference tail.
  Slide/posture retain absent fields as zeros; dynamic velocity publishes under
  the native non-dynamic hook label. No bit is reread to obtain a snapshot.
- decode_native_component_with_movement takes an optional observer slot; None
  disables only movement publications. Reader capture is opt-in, so speculative
  paths remain silent until their exact native enablement is audited. Production
  traversal supplies its existing capture_slot: Delta updates it, New inherits
  the last slot (or zero), and rejected admissions do not publish.
- The production movement scanner now consumes these typed snapshots instead of
  reconstructing values from raw component fields. Existing semantic transition
  and jump policies are unchanged.
- Direct native oracle: 4,096 reads, 711 movement callbacks across all seven fields
  and 667 slot values, including 0xffffffff; all three native suppression modes
  emit zero movement callbacks. Corrected a native dispatch mismatch: the
  unsupported biped-spartan-ability alias no longer consumes a body; the canonical
  -component name is supported. Slide and mobility aliases remain supported.
- Production oracle: 512 cases, 8,388 total callbacks including 2,326 movement
  callbacks, all seven fields and slots 0/50/51. It covers New before any Delta,
  New following each of two Delta slots, unbound admission and truncated data.
  Initial fixture used native's default 13-bit IDs against an 11-bit writer;
  corrected to cfg.IDLowBits=11 before accepting positive production evidence.
  Both focused comparisons pass in 0.51s.
- Registered movement-hook-reads and movement-hook-production harnesses/fixtures.
  Full suite: 330 passed, 29 ignored, in 46.57s; library/test Clippy with warnings
  denied, WASM compilation, formatting and diff checks passed. Logs:
  /private/tmp/halo-movement-hooks-suite.log,
  /private/tmp/halo-movement-hooks-clippy.log,
  /private/tmp/halo-movement-hooks-wasm.log.
- Movement hook is not yet counted as fully verified: inference/harvest/resync
  and keyframe routing remain. Native ScanFrameTargets suppresses movement for
  trials/confirmation and enables it for accepted rereads. Raw scanForTargetDelta
  is live when directly called, but DecodeFrameResync suppresses its trials and
  enables the accepted TryDeltaAt reread. Its new main reader after resync resets
  the accumulator slot to zero. Chain inference and repair suppress movement
  throughout trials; production locators suppress it too. Preserve these exact
  distinctions when wiring the remaining opt-in call sites.

- Captured movement regression also passed: movement_scan_matches_production_films
  explicitly run with --ignored, 139.64s. Bazaar, Aquarius, Bandit Evo and ranked
  Oddball exactly match native movement records and every counter. Bandit retains
  181,689 records/5,994 transitions/471 derived jumps; Oddball retains 321,862
  records/11,601 transitions/1,160 derived jumps. Log:
  /private/tmp/halo-movement-hooks-corpus.log. All processes are terminal.
- Changes remain uncommitted on simbleau/theater-experiments. Observer completion
  stays at 26/30 until the remaining movement routing/suppression cases pass;
  the overall v41 parser-parity goal remains active and incomplete.


### Movement recovery routing checkpoint

- Added contextual bound-record reads. Known inference Deltas update the movement
  slot, New records inherit it, and speculative unbound inference leaves it alone.
- Harvest trials and successor confirmation suppress movement. Accepted rereads
  publish it. Direct raw target scans publish candidate callbacks, including
  rejected candidates; the scan inside full resync suppresses them and publishes
  only the accepted reread. Resync resets the sequential reader slot to zero.
- Added four 512-case native movement fixtures: harvest (357 movement callbacks,
  all seven fields), raw resync (753, all seven fields), chain and repair (zero
  movement callbacks, with ordinary callback comparisons retained).
- Added a 512-case inference fixture: 2,327 movement callbacks and two successful
  inferred records, toggling view admission and chain inference. It compares
  ordered callbacks, record IDs/end positions, overall cursor, inferred count and
  termination. Focused inference comparison passes.
- The first broad run found three older observer fixtures without movement
  callbacks installed (ability harvest/resync and mobility harvest). Their native
  harnesses now use the movement-enabled recorder; regenerated the three fixtures.
  No callback comparisons were weakened.
- Added a dedicated 512-case recovery-slot oracle with New records before Delta,
  after Delta and after recovery from an unbound Delta. Native emits 913 movement
  callbacks at slot zero, 980 at slot 50 and 427 at slot 51. This explicitly tests
  reader replacement rather than inferring reset coverage from Delta-only scans.
- Generator registers all new fixtures and the movement-enabled shared recorder
  wrapper. Production code remains within the theater module and uncommitted.
- Final validation results for this checkpoint are recorded below. Overall goal
  remains incomplete; keyframe movement routing, the three remaining observer
  audits, contextual/fallback integration and positive full objective captures
  still require work. Optional inference repair integration is not claimed here.

- Final broad suite: 335 passed, 29 ignored, 45.28s. After adding the isolated
  reader-reset test (test-only change), all eight focused movement comparisons
  passed in 2.63s, including exact resync slot inheritance and record endpoints.
  WASM compilation, formatting, generator Python compilation and diff checks pass.
  Logs: /private/tmp/halo-movement-recovery-suite-final.log,
  /private/tmp/halo-movement-recovery-all-focused.log,
  /private/tmp/halo-movement-recovery-wasm.log. Final library/test Clippy with warnings denied also passed
  (/private/tmp/halo-movement-recovery-clippy-final.log).


### Keyframe movement and record-mask publication checkpoint

- Native WalkKeyframeFullState creates a fresh reader and does not set its capture
  slot from the keyframe ID. Both Rust full-state reader policies now emit movement
  at slot zero. Regenerated the ten hook-read/keyframe fixtures with movement
  capture installed for keyframe calls, leaving direct step observers unchanged.
  The movement keyframe oracle contains 340 positive callbacks across all seven
  fields, all at slot zero. Full Theater suite passed: 336 tests, 29 ignored,
  40.79s; /private/tmp/halo-movement-keyframes-suite.log.
- Implemented RecordMaskHook publication as BipedPositionStream.record_masks with
  source packet, component indices and payload-relative after_position_bit.
  Source coordinates retain access to the exact payload without copying it per
  record. Publications occur after saturation rejection, before isolation/speed
  filtering, and only when capture_dirs is enabled. Disabling capture also disables
  companion output. Existing Rust companion-capture default remains true; explicit
  false matches native CaptureDirs=false (native zero-valued option is false).
- Added the native record-mask-hook generator and fixture: 256 cases, 2,647 mask
  callbacks, 103 cases where later filters remove already-published records.
  Tests compare callback order, masks, offsets, original payload bytes, filtered
  timestamps and serde round trips; capture-disabled outputs are empty. Includes
  dynamic orientation, saturated vectors, sparse masks, isolation and speed gates.
  Focused comparison passes in 0.62s; /private/tmp/halo-record-mask-focused.log.
- RecordMaskHook is now verified for supported scan contexts. Observer completion
  is 27/30: movement still needs generic record/view entry-point auditing, and
  PosCaptureHook/UnitRefHook still require the complete routing/suppression audit.
  Full parser parity, profile/fallback integration, source inventory and positive
  complete objective-film evidence remain incomplete. No commits or pushes.

- Final combined validation passed: 337 Theater tests, 29 ignored, 40.87s;
  library/test Clippy with warnings denied; WASM library compilation; cargo fmt,
  git diff check and generator Python compilation. Logs:
  /private/tmp/halo-observer-mask-suite.log,
  /private/tmp/halo-observer-mask-clippy.log,
  /private/tmp/halo-observer-mask-wasm.log. All validation processes are terminal.
- Film.biped_positions retains the stream, so record-mask observations are included
  in Film serialization via the existing replication enrichment path.
- Next movement routing audit: native DecodeFrameRecords sets capture slot from
  every non-End record, including New, unlike production/inference New inheritance.
  Rust bounded decode_entity_view currently suppresses movement. Preserve the
  distinction rather than changing the shared record reader globally. Native
  direct-component default context and internal speculative users also need
  classification before changing decode_native_component's default suppression.


### Generic record and direct native movement routing checkpoint

- Generic DecodeFrameRecords assigns every non-End record its own slot, including
  New. Bounded decode_entity_view now does so in its generic read policy, without
  changing production/inference inheritance or speculative native-policy reads.
- Added 512 native generic-record cases: 2,466 movement callbacks at slots
  100/50/101/51/102, all seven types, New-before-Delta and New-after-Delta, plus
  missing bindings. Compares all ordered callbacks, record IDs/end bits, final
  cursor and complete/failure status. Input bodies are complete; this oracle does
  not claim native zero-padding parity for the deliberately bounded generic API.
- Fresh decode_native_component now emits movement at reader slot zero. Extended
  direct native fixture with 344 fresh-reader calls, 239 positive movement
  callbacks, compared with the default API as well as explicit contextual reads.
  Existing bounded field-oriented helper and internal attempts retain their prior
  suppression behavior; callers seeking native callback streams use native APIs.
  The two production callers of decode_native_component consume managed-property
  and navpoint-radius components (no movement emissions), so no speculative caller
  context changed. No global change to shared production/New/repair slot handling.
- All nine focused movement tests passed in 2.58s. Generator registers the new
  generic fixture and refreshed direct fixture. Full final validation follows.
- Movement observer is verified for tested v41 contexts: observer count is now
  28/30. Profile/context integration still has its own audit. Remaining observer
  work is UnitRefHook and PosCaptureHook publication/suppression, not just values.
- Next audit evidence: native neutraliserCaptures suppresses position, unit refs
  and movement (repair, single-step inference, validated resync). The narrower
  neutraliserCapturePosition suppresses position/movement but preserves unit refs
  (chain inference, harvest trials and internal raw resync scans). Rust's existing
  references vectors retain raw decoded reference fields but are not yet a
  separate ordered callback stream in diagnostics. Preserve raw references while
  adding exact callback gating; do not delete fields to simulate suppression.

- Final validation: 338 Theater tests passed, 29 ignored, 42.36s. Library/test
  Clippy with warnings denied, WASM library compilation, cargo fmt, git diff check,
  and generator Python compilation all pass. Logs:
  /private/tmp/halo-movement-generic-suite.log,
  /private/tmp/halo-movement-generic-clippy.log,
  /private/tmp/halo-movement-generic-wasm.log. All processes are terminal.
- Changes remain uncommitted on simbleau/theater-experiments; full goal remains
  active and incomplete, with objective-film corpus and integration gates open.


### Unit-reference publication checkpoint

- Added FilmComponentObservation::UnitReference in the same ordered stream as
  component callbacks, emitted at the native reference primitive's completion.
  Existing raw NativeUnitReference vectors remain unchanged and separately exposed.
- Added NativeComponentCapture with independent reference/movement switches and
  decode_native_component_with_capture. Direct native observer fixtures now
  explicitly record suppression rather than inferring it from movement slots.
- Repair, single-step inference and validated resync filter reference publications;
  chain inference, harvest and raw resync preserve them as native does. Successful
  repaired records also suppress their callback stream, retaining raw references.
  No decoded fields are deleted to model observer suppression.
- Shared native recorder now installs 26 callbacks, with movement added by its
  contextual wrapper. Regenerated 66 existing observer fixtures. The default-read
  harness chains its raw-reference recorder with the shared callback recorder so
  both remain independently validated (installing one over the other would have
  erased the raw-reference evidence; this was fixed before accepting fixtures).
- Added unit-reference-hook reads/chain/harvest/resync/repair oracles and registered
  generation. Counts include all three encodings: reads/keyframes 6,022 callbacks,
  chain 3,514, harvest 4,960, raw resync 1,269, repair zero. Comparisons cover full
  callback order, all raw reference fields, record endpoints and existing ordinary
  publications. The shared movement/frame/default families also validate reference
  routing in production, generic records, inference and keyframes.
- Existing broad suite passed 338 tests (29 ignored), 44.83s. All five dedicated
  reference comparisons passed, 3.41s; no production edits after that broad run.
  Logs: /private/tmp/halo-unit-reference-suite.log and
  /private/tmp/halo-unit-reference-focused.log. Final static checks follow.
- UnitRefHook is verified in supported v41 contexts; observer count is now 29/30.
  PosCaptureHook remains: decoded fields plus later reconstruction do not prove
  native callback timing/routing. Its context must carry map ranges and quantum,
  which wire-only PositionEncoding does not contain. Native accumulation is
  reader-local and normally unset; preserved explicit accumulation API still needs
  its contextual equivalence maintained. Do not silently use universal map bounds.
- Overall parity remains incomplete; inherited profile/fallback integration, source
  inventory and full positive objective-film evidence are still open. Changes
  remain uncommitted on simbleau/theater-experiments.

- Final static checks passed: library/test Clippy with warnings denied, WASM library
  compilation, cargo fmt, git diff check and generator Python compilation. Logs:
  /private/tmp/halo-unit-reference-clippy.log and
  /private/tmp/halo-unit-reference-wasm.log. All validation processes are terminal.


### Position callback primitive checkpoint

- Added explicit NativePositionCapture (map bounds, quantum, slot, optional mutable
  accumulator, and emission switch) to NativeComponentCapture. Wire-only precision
  remains separate; no map bounds or quantum are invented for existing callers.
- Added ordered Position observations with IEEE-754 vector bits. Emission occurs
  after a complete vector read and before finite/handle/reference tails, matching
  native i0. Baseline emits only a known accumulator value; absolute paths seed;
  deltas add only to existing seeds with a world, or emit relative values without
  one. Suppressing emission still updates an installed accumulator, as native does.
- Extended the existing 1,024 x 12 direct-read oracle to record callback interleaving
  with unit refs. It verifies 824 position publications across all five kinds and
  213 reference publications, plus every record end and world update. Existing
  reconstruction APIs independently match the same values/world states.
- Added 512 x 16 reader-sequence oracle over i0, simulation, posture, mobility,
  control-context and unit-control reads; varied map ranges/quantum, slots, runtime
  grammar gates, optional accumulation and emission suppression. Native publishes
  316 positions (all five kinds) and 2,198 unit refs, and performs 15 accumulator
  updates while position emission is suppressed. Exact per-step world snapshots,
  callback order and endpoints match. Both focused tests pass in 0.34s.
- Correction: initial inspection of Rust's shared absolute_payload helper suggested
  simulation/media-frame reads would inherit and publish the prior i0 stamp. The
  native sequence oracle disproved that (first mismatch case 1/8), and native call
  sites confirm the shared non-i0 vector reader does NOT publish positions. Split
  pure vector consumption from i0 publication; the regression asserts positive
  non-i0 quantized reads with zero position publications. No stale-stamp claim is
  retained. The current implementation matches native rather than that hypothesis.
- All existing frame/keyframe/recovery Reader constructors remain without position
  capture until their callers provide the explicit map/quantum context. Next work
  must wire and verify those contexts and suppression rules; do not count the
  position observer complete from direct-reader proof. Observer count stays 29/30.
  Full objective-film, profile/fallback and source-inventory gates remain open.
- Logs: /private/tmp/halo-position-publication-focused-final.log. Final broad and
  static validation results follow. Changes stay uncommitted on the film branch.

- Final validation passed: 344 Theater tests, 29 ignored, 47.36s; library/test
  Clippy with warnings denied; WASM compilation; formatting/diff checks and
  generator Python compilation. Logs: /private/tmp/halo-position-publication-suite.log,
  /private/tmp/halo-position-publication-clippy.log,
  /private/tmp/halo-position-publication-wasm.log. All processes are terminal.


### Position callback frame-routing checkpoint

- FrameEncoding now carries optional exact map-bound/quantum capture context;
  V41FilmProfile supplies its configured map and quantum. Generic records assign
  every record's own slot, including New. Production/inference preserve the native
  inherited New slot; recovery recreates the reader at slot zero. Bounded and native
  keyframe APIs support configured capture and use fresh slot zero.
- Ten native fixture families exercise frame/recovery routing: production and
  inference each publish 683 positions in 512 cases; generic publishes 811;
  recovery-slot cases publish 679; 256 keyframe cases publish 28 at slot zero;
  harvest publishes 38; full resync publishes 168. Raw target scans publish no
  caller positions because native replaces that callback with its local collector.
  Chain, repair and validated resync suppress positions. The validated-resync input
  produces 699 callbacks when scanned unsuppressed, proving positive suppression.
- All 11 focused position-hook tests passed in 2.77s. The broad suite passed 351
  tests (29 ignored, 48.42s) before the last three test-only suppression additions.
  Final library/test Clippy, WASM, formatting/diff and generator compilation passed.
  Logs: /private/tmp/halo-position-routing-focused-final.log,
  /private/tmp/halo-position-routing-suite.log,
  /private/tmp/halo-position-routing-clippy-final.log,
  /private/tmp/halo-position-routing-wasm.log. All those processes are terminal.
- Subsequent caller audit found march's accepted record walk used the suppressed
  low-level decoder. It now supplies each record's slot independently, preserving
  New and Delta publication. Two independent 512-case native march fixtures match
  811 position callbacks and 2,466 movement snapshots, ordered with other callbacks,
  record boundaries and unchanged caller world. Both Rust tests pass in 0.12s.
  Logs: /private/tmp/halo-march-hooks-native.log and
  /private/tmp/halo-march-hooks-rust.log.
- March strict/fallback locators suppress movement only in native. Their offset-only
  Rust path optimizes away non-slot-123 trials and repeated generation-check reads.
  A separate observed locator API preserves those publications
  without changing the offset-only path. Broader integrated caller/profile/fallback
  and complete objective-film/source-inventory gates remain open; observer count
  remains 29/30 until the remaining routing audit is closed.

- Observed march locator oracle passed: 512 cases, 668 positions (396 from
  non-signature slots), 688 probes, 170 unit refs, 101 ability callbacks and zero
  movement snapshots. All 418 selected offsets and every callback match native;
  the optimized offset-only API selects the same offsets. Serialization round trip
  also passes. Logs: /private/tmp/halo-position-locator-native.log and
  /private/tmp/halo-position-locator-rust.log (Rust 0.18s).

- Next integration audit targets confirmed by source inspection: replication's
  standalone march encoding and kill starting calibration still use no position
  capture; inherited precision/capture-map consistency needs native validation.
  High-level march/kill callers use the offset-only locator and do not retain its
  trial stream. Do not infer full Film observer parity from the new lower-level
  observed locator API alone.

- Final march-routing validation: 357 Theater tests passed, 29 ignored, 48.15s;
  library/test Clippy with warnings denied, WASM compilation, formatting, diff
  checks and generator Python compilation passed. Logs:
  /private/tmp/halo-march-routing-suite.log,
  /private/tmp/halo-march-routing-clippy.log,
  /private/tmp/halo-march-routing-wasm.log. All processes are terminal. Changes
  remain uncommitted on simbleau/theater-experiments; the full port is incomplete.


### Inherited position capture precision and native context ownership

- Fixed map precision installation for an existing PositionCaptureEncoding: its
  region, axis widths and region-index width now follow the installed native
  WorldObject descriptor. The existing coordinate bounds and delta quantum stay
  unchanged, because native Movement.Range and DeltaQuantum are independent of
  WorldObject. Previously a changed region could suppress valid publications.
- Expanded the independent 512-case native world-precision fixture with exact
  before/after capture contexts and varied float32 ranges/quantums: 128 successful
  installations (85 change the region), 384 zero-axis fallbacks. Rust matches
  every field, original calibrated flags, simulation gate and fallback count.
  The focused test passes in 0.02s. Native/Rust logs:
  /private/tmp/halo-world-capture-context-native.log and
  /private/tmp/halo-world-capture-context-rust.log.
- Correction to the previous integration concern: native CadreDeBalayage copies
  ProfilDeBalayage but leaves FrameConfig.Obs nil, even when FilmContext carries
  installed hooks. NouveauLecteur DOES inherit that observer. The new independent
  512-case TestHaloRustFilmContextObservers confirms both ownership rules and
  inherited quantum/region. Default high-level ScanMarchFacts therefore does NOT
  publish locator callbacks. Rust's offset-only high-level use is not itself a
  missing native publication. Explicit observed locator/frame APIs retain the
  instrumented behavior. Harness is registered in generate_oracles.py; log:
  /private/tmp/halo-film-context-observers-native.log.
- Remaining concrete custom-profile gaps: Grammaire.CorpsAncrageCapacite and
  CorpsActionMobilite, plus Mouvement.MobilityActionExtraBits, are not independently
  represented in Rust's inherited component context. Current ability/mobility
  readers implement their production-enabled defaults. Native mobility still
  reads its reference before the optional body/skip; anchor body suppression must
  preserve its outer tag, level>1 tail and BodyWalked/BodyOK observer state.
  These require independent sequence and wrapper fixtures before claiming custom
  profile parity. Full objective-film and source-inventory gates remain open.

- Final inherited-capture checks passed: focused 512-case native parity fixture,
  library/test Clippy with warnings denied, WASM compilation, formatting/diff and
  generator syntax checks. Logs: /private/tmp/halo-world-capture-context-clippy.log
  and /private/tmp/halo-world-capture-context-wasm.log. All processes are terminal.
  The preceding broad suite remains 357 passed/29 ignored; it was not rerun for
  this focused installer correction. Work remains uncommitted and goal incomplete.


### Inherited ability and mobility body switches

- Added serializable ComponentBodyPolicy to the shared precision/component context.
  Production defaults enable both bodies and skip zero bits; default values are
  omitted from serialized PositionEncoding to preserve existing output shape.
  Custom profiles carry independent ability-anchor/mobility-body gates and the
  signed mobility calibration skip through direct, keyframe and frame readers.
- Disabled tag-3 ability anchors preserve the outer tag, level-dependent three-bit
  tail and BodyWalked=false/BodyOK=false publication. Disabled active mobility
  bodies still read/publish the optional reference before skipping positive extra
  bits; negative/zero skips consume nothing. Skipped positive bytes retain source
  bit positions and raw values in bounded-width fields.
- Native direct/keyframe fixture: 256 profiles, 4,096 component reads, 256 full-state
  keyframes, varied precision/tails and truncation/padding. Comparisons include
  endpoints, status, callbacks, references, mobility flags and absolute-index maps.
  Positive controls assert 978 disabled tag-3 anchor publications and 312 active
  disabled-mobility reads with positive skips. All direct comparisons matched on
  the first run; the reused helper's unrelated absolute-index-count assertion was
  corrected because these two components do not consume indexed absolute headers.
- Independent generic and inference frame fixtures each cover 512 cases, including
  New/default prefixes, unbound slots, incomplete records and inherited reader slots.
  Generic publishes 1,097 ability snapshots and 1,234 movement snapshots; inference
  publishes 1,034 and 1,170 respectively. All callbacks and record boundaries match.
  Three focused tests pass (1.38s). Harnesses and fixtures are registered in the
  oracle generator. Logs: /private/tmp/halo-body-profile-native.log,
  /private/tmp/halo-body-profile-frames-native.log,
  /private/tmp/halo-body-profile-focused-final.log.
- Remaining profile audit includes calibrated i0 skipping, custom New terminal
  bits/default-state routing, generic calibrated/stub widths, baseline scopes and
  their recovery-wrapper propagation. Full objective-film and source-inventory
  completion remain unproven. This checkpoint does not declare full parser parity.

- Final body-profile validation: 360 Theater tests passed, 29 ignored, 48.23s;
  WASM compilation and library/test Clippy with warnings denied passed. Clippy's
  is_multiple_of style finding was corrected without changing skip semantics.
  Formatting/diff checks and generator syntax check passed. Logs:
  /private/tmp/halo-body-profile-suite.log,
  /private/tmp/halo-body-profile-wasm.log,
  /private/tmp/halo-body-profile-clippy-final.log. All processes terminal.
  Changes remain uncommitted on simbleau/theater-experiments. Goal incomplete.


### Calibrated i0 position skip profile

- Added optional calibrated_skip to PositionEncoding, false/omitted by default.
  It ports native Movement.CalibratedSkip exactly: read the first prediction bit,
  consume a total of 47 or 101 bits, and return before ordinary i0 grammar. Raw
  skipped bits remain in named bounded-width fields. This is an explicitly chosen
  map-specific calibration path, not a claim that v41 positions have fixed widths.
  It emits no coordinate/reference callback and does not mutate an accumulator.
- Independent 512 x 16 reader-sequence fixture verifies all endpoints, status,
  callback order and world snapshots while mixing calibrated and regular profiles,
  emission gates, accumulation and other vector-consuming components. It includes
  348 short skips, 338 long skips, 190 normal position callbacks and 2,189 reference
  callbacks. The fixture schedule was adjusted to avoid correlating normal i0 with
  emission suppression; normal position publication is positively represented.
- Independent 512-case generic/inference fixtures compare inherited skipping across
  New, Delta, defaults and failures. Normal branches retain 400 generic and 347
  inference position callbacks. The 256-case full-state keyframe fixture also checks
  4,096 direct component reads; 12 normal keyframe position callbacks remain.
  Every calibrated read is checked against native, not inferred from fixed widths.
- Four harnesses and compressed fixtures are registered in generate_oracles.py.
  Native logs: /private/tmp/halo-calibrated-position-native-final.log and
  /private/tmp/halo-calibrated-position-keyframes-native.log. The three initial
  focused Rust tests passed in 0.61s; final four-test/broad results follow.
- Remaining profile work includes custom New terminal bits/default-state routing,
  calibrated and stub component-width maps, baseline scopes and recovery combinations.
  Full source inventory, captured objective-film parity and completion audit remain
  open. Changes remain uncommitted on the film-experiments branch.

- Final calibrated-position validation: all four focused tests passed (1.10s),
  including 976 matched absolute-index increments from ordinary keyframe/direct
  branches. Full Theater suite: 364 passed, 29 ignored, 45.73s. Library/test Clippy
  with warnings denied, WASM compilation, formatting/diff checks and generator
  syntax check passed. Logs: /private/tmp/halo-calibrated-position-focused-final.log,
  /private/tmp/halo-calibrated-position-suite.log,
  /private/tmp/halo-calibrated-position-clippy.log,
  /private/tmp/halo-calibrated-position-wasm.log. All processes are terminal.
- Next concrete New-record audit: native TraverseEntity skips positive
  BitsDeQueueRecordNew only after a successful component walk. Non-biped defaults
  obey DeserEtatParArchetype and otherwise use the FrameConfig.NewDefaultStateBits
  fallback; biped defaults are unconditional. Rust currently has neither custom
  terminal-bit consumption nor this separate default routing/fallback context.
  The full port remains incomplete; no commit or push performed.


### New-record profile routing and terminal bits

- Added serializable NewRecordEncoding to FrameEncoding: independent
  deserialize_defaults, fallback_default_bits and terminal_bits. Defaults preserve
  production behavior and are omitted from serialized output. Biped defaults remain
  unconditional; Delta/Delete/End do not consume New tails. Positive terminal widths
  are consumed only after successful component traversal. Skipped bytes retain
  exact source positions and raw values in bounded-width fields.
- Corrected the native default routing distinction exposed by oracle case 20:
  known zero-bit stubs are absent from defaultStateDeserByTI, so a configured
  fallback width applies to them too. Added the exact native deserializer inventory;
  ordinary zero-width stubs remain distinct from unported/default-policy fallback
  diagnostics. Generic bounded records now also follow native zero-bit fallback
  behavior for the three unported default archetypes (23/41/44).
- Independent generic/inference oracles each cover 512 cases across ten archetypes,
  empty/default/fallback/biped bodies and unsupported components. Each contains 408
  positive New tails, 58 failed bodies with positive tails (tails must stay unread),
  20 enabled empty-default cases with nonzero fallback widths, and 25 biped cases
  with archetype-default routing disabled (biped still reads its body). Exact
  endpoints and callbacks match: 2,110 EMP timers, 1,640 MPP, 676 references and
  211 equipment-creation publications per fixture.
- Recovery fixture covers 512 cases x three modes (two harvest confirmations and
  full resynchronization), with variable MPP widths, extra prefixes, truncated
  buffers and custom New settings. Ordered streams and selected records match:
  3,518 EMP timers, 2,052 MPP, 867 references and 176 equipment creations.
  All three focused Rust tests pass in 0.62s. Harnesses/fixtures are registered.
  Logs: /private/tmp/halo-new-record-profile-native.log,
  /private/tmp/halo-new-record-profile-recovery-native.log,
  /private/tmp/halo-new-record-profile-focused-final.log.
- Custom calibrated/stub width maps, baseline scopes, remaining profile propagation,
  source-inventory audit and positive complete objective-film comparisons remain
  open. This checkpoint does not establish full parser parity.

- Final New-record validation: 367 Theater tests passed, 29 ignored, 48.84s;
  library/test Clippy with warnings denied, WASM compilation, formatting/diff
  checks and generator syntax check passed. Logs:
  /private/tmp/halo-new-record-profile-suite.log,
  /private/tmp/halo-new-record-profile-clippy.log,
  /private/tmp/halo-new-record-profile-wasm.log. All processes are terminal.
  The first native comparison failures (generic/inference case 20 and recovery
  65/2) were caused by zero-bit stubs ignoring custom fallback widths; corrected
  implementation passes the unchanged native fixtures. Goal remains incomplete;
  no commit or push performed.


### Calibrated component widths and unported stub tails

- Added ComponentWidthOverrides to FrameEncoding with calibrated replacement and
  extra-stub maps. Empty maps are omitted in serialized output. A calibrated entry
  bypasses the component reader and its callbacks entirely; a stub applies only
  after an unsupported result and preserves known prefix fields/publications.
  Zero-width entries are meaningful. Corruption checks follow either accepted
  path; calibrated entries take precedence over stubs. Raw skipped bits retain
  names, source ranges and values, without claiming decoded gameplay semantics.
- Shared policy helper is used by generic/production/inference record bodies,
  chain delta trials and configured native/bounded full-state keyframes. Direct
  component reads remain unaffected by traversal-only override maps, matching
  native. Existing inferred_stub field names remain stable for repair consumers.
- Repair rejects an existing stub for the failed name. Trial redecode and downstream
  confirmation clear other preset stubs, as native does, while retaining calibrated
  replacements. The caller's maps remain immutable. Restored the chain trial's
  component count while consolidating its read path; broad validation follows.
- Native generic and inference fixtures each contain 512 cases, 52 zero-width
  calibrated entries and 85 overlapping calibrated/stub settings, with corruption
  gates and failed prefixes. Each matches 5,628 MPP, 3,143 unit refs, 414 EMP,
  207 movement and 202 ability publications, plus boundaries and returned records.
  Keyframe fixture covers 256 configured full-state records and 4,096 direct reads;
  it also validates 602 absolute-index increments and bounded/native equality.
- Native repair fixture has 512 cases, 94 successful repairs and 47 preset-name
  refusals. Selected widths, repaired boundaries, diagnostics, callbacks and world
  preservation match. All four focused tests pass in 3.49s. Harnesses and fixtures
  are registered in generate_oracles.py. Logs:
  /private/tmp/halo-width-overrides-native.log,
  /private/tmp/halo-width-overrides-repair-native.log,
  /private/tmp/halo-width-overrides-focused-final.log.
- Baseline scope, remaining custom-profile/caller propagation, source inventory,
  and positive complete objective-film validation remain open. These tests cover
  supported named component entries and nonnegative width overrides; arbitrary
  malformed/custom profile behavior is not implied. Full parser parity unproven.

- Final width-override validation: 371 Theater tests passed, 29 ignored, 49.27s;
  library/test Clippy with warnings denied, WASM compilation, formatting/diff and
  generator syntax checks passed. Logs: /private/tmp/halo-width-overrides-suite.log,
  /private/tmp/halo-width-overrides-clippy.log,
  /private/tmp/halo-width-overrides-wasm.log. All processes terminal.
- Next baseline-scope audit: native fullPrecisionGate is PorteeBaseline OR
  Movement.FullPrecision. Rust currently carries only full_precision, with reads
  in position.rs, orientation.rs, ability.rs and scene.rs. The independent baseline
  scope needs profile representation and positive native fixtures; production's
  false baseline default does not prove custom-scope parity. Goal incomplete;
  changes remain uncommitted on simbleau/theater-experiments.


### Independent baseline scope

- Added serializable baseline_scope to PositionEncoding, false and omitted by
  default. Vector paths now use the native fullPrecisionGate (baseline scope OR
  global full precision): i0, simulation/media frames, mobility vectors and flock
  position. The biped control-context field deliberately continues consulting only
  the global flag, matching the native exception.
- Independent 512 x 16 reader-sequence oracle checks callback order, all boundaries,
  status and accumulator snapshots. Positive baseline-only controls include 468
  control-context reads of three bits and 469 raw flock vectors of 96 bits. The
  complete fixture retains 279 position and 1,960 reference callbacks.
- Generic/inference frame fixtures each cover 512 profiles and retain 400/347
  ordinary position callbacks. The keyframe fixture covers 256 full-state records
  and 4,096 direct reads, including mobility, flock and control-context components.
  When adding mobility to the keyframe fixture, its native recorder initially
  omitted EtatMouvementHook; that omission was corrected before final comparison.
  No parser semantics were changed to hide those native movement publications.
- Four harnesses and fixtures are registered in generate_oracles.py. Native logs:
  /private/tmp/halo-baseline-scope-native.log and
  /private/tmp/halo-baseline-scope-keyframes-native.log. Final validation follows.
- Next confirmed context gap: configured keyframe traversal currently passes a
  hardcoded simulation-complete=true into the width-policy helper, whereas native
  consumeByName consults Grammaire.SimStateComplet. Explicit keyframe simulation
  policy and its inherited Film callers need native gate/refusal fixtures. Other
  profile/propagation, source-inventory and complete objective-film gates remain
  open; full parser parity is not established.

- Final baseline-scope validation: all four focused tests pass (1.25s), including
  262 absolute-index increments and native keyframe streams of 102 movement,
  52 unit-reference and five position callbacks. Full Theater suite: 375 passed,
  29 ignored, 48.06s. Library/test Clippy with warnings denied, WASM compilation,
  formatting/diff and generator syntax checks pass. Logs:
  /private/tmp/halo-baseline-scope-focused-final.log,
  /private/tmp/halo-baseline-scope-suite.log,
  /private/tmp/halo-baseline-scope-clippy.log,
  /private/tmp/halo-baseline-scope-wasm.log. All processes are terminal. Changes
  remain uncommitted on simbleau/theater-experiments; full goal remains incomplete.

### Keyframe simulation completion policy

- FrameEncoding now carries optional keyframe_simulation_complete. Legacy reader
  calls retain true; configured keyframes use the supplied gate. Film replication
  passes inherited precision's simulation_complete, including zero-axis fallback.
- Native 256-case fixture covers 128 enabled completions, 32 disabled stops at
  component index 1 and 96 disabled completions through width overrides. Checks
  include callbacks, stop index, endpoint, round trip and sequential-table bindings.
- The first suite caught a generation-zero test header rejected by the sequential
  table. The native fixture now uses a valid generation; parser header validation
  was retained. Final suite: 376 passed, 29 ignored, 45.51s. Focused test, Clippy,
  WASM, format/diff and generator syntax checks passed. Captured Bazaar zero-axis
  complete document also matches native (8.14s).
- Logs: /private/tmp/halo-keyframe-simulation-suite-final.log,
  /private/tmp/halo-keyframe-simulation-focused-final.log,
  /private/tmp/halo-keyframe-simulation-clippy.log,
  /private/tmp/halo-keyframe-simulation-wasm.log,
  /private/tmp/halo-keyframe-simulation-zero-axis.log.
- Remaining gates: inherited/custom profile propagation, source inventory,
  positive complete v41 CTF/VIP/bomb documents and all-data publication audit.
  Full goal remains incomplete; changes remain uncommitted.

### Empty-name traversal overrides

- Native traverseComponentLoopFrom looks up calibrated/stub widths for every
  present registry entry, including an empty name. Frame traversal and configured
  keyframes now follow that ordering instead of rejecting/skipping the entry
  before override lookup. An unconfigured legacy keyframe reader retains its
  existing empty-slot behavior.
- Expanded the existing generic/inference (512 each) and keyframe (256 plus
  4,096 direct reads) native fixtures with empty names. Generic/inference each
  include 51 empty-name calibrated maps and 51 stub maps; keyframes include 25
  and 26 respectively, including zero-width and overlapping overrides.
- Initial focused validation exposed the same early rejection in bounded generic
  frames and bounded configured keyframes. Both now follow native dispatch and
  override semantics; final four width-override tests pass in 6.34s.
- Native log: /private/tmp/halo-empty-component-native.log. Focused log:
  /private/tmp/halo-empty-component-focused-final.log. Full validation follows.
- Final empty-name validation: 376 Theater tests passed, 29 ignored (47.80s).
  Library/test Clippy with warnings denied, WASM compilation, formatting and
  diff checks pass. Logs: /private/tmp/halo-empty-component-suite.log,
  /private/tmp/halo-empty-component-clippy-final.log,
  /private/tmp/halo-empty-component-wasm-final.log. All processes terminal.
  Updated the three relevant manifest entries with specific evidence; no full
  source-parity claim. Remaining acceptance gates above are still open.

### Native medal identity catalog

- Source inventory found medalname/table.go still pending: its lookup uses the
  exact (type_hint, medal_type) pair, unlike the existing SPNKr/CMS code catalog.
  native_medal_names.rs ports all 124 measured pairs and the unknown-pair refusal.
  Signed inputs outside the byte range remain unknown, matching native Lookup.
- NativeHighlightEvent::medal_name resolves only medal events. FilmHighlightStream
  retains aligned optional medal_names in the Film portable JSON; duplicate events
  keep separate entries. The broader existing medal catalog remains separate.
- Independent native oracle exhaustively enumerates -1..256 on both axes (66,564
  pairs) and records native Len. Rust tests compare every answer and exercise
  scanning/export/round-trip with a known pair, same-code unknown pair, non-medal,
  and duplicate event. Harness is registered in generate_oracles.py.
- Initial Rust test build exposed an i32/i64 mismatch in fixture chunk size; fixed
  the fixture to the actual FilmChunk schema. Final validation follows.
- Focused tests passed (0.18s), library/test Clippy and WASM compilation passed.
  Formatting, diff and generator syntax checks pass. Logs:
  /private/tmp/halo-medal-names-native.log,
  /private/tmp/halo-medal-names-focused-final.log,
  /private/tmp/halo-medal-names-clippy.log,
  /private/tmp/halo-medal-names-wasm.log.
- Next concrete source gap: killicon/killicon.go and neutral.go have no Rust
  resolver. Existing kill_damage_tags supplies their input labels, but does not
  resolve the native ordered NOM/GGGL/PORTEUR/BANQUE/CLASSE rules, sprite/weapon
  keys, rules/provenance tables, or neutral-death icons. Port these with native
  catalog and ambiguity/priority fixtures; do not equate labels with icon parity.
  Complete objective films and inherited-profile/source audits remain open.
- Final suite: 378 passed, 29 ignored (45.53s), recorded in
  /private/tmp/halo-medal-names-suite-final.log. All processes terminal. Manifest
  marks only medalname/table.go ported-v41 based on exhaustive lookup and export
  evidence. Full parser goal remains incomplete; branch changes uncommitted.

### Kill-feed icon resolution

- Ported killicon rule parsing, ordered resolution, catalog/provenance, sorted tag
  lookup and neutral-death sprite lookup in kill_icons.rs. The pinned TSV remains
  under the film reference directory. Exact ASCII capture scanners reproduce
  native RE2 patterns without an additional crate dependency.
- Precedence is name, first grenade-list match, unique carrier, unique long bank,
  unique short bank, class. Carrier +N (including +0) rejects uniqueness. Last
  duplicate rule wins. Only publishable labels get weapon icons; neutral-death
  classification intentionally checks class independently of label status.
- Native oracle covers the complete catalog (193 resolved tags, 265 neutral tags),
  rules, provenance, sorted tags and neutral sprite table; 1,024 synthetic rule/
  label cases and 13 parser cases cover ambiguity, precedence, duplicate rules,
  ASCII captures, invalid rules and header replacement. Focused comparison passes.
- FilmKillSourceResult now carries icons aligned with attribution kills/unclaimed
  deaths. This does not bypass the existing health/publication gate. Tests check
  alignment against native catalog answers for every label and JSON round trips.
  Generator registration includes the new killicon-package harness.
- Logs: /private/tmp/halo-kill-icons-native.log,
  /private/tmp/halo-kill-icons-focused.log. Broad/captured validation follows.
- Library/test Clippy with warnings denied and WASM compilation passed; format,
  diff and generator syntax checks passed. Captured regression is the existing
  full kill-source comparison for bandit/01-evo and ranked-arena/02-oddball, not
  positive CTF/VIP/bomb evidence. Logs: /private/tmp/halo-kill-icons-clippy.log,
  /private/tmp/halo-kill-icons-wasm.log, /private/tmp/halo-kill-icons-suite.log,
  /private/tmp/halo-kill-icons-captured.log.
- Retraction from the source-gap note: replay_kill_inputs.rs already implemented
  the neutral classification and image URLs. The missing portion was the native
  standalone Icon/NeutralSprites surface and the general rule catalog/resolver.
  Both replay and the new API now share neutral_death_sprite; existing replay
  image paths remain intact. Validation is rerunning after this consolidation.
- First broad suite passed: 379 tests, 29 ignored (48.86s). Captured comparison
  has matched Bandit's 122 kills; Oddball comparison is still running.
- Final consolidated suite passed: 379 tests, 29 ignored (50.97s), at
  /private/tmp/halo-kill-icons-suite-final.log. Final Clippy/WASM passed at
  /private/tmp/halo-kill-icons-clippy-final.log and
  /private/tmp/halo-kill-icons-wasm-final.log. Format/diff checks pass.
- Captured regression remains live in session 8618 (poll this handle; do not
  restart merely because output is quiet). Bandit 122 kills passed; Oddball still
  pending in /private/tmp/halo-kill-icons-captured.log. The captured binary predates
  only the neutral helper consolidation, covered by the final active suite.
  Full goal remains incomplete; source/profile and positive objective-film gates
  remain open. Changes stay uncommitted on simbleau/theater-experiments.
- Captured kill-icon regression is terminal and passed: Bandit 122 kills and
  Oddball 247 kills, all kill/assist/health/pass outputs matching (202.36s).
  /private/tmp/halo-kill-icons-captured.log. No live icon-validation processes.

### Recorded film-key verdict

- Source audit identified the distinct native CleDeChunk0/CleFilm diagnostic:
  written format/build/major, readability, combined refusal reasons and metric
  deltas. This was not supplied by V41FilmProfile's precision-installation issues.
- Added FilmKey and decode_v41_film_key, preserving missing header state and
  rejecting other major versions. Native table membership is reported separately
  from Rust precision availability; it does not enable older-major decoding.
  Known-format/unknown-build and unknown-format/known-build are independent, and
  both counter deltas are retained when both keys are unknown.
- Film construction uses its already-read registry/identity to retain key in the
  portable output. No extra registry walk or implicit new decode refusal is added.
  Standalone key inspection treats malformed identity as absent, as native does.
- Native 512-case oracle mutates the captured bootstrap's format/build and bounds:
  partial headers, truncated identity, known/unknown builds, unknown formats and
  non-ASCII build refusal. Generator registration and focused validation added.
  Initial compile caught borrowing registry/identity after moving them into Film;
  key is now computed alongside identity before construction. Validation follows.
- Focused film-key test passed (1.21s): 143 readable accepted keys, 107 dual-reason
  refusals, 52 unreadable inputs with no false refusal. Every field, reason and
  metric delta matches native; JSON round trips pass and v39 decoding is rejected.
  Native/focused logs: /private/tmp/halo-film-key-native.log,
  /private/tmp/halo-film-key-focused.log. Broad validation still running.
- Final film-key validation: 380 Theater tests passed, 29 ignored (47.90s),
  library/test Clippy with warnings denied, WASM compilation, format/diff and
  generator syntax checks passed. Logs: /private/tmp/halo-film-key-suite.log,
  /private/tmp/halo-film-key-clippy.log, /private/tmp/halo-film-key-wasm.log.
  All processes terminal. Full source/profile/objective-film gates remain open;
  changes remain uncommitted on the requested branch.

### Registry catalog provenance and classification audit

- The existing replay registry classification already covered v41 behavior. The
  source gap was the native complete catalog: keys, fingerprints, block/slot
  counts, measurement status/source, witness films, proof text and date.
- registry_catalog.rs exposes all nine records in native order and v41 expected
  entry/catalog-membership/classification APIs. Replay classification now uses
  this table instead of its separate hash list. Earlier-major catalog rows remain
  as fingerprint evidence for the native presumed-vs-unknown distinction; no
  earlier-major decoder is enabled.
- Production JSON is extracted from the native Go source literals by a bounded
  extractor that requires all nine rows. Independently executed native APIs
  produce the oracle: complete table plus 252 v41 lookup/classification cases,
  covering unknown/missing builds, all native hashes, one-bit mutations, zero and
  u64::MAX. Catalog measurement status remains distinct from film classification.
- Both extraction and native harness are registered in generate_oracles.py.
  Logs: /private/tmp/halo-registry-catalog-native.log,
  /private/tmp/halo-registry-catalog-focused.log. Broad validation follows.
- Final registry catalog validation: focused test passed (0.02s), full suite
  381 passed, 29 ignored (49.50s); library/test Clippy with warnings denied, WASM,
  formatting/diff and both Python syntax checks passed. Logs:
  /private/tmp/halo-registry-catalog-suite.log,
  /private/tmp/halo-registry-catalog-clippy.log,
  /private/tmp/halo-registry-catalog-wasm.log. All processes terminal. Manifest
  updates cover only the two audited native profile files. Full parser parity
  remains incomplete; source/profile/captured-objective gates remain open.

### Source-derived component dispatch coverage

- Audited all native dispatch files, resolving named string constants as well as
  literal case names: 223 names. Every name is represented in Rust; no missing
  dispatch reader was found. The broad component oracle previously omitted 18
  names (mostly aliases/body-policy readers tested in specialized suites).
- Expanded that oracle to 225 names, 722 archetype/level contexts and 18,144
  native cases. The two additional names are retained existing aliases. Every
  selected name has at least one positive native case; all native refusals still
  compare attempted endpoint and unsupported status. Successful cases compare
  boundaries, contiguous exported raw bits and bounded truncation refusal.
- audit_dispatch_names.py derives the pinned list from native sources and rejects
  unresolved constants. Generator requires an explicit reviewed archetype mapping
  for added names; future unknown names cannot silently inherit a biped type.
  The Rust test asserts that every native dispatch name appears in the oracle.
- Expanded native and focused Rust tests pass (0.75s focused). Logs:
  /private/tmp/halo-dispatch-audit-native.log,
  /private/tmp/halo-dispatch-audit-focused.log. This is named dispatch/bit-boundary
  evidence, not a claim of complete callback/context or captured-film parity.
  Broad validation follows. Full source/profile/objective-film gates remain open.
- Final dispatch audit validation: 17,760 accepted and 384 refused native cases;
  added-name coverage accounts for 2,280 cases. Full suite: 381 passed, 29 ignored
  (45.05s). Library/test Clippy with warnings denied, WASM, format/diff and Python
  syntax checks pass. Logs: /private/tmp/halo-dispatch-audit-suite.log,
  /private/tmp/halo-dispatch-audit-clippy.log,
  /private/tmp/halo-dispatch-audit-wasm.log. All processes terminal. No parser
  semantics changed in this audit; it closes the broad oracle's name inventory
  gap while preserving the remaining full-parity acceptance gates.

### Keyframe profile framing

- Added default-omitted KeyframeLayout to FrameEncoding: header_bits=108 and
  size_word_bits=32 preserve current v41 framing. Configured keyframe readers now
  use it for the header, both signed size guards and the unconditional default
  corruption-control word. Per-component corruption remains its native R(1)/R(32).
  Header extensions remain explicit raw fields; shortened headers retain every bit.
- Supported custom layouts require header_bits>=64 and size_word_bits<=32, including
  zero-width guards. Sequential tables validate the layout and inherit it through
  the configured reader. Legacy unconfigured API keeps default framing.
- Native 512-case oracle covers headers 64/72/96/108/128/160 and size widths
  0/1/5/16/24/31/32, signed/zero/positive guards, 256 corruption-enabled profiles,
  simulation gates and width overrides. All 463 completions, 49 component stops,
  78 zero-width-word cases, callbacks/endpoints, JSON round trips and shifted table
  binding results match. Focused test passes (0.33s); generator registration added.
- Logs: /private/tmp/halo-keyframe-layout-native.log,
  /private/tmp/halo-keyframe-layout-focused.log. Full validation follows.
- Next confirmed header audit target: native isolated WalkKeyframeFullState extracts
  only six type bits at record+58, while the Rust isolated padded reader currently
  reads the whole second u32. Sequential table headers independently validate that
  word, so any fix must preserve the table's stricter admission and sentinel path.
  Existing valid-header oracles do not prove this malformed/field26 distinction.
  Full source/profile/captured-objective gates remain open.
- Final keyframe-layout validation: 382 Theater tests passed, 29 ignored (50.02s).
  Library/test Clippy with warnings denied, WASM, formatting/diff and generator
  syntax checks passed. Logs: /private/tmp/halo-keyframe-layout-suite.log,
  /private/tmp/halo-keyframe-layout-clippy.log,
  /private/tmp/halo-keyframe-layout-wasm.log. All processes terminal; full goal
  remains incomplete, with the explicit header-field audit above next. Changes
  remain uncommitted on the requested branch.

### Isolated keyframe type word and absent defaults

- Native isolated WalkKeyframeFullState reads six type bits at record+58. The
  padded Rust reader now does the same while retaining the complete second u32
  in raw header fields. Bounded/table readers still admit the full word and retain
  the no-archetype sentinel; an isolated u32::MAX word decodes type 63 and refuses.
- The audit also found absent default deserializers: native keyframe traversal
  does nothing when the default table lacks the archetype, while Rust stopped at
  23/41/44. Keyframe traversal now routes only registered defaults (including the
  special biped). Direct default-component APIs remain explicit about unsupported
  standalone bodies. Default corruption control and n2 still follow the no-op.
- Native 512-case oracle independently compares isolated and sequential table
  paths, raw/type/header/guard variants, invalid generation/slots, ID/table
  sentinels, callback streams, endpoints, admission and binding behavior. Positive
  controls include 139 accepted nonzero-upper-bit words, 19 no-archetype table
  records and 103 positive-guard absent-default cases. Focused test passes (0.07s).
- Initial test compilation referenced a nonexistent binding accessor; corrected
  it to inspect the actual slot map. No parser behavior was changed for that error.
- Harness registered in generator. Logs:
  /private/tmp/halo-keyframe-type-word-native-final.log,
  /private/tmp/halo-keyframe-type-word-focused-complete.log.
  Broad and six-film complete-document validation are running. This closes the
  specific header-field audit identified above, not the full parser goal.
- Full suite passed: 383 tests, 29 ignored (50.25s), and WASM passed. Clippy
  requested as_chunks instead of chunks_exact in the new test's hex reader;
  applied that test-only cleanup and rerunning focused/Clippy checks. No parser
  semantic changes followed the broad suite.
- Six-film complete-document process is live in session 5669; poll the same handle
  while it is quiet. Bazaar and Aquarius documents have matched; remaining films
  are running. Log: /private/tmp/halo-keyframe-type-word-documents.log. This run
  excludes integrated kill decoding and is not CTF/VIP/bomb capture evidence.
- Final focused test (0.05s) and Clippy pass after the test-only lint cleanup.
  Format/diff and generator syntax checks pass. Logs:
  /private/tmp/halo-keyframe-type-word-suite.log,
  /private/tmp/halo-keyframe-type-word-focused-lint.log,
  /private/tmp/halo-keyframe-type-word-clippy-final.log,
  /private/tmp/halo-keyframe-type-word-wasm.log. Only captured-document session
  5669 remains live; all other checks terminal. Full goal still incomplete and
  changes remain uncommitted on simbleau/theater-experiments.


### Deterministic native keyframe chaining

- Added `chain_keyframe_records` with typed stop reasons, skipped/no-archetype
  counts, per-attempt records and accumulated read diagnostics. Native source:
  `internal/grammar/keyframe_record_walk.go` lines 279-315 at the pinned commit.
- Matches equality-before-read (including negative equal offsets), overshoot,
  strict full-word admission, increasing slots, no-archetype header-only advance,
  padded body reads and failed-body callbacks. The native `skipped <= 16` loop
  consumes up to 17 records; reaching the target after record 17 still stops with
  Budget. Sentinel ID is Header here, unlike sequential-table End.
- Independent 512-case Go fixture/harness registered in `generate_oracles.py`.
  Every result and callback stream matched. Positive controls: 97 reached,
  7 budget stops, 186 header stops, 46 slot stops, 101 failed-body callback cases.
  JSON round trips included. Initial aggregate success assertion used 102 rather
  than the actual native 97; corrected the test count, with no parser change.
- Full Theater suite, Clippy and WASM checks running; logs:
  /private/tmp/halo-keyframe-chain-suite.log,
  /private/tmp/halo-keyframe-chain-clippy.log,
  /private/tmp/halo-keyframe-chain-wasm.log.
- Next concrete source gap: existing `decode_keyframe_table` intentionally uses
  bounded reads, while native `WalkKeyframeRecords` calls the padded isolated
  reader. Existing table comparisons use sufficiently long payloads; audit/add a
  separate native table API with positive truncated-body evidence before claiming
  this source file complete. Chain now uses padded native reads correctly.
- Captured complete-document session 5669 remains live; Bazaar, Aquarius and
  Bandit have matched. Not objective-film evidence and excludes kill decoding.
  Full parser goal remains incomplete; changes remain uncommitted.


### Native sequential keyframe tables with padded bodies

- Added `decode_native_keyframe_table`, matching `WalkKeyframeRecords` header
  admission, prefix bit, sentinel End, increasing slots, retained failing record,
  and padded isolated body reader. The existing bounded replication-table API
  retains its established behavior. Chain/table share the single-record helper.
- Both APIs retain raw headers, per-record fields and diagnostics. Native table
  results round-trip through JSON and expose aggregated ordered callbacks.
- New independent 512-case oracle compares every record start/end, slot,
  generation, full archetype, desync index, stop and callback stream. Positive
  controls: six padded-body overruns, 172 failed-body callback cases, 766
  no-archetype entries. Stops: 176 End, 172 Desync, 128 Header, 36 Slot.
- Initial oracle truncated with floor-byte rounding, accidentally excluding the
  final complete header. The positive overrun assertion caught this. Corrected
  ceiling-byte rounding and regenerated native output; parser comparisons already
  matched the original cases. Final focused keyframe suite: 9 passed (0.55s).
- Native harness/output registered in generator; syntax check passes. Logs:
  /private/tmp/halo-keyframe-native-table-native.log,
  /private/tmp/halo-keyframe-native-table-focused-final.log.
- The previous chain-only full suite passed 384 tests, 29 ignored (59.04s), with
  Clippy, WASM, fmt/diff clean. Final table-inclusive suite/Clippy/WASM running in
  /private/tmp/halo-keyframe-native-table-{suite,clippy,wasm}.log.
- This closes the concrete padded sequential traversal API gap above. Full source
  inventory, integrated observer/profile audit and complete captured v41 CTF/VIP/
  bomb parity remain open. No full-parser completion claim.

- Final table-inclusive validation passed: 385 Theater tests, 29 ignored (55.90s);
  Clippy with warnings denied, WASM library check, fmt/diff and generator syntax.
  No changes after these checks except this status update. Only the older captured
  complete-document run (session 5669) remains live: Bazaar/Aquarius/Bandit match,
  other three pending. Next work: finish inherited profile/observer and native
  source inventory audits plus positive complete captured CTF/VIP/bomb gates.


### Film-derived corruption policy across profile replacement

- Source audit of `controle_corruption_du_film.go`, `film_context.go` 165-180/
  221-235 and `profil_balayage.go` found two distinct missing-identity rules:
  standalone composition retains its inherited flag; FilmContext memoizes the
  invariant false before replacing its profile and reapplies the film decision
  whenever producing a scan frame/direct reader. Present identity always wins.
- Added serializable `FilmCorruptionControl` (effective flag and declaration
  provenance), fallback query, and application to an encoding. Film exposes its
  context policy through `corruption_control()`. Kill calibration uses the same
  standalone resolution rule with its false invariant.
- Fixed `try_from_chunks_with_encoding`: missing identity now restores the native
  false invariant rather than retaining a caller true flag. Map-aware replication
  and march now reapply the film decision after inherited profile selection;
  default march no longer silently hardcodes false when the film declares true.
- Independent 256 native cases compare standalone composition, declaration status,
  fallback and four successive profile replacements, including prior values.
  Native direct-reader and scan-frame flag inheritance are asserted each time.
  18 cases positively distinguish inherited-true standalone fallback from context
  fallback. Fixtures and harness registered in the generator; native rerun matches
  stored fixture bytes.
- Integrated Rust test feeds missing identity plus a true custom flag through the
  Film constructor. Initial test setup incorrectly truncated the registry at its
  end offset without its structural terminator; changed fixture setup to remove
  only the build anchor. An initial metadata index type typo was test-only.
- Next source issue to audit: strict `V41FilmProfile::frame_encoding` refuses
  missing identity, while native context composition retains invariants. The
  map-aware high-level API still routes through this strict method. Establish
  native partial-identity end-to-end behavior before changing that separate gate.
- Focused test and broader validation pending in
  /private/tmp/halo-corruption-context-focused-verified.log. Full parity remains
  incomplete; no commits/pushes, and the older captured-document session 5669 is
  still live, with three of six complete documents matched.

- Final validation: focused policy/integration tests 2 passed (0.40s); full Theater
  suite 387 passed, 29 ignored (57.48s); Clippy with warnings denied, WASM library,
  fmt/diff and generator syntax all pass. Native final fixture regeneration agrees
  byte-for-byte. Logs: /private/tmp/halo-corruption-context-suite.log,
  /private/tmp/halo-corruption-context-clippy.log,
  /private/tmp/halo-corruption-context-wasm.log. All new processes terminal.
  Captured-document session 5669 remains live (three of six matches). Full goal
  remains active/incomplete; next concrete gate is missing-identity map-aware
  profile fallback, followed by the broader source/profile/observer audit.


### Missing-identification v41 decoding through complete replay output

- Native `profile.Resoudre` preserves format-27 MPP widths and map context when
  identification is absent, with a typed unknown-build issue. Removed Rust's
  extra identity requirement in `V41FilmProfile::frame_encoding`; corruption
  defaults false while `MissingIdentity` and unavailable personalization remain
  explicit. Map/format precision requirements otherwise remain unchanged.
- Expanded the existing independent native profile oracle from 158 to 237 rows:
  both known v41 builds plus missing identity on all 79 maps. Compares declared
  identity/personalization availability alongside map/MPP/position widths.
- Added a complete-document oracle using the captured Bazaar clip with only its
  build anchor cleared. Native still decodes. The first Rust attempt reached
  replay assembly but failed because player-team scanning had been skipped without
  identity. Native `ScanPlayerTeams` needs registry and context, not identification;
  the scan now runs with the film/default corruption flag. Ground-weapon creation
  scanning had the same extra gate; it now uses the effective context policy.
- The next document comparison found only an extra Rust `identity` section.
  Native `IdentitySection.Empty` ignores coverage alone and publication requires
  at least a player/biped/statborg link. Added the matching `is_empty` gate in
  replay assembly. Identity diagnostics remain available in the player result.
- Complete missing-identity Bazaar document now matches (7.77s). Native run:
  /private/tmp/halo-missing-identity-document-native.log; Rust final:
  /private/tmp/halo-missing-identity-document-rust-verified.log. The opt-in test is
  `local_missing_identity_document`, and generator registers both harness/output.
  This is a controlled mutation of a captured film, not a naturally missing-ID
  v41 capture or a CTF/VIP/bomb capture.
- Initial fixture-generation diagnostic assumed a nonexistent top-level players
  field and aborted before writing the oracle; corrected to inspect actual schema
  and write it. The resulting missing include-file compile attempt is terminal.
- Broad checks running in /private/tmp/halo-missing-identity-{suite,clippy,wasm}.log.
  fmt/diff/generator syntax checked. Next concrete profile gate: unknown formats
  retain native scan defaults/calibration while the Rust map-aware encoding still
  requires known MPP widths. Keep the v41 major guard and diagnostics; verify
  complete native output before changing this separate path.

- Final broad validation passes: 387 Theater tests, 30 ignored (53.53s), including
  the expanded 237-profile oracle; Clippy/WASM/fmt/diff/generator syntax pass.
  The separately run ignored missing-identity document test passed. Its native
  document has 71 frames, one track, loadout/inventory, grenade/stance and
  weapon-pad/pickup publications: positive retained data, not an empty-success
  fixture. All new sessions terminal. The older six-film document session 5669
  remains live; three matches so far. Full goal remains incomplete.


### Unknown-format MPP fallback and independent power-up pad scans

- Native source `InstallFilmFormatMPP`, `MPPWidthsForFilm`, `gwWidthsForFilm` and
  `gwInstallMPPWidths` distinguishes declared widths, calibrated widths and the
  inherited scan invariant. Added `FilmMppResolution` with format membership,
  optional declared widths and effective creation-width selection. Formats
  20/21/24/25 are known metadata with undetermined widths, not unknown-format
  errors; this does not remove the v41-only major-version guard.
- Map-aware encoding now retains 9/5 scan invariants when declared MPP is absent,
  with profile issues and unavailable declaration preserved. Equipment calibration
  runs before weapon/vehicle creation scans; its valid widths take precedence over
  the invariant when format lookup supplies none. Missing/invalid calibration
  leaves inherited widths. No neighboring build or map is selected.
- The native 512-case resolution/installation oracle checks format/readability,
  width selection and restoration. Positive controls: 154 effective calibrations
  different from inheritance, 217 inherited fallback cases and 256 known-format
  cases with undetermined widths. JSON round trips and v41 profile diagnostics
  also checked. Focused test passed (0.24s).
- Complete captured Bazaar documents with format changed to 0, 28 and 0xffffffff
  first exposed missing power-up pads. Placement confirmation returns no readings
  if both format widths and calibration are inconclusive, but native independently
  scans power-up creations for pads under inherited widths. Rust had reused the
  empty placement stream. Added `scan_equipment_creations_for_band` and retained
  `Film.equipment_pad_creations`; pad publication uses it. Successful placement
  scans reuse identical raw creations, avoiding an extra scan of normal films.
- All three complete native replay documents now match (23.93s): 71 frames/one
  track each, with retained weapon-pad and pickup publications. New native harness
  and 512-case width oracle registered in generator. Logs:
  /private/tmp/halo-unknown-format-document-native.log,
  /private/tmp/halo-unknown-format-document-rust-complete.log,
  /private/tmp/halo-mpp-resolution-native.log,
  /private/tmp/halo-mpp-resolution-rust.log.
- Rerunning final unknown-format documents after normal-path reuse and positive
  count assertions, plus the known-format/missing-identity document, full suite,
  Clippy and WASM. All modifications remain in the film module and uncommitted.
- Earlier captured six-film complete-document session 5669 finished successfully:
  Bazaar, Aquarius, Bandit, Oddball, Cadet Blue and Cadet Brick all matched
  (1469.70s). That binary predates these profile-fallback changes; these new
  controlled-mutation checks validate the current changes separately. No live
  handle 5669 remains. Positive complete CTF/VIP/bomb captures still outstanding.

- Final checkpoint: 388 Theater tests passed, 31 ignored (63.66s), including the
  512-case width-resolution oracle. Final complete unknown-format documents all
  match (47.33s); known-format missing-ID document also matches (13.37s). Clippy,
  WASM, fmt/diff and generator syntax pass. Logs:
  /private/tmp/halo-unknown-format-suite.log,
  /private/tmp/halo-unknown-format-clippy.log,
  /private/tmp/halo-unknown-format-wasm.log,
  /private/tmp/halo-unknown-format-document-rust-final.log,
  /private/tmp/halo-unknown-format-known-profile-document.log.
  Every process from this checkpoint is terminal. Full source/profile/observer
  audit and positive captured CTF/VIP/bomb parity remain open. No full-parity claim.


### Creation-reader world descriptors independent of map bounds

- Source audit of `equipCreationWalk.decodePos/posAdvance/readCreation` found that
  native equipment/ground-weapon position admission and advancement use the
  reader profile's world descriptor. Rust used catalog/map widths even when a
  different PositionEncoding was supplied for ground-weapon component reads.
- Added serializable `WorldObjectPrecision` and explicit selective position reader.
  Bounds remain the coordinate range; descriptor carries region/index/axis widths.
  Native zero-bit region indices and 31/32-bit axes are now represented, including
  padded tails. Existing catalog entry points retain their effective minimum
  one-bit region convention. Checked offset arithmetic rejects overflow.
- Creation readers now use the supplied descriptor for ti37/42 admission and
  `after_bit`; ti40 retains its separate catalog dynamic-precision position rule.
  Added equipment payload/chunk APIs with PositionEncoding and routed the Film
  pad scan through them. Reuses the established first-world-region convention from
  replay precision installation when adapting a general PositionEncoding.
- Independent 512-case native oracle covers ti37/42, distinct profile/catalog
  widths and regions, zero region bits, 31/32-bit axes, MPP widths, wrong regions,
  boundary quanta and truncated tails. All records, float coordinates, bounds and
  counters match; JSON round trips pass. Positive controls: 452 accepted records,
  117 zero-index records and 92 wide-axis records. Focused test passes (0.06s).
- Native harness/output registered in generator. Initial test include path used
  the components-subdirectory convention from another fixture; corrected it to
  this module's fixtures path. No parser change for that compilation error.
- Broad suite/Clippy/WASM and complete zero-axis/missing-ID document checks running
  in /private/tmp/halo-creation-world-profile-{suite,clippy,wasm,zero-axis,missing-identity}.log.
  fmt/diff and generator syntax pass. Full goal remains incomplete; remaining
  source/profile/observer audit and positive complete CTF/VIP/bomb captures remain.

- Final checkpoint validation: 389 Theater tests passed, 31 ignored (50.88s);
  Clippy with warnings denied and WASM library checks passed. Separately executed
  complete zero-axis and missing-ID document comparisons passed (15.12s/14.25s).
  fmt/diff and generator syntax passed. Logs use the creation-world-profile prefix
  listed above. All checkpoint sessions are terminal. Changes remain uncommitted;
  full source/profile/observer audit and positive CTF/VIP/bomb captures remain.


### Replay input retention and loaded-source clock audit (2026-09-24)

- Added [FILM_INPUT_RETENTION.md](FILM_INPUT_RETENTION.md), mapping all 44
  pinned native FilmInputs fields to retained Rust data and assembly consumers.
  Checked the names against the native struct and gameplay-parity inventory.
  This is a wiring audit, not blanket completion of historical per-field gates.
- Confirmed replay uses chunk number 1's first packet, independently of
  Film.origin_timestamp_us. Existing native grid and automatic identity oracles
  cover origin refusal, death-clock consistency, missing chunks and zero clocks.
- Added scan_loaded_replay_clock_origin for FilmSource. The existing 512-source
  native chunk-bridge oracle now checks clock lookup for metadata-free, short,
  long, reordered and duplicate metadata. Empty first matches do not fall through.
- Updated only replay/film_inputs.go (partial with explicit retention evidence)
  and replay/origin.go (ported-v41 with existing and new oracle evidence).
  NEXT_PHASE.md remains queued; no architecture refactor was started.
- Clippy with warnings denied, WASM library check and format/diff checks pass.
  Full Theater suite passed: 397 passed, 32 ignored (83.93s), recorded in
  /private/tmp/halo-input-audit-suite.log; companion logs use the same prefix
  with clippy/wasm suffixes. All checkpoint processes are terminal.
  Full profile/source/observer audit and positive complete captured CTF/VIP/bomb
  parity remain open. Changes remain uncommitted on simbleau/theater-experiments.


### Resolved position-capture profile and observer closure (2026-09-24)

- Audited all non-test PosCaptureHook installers, neutralizers and slot setters.
  The raw target scan's local replacement collector is the sole installer;
  inference/chain/harvest/repair suppression and direct/context inheritance have
  existing independent ordered-stream evidence. See OBSERVER_STATUS.md.
- Extending the profile oracle exposed a factory mismatch: Rust substituted map
  bounds for the native observer's invariant QuantRangeCEBiped. Native map
  installation changes the descriptor, not Movement.Range or DeltaQuantum.
  Corrected V41FilmProfile::frame_encoding and its effective minimum region-index
  width. Explicit capture profiles and dedicated map-position scans remain separate.
- First diagnostic used immutable Profile.Movement; the final oracle installs
  the map descriptor with the native helper before comparison. This confirms the
  range mismatch independently of descriptor selection. All 237 map/build capture
  contexts now match exact floating-point bits and descriptor values.
- Added native position-hook-resolved fixture: 512 production frames, all 79 maps,
  863 position callbacks. Rust uses its own profile factory; compares all ordered
  callbacks, records, view counts and final cursors. Registered regeneration.
  The initial short exact test selector selected zero tests; corrected to the
  fully qualified name and reran successfully, then ran the full suite.
- All 30 observer hooks now have publication/routing evidence for tested v41
  contexts. This does not close broader custom-profile combinations or imply
  complete native source/output parity.
- Validation: 398 Theater tests passed, 32 ignored (48.51s). Separately executed
  complete missing-identification Bazaar document comparison passed (13.17s).
  Clippy with warnings denied, WASM library check, fmt/diff and generator Python
  syntax passed. Logs: /private/tmp/halo-position-profile-{suite,captured,clippy,wasm}.log;
  native frame/profile checks: /private/tmp/halo-position-resolved-native.log.
  All checkpoint processes are terminal. Work remains uncommitted on
  simbleau/theater-experiments. Full source/custom-profile audit and positive
  complete captured v41 CTF/VIP/bomb parity remain open. NEXT_PHASE remains queued.


### Standalone component probe context (2026-09-24)

- Added public consume_component_at alongside the existing keyframe-biped probe.
  It matches native ConsumeComponentAt's direct dispatch under ContexteDeLecture,
  retaining raw fields, reference reads, padded endpoints and ordered callbacks.
- Native direct dispatch bypasses traversal calibrated/stub widths and corruption
  guards. Simulation completion still controls the ported verdict, after body
  consumption; its unspecified standalone default is false. Frame ID/MPP settings
  are not used by this API and therefore do not invalidate a standalone read.
- Independent 1024-case native oracle compares status, endpoints and all 318
  callbacks, including 142 padded reads and 85 refused simulation bodies. Deliberate
  width overrides (77/91 bits), corruption flags, unknown/empty names, movement,
  positions, references and dead-state readers exercise the direct-vs-traversal
  distinction. Rust also supplies invalid outer frame IDs/MPP widths to verify
  those unused settings do not suppress direct reads.
- Native harness and fixture are registered in generate_oracles.py. A first
  harness spelling used an unsupported crouch alias; corrected it to the native
  unit-crouch-component before the final fixture was generated.
- probe_export.go remains partial: the separate keyframe trace typed-payload
  projection audit remains open. Both native probe tests pass with the final
  irrelevant-frame-context check (0.15s). Clippy, WASM, fmt/diff and generator
  syntax pass. Logs: /private/tmp/halo-component-probe-{focused,clippy,wasm}-final.log.
  All checkpoint processes are terminal. The latest complete suite remains 398 passed and
  32 ignored from the prior checkpoint; it has not been rerun for this additive API.


### Native typed component payload projections (2026-09-24)

- Added CapturedComponentPayload and captured_payload accessors on decoded
  components, ordered entity attempts and keyframe-biped probe components. All
  six capture.go kinds are represented: body/shield vitality, object parent,
  dissolver, respawn timer and round timer. Values come from retained fields and
  observations; no bytes are reread and no timeline/architecture work is introduced.
- Existing vitality dequantization is reused. RoundTimer preserves both quanta
  and exact native endpoint seconds without assigning elapsed/remaining meaning.
  Respawn timers remain raw words with unknown units. Dissolver retains the
  96-bit raw body omitted by native's typed payload. Calibrated skipped components
  have no fabricated typed value; source fields remain available separately.
- Native 1024-keyframe fixture covers all 6144 component entries: 5997 typed
  payloads and 147 absent calibration payloads, including padded tails. All
  65536 round-timer quanta compare exact floating-point bits. Probe and direct
  projections match; typed serialization round trips pass. Generator registered.
- Harness corrections: fixed a composite-literal brace; the first mask writer
  used four count bits and selected only one component per record. The final
  native three-bit mask selects all six, enforced by the 5997/147 counts. Native
  parent EndBit is filled by its observer: with no listener it stays zero. The
  final oracle installs the parent observer to match Rust's retained observation,
  and the public payload documentation states that distinction.
- Focused payload test passes (0.28s). Full Theater suite passes: 400 passed,
  32 ignored (52.58s). Clippy with warnings denied, WASM library check, fmt/diff
  and generator syntax pass. Logs: /private/tmp/halo-captured-payloads-{suite,clippy,wasm}.log.
  All checkpoint processes are terminal; changes remain uncommitted on the
  requested branch. No next-phase architecture work was started.
  capture.go is marked ported for these six kinds; probe_export.go remains partial
  pending Variant/Dead trace projection audit. Broader profile/source and positive
  complete captured v41 objective parity gates remain open.


### Keyframe probe trace values and source closure (2026-09-24)

- KeyframeBipedProbeComponent now retains native Variant, distinguishing ordinary
  no-variant u32::MAX from calibrated skips' zero-initialized value. The probe
  retains the last decoded ObjectDeadState, including Mort=false and observations
  before a subsequent failed component. Raw skipped/default fields remain separate.
- Dedicated native fixture: 1024 records, 5486 component results, 847 decoded
  variants, 222 calibrated skips, 573 Mort=false states, 198 padded reads and
  682 failed traces. Covers duplicate weapon/death entries, unknown stubs,
  corruption guards, light/heavy death layouts, and simulation policy.
  Endpoints, masks, trace metadata, every variant/death field and serialization
  round trips match. Generator registered. All three probe tests pass (0.63s).
- Corrected keyframe context validation: record IDs are not read by this probe;
  non-biped probes also do not read the MPP default block. Invalid unused settings
  no longer reject a valid probe. Unspecified simulation completion now uses the
  native standalone false default. Native trace DefaultBits/Gate are fixed zero
  here; actual default reads and gate values are retained in fields and documented.
- probe_export.go is now ported-v41 with combined standalone, structural, typed
  payload and trace-value evidence. Source audit also closes traverse_precision.go:
  its only two functions forward descriptors already covered by the profile and
  position/creation oracles. No new decoder behavior was hidden in that file.
- Full Theater suite passed: 401 passed, 32 ignored (48.32s). Clippy with
  warnings denied, WASM, fmt/diff and generator syntax passed. Logs:
  /private/tmp/halo-probe-trace-suite.log; focused/clippy/wasm share that prefix.
  All checkpoint processes are terminal.
  Broader source/custom-profile and positive complete captured objective gates
  remain open. Architecture work remains queued; changes stay uncommitted.

### Equipment calibration inherits world precision (2026-09-24)

- Audited `equipment_creation_width.go` end to end against the pinned source.
  Candidate preference, per-axis epsilon, nearest-time life matching, conclusive
  threshold/margin, per-chunk early stopping, and inconclusive score retention
  match Rust. `Lives` counts distinct slot/generation keys, not individual lives.
- Confirmed a descriptor propagation gap: rebuilding an inherited descriptor via
  `FilmMapBounds` normalizes zero index bits to one. The first native fixture has
  seven best-candidate agreements; the old Rust path reports zero. Normal catalog
  installation already supplies the same widths, so this is specifically an
  inherited/custom-profile mismatch, not evidence of broken normal-map captures.
- Added explicit-position calibration and placement APIs. Map-aware Film assembly
  now supplies its resolved descriptor through mobile tracks, MPP calibration,
  and creation admission. Existing map-only convenience functions remain.
  The mobile-track scanner now supports native zero index bits and 31/32-bit axes.
- Removed an additional early return for a present but empty equipment archetype:
  native calibration still counts chunks/anchors while rejecting its masks.
- Independent native harness `halo_rust_equipment_calibration_profile_test.go.txt`
  exercises 256 loaded films and all 63 MPP candidates. The final fixture includes
  213 conclusive calibrations, 189 early stops, 15 empty archetypes, 11 empty bands,
  eight empty span maps, truncations, and 2,940 mobile lifetimes / 14,700 samples.
  It compares every candidate score, winning/runner widths, all diagnostics and
  ordered tracks. Native profile restoration is asserted; Rust also checks JSON
  round trips and the original zero-index regression. The generator registers
  the pinned harness and compressed fixture.
- Final validation: 402 Theater tests passed, 32 ignored (49.18s); the captured
  zero-axis complete document regression passed separately (12.98s). Clippy
  with warnings denied, WASM library, formatting, diff checks and generator
  syntax all pass. All checkpoint processes are terminal. Logs are under
  `/private/tmp/halo-equipment-profile-*`.
- The architecture in `NEXT_PHASE.md` remains queued. Full parser parity is still
  incomplete; this checkpoint does not close the other inventory/corpus gates.

### Keyframe closure diagnostics (2026-09-24)

- Ported native `keyframe_closure.go` as `measure_keyframe_closure` and the public
  serializable `KeyframeClosureStat`. It uses the existing native anchor selection
  and full-state reader, measures every bounded record, excludes each payload's
  final anchor, and picks the most frequent blocker with lexical tie-breaking.
  Missing archetypes retain the native bare `i0` label. Only type-2 packets count.
- The API installs format-27 MPP widths on a local profile copy, preserving the
  caller's inherited context; unknown and older known formats keep their widths.
  Empty native chunk prefixes return an empty measurement. This diagnostic has
  no non-test native caller, so it remains an explicit API rather than adding
  automatic scanning cost or derived claims to Film records.
- Closure is agreement with a recovered next anchor, not proof that recovery found
  every record. It does not establish byte-for-byte fidelity, replace sequential
  parsing, or discard records/unknown regions. The future fidelity refactor stays
  queued in `NEXT_PHASE.md`.
- Pinned native harness `halo_rust_keyframe_closure_test.go.txt` supplies 256
  three-chunk cases, 17,112 bounded records, 6,830 exact closures, missing
  archetypes, closed component guards, unsupported components, truncations,
  corruption/simulation policies and known/unknown MPP formats. A complementary
  native blocker-ranking oracle includes 140 top-count ties.
- Focused native/Rust comparisons pass, including every closure count, blocker,
  unchanged caller context and JSON round trips. An initial harness comparison
  omitted Rust's movement encoding; the final fixture supplies the same native
  movement descriptor. This was a harness mismatch, not a decoder regression.
  Both new harnesses are registered in `generate_oracles.py` and formatted with Go.
- Final validation: 403 Theater tests passed, 32 ignored (49.75s); Clippy with
  warnings denied, WASM library, formatting, diff checks and generator syntax
  passed. The earlier equipment checkpoint also passed the complete captured
  zero-axis document comparison (12.98s). All processes are terminal; final logs
  are `/private/tmp/halo-keyframe-closure-{suite,clippy,wasm}.log`.
- Full v41 parity remains incomplete. Fifteen non-research grammar entries still
  have pending source-audit status, alongside partial entries and the positive
  complete captured CTF/VIP/bomb gates. Do not start the queued architecture.

### Loaded march/death source audit (2026-09-24)

- Audited `object_deaths.go` and `object_deaths_calibrate.go` against the pinned
  source and existing Rust harvest, timeline, calibration and march code. The
  six candidate widths, 3,000-packet / 150-event budgets, default-width tie
  preference, twofold domination gate, chronological stable sorting, duplicate
  selection and all death/occupancy coverage denominators are retained.
- The calibration source comment claiming NEW/DEL bindings persist is stale:
  `marchRecordsOf` snapshots and restores the world on every call. Its trial
  configuration has no observer. Rust may omit those discarded trial walks;
  only the event-localization result and keyframe timeline influence calibration.
  This is verified from the implementation, not inferred from the comment.
- Fixed `scan_film_march_facts_with_simulation_policy` on an empty contiguous native
  data prefix. Native `ScanMarchFacts` returns an empty successful measurement;
  Rust previously returned `Missing("readable film chunks")`. Keyframe-only
  measurements preserve their keyframe count and leave calibration absent.
- New pinned `halo_rust_march_loaded_test.go.txt` / `march-loaded-v41.json.zlib`
  compare 256 loaded sources: 55 no-delta cases, missing/duplicate chunk numbers,
  shuffled and tied timestamps, packet event localization, truncated payloads,
  explicit corruption/simulation policies, 560 deaths and 934 occupancy readings.
  Every event field, order, coverage count, calibration denominator and retained
  profile field matches; JSON round trips pass. The pre-fix test fails on case 0
  with the missing-chunks error; the final focused test passes (8.68s).
- Existing independent evidence complements this integrated fixture: 128 native
  timeline/calibration cases include both budget limits; 1,024 harvest cases
  cover clean and failed tails, mask declarations and deduplication; the four-film
  captured march oracle and complete-document regressions cover production use.
- March checkpoint validation: 404 Theater tests passed, 32 ignored (59.13s);
  Clippy, WASM, formatting/diff and generator syntax passed. A separate four-film
  captured march regression is still running; see the live-check note below.

Next source audit: `keyframe_ground_weapons.go` combines production family scans
(already covered by 512 direct cases and four captured films) with explicitly
research-only raw position grouping/nearest-sample helpers. The latter must be
classified and mapped explicitly; their noisy delta candidates must not be
published as confirmed ground-weapon positions. The native comments distinguish
those candidates from validated creation positions. No implementation or blanket
source-completion claim was made for those helpers at this checkpoint.

Additional next-target finding from read-only audit: native `objective_scan.go`
exposes `ScanObjectives` (only its directory wrapper calls it outside tests), but
Rust currently has no equivalent scan adapter. Its six typed Objective callbacks
are already retained in `FilmComponentObservation::Objective` through the
component readers. The missing work is grouping/ordering and the delta/keyframe
acceptance counters, observed-only slot band, partial-delta versus whole-keyframe
rollback, and chained/closed flags. Native's comments explicitly warn that these
readings are not validated objective-progress semantics. Port this as a faithful
explicit scan API; do not reinterpret it as capture events or begin the queued
architecture. Useful existing building blocks: `consume_component_at`,
`decode_native_keyframe_record`, `recover_keyframe_anchors`,
`navpoint_radial_scan::{indices,header_at}`, and Objective component observations.

### Managed-objective scan adapter (2026-09-24)

The next-target finding above is now implemented as the explicit `scan_objectives`
API in `objective_scan.rs`, without wiring candidate readings into gameplay events
or beginning the queued architecture. It reuses existing production component
readers and Objective callbacks; there is no copied component grammar.

- Preserves native chunk/packet/publication order, the observed-only slot band
  with cross-archetype exclusions, all six field values and second-timer presence,
  delta/keyframe provenance, chained flags and every scan/closure denominator.
  The field enum uses registry names, with `native_code()` preserving its exact
  native ordinal. No semantic interpretation of the raw progress values is added.
- Delta reads retain publications preceding a later failed component. Keyframe
  reads roll back the entire failed record. Calibrated/stub widths and corruption
  guards affect full-state traversal; the direct delta reader intentionally
  bypasses them, matching native `consumeByName`.
- `ObjectiveScanFailure` retains native partial counters on setup errors, including
  observed slot counts when the registry lacks ti=11. The report remains available
  for serialization even when the scan returns an error.
- The pinned native `halo_rust_objective_scan_test.go.txt` final oracle contains
  256 loaded sources, 40 setup failures, 40,641 ordered readings (12,891 from
  keyframes), 1,500 closed keyframes, 201 chained deltas and 963 broken keyframes.
  It includes sparse observed slots, cross-archetype reuse, mask-domain refusals,
  missing chunks/slots/archetypes, truncations, width overrides, empty/unknown
  component names and corruption gates. Native field codes and all raw fields,
  counters, errors' partial reports and JSON round trips compare successfully.
- Final focused native/Rust comparison passes (1.69s). The generator includes
  the new harness/fixture. Latest complete suite: 405 Theater tests passed,
  32 ignored (54.41s); Clippy with warnings denied, WASM, formatting/diff and
  generator syntax passed. Unit/build processes are terminal. The separate
  captured march regression remains live, as recorded below.
- `slot_band_observed.go` has exactly two non-test callers: objective scanning and
  managed-property scanning. Both Rust adapters derive observed slots from the
  keyframe census, intersected with its cross-archetype exclusions. No gap filling
  is used for these long-lived objects; the new sparse/reused-slot oracle verifies
  this distinction independently of the filled world-object bands.

### Captured march verification completed (2026-09-24)

The previously running four-film captured march regression completed successfully:
1 passed, 0 failed, in 1,312.63s. Session 27906 is terminal; do not poll or restart it.

- Command: `cargo test --lib theater::march_scan::tests::local_native_march_corpus -- --exact --ignored`.
- Log: `/private/tmp/halo-march-loaded-captured.log`.
- The assertions compare every death and occupancy reading, packet/keyframe/delta
  counters, record/clean-record/mask coverage and the chosen calibration results
  against the pinned four-film native oracle. This run used the binary from the
  march-loaded checkpoint; subsequent aim-census and vehicle-layout edits have
  their own current-suite validation below.
- Later historical notes saying this handle was live describe intermediate polls;
  this terminal result supersedes them.

The goal remains incomplete. The remaining inventory includes research utilities
requiring explicit classification as well as core/partial entries and positive
complete captured CTF/VIP/bomb gates. Keep all changes uncommitted and preserve the
queued architecture boundary in `NEXT_PHASE.md`.

### Aim accessors and orientation source audit (2026-09-24)

- Added `BipedAim::aim_heading_degrees` and `aim_pitch_degrees`, preserving the
  native midpoint formulas and the documented uncertainty outside the observed
  central pitch range. Position-track and occupant-aim publication now share the
  same conversion owners. Raw quanta remain intact.
- Extended the existing native aim oracle with every 12-bit yaw and 11-bit pitch
  value: 4,096 heading values and 2,048 pitch values, alongside the existing 2,048
  scanner payloads. Regenerated directly with the pinned Go parser; no expected
  values were generated by Rust. Existing generator wiring retains these fields.
- Audited `orientation_frame.go` against its Rust implementation and the 4,096
  native cases, including missing/default orientation, zero up, both pi branches,
  mode widths and component projections. Marked this source ported-v41. Preserve
  its explicit ARM64 fused arithmetic order. Both source hashes match the manifest.
- The aim scanner's preceding vitality readers do not consult reader context:
  `decodeObjectBodyVitality` and `decodeObjectShieldVitality` read fixed fields
  directly. No inherited-profile propagation change is required there.
- Current verification is terminal: 405 Theater tests passed, 32 ignored (52.26s);
  Clippy with warnings denied, WASM library check, formatting and diff checks pass.
  Logs: `/private/tmp/halo-aim-audit-{suite,clippy,wasm}.log`.
- The separate captured march session 27906 remains live after another poll;
  its existing handoff above still applies. Do not restart it.

Next concrete gate: loaded aim scanning and biped census parity. The manifest
keeps `offline_aim_only.go` partial. Native `bipedSlotBand` in
`offline_biped_band.go` scans the contiguous chunk numbers plus the next number,
uses the first keyframe per selected chunk, and fills the min/max band. Rust's
`biped_scan::biped_slot_band` currently scans the full packet index and builds a
last-value-by-number byte map. Add loaded native sources covering missing,
duplicate and shuffled chunk numbers before changing this shared helper; several
scanners depend on it. Also verify no-source/empty-band error ordering. Passing
payload-level aim tests alone does not establish this wrapper parity.

Additional read-only audit: native `quantize.go::ReadQuantizedVec3` has only test
callers in the film tree; `bitLen` is used by variable-width decoding and fixed
component widths. Generic quantizer utility scope remains to be classified;
no completion claim or implementation change was made for that file.

Parity remains incomplete; the architecture in `NEXT_PHASE.md` is still deferred.
All changes remain uncommitted on `simbleau/theater-experiments`.

### Loaded aim parity and biped census repair (2026-09-24)

The preceding loaded-source gate is implemented and verified. Added
`scan_film_biped_aim`, which applies native source/empty-band gates and delegates
the actual record grammar to the existing aim scanner. The shared
`biped_scan::biped_slot_band` now uses the contiguous chunk prefix plus the next
number, the first matching chunk for that number, and its first keyframe. It
returns an empty census for no data prefix. It no longer requires the strict full
packet index just to compute this native census.

- New native harness `halo_rust_biped_aim_loaded_test.go.txt` and fixture
  `biped-aim-loaded-v41.json.zlib` cover 256 loaded v41 sources, 1,929 ordered aim
  readings, 79 absent-prefix errors and 38 empty-band errors. All slot sets,
  source timestamps/chunk/packet indices, raw quanta, errors and JSON round trips
  match. Cases include duplicate and shuffled numbers, gaps, negative metadata,
  truncated packet tails, two keyframes per chunk, non-delta packets and tied
  timestamps. Explicit duplicate-prefix cases retain a following numbered chunk
  to verify the extra-keyframe census lookahead.
- Before repair, the new test failed because the old census rejected the empty
  source as missing replication packets. The repaired initial fixture passed;
  the expanded final fixture also passes. Native generation is registered in
  `generate_oracles.py`, and the Go harness is formatted.
- Marked `offline_aim_only.go` ported-v41. `offline_biped_band.go` remains partial
  for its other position-scanning wrappers; this change does not claim those are
  complete. Both source hashes match the pinned manifest.
- Current suite: 406 Theater tests passed, 32 ignored (52.06s). Clippy with warnings
  denied, WASM, formatting/diff and generator syntax checks pass. These jobs are
  terminal. Logs: `/private/tmp/halo-aim-loaded-{suite,clippy,wasm}.log`.
  Before/after focused logs: `/private/tmp/halo-aim-loaded-{before,after}.log`.
- Captured march test session 27906 is still live after polling; its log remains
  `/private/tmp/halo-march-loaded-captured.log`. Preserve that running handle.

### Highlight source closure and next source findings (2026-09-24)

Audited the full pinned `highlight_events.go` against `highlight_events.rs` and
its 9,323-input native fixture (passing in the current suite). The v41 layout,
seven fields, marker/window boundaries, candidate fallback, event precedence,
UTF-16 and compressed-input behavior are implemented; marked ported-v41. Native
seen-position bookkeeping is redundant in Rust's strictly increasing one-pass
marker loop, which visits each XUID start once. Other major-version layouts stay
outside scope. Captured-corpus evidence remains recorded above; no captured
corpus rerun was claimed in this source-closure audit.

Read-only next-source findings:
- `keyframe_entity_queue.go` is a research adapter, not entity-state storage.
  It contains packet content matching, first/all packet selection, frame-variant
  measurements/ranking, and keyframe anchor-gap measurements. Its comment says
  `FirstPacketOfType` includes chunk zero, but the implementation starts at one.
  No corresponding KFQ measurement types were found in Rust. Classify/port the
  observable diagnostic outputs explicitly; do not mistake it for the queued
  architecture or silently omit it.
- `vehicle_creation.go` shares equipment creation traversal and has explicit
  dynamic i0 validation, loaded wrapper/setup refusals, and position-layout
  selection. Read its remaining wrapper source and compare the existing
  equipment/vehicle creation fixtures before closing that pending entry.

All changes remain uncommitted. Full parity and positive captured objective gates
remain incomplete; the architectural phase is still deferred.

### Explicit vehicle i0 layouts (2026-09-24)

Audited `vehicle_creation.go` and split its dynamic position gate from map catalog
normalization. `decode_vehicle_i0_position` accepts the resolved `I0Layout` and
coordinate bounds directly; `decode_vehicle_creations_with_layout` applies that
same layout to admission and `AfterBit`. Default-state `PositionEncoding` remains
independent. Existing map-based creation entry points delegate to the same gate
with their existing effective region width, preserving catalog behavior.

- Native `halo_rust_vehicle_i0_test.go.txt` covers 2,560 cases with negative and
  unaligned starts, absolute-spine rejection, zero-axis saturation, truncation,
  0..4 region bits and 8..63-bit axis fields. Its final 538 accepted positions
  include 105 with no region field and 462 with at least one axis wider than
  32 bits. Wider native reads retain only the low 32 bits, including the native
  saturation rule, while dequantization uses the full field width. All match.
- Native `halo_rust_vehicle_creation_layout_test.go.txt` covers 2,048 complete
  payloads with a dynamic layout distinct from the map/default-state profile.
  All fields, rejection/acceptance counters and advancement match for 4,096
  accepted creation records. Includes zero-bit region layouts, wrong regions,
  saturated axes, full/sparse masks, optional defaults, garbage and truncations.
  The legacy map-only decode differs in more than 100 cases; the test asserts
  that this fixture actually exercises the explicit-layout path.
- Both formatted native harnesses and compressed fixtures are registered in the
  oracle generator. Focused creation suite: seven tests pass (0.60s), including
  the existing equipment, ground-weapon and vehicle/default-profile tests.
  Initial direct-gate comparison matched every native value but failed a
  hand-written aggregate expectation: some randomly cut payloads remained whole.
  The final aggregate counts are taken from the native fixture, not Rust output.
- `vehicle_creation.go` is now explicitly partial-v41 in the manifest rather
  than pending. No claim of loaded-wrapper completion: native resolves layout
  before archetype lookup and preserves `Slots` on subsequent setup failures;
  the existing Rust loaded API drops that partial report. The top-level native
  wrapper also derives the vehicle census. These remain the next concrete gate.
  Payload-level explicit layouts are implemented; loaded auto-layout/error/stats
  parity still needs its own native source fixture and production wiring.
- The former long-running captured march test is terminal and passed: four films,
  309 deaths, 150 occupancy readings, full counters/coverage/calibration checks.
  Its superseding terminal result is recorded above. Do not poll session 27906.

Full parity remains incomplete, including positive captured objective gates and
remaining source inventory. Keep changes uncommitted and keep `NEXT_PHASE.md`
deferred.

Final explicit-layout verification: 408 Theater tests passed, 32 ignored (47.56s).
Clippy with warnings denied, WASM library check, formatting/diff and generator
syntax checks pass. All test/build sessions from this checkpoint are terminal.
Logs: `/private/tmp/halo-vehicle-layout-{suite,clippy,wasm}.log`.

### Vehicle creation wrapper reports (2026-09-24)

Added `VehicleCreationScanConfig`, `VehicleCreationScanFailure`,
`scan_vehicle_creations`, and `scan_vehicle_creations_for_band_report`. These are
native scan adapters, not the deferred resolved-replay architecture. The config
accepts resolved layout/registry results so their failures remain deferred until
the corresponding native setup step. It keeps component precision and dynamic
position layout independent.

- Top-level data-prefix and census refusals precede bounds validation. ForBand
  validates bounds first, then the data prefix, records Slots, then checks layout
  and registry/archetype. An explicitly empty band is allowed for ForBand. Errors
  retain a serializable `EquipmentCreationStream`, including native partial stats.
- Existing map-based compatibility APIs delegate to the report API and project
  failures to their existing error-only signature. `FilmVehicleFacts` now retains
  the partial creation stream alongside its issue message, while leaving scanned
  false. A fixture-backed integration assertion verifies four census slots survive
  an absent vehicle archetype.
- Native `halo_rust_vehicle_creation_loaded_test.go.txt` consumes the independently
  generated creation-layout payloads and builds 256 loaded sources. The final
  fixture has 5,262 records and 121 setup failures: 19 missing prefixes, 37 missing
  bounds, 38 layout refusals, eight registry refusals, eight missing archetypes and
  11 empty top-level bands. Sixteen cases have empty explicit ForBand bands.
  Entry-point choice is independent of failure triggers. Duplicate/missing chunk
  numbers, truncated tails, non-delta packets and tied timestamps are covered.
- Every ordered record, all stats, failure category and typed JSON round trip
  matches. Initial harness failures were corrected: the Rust registry fixture
  needs 41 indexed entries, and native float32 coordinates must compare as typed
  float32 values rather than different JSON decimal representations. These were
  harness problems, not changes to coordinate decoding.
- The formatted harness/fixture are registered after their creation-layout input
  in `generate_oracles.py`. Focused comparison passes (0.42s). Current full suite:
  409 Theater tests passed, 32 ignored (49.51s). Clippy with warnings denied, WASM,
  formatting/diff and generator syntax checks pass. All jobs are terminal.
  Logs: `/private/tmp/halo-vehicle-loaded-{focused,suite,clippy,wasm}.log`.
- `vehicle_creation.go` remains partial for end-to-end automatic context resolution
  and captured creation coverage. Its resolved-input wrapper output is now verified;
  do not confuse that with testing the complete loader/profile-to-wrapper path.

Inventory clarification: pinned `doc.go` and the six `rev_chronique*.go` files
contain only comments and their package declaration. Verified each source hash
and absence of runtime declarations; marked them `reference-only`. No executable
output was dropped, and this classification does not close the code described by
those historical notes. `rev.go` is different: its revision constant is already
published in Rust decoder coverage, but its remaining source/test-infrastructure
mapping still needs explicit audit.

Keep all changes uncommitted on the current branch. Full parser parity and the
captured objective gates remain incomplete; `NEXT_PHASE.md` remains deferred.

### Automatic i0 detection: duplicate-source repair (2026-09-24)

Tracing the vehicle context path exposed a remaining duplicate-chunk mismatch in
`detect_film_i0_layout`. Its candidate census scanned every chunk with a number
in range, whereas native `bipedSlotBand` resolves the first chunk by number. Later
duplicates could enlarge the band and admit unrelated candidate positions. The
new native loaded-source test reproduced 142 Rust samples versus 20 native samples
in case 1 before repair. The detector now resolves each selected number exactly
once, including the next-number keyframe after its six-chunk sample prefix.

- `halo_rust_i0_loaded_test.go.txt` and `i0-loaded-v41.json.zlib` cover 128 loaded
  sources, duplicate/missing numbers, eight source chunks against the six-chunk
  sampling limit, keyframe lookahead, other archetypes, non-delta packets and
  truncated tails. Final oracle: 11,046 samples and 4,776 paired observations.
  Every denominator, flip rate, boundary, candidate layout and refusal matches.
  These synthetic patterns all refuse a valid layout; they verify negative
  detection and source selection, not positive map-layout inference.
- Complementary captured regression: `local_i0_layout_corpus` passed (14.89s).
  Its six existing captured windows verify 66,584 samples and 66,164 pairs, three
  valid layouts (Bazaar and the two Recharge films) and three refusals (Aquarius,
  Cadet Blue and Cadet Brick). The last retains the implausible 47/5/2 candidate.
  Each window reads up to six data chunks plus the next keyframe; this is not a
  claim of full-film creation parity.
- Registered/formatted the native harness and fixture. Current full suite:
  410 Theater tests passed, 32 ignored (49.42s); Clippy with warnings denied,
  WASM library, formatting/diff and generator syntax checks pass. All sessions
  from this checkpoint are terminal.
  Logs: `/private/tmp/halo-i0-loaded-{before,focused,captured,suite,clippy,wasm}.log`.

Next vehicle gate remains captured creation output through the resolved context.
Useful existing wiring: `replication.rs` resolves `ReplayPrecisionContext`, passes
its sampling map into vehicle facts and retains the independent world-object
`PositionEncoding`; `resolve_replay_precision_context` invokes the detector only
when map precision is not installed. Native `NewFilmContextForMap` uses imposed
catalog/forced layout, while the separate D2 `contexteDeBobine` detects then installs
world-object widths. Do not conflate these paths when generating captured oracles.
The existing native march-corpus harness demonstrates loading the local film.json
chunks with `source.Load`; adapt that transport for creation scans without running
the entire Film assembler for every native fixture.

Full parity remains incomplete. No architectural refactor or commits were made.

### Captured vehicle creation gate started (2026-09-24)

Added `scan_vehicle_creations_with_detected_layout`, the native offline-wrapper
path: detect i0 from loaded chunks, install only successful widths/region on a
copy of inherited PositionEncoding, and pass detection failure through the native
report setup order. Failed detection preserves inherited precision; coordinate
bounds remain an independent external input. This does not change the queued
architecture.

New native `halo_rust_vehicle_creation_corpus_test.go.txt` loads real film.json
chunks through `source.Load` and runs both catalog and D2 automatic contexts.
The first four films (Bazaar, Aquarius, Bandit and Oddball) contain no native ti40
slots, so their eight cases are refusal tests, not positive creation coverage.
The hour-long raid supplies the positive automatic case:

- 150 creation records, 87 census slots, 5,670 anchors; 63 overflow, 4,984 mask
  and 473 position refusals; 64 sparse / 86 full masks; 40 records without i0
  declared in the mask. The detected layout is gate 5, axes 15/15/17, region 0.
- Raid map bounds are unknown. The fixture explicitly uses [0,1] per axis with
  module `normalized-coordinate-probe-not-map-bounds`. This tests exact parser
  values/advancement and coordinate normalization, not physical map positions or
  independently verified vehicle actions. Do not present those XYZ as map units.
- Native generation passed in 19.183s. The fixture retains every record and stat,
  layouts, context precision and error status. Generator wiring includes it and
  the formatted harness. No expected creation values were generated by Rust.
- Rust `local_vehicle_creation_corpus` independently reads registry and source
  chunks, detects layout, starts from the inherited native Cliffhanger precision,
  and calls the new automatic wrapper. It compares all ordered records/stats and
  asserts 150 total positive records so an empty-only pass cannot satisfy it.
  It also checks version 41 and registry counts. This test is opt-in because it
  needs the full local films, including the raid.

### Captured vehicle verification complete

Session 52569 completed successfully: all nine captured cases passed, including
150 ordered raid creation records and all scan counters (345.57s). The eight
non-raid cases remain refusal-only coverage. Log:
`/private/tmp/halo-vehicle-corpus-rust.log`.

Audited every runtime function in `vehicle_creation.go`: dynamic i0 decoding,
shared creation walk, both loaded entry points, registry lookup and offline
loading composition have Rust mappings. Marked this source ported-v41. Normalized
raid test bounds do not establish physical map coordinates or gameplay actions.
The previous regular checkpoint passed 410 tests (33 ignored), Clippy, WASM,
formatting/diff and generator syntax; endpoint validation below supersedes it.

Additional source closure: `rev.go` has exactly one runtime declaration, the
pinned `grammar-2026-09-22.12` constant. Verified its hash and declaration-only
body, mapped it to `build_replay_decoder_coverage`, and confirmed the existing
native provenance oracle passes. Marked ported-v41. This is reference provenance,
not a Rust source fingerprint; Go revision-maintenance tests are build tooling.

Full parser parity and captured objective gates remain incomplete. All changes
remain uncommitted on the current branch. Do not start `NEXT_PHASE.md` yet.

### Generic endpoint quantization completed (2026-09-24)

`dequantize_native_endpoint` now ports the full pinned `DequantEndpoint`, rather
than only vitality parameters. It retains unsigned wrapping, unclamped quanta,
exact endpoints, excluded midpoint override in double precision, and IEEE
exceptional results for degenerate widths. Float operation ordering matches the
pinned Go ARM64 oracle. Body/shield and round timers use the shared conversion.
`native_navpoint_manual_timer_seconds` also ports the inclusive dead zone without
assigning gameplay meaning to the recorded timer.

Independent native harness `halo_rust_endpoint_test.go.txt` covers 4,096 generic
cases with independently varied flags, endpoint/interior/out-of-domain quanta,
widths 0/1/2/8/16/17/31/32/63/64/65, reversed/equal/tiny/random bounds; exact f32 bits
match except NaN payloads, for which classification is compared. All 131,072
navpoint timer quanta match exact bits. Existing exhaustive body/shield and
65,536-quantum round-timer oracles also pass. Fixture and harness regeneration are
registered in `generate_oracles.py`. Marked `quantize_endpoint.go` ported-v41;
`components_navpoint.go` retains its broader partial status.

Regular suite: 411 passed, 33 ignored (51.42s). WASM library check passed.
Logs: `/private/tmp/halo-endpoint-{suite,clippy,wasm}.log`.
Full parity remains incomplete and the architecture phase remains deferred.

Final endpoint checkpoint: Clippy with warnings denied, WASM, formatting,
diff checks and oracle-generator syntax all passed. All processes from this
checkpoint are terminal; no captured comparison remains running. Changes remain
uncommitted on `simbleau/theater-experiments`.

### Remaining generic quantization source completed (2026-09-24)

Added `quantization.rs`, exported within Theater. It ports every runtime primitive
in pinned `quantize.go`: equal-width midpoint vectors, bitLen/BitLenExport, and
readQuantStat. Position precision now uses the shared native bit-length helper.
Category bases remain deliberately absent, as in the pinned reference; the packed
word preserves two tail bits in bits 31:30 and the category-dependent raw value.

`halo_rust_quantize_test.go.txt` independently generated 4,096 cases. The final
oracle covers widths 0..128 (including low-64-bit retention for wider fields),
varied bounds, byte lengths 0..32, starting offsets 0..18, categories -3..12,
category-1 probe branches, random uint32 bit lengths and powers of two. Rust
matches f32 bits (NaNs by classification), packed words, and exact cursor ends.
The same cases additionally verify atomic bounded-reader truncation failures.
Explicit internal native-padding mode preserves zero-tail semantics. Source hash
matches the pinned manifest. Harness and fixture regeneration are registered.
Marked `quantize.go` ported-v41; this does not close `lecteur.go` profile/capture/
observer context propagation or the rest of the parser inventory.

Final validation: 412 Theater tests passed, 33 ignored (51.58s); Clippy with
warnings denied, WASM library check, formatting/diff, and generator syntax pass.
Logs: `/private/tmp/halo-quantize-{suite,clippy,wasm}.log`. All processes terminal.

Next source audits remain `lecteur.go`, `keyframe_entity_queue.go` diagnostic
walkers and packet discovery, `keyframe_ground_weapons.go` research grouping and
nearest-sample helpers, and explicit `slot_band_dense.go` semantic/performance
mapping. Positive captured CTF/VIP/bomb gates and the full inventory remain open.
Keep the goal active and architecture deferred. Changes remain uncommitted.

### Ground-weapon research sample utilities (2026-09-24)

Audited `keyframe_ground_weapons.go` against its pinned hash. Existing Rust
keyframe family decoding/loading is already exercised by 512 generated payloads
and four captured films. The source also includes explicitly research-only world
position grouping and nearest-sample queries, previously missing as distinct APIs.

Added `ResearchWorldObjectSample`, `scan_research_world_object_samples` and
`nearest_research_world_object_sample` in `world_object_research.rs`. The loaded
scanner takes explicit resolved precision, preserves every candidate per slot
across generations and native timestamp-only sort tie ordering, and returns empty
for absent bounds, empty bands or unreadable prefixes. Nearest scans unsorted
inputs and keeps the first equidistant sample, including u64 timestamp extremes.
These candidates are not promoted into Film playback or asserted physical actions.
Native comments report substantial false positives; the API preserves that scope.

Independent Go harness uses native source.Load, chunk/packet walking and
scanProjectileRecords, followed by the source's timestamp-only sort. It directly
calls NearestWorldObjectSample for query expectations. All 256 cases / 17,172
candidates match in Rust, including duplicate/missing chunks, truncated packets,
repeated timestamps, empty and missing-input cases, nearest ties and extremes.
This is explicit-profile loaded composition evidence, not direct directory-wrapper
or inherited-default-profile validation. Harness/fixture regeneration is registered
immediately after its WorldTracks input oracle. `keyframe_ground_weapons.go` is
now partial-v41 with explicit mappings and remaining wrapper obligations.

Final research-utility checkpoint: 413 Theater tests passed, 33 ignored (46.36s).
Clippy with warnings denied, WASM library check, formatting/diff and generator
syntax passed. Logs: `/private/tmp/halo-world-research-{suite,clippy,wasm}.log`.
All processes terminal. Current changes remain uncommitted on the requested
branch. Full v41 parity, the remaining source audits and positive captured
CTF/VIP/bomb gates are incomplete; keep NEXT_PHASE.md deferred.

### Keyframe queue diagnostics audit and measurement port (2026-09-24)

Verified the pinned `keyframe_entity_queue.go` hash. This file contains research
packet reconciliation and frame diagnostics; it does not implement a separate
entity-state queue transformation. Added `keyframe_diagnostics.rs` with native
variant enumeration, walk measurement types, coverage/type-index accessors,
ranking of supplied walk measurements, and recovered-anchor spacing/alignment.
Anchor measurements reuse the existing native-policy recovery scanner.

Native `BestVariant` only excludes Overrun and requires strictly greater EndBit;
it does not reject desync despite its prose description. The Rust measurement
ranker preserves that rule, including first-tie retention and the zero-value
result when no positive candidate exists. It is explicitly not a traversal API.

New Go oracle directly calls KFQFrameVariants, BestVariant, Coverage, SortedTIs,
and MeasureKeyframeAnchors. All 256 anchor cases (1,462 recovered anchors) and
256 layout cases (2,832 native walks, 523 overruns, 43 clean NEWs) pass in Rust.
Actual native walk outputs are inputs only to measurement/ranking checks; this
is not evidence that the Rust traversal wrapper is complete. The fixture also
retains payloads for the next traversal comparison. Generator registration added.

Source remains partial-v41. Remaining runtime APIs: WalkPacketRecords/WithWorld,
seeded BestVariant traversal, FindPackets/KFQEqual/KFQPrefix, FirstPacketOfType and
AllPacketsOfType. Audit note: FirstPacketOfType's comment claims chunk0 inclusion,
but the implementation starts at chunk1; port executable behavior. Directory
search ignores unreadable chunks and preserves its CountFilmChunks range rather
than using the production readable-prefix rule indiscriminately.

Final KFQ measurement checkpoint: 414 Theater tests passed, 33 ignored (46.12s).
Clippy with warnings denied, WASM, formatting/diff and generator syntax passed.
Logs: `/private/tmp/halo-kfq-{suite,clippy,wasm}.log`. All processes terminal.
Changes remain uncommitted on simbleau/theater-experiments. Full parity remains
active and incomplete; the architecture phase remains deferred.

### Native packet-discovery helpers completed (2026-09-24)

Added filesystem-only `packet_discovery.rs`: native FindPackets content search,
KFQEqual/KFQPrefix predicates, FirstPacketOfType and AllPacketsOfType. The search
lists immediate film directories lexically and scans all predicates in one pass.
It includes chunk zero; first/all-by-type start at one. All use native numbered
path counting and skip unreadable chunks. Empty prefix is rejected, while exact
empty payloads can match end packets. First-query errors distinguish missing
chunks from missing packet types. Full decompressed chunks are retained through
shared Arc buffers for matched packets, without copying a chunk per match.

Go harness directly invokes all five native APIs on a generated directory corpus:
12 films, compressed/raw files, missing intermediate chunks, numbered directories
that count as paths but cannot be read as chunks, truncated tails, terminal packets
and trailing packets that must remain invisible. Rust reconstructs those inputs
in a temporary directory. Predicate result counts [32,0,48,1,192] match exactly,
including per-result film/chunk/ordinal/type/range/timestamp; all 52 first/all
queries match full decompressed bytes and error category. Missing-root and empty
predicate-list checks also pass. Harness and fixture regeneration registered.

Remaining keyframe_entity_queue.go work is the actual WalkPacketRecords/WithWorld
and seeded BestVariant traversal adapters. The existing measurement ranker is not
substituted for those traversals; the source remains partial-v41.

Final packet-discovery checkpoint: 415 Theater tests passed, 33 ignored (46.61s).
Clippy with warnings denied, WASM library, formatting/diff and generator syntax
passed. Logs: `/private/tmp/halo-packet-discovery-{suite,clippy,wasm}.log`.
All processes terminal. Work stays uncommitted on simbleau/theater-experiments;
full parity remains incomplete and NEXT_PHASE.md remains deferred.

### KFQ traversal and seeded search adapters (2026-09-24)

Added `walk_keyframe_queue_records`, which invokes Rust's native-policy record
body reader with explicit profile and mutates FilmWorld only on clean NEW/DEL.
Unlike production view admission, generic unbound DELTA reads its baseline first
when strict generation checking is disabled. Native padded endings, per-kind
counts, clean-NEW counts/type histogram, exact stop text and overrun prefix match.
`best_keyframe_queue_variant` walks each candidate from a fresh caller seed and
returns all measurements plus the native winner. A pinned research-default
encoding factory is verified against native profile values; these defaults are
not a claim of universal map precision for v41 films.

Extended the existing native KFQ harness with profile values and 128 seeded
record sequences. Rust now generates (rather than just ranks supplied) all 2,832
random native walk results. The seeded cases compare direct results and final
world bindings, plus all 2,304 alternative candidate walks and their winners.
They cover hard, wrong-generation, soft and absent initial bindings; 80 clean
NEWs; DEL/recreation; extra fields; preambles; and truncated packets. Tests passed.

Source remains partial pending broader component-rich/captured traversal context
and out-of-range diagnostic configuration audit. The fixtures do not yet justify
full source closure merely because the empty-component and high-frequency paths
match. Other v41 parity gates and NEXT_PHASE.md remain unchanged.

Final KFQ traversal checkpoint: 416 Theater tests passed, 33 ignored (49.42s).
Clippy with warnings denied, WASM, formatting/diff and generator syntax passed.
Logs: `/private/tmp/halo-kfq-traversal-{suite,clippy,wasm}.log`. All processes terminal.
Changes remain uncommitted on simbleau/theater-experiments; full parity remains
active and incomplete, and the architectural phase stays deferred.

### KFQ component and captured-film closure (2026-09-24)

Extended generic traversal beyond the empty/high-frequency fixtures. Native
`halo_rust_kfq_components_test.go.txt` reads the actual v41 registry and exercises
all 1,067 slots with their archetype and level: three zero/random/truncated modes,
3,201 cases. Every Rust traversal summary matches, including stopping fields.

Native `halo_rust_kfq_corpus_test.go.txt` reads four local films (Bazaar, Aquarius,
Bandit, Oddball), selecting six delta and two keyframe packets each. Each packet
is tested against empty and recovered-keyframe seed worlds across 24 layouts.
Rust independently loads the real files, checks version41, parses the registry
and recovers seed anchors. All 64 cases / 1,536 walks and winners match: 704 returned
records, 110 clean NEWs, 512 overruns, 55 distinct stop diagnostics; longest walk
returns 48 records. Captured opt-in test passed (1.15s), log
`/private/tmp/halo-kfq-corpus-rust.log`. These diagnostic comparisons do not claim
complete sequential parsing of the selected films or canonical seed correctness.

Completed source-by-source mapping for every runtime declaration in
`keyframe_entity_queue.go`; pinned hash still matches. Marked ported-v41. Rust
accepts valid entity ID layouts with 0..30 low bits and explicitly rejects
nonrepresentable/invalid research configurations instead of reproducing Go
integer overflow or panic behavior. Empty-world construction is FilmWorld::default;
optional seed callbacks are represented by the supplied factory. Filesystem
reconciliation, pure predicates, variants, anchor measurements, traversal and
ranking all have native evidence. Both new harnesses are registered for regeneration.

This closes the earlier component-rich/captured KFQ gap, not the broader parser
goal. Positive captured CTF/VIP/bomb, remaining source/profile audits and the
existing acceptance gates are still outstanding. Keep NEXT_PHASE.md deferred.

Final broad-KFQ checkpoint: 417 Theater tests passed, 34 ignored (46.18s), plus the
captured KFQ test passed separately. Clippy with warnings denied, WASM, format/diff
and generator syntax passed. Logs: `/private/tmp/halo-kfq-broad-{suite,clippy,wasm}.log`.
All processes terminal; all edits remain uncommitted on the requested branch.

### Dense slot-band source port (2026-09-24)

Audited pinned `slot_band_dense.go` and added FilmSlotBand in `slot_band.rs`.
Native NewSlotBand/Has/Count/Slots map to flag/set/slot constructors, contains,
count and ascending slots. False flags (including u32::MAX) do not increase the
domain. Duplicate slot inputs count once. Allocation is at least 8,192 booleans
and grows for higher present slots; cardinality is tracked separately. Default
empty state behaves like native's zero value. Allocation/representability errors
are explicit Rust errors. Membership is O(1); construction and enumeration scale
with the allocated domain, not just the number of present slots.

The biped payload scanner now converts its allowed-slot set once per payload and
uses indexed membership inside the bit-candidate loop. Other existing BTreeSet
users preserve semantic membership/order; no whole-parser performance claim is
made and no unrelated scanner was rewritten.

Native `halo_rust_slot_band_test.go.txt` directly exercises 256 flag maps with
263,680 membership queries, counts and sorted slots. All Rust outputs match;
set conversion and duplicate input construction agree as well. Includes empty
maps, false entries, normal 13-bit boundaries, wider world slots and out-of-range
queries. Harness and fixture are registered in generate_oracles.py.

Final dense-band checkpoint: 418 Theater tests passed, 34 ignored (46.05s).
Clippy with warnings denied, WASM, formatting/diff and generator syntax passed.
Logs: `/private/tmp/halo-slot-band-{suite,clippy,wasm}.log`. All processes terminal.
Marked `slot_band_dense.go` ported-v41. Changes remain uncommitted; full parser
parity and captured objective gates remain open. Architecture remains deferred.

### Profile value sources and provenance audit (2026-09-24)

Added `profile_values.rs` with FilmMppWidths (signed values, native positive-only
Valid semantics, default and lead/index display) and named native quantization
ranges. Reader width limits remain separate from profile-value validity. The
build range maps to existing BUILD_DEFAULT_POSITION_BOUNDS; the captured biped
range now supplies the existing resolved-profile capture bounds. The rejected
historical Cliffhanger range is explicitly named/documented as rejected evidence
and is never substituted into decoding.

Native oracle directly calls exported profile values/functions from the working
grammar test package: all 81 signed MPP combinations (including zero, negative,
wide and i64-extreme values), default widths and exact float32 bits of all six
ranges match. The profile test package itself encountered an unrelated ooz/x86
intrinsic build failure on this host; importing the actual profile runtime through
the grammar harness succeeded without modifying its implementation. Registered
harness and fixture regeneration in generate_oracles.py.

Verified pinned hashes and full declaration inventories for five profile sources.
`precision.go` has only IndexW/AxisW/Region: all are represented by
WorldObjectPrecision, with existing direct and loaded descriptor oracles; marked
ported-v41. `doc.go` is comments/package declaration only; marked reference-only.
`rev.go` has one runtime constant, already matching decoder coverage provenance;
marked ported-v41. This is reference provenance, not a Rust-source fingerprint.
MPP/range sources await final regression checkpoint below.

Final profile-value checkpoint: 419 Theater tests passed, 34 ignored (46.76s).
Clippy with warnings denied, WASM, formatting/diff and generator syntax passed.
Logs: `/private/tmp/halo-profile-values-{suite,clippy,wasm}.log`. All processes
terminal. Marked mpp_widths.go and plages_quant.go ported-v41. The next profile
source is profile_table.go (ordered provenance rows and layout mappings).
Full v41 parity remains incomplete; changes stay uncommitted and architecture
remains deferred.

### Ordered profile table and provenance port (2026-09-24)

Audited complete pinned `profile_table.go`: all six row builders feed TableProfil,
followed by the gamertag layout type/constants/selector. Added typed table rows
and provenance in `profile_table.rs`, backed by the direct native TableProfil
export in `reference/profile-table.json`. All 33 rows retain all six fields in
native order (format, build, major, invariants, grenade-build, grenade-major).
Historical rows are retained as reference metadata, not additional decode support.
The table returns fresh owned values; changing a result does not mutate later calls.

The schema round-trip retains every native field. Source-audited assertions cover
row order/values, unresolved MPP values, v41 gamertag layout, major31's distinct
0x20400 grenade prefix, and the exact 7 reviewed / 21 measured / 5 presumed
provenance counts. These categories are the reference's claims, not stronger new
certainty. The named gamertag layout selector matches 103 native resolved-profile
outputs (majors -2..100); the actual highlight decoder remains restricted to v41.

Native generation calls public TableProfil and Resoudre/Highlight through the
working grammar package, with no Rust-generated expectations. The catalog and
layout oracle are registered in generate_oracles.py. Focused table/schema/layout
comparison passed. Full runtime row declaration mapping is complete; final static
checks are recorded below.

Final profile-table checkpoint: focused test, Clippy with warnings denied, WASM,
formatting/diff and generator syntax passed. No decoder path changed, so the full
419-pass/34-ignored prior suite was not repeated or relabeled. Logs:
`/private/tmp/halo-profile-table-{clippy,wasm}.log`. All processes terminal.
Marked profile_table.go ported-v41. Full parser parity remains incomplete; all
changes remain uncommitted and the architecture phase stays deferred.

### Native source octets audit (2026-09-24)

Added `source_octets.rs` with all seven octets.go primitives: four native endian
integer reads, unaligned complete-byte reads, little-endian words assembled from
independently bounded bytes, and first fully bounded 64-bit pattern search.
Incomplete octets return zero instead of padding their available bits. Negative
word starts can still expose later complete bytes. Integer-offset reads retain
the native panic contract for incomplete input; signed addition overflow in the
unaligned helpers safely produces zero instead of reproducing native overflow.
Existing readers have not been migrated wholesale; no decoder path changed.

Pinned native grammar harness exports 96 buffers (empty, repeated, randomized),
49,056 unaligned read pairs, 32,248 searches (including every available bit-aligned
window and random misses), and 3,916 four-integer tuples. Rust comparisons pass;
Clippy with warnings denied, WASM library check, fmt/diff and generator syntax
pass. Native harness/fixture registered in generate_oracles.py. Logs:
`/private/tmp/halo-source-octets-{clippy,wasm}.log`. All processes terminal.
Full suite was not repeated for these standalone utilities; previous full-suite
results remain at their own checkpoint.

Marked octets.go ported-v41; source/doc.go is documentation only; source/rev.go's
sole constant matches existing coverage provenance. source/bits.go remains
partial: audit remaining signed/tolerant/truncated/wide reads and reader accessors
without conflating boundary conventions. Overall v41 parity remains incomplete,
including captured CTF/VIP/bomb gates and remaining source inventory. Architecture
requirements remain deferred in NEXT_PHASE.md; all changes remain uncommitted.

### Native source bit conventions (2026-09-24)

Completed source/bits.go declaration mapping with `source_bits.rs`. The native
borrowed reader exposes original octets, bit length, signed position, absolute
resynchronization, backward/unbounded skips, signed remaining count, bit reads,
and wide reads retaining low 64 bits. Separate scalar, single-bit, two-sided
zero-padded, and truncated helpers preserve their distinct boundary contracts.
The bounded FilmBitReader and its production users remain unchanged.

Pinned native oracle: 34,632 cases over 24 buffers, starts -9 through tail+9 and
widths -1,0,1,7,8,9,31,32,63,64,65,96,128. Values, panic outcomes, final cursor and
remaining counts all match. In particular, truncated negative starts -7..-1
produce leading zeros due to native signed division; earlier negative starts
panic. Sequential negative nonempty reads panic without advancing. Borrowed
buffer identity and sequential bit reads are also tested. Native width arguments
above u32 and overflowing signed read spans are explicitly outside the v41 API;
overflowing read spans are rejected, not silently interpreted as wire data.

Focused oracle test, Clippy with warnings denied, WASM, formatting/diff and
Python syntax checks passed after the final guard edit. Logs:
`/private/tmp/halo-source-bits-{test,clippy,wasm}.log`. All processes terminal.
No existing decoder path changed; full suite not repeated. Marked bits.go
ported-v41. The native source directory inventory is now mapped, but complete
parser parity is still open. Next audit: remaining lecteur.go profile/observer
context and ground-weapon directory/default-profile paths. Architecture remains
deferred, and all changes remain uncommitted.

### Ground-weapon loaded and directory wrappers (2026-09-24)

Added scan_source_keyframe_ground_weapons and native-target
scan_directory_keyframe_ground_weapons. These use FilmSource's native numbering
and packet lookup directly and retain boolean catalog flags: an empty catalog
bypasses I/O, whereas nonempty all-false catalogs still load and check for a
readable data chunk. GroundWeaponScanError separates source-loading failure from
no-readable-data failure; successful empty results remain distinct. The existing
FilmChunkData convenience scanner is unchanged.

Native directory oracle covers 128 cases and 337 complete records, including
missing/empty directories, registry-only, missing registry, a numbered gap,
malformed data, truncated final packets, non-keyframe prefixes, exact packet
indexes/timestamps and family ordering. Rust results and error categories match.
Harness and fixture are registered in generate_oracles.py. Focused test, Clippy
with warnings denied, WASM, formatting/diff and Python syntax passed. Logs:
`/private/tmp/halo-ground-directory-{test,clippy,wasm}.log`. All processes terminal.
Full suite not repeated because existing decoder paths were unchanged.

keyframe_ground_weapons.go remains partial: its research-position directory
wrappers must still use the native context's detected I0 layout or invariant
fallback (not arbitrarily supplied map precision). Existing world_precision.rs
contains detected-layout support to inspect before implementing those wrappers.
lecteur.go declaration audit maps source reader promotion and signed codec, but
profile/observer replacement and context inheritance still require explicit API
and caller evidence. No architecture refactor has started; parity remains active
and all changes remain uncommitted.

### Ground-weapon research directory closure (2026-09-24)

Completed the remaining keyframe_ground_weapons.go research wrappers in
world_object_research.rs: directory slot bands, explicit-band samples, and
GroundWeaponPositions-equivalent samples. These preserve silent empty results
on loading failures, absent bounds and empty bands. Directory conversion uses
native numbered-prefix lookup; unknown/oversized file numbers cannot alias valid
chunks. The helper installs successful recording I0 detection, or retains native
index1/axes13,13,14/region0. Coordinate bounds do not choose scanner widths.
Research candidates remain explicitly separate from validated playback positions.

Native harness directly invokes all three directory APIs. Thirty-two generated
cases match 90 explicit-band and 162 ground-band candidates. Two captured
prefixes (chunks through 7) match band membership, precision and all 425
explicit-band plus one ground-band candidate: Bazaar detects axes17,17,16;
Aquarius retains invariant axes13,13,14. Rust deliberately passes unrelated map
widths/region to demonstrate that only coordinate bounds come from that input.
This is research-wrapper parity, not evidence that those candidates represent
physical ground-weapon locations.

Both focused tests, including the opt-in captured test, passed (3.25s). Clippy
with warnings denied, WASM, fmt/diff and generator syntax passed. Logs:
`/private/tmp/halo-research-directory-{test,clippy,wasm}.log`. All processes terminal.
Full suite not repeated because existing production decoder paths are unchanged.
Marked keyframe_ground_weapons.go ported-v41 after full declaration mapping.
Overall parser parity remains incomplete; architecture is deferred, all edits
are uncommitted. Next: keyframe_loadout.go uses a different existing packet-index
path than native prefix scanning; audit its missing/unreadable/duplicate/gapped
input behavior and add loaded-source/directory wrappers with direct evidence.

### Keyframe loadout source parity and production scan repair (2026-09-24)

The native source oracle exposed a real existing-path mismatch: loadouts used
packets::index, which rejected duplicates and truncated tails, sorted/scanned
past numbering gaps, and filtered on metadata role. Native loadouts walk the
input-order numbered prefix and retain complete packets before a malformed tail.
The initial Rust fixture failed on case2, where readable empty buffers should
produce a successful empty scan rather than a strict packet-index error.

Changed the existing FilmChunkData loadout scanner to shared native prefix and
packet rules. Added direct FilmSource and offline directory APIs with boolean
family maps. Empty maps bypass I/O; nonempty all-false maps still require readable
chunks. Native loading and no-readable-data failures remain separate typed cases.
The full suite caught the top-level partial-film constructor propagating this
newly faithful no-readable error; it now handles unavailable loadouts consistently
with its existing ground-weapon/fire scan handling. The scanner itself retains
its native error contract. No fabricated records or inferred loadouts are added.

New native oracle: 128 directory/source cases, 178 directory records and 152
loaded records, covering role flags, duplicates, gaps, out-of-order metadata,
missing directories/registry, malformed/truncated buffers, catalog flags,
packet ordinals and family ordering. Existing pure 256-case attribution test
passes. Full source declaration audit includes recordContaining equivalence to
the monotonic last-anchor selection in families_by_entity.

Final full Theater suite: 426 passed, 36 ignored (51.92s). The separate four-film
captured test matches all 501 loadouts through both loaded-source and existing
FilmChunkData APIs (161.14s). Clippy with warnings denied, WASM, fmt/diff and
Python syntax passed after the constructor fix. Logs:
`/private/tmp/halo-loadout-{before,after,suite,corpus,clippy,wasm}.log`.
All processes terminal. Marked keyframe_loadout.go ported-v41. Full parser parity
remains incomplete; current inventory still includes 132 partial, 174 pending,
23 partial-v41 entries, plus the captured objective gates. Architecture remains
deferred and all changes stay uncommitted on simbleau/theater-experiments.

### Objective and movement data-contract audit (2026-09-24)

Added TYPE_CONTRACTS.md with field-level mappings for all eight data-only
objective structs, both movement structs and the two ability stats structs.
The native harness serializes actual Go types: 768 instances across 12 types
match typed Rust round trips, including presence flags, scan status, nested
ordered spans, component maps and arrays. This is declaration/schema evidence,
not a substitute for objective algorithm or independently annotated action tests.
Native nil versus allocated-empty collection identity is outside this value-shape
comparison; source element values/order are the relevant decoding contract.

Exported the native five movement labels and four jump-derivation constants.
Existing derivation now uses the named constants with identical arithmetic and
values. RiseMinMS is a speed in metres/second, not milliseconds; the Rust name
makes that explicit. `jumpDerived` remains distinct from recorded input.
All nine constants match the native runtime export. Existing velocity/jump
sequence comparison passes after the substitution.

Two focused tests passed (0.62s); Clippy with warnings denied, WASM, fmt/diff and
generator syntax passed. Logs: `/private/tmp/halo-type-contracts-{tests,clippy,wasm}.log`.
All processes terminal. Full suite not repeated for declaration coverage and
literal-to-equivalent-constant substitution; the prior 426-pass/36-ignored suite
remains its own checkpoint. Marked types/objectives.go and
 types/grammar_mouvement.go ported-v41, types/doc.go reference-only.
Ability data declarations remain partial beyond the two verified stats structs.
The complete v41 parser goal remains active; architecture remains deferred and
all changes remain uncommitted.

### Kill-source loaded view and helper audit (2026-09-24)

Ported chunks.go's native service view in kill_source_film.rs. KillSourceFilm
borrows FilmSource and exposes all non-CHUNK_END packets, source-position/original
ordinal identities and timestamp-sorted replication indexes. It retains native
no-chunk/no-replication errors, optional recorded major version, unsigned
millisecond subtraction, and the event-list marker. Payloads borrow the original
source bytes; pointer assertions confirm no payload copying. This is the native
kill-source adapter, not the deferred resolved replay architecture. Existing
production consumers have not been migrated wholesale.

Native oracle: 128 source cases, 8,484 packets and 4,142 ordered deltas match,
including sources with 66 chunks, numbering gaps/duplicates, missing metadata,
missing/short registry headers, timestamp ties/u64 extremes, earlier non-delta
packets causing unsigned wrapping, malformed tails and nil sources. Native sort
requires a noncapturing comparator; the adapter sorts index/timestamp pairs to
preserve tie behavior. Native fixture/harness registered in generate_oracles.py.

Full declaration audit also closes botmeta.go and paquet_identite.go against
existing implementations. Rerun evidence: 512 generated bot cases and 34 captured
bot fixture entries pass (2.25s); 1,024 native kill matching/hybrid cases pass
(1.28s). The new source test passes (0.19s). Clippy with warnings denied, WASM,
formatting/diff and generator syntax pass. Logs:
`/private/tmp/halo-kill-chunks-{test,bots,matching,clippy,wasm}.log`.
All processes terminal. Full suite not repeated for the standalone adapter and
source mapping; last full suite remains 426 passed/36 ignored. Marked chunks.go,
botmeta.go and paquet_identite.go ported-v41; doc.go reference-only. Overall
v41 parity remains incomplete. Architecture stays deferred and edits uncommitted.

### Named statistic components and score-source closure (2026-09-24)

Added StatborgComponent with all four native StatComponent fields, key conversion,
round/total methods and seven named native descriptors: mode score, personal
score, kills, deaths, assists, skull ticks and skull grabs. Strict growth and
unitary action bounds remain independent: personal-score cadence steps are not
subject to the action-counter limit. Caller-supplied descriptors retain both
policy switches. Existing series functions remain the underlying implementation.
Extracted the equivalent shared two-channel mode-score domain guard.

Native oracle invokes real SeriesByRound/SeriesTotal: 128 record sets, 1,920
queries, 96,394 total-series points, all seven catalog descriptors, custom
components/policies, player/team filtering, repeated values, reversed time order,
multiple rounds, invalid counters and 64 domain-boundary/extreme signed pairs.
All values and ordered points match. Eight focused statborg tests passed (8.74s),
including existing decoder, round, identity, named-event and event-budget checks.
Clippy with warnings denied, WASM, fmt/diff and generator syntax passed. Logs:
`/private/tmp/halo-stat-components-{tests,clippy,wasm}.log`. All processes terminal.
Full suite not repeated; existing domain predicate was extracted without changing
its condition, and the new descriptor API delegates to existing series routines.

Marked score.go ported-v41 after complete declaration mapping. Audit found a
remaining named_bounds.go gap: the current Rust budget carries counts/remaining/
truncated state, but not the native pass-origin and bounded rejection/exhaustion/
summary diagnostics. That file remains partial-v41, explicitly recording the
missing output instead of relying on count agreement. Next work should preserve
those diagnostics with native publication-order evidence. Full v41 parser parity
remains incomplete; architecture is deferred and changes remain uncommitted.

## Objective-budget diagnostic checkpoint (2026-09-24)

Added opt-in `StatborgBudgetDiagnostics` and ordered `StatborgBudgetDiagnostic`
records. The existing budget delegates to the same increment implementation with
a no-op observer, preserving its serialization, equality and allocation behavior.
The observer retains pass origin, exact native WARN text and ordered attributes,
the eight-detail rejection limit, exhaustion, and repeated explicit summaries.
Clean summaries and increment calls after exhaustion remain silent. This is
native diagnostic publication, not an inferred gameplay event stream.

The pinned Go harness captures real slog records: 64 cases / 256 calls, 1,087
rejected steps, and 638 publications (327 detailed rejections, 41 exhaustion,
270 summaries). Rust matches messages, attribute values/order, publication order,
event times and final budget state. Nine focused statborg tests passed (8.55s).
Clippy including tests with warnings denied, WASM compilation, fmt/diff checks
and generator Python syntax passed. Logs:
`/private/tmp/halo-budget-{tests,clippy,wasm}.log`. No live processes remain.
The full Theater suite was not repeated at this checkpoint.

Keep named_bounds.go partial: thread diagnostics through the native-equivalent
named-events, slot-identity and cross-check entry points next, verifying their
origins (`named_events:<mode>`, `slot_identity`, `cross_check:<mode>`) and summary
placement against the reference. The new API currently observes explicit budget
calls only. Full v41 parity, including captured CTF/VIP/bomb gates and the source
inventory, remains incomplete. Changes remain uncommitted; NEXT_PHASE.md still
queues the requested architecture proposal after parity, with no refactor begun.

## Objective-budget pass integration and exhaustion (2026-09-24)

Added diagnostic-returning named-event, slot-identity and cross-check APIs. Each
shares the original implementation, preserves sorted budget-consumption order,
uses the native pass origin, and summarizes at the native call site. Legacy APIs
still disable observation and preserve their output contracts. The cross-check
result has a public type alias to keep its diagnostic-returning signature readable.

The pinned native pass oracle covers 64 inputs, 108,418 complete ordered events,
639 ordered WARN records, identity values, and cross-check differences. Cases
include unsupported modes, empty inputs, rejected steps, multiple slots/rounds,
and a rejection-detail cap shared across counters. A separate stress oracle uses
62,503 input records and the real 1,000,000-event limit. It compares count plus
FNV-1a over every ordered named-event field, the resulting identity/cross-check
values, and six exhaustion/summary warnings. Expectations are generated by the
pinned Go implementation; the controlled stress input is reconstructed in Rust.
Both harness operations and fixtures are registered in generate_oracles.py.

Validation: eleven focused statborg tests passed (8.59s). Full Theater suite:
432 passed, 36 ignored (51.57s). Clippy with tests and warnings denied passed after
factoring the complex cross-check signature into a type alias. WASM compilation
passed before that alias-only change; fmt/diff checks and generator syntax passed.
Logs: `/private/tmp/halo-budget-passes-{tests,suite,clippy,wasm}.log`.
All processes are terminal. No captured-film tests were rerun at this checkpoint.

named_bounds.go is now ported-v41 based on complete declaration mapping and the
low-level/public-pass/exhaustion evidence. The audit of named_series.go found the
next concrete gap: ChronologicalTotal drops backward timestamps correctly but
omits the native warning containing slot, dropped/retained counts and first
backward timestamp. It remains partial-v41; port and test that publication and
its inherited cumulation callers next. Broader source/output inventory and
positive captured CTF/VIP/bomb gates remain open. Full parity is incomplete,
changes remain uncommitted, and the queued architecture has not begun.

## Chronology diagnostics checkpoint (2026-09-24)

Preserved native backward-timestamp WARN records and propagated observation
through raw cumulative series, statistic descriptor totals, named-event,
slot-identity and cross-check passes, plus replay round-series construction before
clock clipping. The shared diagnostic record is now named StatborgDiagnostic;
the former budget-specific name remains an alias. Existing value APIs delegate
without observation and their outputs remain unchanged. This is diagnostic
parity work, not the queued Film/ResolvedFilm architecture refactor.

Pinned native oracle: 128 direct/cumulative cases, 67 direct +213 cumulative
warnings, and 3,814 warnings through descriptor/objective passes. Covers negative
and extreme timestamps, first-point retention, repeated/equal timestamps,
backward drops, discarded rounds, multiple slots and interleaved budget warnings.
A refreshed replay oracle captures all 976 warnings in 1,024 cases; every old
score output was compared unchanged before adding diagnostics to that fixture.
Native Go's cumulative slot-map warning order is unspecified. Rust emits those
independent warnings in ascending slot order, documented and tested; the per-slot
pass fixtures assert exact chronology/budget interleaving and attribute order.

Validation: initial12 focused statborg tests passed; the expanded chronology
pass test passed. Full Theater suite433 passed,36 ignored (53.85s), including
updated replay warnings. Clippy including tests with warnings denied, WASM,
fmt/diff and generator syntax passed. Logs:
`/private/tmp/halo-chronology-{statborg,focused,suite,clippy,wasm}.log`.
All handles are terminal. No local captured-film tests were rerun.

named_series.go now has complete declaration mapping and source-level evidence.
Next concrete audit: objectives/film.go and statborg.go loaded-source adapters,
missing-manifest INFO publication and record-cap WARN publication. Existing
scan_film_statborg takes aligned FilmChunkData and retains truncation state but
does not expose those diagnostics or the nil/partially aligned FilmSource entry
point. Verify selection/timing/order against native source loading before adding
named-event/identity loaded wrappers. Reader profile/observer replacement,
remaining inventory, full document propagation, and positive captured CTF/VIP/bomb
validation remain open. Full v41 parity remains incomplete; no commit or refactor.

## Loaded statborg source and diagnostics (2026-09-24)

Added scan_source_statborg and its diagnostic-returning variant. They consume
FilmSource metadata by position, including shorter/longer metadata slices,
negative/duplicate file numbers and arbitrary nonzero types. Nil or undescribed
sources preserve the native INFO, distinct from a described film with no records.
Per-chunk time starts at its first FRAME, including record-free frames; subtraction
wraps as u64 before conversion and the signed metadata offset wraps as in Go.
The shared scanner is also used by the existing FilmChunkData path.

Record-cap handling preserves complete frames. The limit remains33,076; a frame
may overshoot it. Diagnostic WARN attributes retain match ID, actual record count,
limit and metadata chunk index. Stable sort remains time/slot/round. Legacy stream
serialization and truncation fields are unchanged.

Native oracle:128 sources,680 records across63 positive sources,55 missing-manifest
INFO records; covers zero-size packet termination, malformed tails, CHUNK_END,
record-free first frames and unsigned/signed timestamp extremes. The initial
harness accidentally made every first FRAME zero-size (which terminates native
walking); it was corrected to use nonempty record-free payloads, retaining explicit
zero-size cases. A record-count assertion prevents that empty fixture regression.
The real-cap fixture compares all33,078 records after whole-frame overshoot and
its exact WARN, and confirms subsequent chunks are not consumed. Both loaded and
legacy aligned APIs agree. Harnesses are registered in the generator.

Validation: two focused source tests passed (0.72s). Full Theater suite435 passed,
36 ignored (50.96s). Then added and explicitly ran the opt-in loaded-source corpus
test: all16,113 records and truncation flags match the pinned native oracle across
32 v41 films, including ranked games and the hour-long raid (70.89s). This adds one
ignored test to the suite inventory; the full suite was not repeated merely for
that opt-in addition. Clippy including the new test, WASM, fmt/diff and generator
syntax passed. Logs: `/private/tmp/halo-statborg-source-{tests,suite,corpus,clippy,wasm}.log`.
All handles are terminal.

Corrected film.go's historical ported-v41 status to partial-v41: an empty input did
not preserve its missing-manifest log. The new statborg adapter preserves that
publication, but other chunksDatables consumers still need loaded-source/diagnostic
mapping. statborg.go stays partial pending complete declaration/constant audit and
loaded downstream wrapper closure. Next: native NamedEvents/CrossCheckNamedEvents/
SlotIdentity loaded wrappers and other objective source primitives. Broad inventory,
reader context/profile replacement, document propagation and positive captured
CTF/VIP/bomb gates remain open. Full parity is incomplete. Changes stay uncommitted;
the architecture phase remains queued.

## Loaded named-event and identity entry points (2026-09-24)

Added statborg_source_named_events, statborg_source_cross_check_named and
statborg_source_slot_identity. Each scans once, then invokes the existing
record-based diagnostic pass. Source diagnostics precede pass diagnostics; source
scanning still occurs for unsupported objective families, as in native wrappers.
StatborgSourceOutput retains the native value, source truncation flag and ordered
diagnostics. Its truncation flag is explicitly source-only; event-budget exhaustion
has its own diagnostic. Consumers needing several projections can scan once and
reuse the record-based APIs. Existing Film/document consumers are unchanged.

Expanded native source oracle:128 sources,102 named events,55 source diagnostics
per wrapper. The negative identity/cross-check cases are complemented by an
explicit encoded positive control yielding six events, one named actor and two
cross-check differences. A strengthened real-record-limit fixture retains33,078
records with rejected counter steps; each wrapper's five warnings prove source
truncation is published before three pass rejections and its summary. Source
truncation stays visible in each returned report. The native harness asserts the
intended positive control outcome independently of Rust. All fixtures are generated
from the pinned Go entry points. Initial Rust test deserialization hit Serde's
flattened integer-map-key limitation; test rows now deserialize directly from JSON
values, without weakening any output comparisons.

Validation: five focused source/pass tests passed, one local-corpus test ignored
(1.55s). Clippy including tests, WASM, fmt/diff and generator syntax passed. Logs:
`/private/tmp/halo-statborg-source-passes-{tests,clippy,wasm}.log`.
No full-suite or captured run was repeated for these additive wrappers; the prior
435-pass suite and32-film/16,113-record source comparison remain the last broader
checks. All process handles are terminal. Changes remain uncommitted.

The complete declaration audit closes named.go and slotidentity.go, using their
existing series/identity/attribution oracles plus the new loaded and diagnostic
checks. Corrected the prior caller-audit assumption: chunksDatables has exactly
ONE production caller (StatRecordsCtx), now covered. There are no other users of
that missing-manifest logger. Remaining film.go/extract.go work is loaded footer
selection and capture aggregation, using manifestChunks and framesOf. In particular,
footerData selects the first strictly highest index above -1; metadata beyond the
loaded buffer can select a nil footer and must not fall back to a lower cached one.
Next port those loaded primitives and compare selection, borrowing, native nil
behavior and complete capture/footer outputs. Broader inventory, reader context,
document propagation and positive captured CTF/VIP/bomb gates remain open. Full
v41 parity is incomplete; NEXT_PHASE architecture remains deferred.

## Loaded footer, capture and objective extraction (2026-09-24)

Added objective_manifest_chunks, objective_source_footer,
scan_source_objective_footer, scan_source_capture_bursts and
extract_source_objective_events. Metadata positions select buffers. The footer
selector keeps the first strictly highest type-3 index above -1, including a row
beyond loaded buffers; it does not fall back to a lower cached footer. Selection,
missing bytes, and loaded-empty bytes remain distinct. Selected bytes are borrowed
from FilmSource. Type-2 capture aggregation preserves first-FRAME exclusion and
native signed wrapping before microsecond-to-millisecond division. The extraction
entry point retains the existing explicit v41 guard.

Independent source oracle:128 cases,40 footer events,224 capture timestamps and
18 selected-but-unloaded footers. Tests compare complete metadata selections,
nil/empty semantics, bytes, pointer borrowing and decoded outputs. The existing
1024-case objective oracle now exercises both legacy and loaded APIs, comparing
all footer/capture/extracted fields and team controls. The local32-film test also
checks both paths:159 footer events,4 capture timestamps,4 extracted events and
all team controls match the pinned native oracle (53.35s). Those native capture
observations are not independent proof of completed real CTF matches.

Validation: four focused objective tests passed, one local test initially ignored
(1.70s), then the local corpus test was run explicitly and passed. Clippy including
tests, WASM, fmt/diff and generator syntax passed. Logs:
`/private/tmp/halo-objective-source-{tests,corpus,clippy,wasm}.log`.
All handles terminal. No full suite repeated for the additive source wrappers;
prior435-pass suite remains the last broad run. The source harness is registered
in generate_oracles.py; it uses the registered statborg source packet helper.

film.go's source-level gap is now closed after complete primitive mapping;
extract.go's entry-point mapping includes loaded footer/capture/extraction APIs.
The earlier missing-manifest uncertainty is resolved: StatRecordsCtx is the sole
chunksDatables caller, and its INFO is covered by the preceding source work.
Next priorities remain explicit declaration/constant audit of statborg.go and
reader profile/observer replacement in lecteur.go, plus broader inventory and
full document propagation. Positive captured CTF/VIP/bomb gates remain open.
Full v41 parity remains incomplete. Changes are uncommitted and the queued
three-layer architectural proposal/refactor has not begun.

## Statborg round helpers and declaration closure (2026-09-24)

Exposed the native mode-score run lengths, written-round set, material-round set,
future-admission query and contiguity decision. The existing round resolver now
composes these helpers rather than discarding their intermediate results. Named
five native thresholds and retained all previous written/real/contradicted/decreed
outputs. This is current reference-parser parity work, not the queued replay-model
architecture. STATBORG_PORT.md maps every statborg.go declaration and constant to
its Rust implementation and evidence.

A direct pinned Go oracle covers128 record sets with116,670 records,905 run entries,
422 material-round entries and1,536 future-round queries. It tests count boundaries
24/25/26 and249/250/251,10-percent shares, player/team distinctions, invalid mode
channels, stable timestamp order and rounds outside the admitted0..7 interval.
Another256 independently supplied helper maps cover false boolean entries and
contiguity gaps. All five threshold values are compared directly with native Go.
The prior1,024 round-decision/boundary cases also pass unchanged.

Validation: two focused round tests passed (1.43s). Full Theater suite440 passed,
37 ignored (52.14s). Clippy with tests and warnings denied, WASM, fmt/diff and
Python generator syntax passed. Logs:
`/private/tmp/halo-round-helpers-{tests,suite,clippy,wasm}.log`.
All handles terminal. No captured scans were rerun for the helper extraction;
previous32-film statborg source and objective source checks remain current evidence
for those unchanged scanners. statborg.go is now ported-v41 at source level.

Next reader-context audit: native PoserContexte/poserCadre have production callers
throughout frame/keyframe/scanner paths; no non-test calls of PoserProfil/Profil/
poserMouvement were found. equipment_creation.go installs its context and then
reinstalls the same w.obs pointer; this is redundant, not a distinct override.
Verify inherited profile/capture state and the explicit replacement contracts before changing
reader representation. lecteur.go stays partial. Full document propagation,
remaining inventory and positive captured CTF/VIP/bomb gates remain open. The full
v41 goal remains incomplete, changes uncommitted, and architecture deferred.

### Reader-context creation audit (2026-09-24)

Added [READER_CONTEXT_PORT.md](READER_CONTEXT_PORT.md), mapping every lecteur.go
operation to its contract and current Rust representation or explicit gap.
The equipment observer installation is redundant: contexte() already includes
the exact same w.obs pointer. Each candidate and ammo traversal starts a fresh
reader; there is no previous candidate's capture state to inherit.

The frame multiplayer-properties component does not invoke the default-state MPP
hook. Expanded the registered native ground-creation profile harness to traverse
component9 before ammo in selected cases. All512 cases match complete records and
counters:403 ammo records, including36 with MPP, and246 cases affected by inherited
profile settings. The Rust assertion requires positive MPP-plus-ammo coverage.
Native Go oracle passed; focused Rust test passed (0.19s after compilation).
Log: `/private/tmp/halo-reader-creation-test.log`. Production code is unchanged.
The previous440-pass full-suite checkpoint remains the latest broad validation.
lecteur.go remains partial; arbitrary replacement APIs, integrated caller mapping,
document propagation and captured objective modes remain open. No architecture
refactor was started.

### Damage-tag catalog parser parity (2026-09-24)

Added `parse_kill_damage_catalog` and structured table/line/input/failure errors.
The embedded catalog now uses this same parser, replacing a pinned-input-only
implementation. IDs are sorted/deduplicated; labels preserve unknown classes,
statuses and all six raw fields, use last-row precedence, and need not belong to
the ID set. Header dates follow native first-token and later-nonempty rules.
Rust error display wording is explicitly distinct from Go; typed error data is
retained. Inputs are UTF-8 text tables. Complete declaration mapping is in
[DAMAGE_CATALOG_PORT.md](DAMAGE_CATALOG_PORT.md).

The registered independent Go harness invokes the pinned private parsers on512
edge cases plus the exact embedded tables. All513 comparisons pass:204 accepted,
128 column-count failures,140 syntax failures and41 overflow failures. Checks
include ordered fields, dates, publication flags and exact error table, physical
line, input and column count. All468 embedded IDs and labels match native data.

Validation: full Theater suite441 passed/37 ignored (54.21s); Clippy with tests
and warnings denied, WASM library check, fmt/diff and generator syntax passed.
Logs: `/private/tmp/halo-damage-catalog-{test,suite,clippy,wasm}.log`.
All processes terminal. No captured scans were rerun because the independent
embedded-table comparison confirms the catalog consumed by those scans is
unchanged. damagetag.go is now ported-v41; broad parser parity remains incomplete.

Next confirmed integration gap: `Film::from_chunks` still assigns only
`scan_film_statborg(chunks)` to Film.statborg (decode.rs), while loaded-source
statborg APIs expose diagnostics separately. Audit the legacy chunk metadata
contract and thread native source diagnostics into Film/portable output without
changing the deferred architecture. Reader replacement/caller contracts and
positive complete captured CTF/VIP/bomb validation remain open as well.

### Film statborg diagnostics and portable export (2026-09-24)

Added `scan_film_statborg_with_diagnostics`, sharing the existing StatborgScan
implementation with loaded-source and non-observing APIs. Legacy chunk metadata
is attached per buffer, so this API does not claim to represent extra manifest
rows or unloaded buffers. Every nonzero metadata type is scanned; corrected the
older comment implying unknown nonzero types were excluded.

`Film::try_from_chunks` scans once and retains ordered source warnings in
`statborg_diagnostics`, using DecodeOptions.match_id or the native empty string.
The serde field omits empty vectors and defaults missing fields for older exports.
Source truncation remains in Film.statborg.truncated. Downstream pass warnings are
not conflated with this source-only diagnostic collection.

Validation reuses the pinned128-source oracle for representable legacy paths and
the33,078-record whole-frame cap oracle. The cap test now constructs a real Film,
compares all records and the exact warning, round-trips JSON, reads an older export
without the new field, and checks a non-truncated Film emits no warning. The first
synthetic bootstrap omitted required pawn component names and correctly failed
registry validation; the fixture was corrected without weakening that guard.
Two focused tests passed (2.29s). Full Theater suite441 passed/37 ignored (48.46s).
Clippy with tests and warnings denied, WASM, fmt and diff checks passed.
Logs: `/private/tmp/halo-film-statborg-{test,suite,clippy,wasm}.log`.
All processes terminal. No captured rerun was needed for the shared unchanged
scanner; the captured source oracle remains separate prior evidence.

Next diagnostic propagation boundary: replay_score_players.rs uses unobserved
statborg_series_total and replay_score_series_of_rounds, and build_replay_score
also invokes unobserved identity/team/hold passes. Audit native warning order
through those calls and expose retained diagnostics without altering the native
replay document schema accidentally. Full parser parity, reader contracts,
remaining inventory and captured objective gates remain open. Work remains
uncommitted on simbleau/theater-experiments; architecture is still deferred.

### Player-score diagnostic propagation (2026-09-24)

Added diagnostic-returning flat, per-round and automatic player-score APIs.
Existing APIs share the same implementations with observation disabled. Flat
counter loading retains personal/kills/deaths/assists chronology messages before
clock clipping. Per-round publication retains sorted-XUID, then counter order.
The automatic wrapper now computes the flat identity pass even in multi-round
matches, matching its native caller; its warnings precede player publication.
No replay document schema or queued architecture changes were made.

Expanded the registered player-score native harness from1,024 to1,536 cases,
using the existing native slog capture handler. Every original input/output row
was checked unchanged before fixture refresh. The512 additional cases include
reversed round timelines and use one slot to make native callback order fully
deterministic without normalization. All output values and all1,536 direct plus
1,988 automatic chronology warnings match with exact attributes/order. Direct
and automatic legacy outputs also match their diagnostic-returning counterparts.
The focused test passed (16.84s); full Theater suite441 passed/37 ignored (51.84s).
Clippy with tests and warnings denied, WASM, fmt/diff and generator syntax passed.
Logs: `/private/tmp/halo-player-score-diagnostics-{test,suite,clippy,wasm}.log`.
All processes terminal.

Next confirmed gap: native buildScoreTimeline loads team score, team frags and
player frags before resolving flat identity once. Team/player/optional-hold
publication reuse those values. Rust build_replay_score invokes convenience
helpers in a different order and repeats identity/series calculations. To expose
correct top-level diagnostics, factor shared prepared values and reuse flat
identity (including across the multi-round player path); do not concatenate the
current helper warning vectors and call it parity. Team/hold diagnostics and
Film/document report propagation remain open. Reader contracts, broader inventory
and positive captured objective-mode gates are also still incomplete. Changes
remain uncommitted; the architecture phase stays deferred.

### Complete score diagnostic order and shared preparation (2026-09-24)

Refactored the existing score implementation to prepare team score, team kills,
player kills and flat identity in native order. Publication reuses those values;
the player path accepts prepared flat identity and hill-hold reads occur after
player publication, under the existing mode/team gates. Removed repeated flat
identity scans from the top-level composition. Standalone APIs share these same
helpers. Added `build_replay_score_with_diagnostics` and the corresponding Film
wrapper. Both retain pass warnings; Film's preceding source diagnostics remain
separate and no source scan or replay document schema change is introduced.

Expanded the registered complete-score oracle to1,536 cases. All original1,024
rows were compared unchanged before fixture refresh. The512 new cases use one
team and one player slot, with reversed/opposing round timelines, valid/invalid
clocks, absent input and mode/team gates. All values and2,003 ordered chronology
warnings across482 positive cases match native exactly. There are119 positive
hold-publication cases across the fixture. Budget-specific messages remain
covered by the independent identity/budget fixtures; the new fixture's warnings
are chronology messages. Legacy and diagnostic-returning score outputs agree.

Focused complete-score test passed (11.03s). Full Theater suite441 passed/37
ignored (52.66s); Clippy with tests and warnings denied, WASM, fmt/diff and generator
syntax passed. Logs: `/private/tmp/halo-score-diagnostics-{check,test,suite,clippy,wasm}.log`.
All handles terminal. No captured scans rerun; native output regression checks
cover this pure score composition and the unchanged source inputs.

Next reporting gap: StatborgRoundIdentity::completed_by_lines still invokes
unobserved statborg_slot_identity behind its empty-lines/single-round gates;
build_film_replay_players invokes it before identity publication. Retain those
warnings and trace subsequent identity registry calls before exposing a complete
document diagnostic report. build_film_replay_document also still invokes the
unobserved score wrapper. Full parser parity, reader contracts, broader inventory
and positive captured objective-mode gates remain incomplete. All changes stay
uncommitted on the experiments branch; architecture remains deferred.

### Identity completion warning retention and provenance repair (2026-09-24)

Added `StatborgRoundIdentity::completed_by_lines_with_diagnostics`; the existing
API shares its implementation with observation disabled. Empty lines and anything
other than one identity round return before scanning counters. Nonempty triplet
results now rebuild origins only for returned links, matching native behavior;
unrelated input provenance previously survived incorrectly. Existing links are
not replaced and a player already claimed by another slot is not reassigned.
The Film player builder retains warnings in `FilmReplayPlayers.statborg_diagnostics`
with a defaulted, omitted-when-empty serde field.

The registered512-case native oracle compares full results, unchanged input,
legacy API equivalence and1,647 ordered messages (657 chronology,880 budget detail,
110 summaries).329 provenance rebuilds are checked, including stale origins,
incumbents, duplicate XUIDs, missing origins and guarded early returns. Only native
private nil origin maps are mapped to empty Rust maps, consistent with Origin's
lookup behavior. Native callback order and all message attributes are exact.
Focused test passed (1.04s). Full Theater suite442 passed/37 ignored (54.04s);
Clippy with tests and warnings denied, WASM, fmt/diff and generator syntax passed.
Logs: `/private/tmp/halo-identity-completion-{test,suite,clippy,wasm}.log`.
All processes terminal. No captured corpus rerun; the new provenance cases use
explicit caller-supplied identities absent from the captured scanner fixtures.

Further caller audit found no additional statborg counter pass in identity
registry construction (it consumes round boundaries). It did find unobserved
named-event passes in replay_objective_actions.rs, replay_flags_film.rs and
replay_vip.rs. Those mode/early-return boundaries are the next diagnostic gate;
then compose source/identity/objective/score diagnostics at document assembly
without implying unrelated decoder reports are complete. Full v41 parity,
reader contracts, broader inventory and positive captured objective-mode coverage
remain open. Changes stay uncommitted; the queued architecture is not started.

### Objective, flag and VIP named-event warning propagation (2026-09-24)

Added diagnostic-returning general objective identification/publication, flag
publication (default/explicit catalog) and VIP publication entry points, sharing
the existing algorithms. A common named-event sink preserves chronology and
budget messages without changing replay output types. General objective warnings
are retained before roster/death-evidence/clock filtering. They do not claim the
outer replaybuild application's match-row/error logs, whose context these APIs
never receive.

The flag caller audit found two native named-event passes inside attachFlagCarries:
flagFilmSignalsOf extracts once and scan.Events extracts again, before the Scanned
and recognition checks. Rust now reproduces this order; it drops the first event
vector after forming signals to avoid retaining two large event vectors. VIP
returns before extraction when unrecognized, and otherwise extracts before
resolving death identity. Earlier carrier-marker source scanning remains a distinct
boundary; no claim of its logs is made by these attachment APIs.

The registered128-case oracle compares native flag-prelude signals/events and
1,548 warnings, plus actual native attachVipCrown output/coverage,320 warnings
and1,863 VIP periods. Exact warning order is checked without normalization. Cases
cover reversed rounds, oversized increments, recognition suppression and invalid
clocks. Existing generic-objective and downstream flag fixtures remain separate
algorithm evidence. Focused comparison passed (1.12s). Full Theater suite443
passed/37 ignored (55.28s). Clippy initially flagged helper placement after a test
module; moving that unchanged helper resolved it. Final Clippy with tests and
warnings denied, WASM, fmt/diff and generator syntax checks passed.
Logs: `/private/tmp/halo-objective-diagnostics-{check,test,suite,clippy,wasm}.log`.
All processes terminal. No captured corpus scans were rerun.

Next: compose the retained statborg source, identity, general objective, score,
flag and VIP diagnostics at document assembly with explicit scope and stage order.
Do not call that a complete report of unrelated grammar/scanner or outer application
logs. Full v41 parser parity, reader contracts, broader source inventory and
positive captured CTF/VIP/bomb coverage remain incomplete. Changes stay uncommitted
on the experiments branch; the requested architecture phase remains deferred.

### Captured position input checkpoint

The integrated native-input comparison now covers 30/44 fields, including all
ordered accepted Positions and every exported companion/vitality/mask field.
The six captured films and complete replay documents pass (97.78s); Clippy,
formatting, diff checks, and 485 pinned reference hashes pass. No oracle changed.
See `film-input-retention-coverage.json` for the fourteen remaining inputs and
`../fixtures/full-document-inputs-v41.md` for the exact projection contract.
PlayerIndices still requires separate retention work: replay identity evidence
currently rebuilds its scan from source. All other previously documented parity
gates remain open; the next-phase architecture remains deferred.

### Native identity input retention checkpoint

Film now retains the death feed, death roster, raw replication player-index table,
and named read failures. Collision rejection applies to a copy, preserving raw
conflicts. Replay reuses matching retained inputs, rescans for changed external
rosters, and preserves the old-export source fallback. This supersedes the prior
note that player-index evidence is always rebuilt during replay assembly.

Integrated comparison now covers 32/44 native inputs, adding Deaths and
PlayerIndices. All six input/document comparisons passed (95.25s); 561 Theater
tests passed, 46 ignored (95.27s). The 1,024-case native evidence test exercises
retained inputs without source, serialization, and changed-roster fallback,
including 92 collision cases, 282 death-read failures and 26 index-read failures.
Clippy and all-features WASM pass. See film-input-retention-coverage.json and
../fixtures/film-identity-retention-v41.md. Twelve integrated inputs and the
previously listed broader parity gates remain; architecture changes stay deferred.

Next audit: Translocations. Replay currently consumes Film.translocations while
Film.native_translocations retains the native padded scan. Check that distinction
against the native input and publication contracts before claiming parity there.

### Native translocation publication checkpoint

Resolved the previous adapter audit: replay now publishes retained native
translocations, including native padded-header admission and native source/order
rules, using a shared borrowed publication loop. Film retains padding provenance;
this compatibility output is not proof that padded actor bits were recorded.
An explicit native scan marker distinguishes empty scans from legacy-only exports.

All six complete-document checks pass (98.24s), bringing integrated field checks
to 33/44. **Translocations are empty in all six captured films.** Positive native
synthetic decode/source/publication tests pass, but captured positive validation
is still missing. This limitation is explicit in the coverage manifest; it is not
a declaration of complete parity. Full Theater: 561 passed, 46 ignored (90.07s).
Clippy, all-features WASM, formatting, diff checks and 485 reference hashes pass.
See ../fixtures/film-translocation-publication-v41.md. Eleven integrated inputs,
all broader parity gates, and the deferred architecture phase remain outstanding.

### Captured world-input comparison checkpoint

Integrated comparison now covers 37/44 inputs, adding Projectiles, Placements,
PlacementStats, and Pads. Positive captured coverage includes 509 projectile
tracks, 548 placements, 2,580 pad creations, and 2,794 pad motion tracks. All
ordered fields, points, scan counters, calibration, and census entries match.
All six complete input/document comparisons passed (100.26s); Clippy, formatting,
diff checks, and 485 pinned source hashes passed. This checkpoint changes test
projections and documentation only, with no production or native oracle changes.

Seven remaining integrated inputs: Vehicles, FlagMarks, FlagGauge,
FlagGaugeScanned, ZoneReads, ZoneScanned, and BombReads. The manifest records the
exact scope. Complete parser parity is still unproven; all previously documented
broader gates and the deferred architectural phase remain outstanding.

### Vehicle input comparison checkpoint

The six-film input/document comparison now covers 38/44 inputs, adding the full
Vehicles structure (101.37s). Positive appearance controls contribute 12 vehicle
creations, 716 positions and two occupant-aim reads. All exported vehicle fields,
including complete death-scan settings/counters, are projected. Clippy passed.

A separate pinned-native raid capture passed (274.512s), retaining 150 creations,
89,945 positions, 71,488 aims, 135 events, 12 deaths and 291 occupancy reads. The
new raid-vehicle-inputs-v41 fixture is linked from the existing full-raid document;
all previous document expectations remain unchanged. The Rust raid input and full
document comparison is currently running, not yet verified. Its log is
/private/tmp/halo-raid-vehicle-inputs-test.log. See the fixture provenance document
and film-input-retention-coverage.json for precise status and hashes.

Remaining integrated inputs: FlagMarks, FlagGauge, FlagGaugeScanned, ZoneReads,
ZoneScanned, BombReads. Broader parity gates and the architecture deferral remain.

### Raid vehicle validation completed

The previously running full-raid vehicle-input and complete-document comparison
passed in 278.98s. This verifies all captured vehicle input fields, including the
12 deaths and 291 occupancy reads, against the pinned native fixture. The prior
pending status is superseded. Broader parity gates remain open.

### All 44 FilmInputs fields compared; full parity remains incomplete

All six captured input/document comparisons passed with all 44 exported fields
(101.76s). Objective fields under default caller options are empty/disabled.
Independent native controls enable the resolved consumers on those same bytes,
checking 16,569 managed-property reads per consumer and 13,805 radial reads, plus
disabled outputs. This does not assign CTF/Assault modes or semantic actions to
those recordings. Captured positive carrier marks and translocations remain gaps.
The positive full-raid vehicle-input/document comparison passed (278.98s).

Fixed a consumer failure-contract omission: zone and flag-gauge replay now reject
reads from a managed scan marked failed while Film retains its partial data/error.
Full Theater passed 561 tests, 46 ignored (107.28s); Clippy, all-features WASM,
formatting, diff checks and 485 source hashes pass. See the machine-readable
coverage manifest and ../fixtures/objective-inputs-v41.md for exact limits.

The integrated field checklist is closed, not the parser objective. Remaining
work includes signed/overflowed cursor continuation, lazy width conversion,
callback mutation/lifecycle and source-failure audits, broader whole-Film
acceptance, and missing controlled semantic evidence. Architecture remains deferred.

### Direct-reader lazy width conversion verified on wasm32

NativeFilmReader::read_component no longer rejects unused oversized profile
widths up front. Raw widths are converted at their actual field reads; gates and
unrelated fields can proceed, while the profile preserves its original values.
A reached unrepresentable width reports an error at the reached cursor position.

648 independently generated native cases pass in host/simulated 32-bit tests and
in an actual wasm32 Node runtime. The runtime also confirms that the old eager
adapter rejects the same profiles. Six focused tests pass; full Theater 563
passed, 46 ignored (100.72s); Clippy and the six complete captured input/document
checks pass (98.69s). Formatting, diff checks and 485 source hashes pass. See
../fixtures/lazy-widths-v41.md and verify_lazy_widths_wasm.py for reproduction.

This is the direct component path only. NativeScanProfile::component_encoding
and NativeFrameConfig::frame_encoding still validate eagerly. Frame-level lazy
conversion and signed/overflowed cursor continuation remain open, alongside the
previous callback/source/whole-Film gates. Architecture remains deferred.

### Live frame position widths and missed world-object read corrected

Live NativeFilmReader frame records now check raw position dimensions when their
fields are reached. The audit also found and fixed object-position-component
still consuming policy placeholders in direct reads. The original 128 context
expectations are unchanged; 128 new unused-width frame cases and 144 world-object
direct/frame cases compare against native output, including axis values and
field boundaries. Both 512-case accumulator fixtures continue to match.

Actual wasm32 execution passes all 1,424 native cases plus a reached-width error
control at bit 35. Full Theater: **565 passed, zero failed, 46 ignored** (82.91s).
Clippy, formatting, diff checks, Python syntax and all 485 pinned source hashes
pass. See ../fixtures/frame-lazy-widths-v41.md and frame-lazy-width-validation.json.
Logs: /private/tmp/halo-frame-lazy-{tests,wasm,suite,clippy}.log.

Other frame adapters (resync, inference, repair and production views) and frame
scalar settings still validate eagerly. Signed/overflowed cursor continuation,
callback/source/whole-Film audits and missing positive captured/independent
semantic evidence remain open. The six captured complete-document checks from
the previous checkpoint were not rerun for this live-reader change. Parity is
not complete; the architecture remains deferred.

### Contextual frame position widths and explicit refusals

All NativeFrameConfig contextual entry points now use raw movement widths at
field reads: bound records, chain trials, repair, inference, validated/raw resync
and production views. NativeWidthRefusal diagnostics retain field, bit position,
raw width and address-domain maximum through failed records/speculative results,
merge and serialization. This makes decoder limitations distinguishable from
missing source data. Signed scalar settings and cursor overflow remain open.

The five expanded native context fixtures contain 576 new unused-width cases.
Every original native expectation is unchanged, and all new cases produce the
same native records/callbacks/world state as their originals. Focused host checks:
17 passed, one corpus test ignored. Actual wasm32: 2,832 native cases passed plus
reached-width/refusal-retention controls. Full Theater: **570 passed, zero failed,
46 ignored** (91.20s). Clippy, fmt/diff, Python syntax and 485 source hashes pass.
The captured six-film comparison passed (103.08s): all 44 native inputs,
objective-consumer controls and complete replay documents match.

See ../fixtures/context-lazy-widths-v41.md, context-lazy-width-validation.json
and verify_frame_lazy_widths_wasm.py. Logs are
/private/tmp/halo-context-lazy-{native,tests,wasm,suite,clippy,captured}.log.
These checks do not close the remaining whole-Film/source/cursor gates or begin
the deferred architecture.

### Native New-record nonpositive tail policy

Native TraverseEntity skips its terminal tail only after successful components
and only when BitsDeQueueRecordNew > 0. NativeFrameConfig now applies that rule
instead of rejecting negative settings during unsigned conversion; the original
signed value remains in the native profile. A 384-case native oracle covers
negative extremes, zero, positive tails, clean/failed New, Delta/Delete/End, and
all starting bit offsets. Endpoints, fields, callbacks and world occupancy match.

All four focused New-record tests pass (0.39s), including existing generic,
inference and recovery fixtures. The expanded actual wasm32 runner passes 3,216
native cases plus prior reached-width controls. Clippy, fmt/diff, Python syntax
and all 485 reference hashes pass. The full 570-test and six-film checks in the
previous checkpoint predate this small policy correction; they were not rerun.

See ../fixtures/new-tail-policy-v41.md and new-tail-policy-validation.json.
Logs: /private/tmp/halo-new-tail-{tests,wasm,clippy}.log. Unused oversized positive
tails, other scalar settings and signed/overflowed continuation remain open.
This is parser parity work; the deferred architecture has not begun.

### Large and unused New-record tails

Contextual New-record tails now convert at successful New completion, using the
raw grammar setting. Unused positive widths no longer reject other records.
Consumed large tails retain every actual source bit and describe their synthetic
padding once, tagged NativeWidthPurpose::NewRecordTail. Address and signed-target
limits retain explicit diagnostics instead of expanding huge padded ranges.

224 new native cases pass on the host; all 384 earlier tail expectations are
unchanged. Five focused New-record tests pass. Actual wasm32 passes 3,392 native
comparisons and 48 explicit address-domain refusals, plus a resulting-cursor
control that preserves native target 2^32 at bit 21. Full Theater: **572 passed,
zero failed, 46 ignored** (92.41s). Clippy, fmt/diff, Python syntax and all 485
reference hashes pass. The previous captured six-film input/document comparison
predates these terminal-tail changes and was not rerun.

See ../fixtures/new-tail-large-v41.md and new-tail-large-validation.json.
Logs: /private/tmp/halo-new-tail-large-{tests,wasm,suite,clippy}.log.
Default-state/MPP/ID scalar handling, negative/wrapped cursor continuation and
broader whole-Film gates remain open. Architecture remains deferred.

### Public facade inventory and measured equipment fallback counter

The decoder facade was inspected with Go ASTs: 166 exports, including 71 single
returned calls forwarding arguments unchanged and in order. Fifty types and all
45 constant/variable exports are qualified aliases. The historical 163-symbol
comment is stale. This establishes delegation, not Rust parity of the targets;
the complete inventory and remaining audit boundary are in PUBLIC_DECODER_AUDIT.md.

The equipment-origin oracle now records the actual native fallback.Compteur
instead of using coverage.ByCause as its only counter proxy. All 1,024 original
case fields remain unchanged. Rust matches 739 positive cases, 285 zero cases
and 2,581 named fallback hits. Focused test and Clippy pass; fmt/diff and all 485
reference hashes pass. The six captured document fixtures have no positive hits
for this particular fallback, so positive complete-document forwarding remains
explicitly open. No production code changed in this checkpoint.

See fallback-callsite-audit.json and logs
/private/tmp/halo-equipment-fallback-{native,tests,clippy}.log.
The full parity goal and deferred architecture boundary remain unchanged.

### Fallible source-provider contract

The public facade audit found that Rust lacked native source.Load's fallible
provider input. FilmChunkProvider and FilmSource::load_from now query the count
once, read/inflate/index each chunk in order, stop on the first read error and
retain its position and original typed cause. Existing slice loading delegates
through this path. Loaded bytes and metadata are owned.

The pinned native source-provider fixture has 256 cases comparing read order,
first-error behavior, errors.Is/original-cause identity, loaded bytes, metadata
and ordered packet payloads/timestamps. It includes negative and zero counts,
empty/malformed/zlib/raw chunks and differently sized metadata. All three focused
source tests pass (0.29s). Full Theater: **573 passed, zero failed, 46 ignored**
(87.23s). Clippy passes (41.90s), all-features wasm32 compilation passes (17.67s),
and formatting, diff, generator syntax and all 485 pinned source hashes pass.
This WASM check compiles the API; it is not a provider runtime oracle on wasm32.

PUBLIC_DECODER_AUDIT.md maps source/profile exports to implementations/evidence
and records the remaining boundaries: i32::MAX count limit, unsigned source
positions, owned errors/data, eager directory file reads, and no compressed-byte
archive. Captured-film tests were not rerun for this change. Broader scalar,
cursor, callback, retention and corpus gates remain open; full parity is not
complete. The three-layer architecture remains deferred in NEXT_PHASE.md.

Evidence: halo_rust_source_provider_test.go.txt;
../fixtures/source-provider-v41.json.zlib; logs
/private/tmp/halo-provider-{tests,suite,clippy,wasm}.log.

### Lazy directory source and first-error propagation

FilmDirectorySource now discovers and numerically sorts paths without reading
contents, implements FilmChunkProvider with owned reads, and retains file paths
and original I/O errors. FilmSource::load_directory aligns metadata and delegates
to load_from, so each file is inflated/indexed before the next file is read.
This supersedes the eager-directory limitation in the previous checkpoint.

The independent pinned native lifecycle oracle covers 64 cases: 17 successful
loads, 24 missing-file errors and 23 directory-read errors. It compares 288 direct
position reads, repeated reads after overwrites, 190 loader reads, stable source
counts after additions, bounds errors, and first-error stopping. Rust also checks
owned previous-read bytes and the error chain down to std::io::Error.

Four focused source tests pass (0.28s). Full Theater: **574 passed, zero failed,
46 ignored** (77.64s). Clippy passes (40.01s), all-features wasm32 compilation passes
(6.69s), formatting/diff/generator syntax and all 485 source hashes pass. Captured
film tests were not rerun. Directory filesystem APIs remain host-only.

Literal directory paths are supported; Go filepath.Glob expressions, enumeration
failure semantics and platform-specific error wording remain unreconciled. These
checks do not close record/field retention or whole-corpus parity. NEXT_PHASE.md
remains deferred; the three-layer refactor has not started.

Evidence: ../fixtures/source-directory-lifecycle-v41.md and its compressed JSON;
halo_rust_source_directory_lifecycle_test.go.txt; PUBLIC_DECODER_AUDIT.md;
logs /private/tmp/halo-directory-lifecycle-{tests,suite,clippy,wasm}.log.

### Automatic Film weapon-hit retention

A concrete propagation gap is closed: Film constructors previously initialized
weapon_hits to None and required a separate retain_weapon_hits call. Base and
explicit-encoding construction now preserve direct shot/damage reads and native
pairing output automatically. Map-aware construction refreshes with resolved
sampling precision to retain tracks and distance statistics. weapon_hits_error
retains independent failure; a failed refresh keeps earlier direct results,
and successful refresh clears its error. Other Film channels remain available.

Four focused tests pass (2.66s), covering positive hits without distance, missing
native chunk-one prefix with other indexed packets retained, explicit failed
refresh/retry, JSON and old exports. The map-constructor regression now verifies
automatic retention. All six captured base-constructor cases match the pinned
native oracle: 4,395 shots, 105 damage observations, landing bases and no-distance
statistics, with packet-source checks and whole-Film JSON roundtrips (20.41s).
The release build took 3m29s. No fixture expectations were generated by Rust.

Full Theater: **574 passed, zero failed, 47 ignored** (95.08s). The new captured
test was explicitly run as above; other captured suites were not rerun. Clippy
passes (12.24s), all-features wasm32 compilation passes (7.66s), formatting/diff and
all 485 pinned hashes pass. Map-aware construction currently rescans hit inputs
when it adds distance evidence, increasing work and export size. This is current
parser retention work, not the deferred architecture.

See FILM_RETENTION_AUDIT.md and ../fixtures/weapon-hit-film-retention-v41.md.
Logs: /private/tmp/halo-hit-retention-{tests,suite,captured,clippy,wasm}.log.
Whole-source fidelity, remaining native contracts and broader corpus acceptance
still prevent a full-parity claim. NEXT_PHASE.md remains deferred.

### Captured map-constructor weapon-hit retention verified

The new six-film map-constructor comparison passes against the pinned native
weapon-hit oracle (83.41s; release build 2m46s). It first confirms identical
sampling bounds, then compares all 4,395 shots, 105 damage observations, landing
bases, distance bases, distance errors, aggregate distance histograms and 590,758
track samples with timestamps/positions. Packet provenance and the retained
FilmWeaponHits JSON roundtrip also pass. The base-constructor comparison was
rerun after sharing test code and passes all six cases (21.07s).

These tests use the native scanner's bootstrap/gameplay prefix; they are not a
new complete replay-document comparison. The map test roundtrips FilmWeaponHits;
whole-Film JSON remains covered by the base corpus and synthetic map regression.
Clippy passes (9.34s), formatting/diff and all 485 reference hashes pass. No
production code changed; the full suite and WASM check from the preceding
production checkpoint were not repeated. One additional ignored test was added.

PUBLIC_DECODER_AUDIT.md now maps the weapon-hit and keyframe-position exports to
Rust implementations and evidence, including explicit chunk-range semantics and
the native position probe's heuristic nature. The phase-boundary note near the
top of CURRENT_VALIDATION_GAPS.md distinguishes all native output parity from the
separately deferred source-hierarchy, resolved-event and playback architecture.
This clarification waives no native output or pending parity gate.

See ../fixtures/weapon-hit-film-retention-v41.md and logs
/private/tmp/halo-hit-map-retention-{captured,base,clippy}.log.

### Lazy New default-state widths and nonnegative rewinds

NativeFrameConfig no longer eagerly converts NewDefaultStateBits when building a
contextual frame encoding. The raw signed option is retained in NewRecordEncoding
and consumed only by the selected New fallback branch. Biped/archetype defaults,
invalid New archetypes and Delta/Delete/End leave it unused. Negative skips with
nonnegative targets now continue at that target and retain a NewRecordDefault
adjustment; large padded skips keep every source bit and compact only padding.
The explicit legacy adapter remains checked. Terminal New tails retain their
separate strictly-positive policy, covered by the existing regressions.

624 independent native cases pass: 480 unused-width cases, 48 rewinds, 48 large
consumed widths and 48 ordinary skips. Ordered records, masks, cursor endpoints,
complete world state, retained source fields and result JSON are checked. Seven
focused New tests pass (2.27s). The actual wasm32 runner passes 3,984 native
comparisons and 80 explicit address-domain refusals (4,064 cases), plus its prior
standalone controls. The new default fixture contributes 592 native comparisons
and 32 reached-width refusals there. Raw widths remain available after refusal.

Full Theater: **575 passed, zero failed, 48 ignored** (71.85s). Clippy passes
(12.15s) after an equivalent test-only range predicate cleanup. Formatting/diff,
Python syntax and all 485 pinned hashes pass. Captured films were not rerun for
this context-option change; the previous map weapon-hit corpus predates it.
The WASM runner reports its existing harmless unused copied-test warning.

Remaining gaps include MPP/ID scalar admission and negative/overflowed cursor
continuation. This fixture compares the raw option at configuration boundaries
and cursor effects, not the native Trace.DefaultBits field on each returned New
record; per-record trace metadata still needs reconciliation. Broader native
output parity remains incomplete. The architectural phase is still deferred.

See ../fixtures/new-default-policy-v41.md; new-default-policy-validation.json;
logs /private/tmp/halo-new-default-{tests,wasm,suite,clippy}.log.

### Native per-New default trace metadata retained

EntityRecord.default_state_bits now retains native Trace.DefaultBits at New
trace entry, including invalid archetypes and dedicated/biped readers that do
not consume the configured skip. It is explicitly decoder context, not recorded
payload length. Non-New/unreached traces and older exports retain None. The
checked legacy width is represented only when it fits the native signed domain.
This closes the per-record metadata gap identified in the preceding checkpoint.

The existing native fixture supplies 384 returned New records, including 120
negative values, 40 zeros and 24 i64::MAX values. All are compared directly; full
view JSON roundtrips and old-export removal preserve the availability distinction.
No native expectations were regenerated. Seven focused New tests pass (2.18s).
The actual wasm32 runner also passes 3,984 native comparisons plus 80 domain
refusals, including 352 returned New metadata values and the unchanged reached-
width refusal cases. Existing generic/inference/recovery and tail checks pass.

Full Theater: **575 passed, zero failed, 48 ignored** (70.60s). Clippy passes
(26.60s, including build-lock wait); formatting/diff and all 485 reference hashes
pass. The actual WASM build/run completed in its existing runner (12.45s build).
Captured films and separate all-features WASM compilation were not rerun for
this additive record field. MPP/ID admission, negative/overflowed cursor targets
and broader native-output reconciliation remain open; architecture stays deferred.

See ../fixtures/new-default-policy-v41.md and new-default-policy-validation.json.
Logs: /private/tmp/halo-new-default-metadata-{tests,wasm,suite,clippy}.log.

### Lazy native MPP widths verified

Native contextual readers now defer original signed MPP lead/index widths to
the reached read instead of applying the legacy 1..=32 gate at setup. Zero and
wide reads match native numeric truncation and preserve discarded source bits.
Unused invalid settings do not reject frames. Explicit platform/signed-cursor
refusals retain original widths, field locations and preceding observations.
The bounded public default-state helper retains its checked contract.

All 1,632 native cases pass (288 consumed and 1,344 unused widths), comparing
1,152 ordered callbacks, records/masks/endpoints, world state, source fields and
JSON roundtrips. Additional negative consumed-width controls pass without trying
those impractical native reads. Actual wasm32 execution passes 5,616 native
comparisons plus 80 existing address refusals and additional MPP refusal controls.

Full Theater: **576 passed, zero failed, 48 ignored** (70.59s). Clippy passes
(13.91s); formatting/diff, Python syntax and all 485 pinned reference hashes pass.
WASM build took 11.09s; its existing unused copied-test warning is unchanged.
Captured films and separate all-features WASM compilation were not rerun.
ID admission, negative/overflowed cursor continuation and broader native output
reconciliation remain open. NEXT_PHASE.md remains deferred.

See ../fixtures/mpp-width-policy-v41.md and mpp-width-policy-validation.json.
Logs: /private/tmp/halo-mpp-width-{tests,wasm,suite,clippy}.log.

### Native contextual record-ID policy verified

Native contextual setup now carries signed IDLowBits in
FrameEncoding.native_id_low_bits and defers its admission until a non-End
header. Nonpositive widths consume no low field. Positive reads truncate to
uint32, add IDBase with wrapping, then mask to 30 slot bits before combining the
separate generation. Ordered frame/inference/production/resync/chain header
pre-reads share this policy when that raw native setting is present.

8,640 native-generated header/frame cases pass: five prefix forms, eight
alignments, three values, five bases and 16 signed/zero/wide widths, with huge
positive settings consumed only by End (which ignores them). The fixture also
compares complete world slots, returned records/masks/endpoints and full-view
JSON roundtrips. Four additional Rust-domain controls cover ID/generation
refusals with/without the optional prefix. Generic reader cursors and structured
inference/resync/production diagnostics retain the reached failure location.
ProductionFrame.header_diagnostics preserves errors before a complete record;
older exports deserialize with empty diagnostics.

Focused tests: **2 passed** (0.71s). Full Theater: **578 passed, zero failed,
48 ignored** (76.44s). Actual wasm32: **14,256 native comparisons + 80 existing
address refusals**, plus MPP/ID standalone controls (21.95s build). Clippy passes
(20.27s). Formatting/diff, Python syntax and all 485 pinned hashes pass. Captured
films and separate all-features WASM compilation were not rerun.

This closes raw native contextual ID handling for the exercised frame paths;
it does not establish parity for every adapter. Native march still uses the
checked frame adapter for locator setup and calibration export. Native APIs
supplied a legacy FrameEncoding without native_id_low_bits still use its checked
ID policy, including checked base arithmetic. These paths need reconciliation;
do not change the bounded public decode_record_header contract inadvertently.
Signed/overflowed cursor continuation and broader native-output reconciliation
also remain open. NEXT_PHASE.md remains deferred.

See ../fixtures/record-id-policy-v41.md and record-id-policy-validation.json.
Logs: /private/tmp/halo-id-{tests,wasm,suite,clippy}.log.

### Legacy native IDs and march adapter reconciliation

Native generic/inference traversal now applies wrapping ID arithmetic even when
a legacy FrameEncoding has no raw native_id_low_bits field. 1,440 compatible
cases from the unchanged native ID fixture compare generic and inference headers,
frame endpoints and completion. The bounded public header helper still rejects
out-of-range sums, verified by explicit controls.

Native march locator setup now uses the contextual frame reader. A new 90-case
oracle calls native calibration, localization, record traversal and harvest for
End-only payloads with/without event markers. It varies unused ID/base, MPP,
New-default and world-index settings through negative/zero/wide values. All
facts, coverage maps, packet/calibration counters and retained configuration are
checked; these cases must emit no deaths or occupancy. This is unused-setting
coverage, not positive action or captured unusual-width evidence.

A completed native scan no longer fails because calibration's checked convenience
encoding cannot represent its settings. MarchCalibration.encoding is now optional;
encoding_error reports conversion failure, while native_config retains complete
calibrated settings. No substitute settings are exported. Existing encoding-object
JSON remains compatible and ordinary successful output shape is unchanged. Rust
callers must handle the optional field. Encoding-only calibration always supplies
Some of its actual encoding. Projection availability is checked across 30 host
and 26 wasm32 cases, with explicit absence/error in the remainder.

Focused ID tests: 2 passed (0.73s); march test: 90 cases passed (0.12s). Full
Theater: **579 passed, zero failed, 48 ignored** (85.73s). Actual wasm32 passes
14,346 native fixture comparisons plus 80 existing domain refusals; the 1,440
legacy-ID extensions and standalone MPP/ID controls run additionally. WASM build
18.85s; Clippy 17.09s. Formatting/diff, Python syntax and all 485 pinned hashes pass.

The four-film captured native-context march comparison is running in release
mode; do not treat it as passed until its terminal result is recorded. Other
captured suites and separate all-features WASM compilation were not rerun.
Negative/overflowed cursor continuation, broader native-output reconciliation
and remaining corpus acceptance still prevent full-parity completion. The
NEXT_PHASE.md architecture remains deferred.

See ../fixtures/march-lazy-policy-v41.md and march-lazy-policy-validation.json.
Logs: /private/tmp/halo-id-legacy-tests.log and
/private/tmp/halo-march-lazy-{tests,wasm,suite,clippy,captured}.log.

### Captured native march comparison completed

The four captured v41 films (Bazaar idle, Aquarius, Bandit Evo, ranked Oddball)
pass the native-context march comparison after the legacy-ID and march adapter
changes. The test compares complete deaths/occupancy, all four coverage maps,
packet/localization counts, native configuration/profile, calibration counters
and FilmMarchFacts JSON roundtrip against the pinned oracle. Execution: 19.83s;
release build: 2m56s. This supersedes the preceding running status. Other captured
pipelines were not rerun; it is not a complete replay-document rerun.

Current baseline remains 579 passed, zero failed, 48 ignored in the regular
Theater suite, with actual WASM and Clippy passing. No compiler/test processes
remain active at this checkpoint. See /private/tmp/halo-march-lazy-captured.log.

### Kill-source input error precedence corrected

The public-target audit found that full kill-source decoding attempted its feed
before checking source/replication presence. The v41 full decoder now follows
native order: no chunks, then no type-zero replication packets, then no usable
kill feed. DecodeError::KillSource retains NoChunk, NoReplicationPacket and
NoKillFeed with exact native sentinel text. The explicit unsupported-version
boundary remains separate. The initial guard adds a packet-presence scan to
successful calls without copying source bytes.

The actual native exported Decode now supplies decode_error in the existing
128-case loaded-source oracle. All earlier fields were verified unchanged.
10 no-chunk, 24 no-replication and 94 no-feed cases pass through the Rust full
entry point; both typed categories and text are checked. This verifies failure
precedence, not cancellation/logging or every successful decoding path.

Focused comparison passes (0.20s); full Theater: **580 passed, zero failed,
48 ignored** (98.82s). Clippy passes (50.08s). Actual WASM passes 14,474 native
fixture comparisons plus 80 existing domain refusals and prior supplementary
controls (47.27s build). Formatting/diff, Python syntax and all 485 source hashes
pass. The two-film complete kill-source result comparison is running in release
mode; its result must be recorded before treating it as passed. Other captured
pipelines were not rerun.

PUBLIC_DECODER_AUDIT.md also maps all seven weaponscan/weaponv3 function exports,
distinguishes marker-based and packet-based clocks, and identifies the kill
catalog/profile/default-option targets. The remaining public target/type and
native-contract audit is still open; NEXT_PHASE.md remains deferred.

See ../fixtures/kill-decode-input-errors-v41.md and
kill-decode-input-validation.json. Logs: /private/tmp/halo-kill-input-
{native,tests,wasm,suite,clippy,captured}.log.

### Captured full kill-source results verified

The two-film full-result comparison passes after input-error precedence was
restored: Bandit Evo (122 kills) and ranked Oddball (247 kills), including the
existing complete result/configuration and publication-gate assertions.
Execution: 10.71s; release build: 3m 06s. This supersedes the preceding running
status. Other captured pipelines were not rerun. The regular suite remains
580 passed, zero failed, 48 ignored; actual WASM and Clippy pass. No build/test
process remains active at this checkpoint. Broader parity remains incomplete.

Log: /private/tmp/halo-kill-input-captured.log.

## Kill timeline registry admission

Native `killsource.newTimeline` accepts successful truncated registry reads;
Rust previously rejected every `FilmRegistry.truncated` result and all short
headers. `KillTimeline::from_chunks` now uses `parse_registry_chunk`, preserves
ordered partial archetypes and the native trailing-byte count, and maps native
compressed-input failure to `KillSourceFilmError::Registry` with its typed cause
and native wrapper text. An empty chunk list reports `NoChunk`.

The 20-case actual-native constructor fixture covers short/header-only inputs,
partial and exact block boundaries, named precision levels, structural endings,
and two compressed-header refusals. It is registered in `generate_oracles.py`
and the public WASM harness. Existing timeline comparisons also pass. See
`../fixtures/kill-registry-v41.md`. This does not close option-domain,
cancellation/warning, complete public-target, or remaining corpus gates.

Registry checkpoint validation: 581 Theater tests passed, zero failed, 48 ignored
(75.47s); focused timeline tests 2 passed (0.21s); Clippy passed (10.43s).
Actual WASM passed the existing 14,474 native comparisons and 80 address-domain
refusals, plus the new 20 registry cases and existing supplementary controls
(build 13.14s). The captured full kill decoder matched all 369 kills across
Bandit Evo and ranked Oddball, including full existing output/configuration and
publication assertions (debug 175.38s). Formatting, diff, Python syntax and all
485 pinned hashes passed. Machine-readable evidence: `kill-registry-validation.json`.
Logs: `/private/tmp/halo-kill-registry-{native,tests,suite,wasm,clippy,captured}.log`.
This checkpoint leaves full parity active; it does not begin NEXT_PHASE.md.

## Signed kill-source options

`KillDecodeOptions` now retains signed i64 multiplicity/restart/view counts.
Native `Options.normalize` replaces all nonpositive counts with defaults; Rust
previously represented only the unsigned subset. The public `normalized` method
matches native while preserving caller-owned raw settings and booleans.
Execution checks conversion to usize and reports typed `DecodeError::KillOption`
with field/value on a narrower target, rather than wrapping or saturating.
See `../fixtures/kill-options-v41.md` for API migration and domain distinctions.

The 512-case 64-bit native oracle compares all option fields and default values,
including negatives, zero, booleans, i64 extremes and 32-bit boundaries. The WASM
harness runs the same public normalization and an actual decoder admission
control for an unrepresentable positive view count. Captured nonpositive-option
results are checked against independently generated native default results.
This closes the previously missing signed normalization domain; it does not
claim to execute enormous loop counts or close source-warning/cancellation and
remaining public-target/corpus gates. NEXT_PHASE.md remains deferred.

Signed-options checkpoint validation: 583 Theater tests passed, zero failed,
49 ignored (72.72s); Clippy passed (26.17s). Actual WASM passed the existing
14,474 native comparisons/80 address-domain refusals plus the new 512 option
normalization cases and actual public execution-domain control (build 26.30s).
With nonpositive counts, full captured decoder results matched all 369 native
default-option kills across Bandit Evo and ranked Oddball, including existing
full result/configuration/publication assertions (debug 175.96s). Formatting,
diff, Python syntax and all 485 pinned hashes passed. The additional ignored
test is the captured nonpositive-options control. No tests remain running.
See kill-options-validation.json and /private/tmp/halo-kill-options-
{native,tests,suite,wasm,clippy,captured}.log. Full parity remains active.

## Complete public type-alias inventory and representation map

inventory_public_types.go.txt resolves all 50 decfilm type aliases to their
native declarations and records fields (including private and embedded fields),
alias targets, JSON tags and receiver methods with source locations. The saved
public-type-declarations.json has 274 directly declared field entries after
expanding grouped names and 98 directly declared exported receiver methods.
These counts are inventory sizes, not completion percentages. Alias-resolution
chains remain explicit; transitive field types require their own audit.

public-type-rust-map.json maps all 50 names/targets to existing Rust storage and
queries, with test pointers and representation distinctions. All names/targets
match decfilm-export-inventory.json exactly. All Rust paths and named test
pointers exist; native regeneration reproduced the declarations byte-for-byte,
and all 485 pinned source hashes passed. The pre-existing tests cited for 34 of
the 35 newly mapped types pass in the preceding 583-test baseline. The health
method evidence is now supplied by the separate new oracle below.

Important distinctions are explicit: BipedCreation.HasIndex is implied by the
native acceptance gate; source packet metadata is outside the primitive record;
BipedPosition world availability is represented by world/quantized result types;
FrameConfig embeds the profile/observer through NativeReaderContext; Registry
trailing bytes and cached parse identity are separate from mutable archetypes;
map width projection and source ownership/index domains are documented. Ordinary
biped i2 roll is skipped by the native companion scanner too; its absence there
must not be mistaken for a Rust-only omission or silently interpreted.

This checkpoint maps representations and locates evidence. It does not claim
all 98 method input domains, mutable aliases, callbacks, transitive types or
remaining source contracts have been proven equivalent. No deferred structural
Film/ResolvedFilm/playback implementation was started.

### Native health-method coverage

The audit found that captured result-field checks did not independently exercise
all health methods. kill-health-methods-v41.json.zlib now contains 2,800 native
health states, crossing warning thresholds/catalog/roster flags with coverage
cases independently. All seven KillSourceHealth methods, PathStats.Ratio and
16,800 Result.LineByLinePublishable decisions are compared. It yields 1,820
ALERTE, 808 HORS DOMAINE MESURE and 172 NOMINAL states. Diagnostics and metric
pairs retain order; ratios compare by f64 bits. Counts other than DeathsReal
are nonnegative in this fixture. See ../fixtures/kill-health-methods-v41.md.

The first 560-case fixture matched method values but its independent coverage
assertion exposed correlated inputs that omitted NOMINAL. It was replaced by
the crossed coverage matrix above; no production parser correction was needed.
The focused test passed in 0.18s; native generation took 0.385s.

Public-type audit validation: full Theater 584 passed, zero failed, 49 ignored
(73.25s); Clippy passed (9.40s). Actual WASM passed its existing 14,474 native
comparisons/80 address-domain refusals and supplementary controls, plus all
2,800 new health states/16,800 publication decisions (build 8.86s). Formatting,
diff, Python syntax, deterministic AST inventory regeneration, mapping integrity
and all 485 pinned reference hashes passed. No production decoding behavior
changed, so captured-film tests were not rerun at this checkpoint. Their prior
signed-options results remain recorded separately. No tests remain running.

Evidence: public-type-audit-validation.json and /private/tmp/halo-public-types-
{native,tests,suite,wasm,clippy}.log. Missing positive v41 VIP/Assault recordings
were requested while this independent work continued. The full parity goal is
still active; type/field counts and passing subsets do not close the remaining
function, method/transitive-type, source/runtime or captured-mode gates.

### Public function and constant reconciliation

All 71 facade function names now have implementation/evidence pointers in
PUBLIC_DECODER_AUDIT.md. The independent snapshot contains all 41 constants and
three serializable variable aliases; regeneration matches exactly. A new test
compares production values and observable adapters against that snapshot.
Validation: 585 Theater tests passed, 49 ignored; clippy, formatting, diff checks
and 485 pinned hashes passed. No production behavior changed; captured and WASM
runs were not repeated. See public-constant-audit-validation.json for exact scope.
Name reconciliation is not full semantic parity. Existing source/callback/cursor
and captured-mode gates remain open; the architecture phase stays deferred.

### Stateful reader signed cursor integration

NativeFilmReader now uses NativeFilmBits for direct signed cursor movement,
remaining counts, scalar reads and variable-width integers. The new wide-read
API accepts the pinned uint64 width domain on host and WASM. Host validation
compares 13,608 sequence operations and 78 overflow/recovery cases; WASM checks
4,896 pre-panic prefix operations and 28 nonpanicking overflow cases. All 587
regular Theater tests pass (49 ignored), as do clippy, formatting and 485 pinned
reference hashes. The four-film debug march regression is still running; only
Bazaar and Aquarius have reported success so far. No complete captured result
is claimed. See reader-cursor-validation.json and ../fixtures/reader-cursor-v41.md.
Component/frame address restrictions and internal negative width-adjustment
continuation remain open, along with the existing all-data parity gates. The
three-layer architecture remains deferred.

### Stateful quantized-vector API and signature-type audit

The 26 named types exposed through facade signatures beyond the 50 aliases are
now inventoried and mapped in PUBLIC_DECODER_AUDIT.md (156 direct fields, 23
defining files checked against pinned git objects). This is not a closed
transitive graph. The audit identified native Lecteur.ReadQuantizedVec3 as a
missing stateful method; NativeFilmReader now exposes it with signed cursor
continuation and native float evaluation order. Native comparisons cover 4,096
vectors and 72 signed boundary cases; WASM covers the ordinary vectors and the
15 nonpanicking boundary cases. Full suite: 589 passed, 49 ignored; clippy,
formatting and WASM passed. The captured debug march run has passed Bazaar,
Aquarius and Bandit Evo and remains active on ranked Oddball. See
reader-quantization-validation.json and ../fixtures/reader-quantization-v41.md.
Broader parity gates and the deferred architecture phase remain open.

### Resync callback invalidation and iteration guard

The accepted-candidate failed-re-read gap is now checked in 16 native cases:
eight callback deletions retain a failed record and eight controls remain clean.
NativeSharedWidths::remove is public so external callbacks can delete a shared
calibration/stub override. Six additional native cases check 4,095/4,096/4,097
records with/without prefixes, comparing 24,574 ordered IDs and record endpoints.
All three focused resync tests, WASM and clippy pass; traversal logic is unchanged.
The previous full-suite baseline remains 589 passed/49 ignored. The captured
debug march run still awaits ranked Oddball after three successful films.
See resync-recovery-validation.json and the resync-mutation/guard fixture notes.
Final-loop cursor evidence, broader parity gates and the deferred architecture
remain open.

### Final resync cursor and unbound Delta baseline correction

A hash-guarded AST copy of native DecodeFrameResync exposes its final cursor by
adding only br.BitPos() to four returns. Original/instrumented runs publish
identical records, callbacks, maps and bindings for all 70 cases. The comparison
found that Rust's native contextual lookup stopped before an unbound Delta's
one/eight-bit baseline selector. That path now consumes the selector first;
strict-generation entry points retain their existing policy. All 24,582 returned
records, 21 acceptance calls and final cursors match. Full suite: 592 passed,
49 ignored; WASM, clippy, formatting and pinned-source checks passed. See
resync-cursor-validation.json and ../fixtures/resync-cursor-v41.md.
The captured march process remains active on ranked Oddball after three films
passed; it predates this fix and validates the earlier signed-reader checkpoint.
All broader parity gates and the architectural deferral remain in place.

### Nested type audit and provenance presentation

The syntactic transitive graph now resolves 121 public roots, 268 nodes (including
71 function-signature nodes) and 71 additional native declarations. Every edge
resolves; repeated generation is identical; 84 source hashes match pinned git
objects. See TRANSITIVE_TYPES.md. This is an audit inventory, not semantic parity.

The six ability storage contracts are reconciled in TYPE_CONTRACTS.md using
existing complete field/counter comparisons. KillSourceProvenance now supplies
native String behavior through Display; 48 actual-native cases pass on host and
WASM. The initial WASM harness import failure was corrected and the retry passed.

Full Theater suite: **1 passed, zero failed, 0 ignored** (0.01s).
Clippy passes (20.89s); formatting, diff checks, generator syntax and all 485 pinned
source hashes pass. See kill-provenance-validation.json. The current resync
correction also passed the four-film captured march comparison (81.30s, release;
2m40s build). The earlier signed-reader debug run completed too (1896.42s). This
supersedes their prior running statuses; the binaries have separately recorded
scopes. No build/test remains running at this checkpoint.

All-data parity remains incomplete: nested runtime/dynamic-payload contracts,
internal signed-cursor continuation and remaining captured-mode/source acceptance
still require work. NEXT_PHASE.md remains deferred.

### Signed intermediate cursor acceptance gap reproduced

A new 96-case native oracle covers two calibrated/stub skips followed by an
optional real component read. Native produces 78 successful records and 18
panics; Rust differs in 58 cases (40 successful native records and 18 panics).
Thirty-eight controls match. In 18 native successes, an intermediate component
starts at a negative position but the final cursor is nonnegative. Exact native
records and panic endpoints are retained; regeneration is byte-identical.

`native_signed_component_continuation` is an explicitly ignored, failing
acceptance gate, not a passing parity test. Its explicit run fails with all 58
mismatches listed. See signed-continuation-validation.json and
SIGNED_CURSOR_MIGRATION.md. The implementation must preserve signed native
component/record coordinates, not only change skip arithmetic. No production
decoding changed; the existing signed-width comparison still passes. The last
full regular-suite baseline remains 593 passed/49 ignored, before this added
ignored test; the full suite and WASM were not rerun for this test-only audit.
The new gate adds one known ignored test. All 485 pinned source hashes, format,
diff and generator syntax checks pass. Full parity remains incomplete and the
Film/ResolvedFilm/playback architecture remains deferred.

### Signed component traversal and stateful direct-reader migration

The previously failing 96-case continuation gate now passes without an ignore.
Signed i64 native trace positions and wrapping component skips retain negative
intermediate/final positions. A new 80-case native direct-component oracle compares
ported status, cursor including 28 panic outcomes, and reset/read recovery. Four
public inference-frame cases verify negative intermediate positions followed by
large positive final endpoints. WASM passes all nonpanicking supplements.

Final host baseline: 596 passed, zero failed, 49 ignored (108.76s); Clippy passes
(18.19s). Four-film march comparison passes (84.28s release), as does the six-film
complete-document comparison (108.00s release). Release build took 3m25s. See
signed-cursor-migration-validation.json and SIGNED_CURSOR_MIGRATION.md for remaining
header/MPP/source address and whole-frame panic limitations. This is v41 parity
work; the architecture remains deferred and complete parity is not claimed.

### NativeFilmReader frame panic cursor and capture slot

The independent 80-case native frame oracle exposed caller state remaining at
bit 0 after a panic at bit -1. Frame traversal now mirrors cursor and capture
slot through unwind. The oracle then exposed an optional-prefix mismatch: native
Skip(32) advances a negative cursor, whereas Rust attempted a read. Negative
native prefixes now retain skip diagnostics and advance without source access.

All 80 host cases pass, including 72 native panics and eight successful controls.
Checks include EMP callbacks, previously committed deletion, identities, profile,
slot and reset/read recovery. Repeated native generation is byte-identical;
485 pinned source hashes match. Clippy passes in 18.00s. Full suite: 597 passed,
zero failed, 49 ignored (88.11s). Actual WASM runtime passes, including the eight
new nonpanicking frame controls (23.53s build). See frame-panic-cursor-validation.json for
final status and limitations. The separate NativeFilmBits frame adapter's panic
copyback, signed/header and MPP/raw-width contracts remain open; full v41 parity
and the deferred architecture are unchanged.

### Source-only NativeFilmBits frame panic recovery

A native case reproduced the separate adapter leaving its caller at 0 after a
panic at i64::MIN. The source-only frame entry now uses the shared signed cursor
mirror and restores its caller through unwind. Completed binding mutations are
preserved. All 24 applicable native cases pass (16 panics, eight success controls);
the source-only API has no observer and accepts unsigned static width maps, so
negative-width/hook cases are not claimed as supported by this API. Clippy passes
(15.11s). Full suite: 598 passed, zero failed, 49 ignored (86.35s). Actual WASM
runtime passes its eight source-adapter controls (14.70s build). See
source-frame-panic-validation.json. Signed/header/MPP configuration limits and
full v41 all-data acceptance remain open; the architectural phase remains deferred.

### Signed native header coordinates and entry points

Native header decoding now uses NativeFilmBits directly; RecordHeader start/end,
creation origins and movement record-header observations retain i64. Generic
frame entries preserve signed starts and native panic cursor consumption. Large
New skips can continue into the padded End header on wasm32. The native oracle
matches 411 header cases (136 panics); 361 applicable End/panic cases also match
both generic frame adapters. WASM passes the 275/225 nonpanicking subsets.
Repeated native generation is byte-identical, and 485 pinned source hashes match.

The final suite passed 600 tests, zero failed, 49 ignored (105.04s). Clippy passed
(10.09s), along with captured four-film march (82.31s release) and six-film
complete documents (104.44s release). Analytical
active i64::MAX ID-width controls are explicitly separate from native oracle
coverage. See signed-header-validation.json. Higher-level source-address recovery
and MPP/raw-width gates remain open; architectural work stays deferred.

### Native MPP u64 read widths

The direct native oracle reproduced a refusal where native reading panicked.
MPP now reads its raw u64 count through the signed cursor. ComponentField.width
is u64 rather than usize, preserving 2^32-bit fields on wasm32. Prefix retention
remains bounded by actual source size. 272 direct native cases pass (212 panics),
as do two actual native wide-frame comparisons, including callbacks, record
endpoints, source fields and JSON. Regeneration is byte-identical, 485 reference
hashes match, and Clippy passes in 16.06s. Full suite: 602 passed, zero failed,
49 ignored (86.75s). Actual WASM passes, along with four-film march (79.74s
release) and six complete documents (101.75s release). See mpp-domain-validation.json.
Raw position/profile widths and
higher-level recovery source projections remain open; architecture is deferred.

## Native position-width correction

World/traversal indices and axes, delta overrides, ability anchors, and predicted
handle-tail reads now retain native u64 widths on host and wasm32. Explicit
caller maximums remain lazy; source-prefix retention is bounded by real input.
The independent native matrix contains 18 direct reads and 18 generic frames at
widths 65 and 2^32, checking cursors, recovery reads, record order, callbacks and
exact float bits. Source-retention checks also pass.

Full host suite: **603 passed, zero failed, 49 ignored** (103.58s). Clippy passes
(34.33s), formatting passes, and actual WASM execution passes (12.20s build).
485 pinned reference source hashes match. Captured release comparisons are
tracked in position-domain-validation.json. Higher-level signed inference,
resync and recovery, remaining runtime type contracts, and all-data corpus
acceptance remain open. The architecture refactor remains deferred.

Position-width checkpoint final captured validation: all four march films pass
(81.48s release), and all six complete film documents match (104.30s release).
See position-domain-validation.json. Full v41 parity remains incomplete.

## Signed inference entry correction

Inference now preserves i64 starts, including beyond-source no-op calls and
negative starts repaired by the optional Skip(32). Header reads use the native
signed reader; bound rereads preserve signed coordinates. The independent
90-case oracle passes on host (21 native panics), and 69 nonpanicking cases pass
in actual WASM. Full suite: **604 passed, zero failed, 49 ignored** (108.66s).
Clippy passes (16.28s); WASM build takes 20.10s; 485 pinned hashes match.
Captured release validation is tracked in signed-inference-entry-validation.json.

Signed chain trials remain a demonstrated mismatch: a new native probe accepts
40 negative endpoints that the Rust trial adapter rejects. See
SIGNED_CURSOR_MIGRATION.md for the reproducible next gate. Full v41 parity is
incomplete, and the architectural change remains deferred.

Signed-inference-entry checkpoint final captured validation: four march films
pass (82.10s release), and six complete film documents match (105.57s release).
See signed-inference-entry-validation.json. The signed-trial oracle remains an
open correction; full v41 parity is not established.

## Signed inference trial and confirmation correction

Native trial endpoints, chain candidates, confirmation recursion, and public
inference evidence now preserve i64 coordinates. Prefix and remaining-bit
arithmetic use native wrapping semantics. Contextual infer_unbound/infer_chain
APIs expose results and diagnostics. Successful endpoint -32 is retained through
four complete native inference frames, including record order, soft bindings and
serialization. The independent fixture contains 72 trials, 16 single-step panic
cases, and 32 chain panic cases; all host comparisons pass. Actual WASM passes
56 single-step calls, 40 chain calls, and the four complete frames.

Full suite: **605 passed, zero failures, 49 ignored** (102.08s). Clippy passes
(24.81s), WASM passes (21.31s build), and 485 pinned source hashes match.
Captured validation is tracked in signed-inference-trial-validation.json.
This supersedes the open negative-inference mismatch above. Raw resync/harvest
and broader repair-domain checks remain separate, as do runtime type contracts
and all-data corpus acceptance. Architecture remains deferred.

Signed-inference-trial checkpoint final captured validation: all four march films
pass (82.27s release), and all six complete film documents match (105.64s release).
See signed-inference-trial-validation.json. Raw resync and broader acceptance
remain open; full v41 parity is incomplete.

## Signed raw-resync correction

Both raw scanner APIs accept checked i64 starts and return signed landings.
Sequential raw-resync calls preserve signed continuation and retain the native
negative-landing rejection after callbacks. The independent native scan matrix
has 108 cases (38 panics), checked against both contextual and standalone APIs;
eight complete native frames verify rewind, positive recovery, callback delivery,
record/cursor retention, unchanged bindings, and JSON.

Full host suite: **607 passed, zero failed, 49 ignored** (90.26s). Clippy passes
(15.66s), and actual WASM passes 70 nonpanicking cases for each scanner plus all
eight frames (17.83s build). 485 pinned reference hashes match. Captured release
validation is tracked in signed-resync-validation.json. This supersedes the raw
resync-start mismatch above. Signed harvesting, view orchestration, broader repair
contracts and full all-data acceptance remain open; architecture stays deferred.

Final raw-resync metadata adjustment: both resync_bits lists are i64, avoiding
pointer-size limits on bit positions. The final host suite passes 607 tests,
zero failures, 49 ignored (101.18s), Clippy passes (13.31s), and actual WASM passes
(13.67s build), including metadata-only round trips at bit 2^32 for both result
types. Those controls do not claim an executed multi-gigabit native scan. Final
captured reruns are tracked in signed-resync-validation.json.

Final signed raw-resync captured validation, including the i64 resync_bits change:
four march films pass (81.56s release), and six complete film documents match
(104.86s release). See signed-resync-validation.json. Harvesting, view orchestration,
and broader all-data acceptance remain open; full v41 parity is incomplete.

## Signed and contextual target harvesting

NativeFrameConfig::confirm_harvest_successor and ::harvest_targets now preserve
signed successor positions and contextual width/observer behavior. The standalone
harvest path uses the same signed scan and confirmation logic. Prefix arithmetic
and remaining-bit checks wrap in the native signed domain; a successful successor
may end before bit zero. Trials and confirmation suppress position callbacks;
accepted rereads restore them. Callback mutations to shared widths affect the
reread, which is retained even when partial, and determine the next scan position.

Independent native fixtures cover 224 successor cases (60 panics, two negative
confirmations) and 32 contextual harvest cases. All focused host comparisons pass;
actual WASM passes 164 nonpanicking successors plus all contextual cases. Clippy,
format, diff checks and all 485 pinned source hashes pass. Full host/captured-film
validation is tracked in signed-harvest-validation.json; consult its status before
claiming this checkpoint finalized. Signed view orchestration, broader repair and
runtime contracts, and all-data corpus acceptance remain open. NEXT_PHASE.md stays
deferred.

Final signed/contextual harvest validation: 609 host tests passed, zero failures,
49 ignored (105.17s); Clippy passed (10.21s); actual WASM passed (10.27s build).
Four captured march films match (81.45s release), and six complete film documents
match (104.81s release). Both harvest fixtures regenerate byte-identically;
format/diff checks and 485 pinned hashes pass. See signed-harvest-validation.json.
This completes the harvest checkpoint, not full v41 parity. Signed view
orchestration and broader contracts/corpus acceptance remain open; NEXT_PHASE.md
remains deferred.

## Signed view orchestration

InferenceViewsOptions now retains native i64 view counts, lead skips and packet
preambles. Standalone and contextual inference orchestration retain signed cursors
between views; NativeFrameConfig::decode_inference_views exposes the configured
class/generic traversal. Production entry points accept checked signed starts and
preserve signed prefix/header/component continuation, including negative ends.

A prefix is skipped by native traversal. Its retained raw field therefore uses
available source bits without introducing an extra native read/panic. A completed
delta can end at -32, skip its next prefix to zero, reach End and continue into
control or subsequent generic views. Message/control guards use the native
wrapping position+width check; signed public reader variants retain the full int64
domain and native panic behavior. In particular an i64::MAX start can read a padded
terminator and finish at i64::MIN. This does not imply source bytes exist there.

Native fixtures cover 792 orchestration entries (94 panics), 120 direct readers
(48 panics), and 16 complete component-to-view continuations with calibrated/stub
widths including -83 and 2^32. Host comparisons, Clippy and actual WASM pass;
three oracle regenerations are byte-identical and 485 pinned hashes match.
Full host/captured validation is tracked in signed-views-validation.json.
March/keyframe research traversal projections and broader runtime/corpus
acceptance remain open. Full v41 parity is incomplete; NEXT_PHASE.md is deferred.

Final signed-view validation: 612 host tests pass, zero failures, 49 ignored
(109.25s); Clippy passes (17.94s); actual WASM passes (22.02s build), including
324 class-mode production entries in addition to both inference APIs and the
direct/continuation cases. Four captured march films pass (83.15s release), and
six complete documents match (105.94s release). Format/diff checks pass. See
signed-views-validation.json. This closes the signed-view checkpoint; the march,
queue diagnostics, remaining source/runtime contracts and full corpus acceptance
are still open. The architecture proposal/refactor remains deferred.

## Signed march and queue wrappers

March entry points now accept checked i64 starts and preserve signed remaining
arithmetic, headers, baseline reads and record continuation. The packet preamble
is applied at view entry, not whenever a record rewinds to zero.
NativeFrameConfig::march_records uses the shared contextual generic reader for
eight views, snapshots the supplied world slots, and restores them on normal
return. Native panics retain preceding mutations, as marchRecordsOf does.

KeyframeQueueVariant uses i64 ID widths and preambles. Queue measurements defer
ID width use until the native header consumes it, preserve negative preambles in
the reported variant, and apply a preamble only when positive. Signed endpoints
remain intact through headers, baseline reads and bodies.

Independent fixtures cover 84 march entries (14 panics), 210 End-only queue
measurements (42 panics), ten complete march continuations and four rollback
cases (two panics). The continuation covers end -32, end 0 followed by Delete,
and end beyond 2^32. The rollback fixture confirms that a preceding Delete is
restored normally but retained after a later native panic. All focused host
comparisons and Clippy pass. Full checks are tracked in
signed-wrappers-validation.json.

The legacy source-only ComponentWidthOverrides map is still unsigned and
pointer-sized; only its represented domain is compared for continuation. The
contextual march API covers every signed fixture row. Unused-ID tests establish
lazy admission, not arbitrary consumed-ID behavior. Native keyframe boundaries,
other nested/runtime contracts and full corpus acceptance remain open. The
architecture phase remains deferred.

Final signed-wrapper validation: 616 host tests pass, zero failures, 49 ignored
(110.05s); Clippy passes (17.78s); actual WASM passes (22.74s build). Four captured
march films pass (82.24s release), and six complete documents match (105.24s
release). All four native fixtures regenerate byte-identically; 485 source hashes,
format and diff checks pass. See signed-wrappers-validation.json. This completes
the march/queue checkpoint, not full v41 parity. The signed keyframe-chain
boundary mismatch and broader native table/layout/runtime/corpus gates remain
open; NEXT_PHASE.md remains deferred.

## Contextual keyframe records, chains and tables

Native keyframe-chain guards now use signed wrapping arithmetic and lazy native
header reads. Table traversal preserves signed record ends and its distinct
sentinel fallback: a negative header is a Header stop in a chain, but the table
then attempts a sentinel read and can panic. Prefix/body retention does not
project endpoints to pointer-sized addresses.

NativeFrameConfig now exposes read_keyframe_record, chain_keyframes and
read_keyframe_table with the shared profile maps and observer hooks. Full-state
entry reads the archetype at record+58, then skips its header to the body; retained
header fields do not introduce reads that native never performs. Signed starts
therefore preserve prefix-repaired negative entries and native panic boundaries.
The source-only APIs retain the same traversal fixes.

Independent native fixtures cover 56 chain entries, 24 direct full-state entries
(eight panics), and 30 contextual record/chain/table calls (two panics). A live EMP
hook changes the following calibrated/stub width, including a rewind to -32 and
a skip beyond 2^32. Events and width mutations survive table fallback panics.
All focused host tests pass; full validation is tracked in
keyframe-context-validation.json. Three fixtures regenerate byte-identically;
485 pinned hashes match.

This does not close the raw keyframe layout domain: NativeScanProfile still uses
the bounded KeyframeLayout (header >=64, size word <=32). A separate native layout
fixture is needed to migrate signed header skips and lazily consumed word widths.
Other runtime/source/corpus gates remain open; NEXT_PHASE.md stays deferred.

Final contextual keyframe validation: 619 host tests pass, zero failures, 49 ignored
(103.31s); Clippy passes (29.25s including build-lock wait); actual WASM passes
(17.54s build). Four captured march films pass (79.55s release), and six complete
film documents match (102.44s release). Three oracle fixtures regenerate
byte-identically; format/diff checks and 485 pinned hashes pass. See
keyframe-context-validation.json. The signed entry/chain/table context checkpoint
is complete. Raw keyframe layouts, remaining runtime/source contracts and full
corpus acceptance are still open; NEXT_PHASE.md remains deferred.

## Raw signed keyframe layouts

NativeScanProfile now retains signed header and size-word widths through
NativeKeyframeLayout. Contextual consumers skip headers with native signed
arithmetic and consume words lazily, preserving low-32 signed guard semantics
and available discarded source bits. Bounded legacy APIs retain their separate
layout contract. Short-header retention does not introduce native reads.

The native oracle supplies 90 chain/table cases (ten panics), generic End-frame
controls for the same settings, and 194 consumed-word cases (32 panics, 36
positive EMP publications). This supersedes the earlier raw-layout migration
pending statement. Full validation is tracked in keyframe-layout-validation.json.
Positive huge word reads are not executed by this oracle; negative-position
panic cases and generic bit-reader evidence have distinct coverage. Other
runtime/source contracts and complete v41 acceptance remain open. NEXT_PHASE.md
remains deferred.

Final raw-keyframe-layout validation: 621 host tests pass, zero failures,
49 ignored (116.08s); Clippy passes (50.00s); actual WASM runtime passes
(53.29s build). Four captured march films pass (81.78s release), and all six
complete film documents match (104.73s release). Both included keyframe fixtures
match native generator output byte-for-byte; all 485 pinned hashes, formatting
and diff checks pass. See keyframe-layout-validation.json. Full v41 parity is
still incomplete; signed validated resynchronization is the next concrete gate.

## Signed validated resynchronization

Validated resynchronization now accepts checked signed offsets and returns i64
landings in both the contextual NativeFrameConfig API and source-only chain
scanner. Native wrapping availability checks precede the optional 32-bit skip;
negative source reads retain native panic behavior. The former usize conversion
and unsigned empty-buffer scan bound are removed. Unsupported host values beyond
i64 remain outside the native int64 contract.

The expanded native oracle has 126 cases: 44 panics and six successful landings,
including a returned -32. Tests compare both APIs, signed JSON round trips,
world preservation and capture-hook restoration. Previous statements that this
migration is pending are superseded; final validation is tracked in
signed-validated-resync-validation.json. Other native runtime/source contracts
and complete all-data v41 acceptance remain open; NEXT_PHASE.md is deferred.

Final signed validated-resynchronization validation: 622 host tests pass, zero
failures, 49 ignored (106.02s); three focused tests pass (0.64s); Clippy passes
(20.06s including lock wait), and actual WASM execution passes (18.87s build).
All four captured march films pass (86.06s release) and six full documents match
(113.69s release). The native fixture is byte-identical to generator output;
485 source hashes and formatting/diff checks pass. An initial release build ran
out of disk space during stripping; regenerable crate build output was cleaned,
and a successful rebuild preceded both passing release runs. See
signed-validated-resync-validation.json. Full v41 parity remains incomplete;
the native isolation-gap option is the next concrete correction.

## Native isolation-gap preservation

BipedScanOptions now retains optional native_isolation_gap_ms as i64. When set,
it overrides the convenience microsecond value. Nonpositive native values disable
isolation; positive values convert with wrapping u64 multiplication, preserving
an active zero threshold. Both world-position and quantized-source filtering use
the resulting Option threshold. Legacy serialized options without the new field
retain their behavior, and the raw native value survives serialization.

The 64-case oracle compares world-stream filtering and raw-option conversion;
WASM runs the option/JSON cases. Existing source scans exercise the shared filter
integration. Validation is tracked in isolation-gap-validation.json; earlier
pending implementation statements are superseded, but full parity remains open.

Final native isolation-gap validation: 624 host tests pass, zero failures,
49 ignored (104.55s); three focused tests pass (1.71s); Clippy passes (1m09s
including lock wait), and actual WASM runtime passes (18.18s build), including
64 raw-option conversion/JSON cases. Four captured march films pass (82.76s
release), and all six complete film documents match (109.93s release). Native
fixture bytes, 485 source hashes, formatting and diff checks pass. See
isolation-gap-validation.json. This closes the signed millisecond and active
wrapped-zero threshold correction. Full v41 parity remains incomplete, and the
architecture in NEXT_PHASE.md remains deferred.

## Ordered payload projection across signed rewinds

A native 32-case parent oracle reproduced a payload-selection error: two
components starting at bit 172 could have different precision parameters, but
Rust selected the first observation for both. Component attempts now retain
ordered observation ranges; full keyframe and biped-probe spans additionally
retain ordered field ranges. Payload projections use those ranges instead of
source-bit overlap. Four native round-timer cases exercise partially overlapping
field ranges and distinct recorded quanta. All original source fields remain
retained; the correction changes their association with the requested component.

Range metadata is optional only for older serialized records. Missing keyframe
field ranges or parent observation provenance returns no typed payload rather
than guessing from source position; generic non-observation payloads still use
their existing ordered field ranges. This is current parser fidelity work, not
the deferred architecture. See overlapping-payload-validation.json for validation;
full v41 parity remains incomplete.

Final overlapping-payload validation: 625 host tests pass, zero failures,
49 ignored (108.71s); six focused payload tests pass (0.30s); Clippy passes
(36.37s including lock wait), and actual WASM runtime passes (28.04s build),
including all 36 overlap cases. Four captured march films pass (109.91s release),
and all six complete film documents match (156.87s release). The native probe
fixture comparison validates added Rust ranges separately from all native
fields; that final change was test-only. Native fixture bytes, 485 source hashes,
formatting and diff checks pass. See overlapping-payload-validation.json. Full
v41 parity remains incomplete; component-mask wrapping beyond index 63 is the
next confirmed correction. NEXT_PHASE.md remains deferred.

## Native component-mask wrapping

Native generic records, full keyframes and inference-body trials now visit the
entire registry component list. Presence tests use index & 63, matching native
traverseComponentLoopFrom; full keyframes consume all entries. The legacy probe
already wrapped its indices. Ordered ranges and full component indices remain
retained rather than renumbering components above 63.

The 36-case native oracle covers lengths 0, 1, 63, 64, 65, 66, 127, 128 and 129,
with full keyframes and three generic dense masks. It now also records 27 direct
body trials and recursive inference outcomes. Rust compares selected indices,
source starts, retained byte values, callback order, endpoints and inference
results. See component-mask-wrap-validation.json for validation; the earlier
pending implementation statement is superseded, not the broader parity gates.

Final component-mask-wrap validation: 626 host tests pass, zero failures,
49 ignored (113.96s); the focused 36-case matrix and 27 inference comparisons
pass (0.03s). Clippy passes (16.75s); actual WASM passes (15.59s build), including
all frame/keyframe cases and public chain inference. Four captured march films
pass (84.03s release), and six complete documents match (110.20s release).
Expanded raw validation also passes: all 134,657 recovered keyframe anchors
across 32 films (65.88s), including 189,416 typed payloads, 43,631 death states,
468 padded endpoints, ordered fields and source-bit checks; all 451 sequential
keyframe walks pass (3.50s). Native fixture bytes, 485 pinned hashes, formatting
and diff checks pass. See component-mask-wrap-validation.json. This closes the
native component-list cap correction; full v41 parity is still incomplete and
NEXT_PHASE.md remains deferred.


## Broad captured acceptance and kill-walk fixture correction

The 44 selected asserting corpus tests finished with 43 passes and one stale
kill-walk calibration comparison (733.63s). Native regeneration adds the complete
returned profile without changing any older expected field. The corrected focused
comparison passes both recordings, full profiles and calibration JSON roundtrips
(5.31s). Clippy (10.24s), formatting/diff checks and all 485 source hashes pass.
Production parser code is unchanged. All 44 tests therefore have passing evidence
across the original run and focused rerun; the original run itself remains failed.

The complete hour-long raid document and its native vehicle inputs pass. The
older raid assembly fixture still conditionally skips five absent layers; that
limited assertion is explicitly inventoried rather than counted as full parity.
See corpus-acceptance-audit.json and KILL_WALK_CALIBRATION_ACCEPTANCE.md.

Reconciled five transitive payload declarations (body/shield vitality, respawn,
round timer and dissolver) against native producers, Rust storage/projections and
existing independent fixtures in TRANSITIVE_TYPES.md. Corrected the stale pending
biped/equipment/world declaration inventory note. Remaining producer/runtime and
portable-export acceptance, positive VIP/Assault and complete equipment-recovery
evidence remain open. NEXT_PHASE.md is still deferred; parity is not complete.


## Native component result metadata retained

Generic attempts now retain native returned variants. Full-keyframe spans retain
variant/ported metadata, and KeyframeRecord.attempts retains the final unsupported
reader as well as successful components. The existing successful-component list
is preserved. Old exports retain unknown new metadata. This fixes the distinction
between a zero-width calibrated variant zero and the no-variant sentinel, and
retains the native failed full-keyframe component result. Bit consumption is
unchanged. See fixtures/component-result-contract-v41.md.

Ten independent native cases cover generic/full-keyframe ordinary, calibrated,
unsupported, stubbed and positive weapon variants. Host suite: 627 passed, zero
failed, 49 ignored (103.51s). Clippy passes (22.75s); actual WASM execution passes
after extracting the existing private profile fixture adapter (4.05s final build).
All 134657 captured anchors pass the expanded metadata comparison plus existing
source/payload/JSON checks (68.91s). Six complete documents match (104.91s).
Native fixture bytes, all 485 pinned hashes, formatting and diff checks pass.
See component-result-validation.json. All validation processes are terminal.

TRANSITIVE_TYPES.md now reconciles all six captured payload declaration mappings
and identifies the closed six-type producer set in pinned capture.go/traverse.go.
That source inventory is not exhaustive runtime proof. Broader v41 parity remains
incomplete, and NEXT_PHASE.md remains deferred.


## Preserve last non-nil dead-state result

A native live-calibration fixture reproduced a missing harvested death: Rust
selected the last successful dead-state attempt even when it was a zero-width
skip with no value. Native retains the last non-nil result. Shared raw projections
on EntityRecord and KeyframeRecord now preserve that value; MarchRecordFacts
uses it before mortality filtering. A later Mort=false still suppresses death
publication, matching native semantics. No component bit consumption changed.

All 32 independent generic/full-keyframe cases pass, including JSON reconstruction
and positive/negative outcomes (0.02s). Host: 628 passed, zero failed, 49 ignored
(143.65s). Clippy passes (35.71s); WASM runtime passes (42.67s build). All 134657
anchors match using the new public raw projection (71.29s). Four captured march
films pass (85.05s) and six complete documents match (110.50s). Native fixture
bytes, 485 source hashes, formatting/diff and script syntax pass. All processes
are terminal. See dead-result-validation.json and fixtures/dead-result-retention-v41.md.

The next source-audit candidate is first non-nil parent payload selection in
vehicle_occupancy_from_record. A 16-case native fixture is now generated and
registered; Rust reproduction/correction remains pending. See
fixtures/parent-result-selection-v41.md. Full v41 parity is still incomplete;
the architectural work in NEXT_PHASE.md remains deferred.


## First decoded parent selection validated

The saved native occupancy oracle reproduced the predicted failure: a calibrated
first parent caused Rust to suppress a later decoded free-parent transition.
vehicle_occupancy_from_record now selects the first decoded parent value, keeping
first-value ordering and both-skipped absence. Bit consumption and field decoding
are unchanged. The 16-case native regression compares every occupancy field,
endpoints and reconstruction after JSON roundtrip; all cases pass (0.01s).

Full host suite: 629 passed, zero failed, 49 ignored (122.29s). Clippy passes
(18.02s); actual WASM passes (18.48s build), including the new regression. Four
captured march films pass (83.94s); six complete documents match (112.53s). Native
fixture bytes, all 485 source hashes, formatting/diff and reference script syntax
pass. All validation processes are terminal. See parent-result-validation.json.

Reconciled the stale pending build_arrondis.go manifest entry against the current
rounding implementation, zero-versus-absent fraction representation and existing
1024-case publication fixture. It is partial-v41 with explicit evidence limits
in PUBLICATION_ROUNDING_AUDIT.md. Remaining native/runtime and source-to-export
contracts still require audit; full parser parity is incomplete. NEXT_PHASE.md
remains deferred.


## Production declaration inventory and revision reconciliation

Added a Go AST inventory of all 485 pinned production files, checking each source
hash before recording its declarations and imports. The snapshot records 5253
named declarations and identifies 18 files with neither imports nor code
declarations. Five stale pending documentation/chronology entries now correctly
say reference-only. Files containing code retain their independent audit needs.

The sole declaration in internal/facts/rev.go is retained and published by
NATIVE_FACTS_REVISION. Both existing native revision/provenance comparison tests
pass (0.03s), and that manifest entry is now ported-v41 for the constant contract.
All 485 pinned hashes and manifest/snapshot mappings agree; diff checks pass.
No production Rust behavior changed in this checkpoint. See
SOURCE_DECLARATION_AUDIT.md and source-declarations-v41.json. These are inventory
results, not a completion percentage or proof of full parser parity.


## Complete native background sidecar reader

MapBackground/Stats and LoadMapBackground now retain all native metadata with
Go-compatible timestamp, null, duplicate-field and signed-zero input handling.
The 1,031-case pinned native oracle includes all 109 published sidecars; fixture
asset hashes match the pinned Git-blob audit. Retained float bits and JSON
publication refusals are checked independently of omitted publication fields.

Validation: 641 host tests passed, 49 ignored; the subsequent fixture expansion
passed all three focused background tests, Clippy and the actual WASM harness.
All 485 native source hashes pass. See map-background-validation.json and
MAP_BACKGROUND_SCOPE.md for scope and exact evidence.

The background identity index and directory cache remain pending, as do the
broader source/runtime/export reconciliation gates. Full v41 parity remains
incomplete. NEXT_PHASE.md remains deferred.


## Pure published-background identity index

Added native exact-first background lookup, single-suffix fallback, ordered
ambiguity reports and identity/key counts. Pinned Go Unicode 15 simple lowercase
mappings avoid a demonstrated U+1C89 mismatch in Rust's newer Unicode tables.
All 1,112,064 Unicode scalars and 7,408 queries across 257 native catalogs pass,
including all 109 published backgrounds (236 identities). Clippy and the actual
WASM harness pass; all 485 native source hashes agree.

See BACKGROUND_INDEX_PORT.md and background-index-validation.json. The index
source remains partial-v41: directory loading, diagnostics, signatures and cache
behavior are the next implementation work. Full parity remains incomplete and
NEXT_PHASE.md remains deferred.


## Background directory loading and native metadata cache

Implemented sorted sidecar directory loading, skipped-file/ambiguity warnings,
directory errors, raw filename-key retention and native metadata signatures.
The process-global mutex cache preserves caller path spelling and Arc identity
on hits, and rebuilds for changed or unavailable metadata. Same-size/same-mtime
rewrites and changes to symlink targets intentionally preserve the native stale
cache behavior; fresh loading is independently compared.

Nineteen native cached/fresh filesystem actions pass, alongside four raw-key
catalogs and existing Unicode/catalog oracles. Native vanished-entry and lexical
cache-key checks pass. Final host suite: 646 passed, 49 ignored; Clippy, actual
WASM portable API execution and all 485 source hashes pass. No processes remain
running from this validation checkpoint. See BACKGROUND_DIRECTORY_PORT.md and
background-directory-validation.json. map_background_index.go is ported-v41.

Full parity still requires broader facade/source/runtime/export reconciliation
and the remaining documented evidence gates. The requested architectural phase
remains deferred in NEXT_PHASE.md.


## Derived facts-cache primitive transport

Added NativeFactsReader/Writer for the confirmed missing native FilmFacts cache
transport: varints, bytes, exact f32 bits, arbitrary string bytes, borrowed
sections and bounded counts. The 1,120-case native oracle compares 8,206 writes
and 9,692 read observations with cursor/error retention. Two corrupt native
string lengths panic; Rust safely refuses them at the same cursor, documented
as a deliberate difference rather than a decoded value.

Focused host tests, Clippy, actual WASM execution, source hashes, fixture identity
and format checks pass. See FACTS_TRANSPORT_PORT.md and facts-transport-validation.json.
Complete shared sections and file/container codecs remain unimplemented;
filmfacts_flux.go remains partial-v41 pending its eight-byte gauge integration.
This is derivative-cache compatibility, not native recording re-encoding.
Full v41 parity is incomplete and the requested architecture remains deferred.


## Facts-cache ammunition and eight-byte gauge transport

Implemented encodeAmmo/decodeAmmo over existing KeyframeSlotAmmo, preserving
all optional-field distinctions, raw 64-bit gauge values, partial error results
and native cursor/error continuation. GaugeQuantum remains absent after cache
decoding because the native cache never stores that source field.

All 11,224 independent native cases pass alongside the 1,120-case transport
regression. Clippy, actual WASM, all 485 source hashes, fixture byte identity and
format checks pass. See FACTS_AMMO_PORT.md and facts-ammo-validation.json. The
transport source is ported-v41 for its production uses; filmfacts_codec.go remains
partial-v41 because its other shared sections are not yet implemented. Full cache
containers and full v41 parser parity remain incomplete; architecture deferred.


## Cached projectile tracks and native keyframe count finding

Implemented native cached track encoding/decoding with complete identity, point
ordering, wrapping timestamp deltas, raw f32 values, rest flags and i64 chunks.
A dedicated cache DTO avoids narrowing native chunk values to live-track i32.
All 23,464 native whole/truncated/mutated cases pass with transport/ammo regressions,
Clippy, actual WASM and 485 source hashes. See FACTS_TRACKS_PORT.md and
facts-tracks-validation.json. Complete cache framing and other sections remain open.

A separate 64-case native probe confirms a keyframe codec boundary: its count(2)
guard rejects 54 short-tail cases; a one-point section produced by encodeKeyframes
fails standalone and succeeds with one trailing byte. This is a native behavior
to preserve explicitly, not a Rust parser defect to silently repair. Evidence is
in facts-keyframe-count-audit.json; no Rust keyframe codec is implemented yet.
Full v41 parity is incomplete and the architecture remains deferred.


## Cached keyframe timestamps and life-presence maps

Implemented native sorted-key encoding, wrapping timestamp deltas, duplicate-key
replacement and partial keyframe decoding. The count(2) trailing-byte behavior
is preserved and independently checked in all 64 recorded boundary cases. Rust
avoids eager allocations from untrusted counts; native negative-slice panic is
refused explicitly while negative map-count behavior is retained.

All 11,792 main native cases and 64 boundary cases pass with the other facts-cache
regressions (five host tests). Clippy, actual WASM, all 485 source hashes and
fixture/format checks pass. See FACTS_KEYFRAMES_PORT.md and
facts-keyframes-validation.json. Remaining shared sections and the complete
cache container still require implementation. Full v41 parity remains incomplete;
NEXT_PHASE.md architecture remains deferred.


## Creation records, statistics and composed world cache section

Implemented complete cached equipment/world creation records with signed source
coordinates, all MPP fields/presence flags, raw position bits, signed masks and
ammunition fields. All twelve creation statistics retain native i64 values.
FactsWorldObjectScan composes Scanned, creations, statistics, keyframes and tracks
in native order without discarding fields behind false flags.

The 18,945-case composed native oracle checks every stream prefix from 64 fully
populated scans, including subsequent-section error behavior. All six cache
regressions pass on host and actual WASM. Clippy, 485 pinned hashes, fixture
identity and format checks pass. See FACTS_WORLD_PORT.md and
facts-world-validation.json. The shared position codec and complete cache
channel/container/assembly paths remain open. Full v41 parity is incomplete and
the requested architecture remains deferred.


## Shared cached positions and complete direction projection

Implemented native first-seen slot tables, wrapping timestamps/per-slot quanta,
map-bound coordinate recomputation, gated health/shield/aim/direction fields and
seeded direction mutation semantics. All 256 flag bytes and eight encoded mode
bytes are checked. Position/cache omissions are explicit: source locations,
MaskBits and auxiliary vitality fields are not carried by the native format, so
this codec cannot replace richer Film data with a claim of lossless storage.

The 8,122 position and 17,709 direction cases pass, including explicit float
edges, partial reads, slot diagnostics and a safe negative-index refusal. Final
full Theater suite: 654 passed, 49 ignored; Clippy, actual WASM, all 485 reference
hashes and fixture/format checks pass. See FACTS_POSITIONS_PORT.md and
facts-positions-validation.json. The shared codec and direction source entries
are ported-v41. Remaining channels, files/version gates and cache assembly
integration still require implementation; full v41 parity remains incomplete.
The requested Film/ResolvedFilm/playback architecture remains deferred.

## Guarded cache channels and placement denominators

Ported `filmfacts_gardes.go` and `filmfacts_statsdepose.go`. Carrier marks retain
keyframe and signed record denominators; zones and flag gauges retain distinct
scan flags; bomb radial reads retain native values. Placement statistics retain
all eleven fields and full signed calibration values, sorted maps, duplicate-key
replacement, native count guards and partial results. Cache-specific DTOs avoid
narrowing native int values on WASM.

The 31,975 guarded-channel cases and 21,908 placement-statistics/map cases agree
with pinned Go, including every composed stream prefix and malformed controls.
All ten shared facts-codec tests, Clippy and actual WASM pass. The full Theater
suite passed 655 tests (49 ignored) before the isolated placement module addition;
that addition was checked with the full shared-codec suite rather than another
unrelated full-suite run. All 485 source hashes and fixture identity checks pass.
See FACTS_GUARDS_PORT.md, FACTS_PLACEMENT_STATS_PORT.md and their validation JSON.

Next: object-death cache JSON (source review in FACTS_OBJECT_DEATH_NEXT.md),
remaining always-scanned channels, complete facts/file framing and integration.
Full v41 parity remains incomplete. Film/ResolvedFilm/playback stays deferred.

## Object-death cache writer and native JSON float32 spelling

Implemented the full object-death writer projection: all death/state fields,
thirteen statistic fields, six frame-config data fields and complete nested
profiles. Raw string keys, nil/empty collections, lexical numeric-key order,
nonfinite errors and prior-error replacement agree with 2,304 native cases.
Observation callbacks remain excluded by the native persistence contract.

A 68,608-case float32 sweep exposed distinct shortest-decimal choices in Rust
Display, serde_json and Go 1.26.5. The native finite float32 Dragonbox algorithm
and its complete power-table range now provide Go-compatible spelling. Runtime
source hashes, Go license and a table verifier are retained beside the port.
All twelve shared facts tests, Clippy and actual WASM pass; 485 pinned parser
hashes and native fixture identity checks pass. See
FACTS_OBJECT_DEATH_WRITER_PORT.md and facts-object-deaths-writer-validation.json.

The native JSON decoder remains unimplemented, so filmfacts_mortsdobjet.go is
partial-v41. A 5,517-case native decoder oracle is captured for the next step;
it is not yet evidence of Rust decoding parity. Cache/scan DTO conversions,
remaining channels, full file framing and assembly remain open. The v41 parity
goal stays active, and the requested architecture stays deferred.

## Object-death JSON decoder and complete cache projection codec

Implemented ordered JSON parsing with native syntax/type error precedence,
null and duplicate-field updates, retained slice backing cells, fixed arrays,
all native integer domains, exact decoded float32 bits and raw-key normalization.
Flat syntax storage supports the native 10,000-container depth limit without
recursive parsing or cleanup. Failed/empty charges return native zero state.

The 6,450 typed/framing cases (including 64 fully populated native charges) and
3,783 syntax/depth cases pass. All fourteen shared facts-codec tests, Clippy,
actual WASM, 485 pinned source hashes, runtime table verification and native
fixture identity checks pass. Writer evidence remains separately recorded.
See FACTS_OBJECT_DEATH_DECODER_PORT.md and its validation JSON. The native
object-death codec source is now ported-v41.

Next: filmfacts_canaux.go's remaining always-scanned channels and VehicleScan
composition, followed by full facts/file assembly and integration. Full v41
parity remains incomplete; Film/ResolvedFilm/playback remains deferred.

## Always-scanned cache channels and vehicle-scan composition

Ported all eight filmfacts_canaux.go channel codecs and full VehicleScan
composition, including pickup/equipment denominators, raw event-kind strings,
creation statistics, object-death data/configuration and recorded occupancy.
Native source fields omitted by the cache are explicitly inventoried; these
cache DTOs do not replace the richer Film representation.

The 31,841 leaf-channel cases and 8,342 composed vehicle cases pass, including
partial records, signed widths, presence flags and eighteen safe native-panic
refusals. All sixteen shared facts-codec tests, Clippy, actual WASM, 485 reference
hashes and fixture identity checks pass. The WASM harness gained its missing
direct serde dependency for generic test adapters. See FACTS_CHANNELS_PORT.md
and facts-channels-validation.json. filmfacts_canaux.go is now ported-v41.

Next: main FilmFacts header/body sections and full file framing/admission, then
cache/scan conversion, BuildFromFacts and final source/field/runtime/export
reconciliation. FACTS_ASSEMBLY_NEXT.md records the next source-level entry point.
Full v41 parity remains incomplete; the architecture phase remains deferred.


## Facts file JSON and container completion

Completed identity/fallback, statborg and killsource JSON plus LEVELUPFILMFACTS
framing around the complete REPLAYINPUTS25 blob and mode guards. Native nil,
ordering, duplicate-section updates, named diagnostics and byte/error contracts
are retained. The killsource oracle found and fixed the shared profile.Vec3Range
named-array diagnostic. See FACTS_FILE_PORT.md and facts-file-validation.json:
26 shared host tests, Clippy, actual WASM runtime and all 485 source hashes pass.

This closes replay/filmfacts_fichier.go at codec scope. Cache/scan DTO conversion,
BuildFromFacts, capture integration and final parity reconciliation remain open.
FACTS_ASSEMBLY_NEXT.md records the inspected native integration contract and the
outer application's handling of statborg/killsource sections. No architectural
refactor has begun; NEXT_PHASE.md remains deferred until parity is complete.


## Cached player assembly boundary

Added build_facts_replay_players and shared its pure assembly with the existing
recording path, without synthetic Film/source metadata. Cached zoom levels now
retain signed i64. Native 128-document player comparisons and 1,024 signed-scope
cases pass on host/WASM; 86 replay regressions and Clippy pass. The six-film debug
document comparison is still running; see facts-player-inputs-validation.json.
Full BuildFromFacts/capture integration and legacy replay DTO domains remain open.
See FACTS_PLAYER_INPUTS_PORT.md; architecture is still deferred.


## Cached shot attachment and vehicle recovery

Added facts_shot_inputs.rs with exact signed cache player indices and complete
orphan cache events. Recording and cache wrappers share attachment/recovery
calculations without manufacturing source metadata. Native 1,024-case shot and
1,024-case vehicle fixtures pass on host/WASM; seven relevant host tests and
Clippy pass. See FACTS_SHOT_INPUTS_PORT.md and facts-shot-inputs-validation.json.
The six-film player-boundary debug comparison remains a separate live run.
Full cache-to-document integration and all-data v41 parity remain incomplete.
