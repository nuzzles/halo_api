# Public decoder and integration audit

Pinned reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
This is an audit of current parity, not the deferred architecture.

## Decoder facade

Go AST inspection of decfilm/decfilm.go finds 166 exported declarations:
71 functions, 50 type aliases, 41 constants and 4 variable aliases. The header's
historical count of 163 is stale. Every function has exactly one return of a
qualified call, with every argument forwarded unchanged and in parameter order.
Every remaining declaration aliases a qualified target. There is no independent
film decoding logic in this facade.

The complete symbol names, signatures, bodies, target qualifiers and source lines
are retained in decfilm-export-inventory.json. inventory_decfilm.go.txt is the
reproduction helper: copy it to a temporary .go file, run it with the pinned
source path as its final argument, then compare its JSON rows with exports in
the saved inventory. The source SHA-256 is included and checked by the existing
485-file verifier.

This does not establish that each target is ported. The manifest keeps the facade
pending for target-by-target Rust/evidence reconciliation. A missing independent
Rust facade is not itself a missing decoder; each underlying implementation and
its observable data must be checked. Do not convert these counts into a parity
percentage or mark the goal complete from this inventory.

## Equipment spawned-piece fallback

The existing equipment-origin oracle derived manifest_fallbacks from native
coverage.ByCause. That checked the published classification, but did not prove
that the actual named fallback counter was triggered. The harness now attaches
a fresh native fallback.Compteur to each of 1,024 placement builds and retains
Rapport independently. All old fixture fields were verified unchanged.

Rust's per-build manifest_fallbacks matches the actual native report: 739 positive
cases, 285 zero cases, 2,581 hits. Existing complete placement, owner, origin,
census-end and coverage comparisons remain intact. Focused test passed (1.08s).
No production change was necessary. The source generator already includes this
harness, so regeneration retains the new counter field.

Final document assembly forwards manifest_fallbacks to
repli_piece_engendree_sans_evenement in replay_document_film.rs. The six captured
complete-document fixtures have no positive hits for this fallback (including
those omitting the coverage field). Thus positive captured end-to-end forwarding
remains a separate gate; the direct native counter comparison does not claim it.

Evidence: fallback-callsite-audit.json; replay-equipment-origin-v41.json.zlib;
logs /private/tmp/halo-equipment-fallback-{native,tests,clippy}.log.

## Source and profile facade targets

| Native export | Rust implementation | Existing comparison |
| --- | --- | --- |
| Decompresser, Inflate | decompress_film_chunk, inflate_film_chunk | native_source_loading: 512 native cases |
| Load, MemoryChunks | FilmSource::load_from, FilmChunkProvider for slices; FilmSource::load convenience API | native_fallible_source_provider: 256 native cases; native_source_loading |
| LoadDir; source.DirSource | FilmSource::load_directory; FilmDirectorySource::open | native_source_directory; native_directory_source_lifecycle |
| Paquets | walk_tagged_film_packets; walk_film_packets for position zero | native_packet_files; native_source_loading |
| CountFilmChunks, ReadFilmChunk | count_film_chunks, read_film_chunk | native_packet_files |
| FilmChunkNumbers, FilmChunkAt | FilmSource::data_chunk_numbers, chunk_by_number | native_chunk_bridge |
| DetectI0LayoutOf | detect_source_i0_layout | native_loaded_i0_selection |
| NewFilmContext, NewFilmContextForMap | NativeFilmContext::new, for_map, with_imposed_layout | native_context_constructor_layout_precedence; native_map_context_eager_profile_and_warning_once; native_source_context_caches_and_registry_identity |
| LoadMapQuantCatalog | load_film_map_catalog, parse_film_map_catalog | native_map_catalog_loading; native_map_catalog_float32_rounding |
| NormalizeMapName | normalize_film_map_name | profile native catalog/name comparisons |

These mappings identify implementations and evidence, not unqualified equivalence
of every API domain or lifecycle. Native source.Film maps to FilmSource, not to
the existing higher-level Rust Film. This is naming reconciliation within the
current port; it does not start the deferred structural Film refactor.

### Fallible provider loading

The facade audit identified a missing public contract: source.Load can consume a
fallible provider, whereas Rust previously required loaded slices or a directory.
FilmChunkProvider and FilmSource::load_from now query the count once and read each
chunk once, in order. Each read is inflated and packet-indexed before the next.
The first provider error terminates loading; ChunkRead retains the source
position and the original typed error as its cause. No partial source is returned.
Slice loading delegates through this same implementation.

The independent pinned Go harness halo_rust_source_provider_test.go.txt produces
source-provider-v41.json.zlib (256 cases). It varies negative/zero/positive counts,
failures before/inside/after the read range, raw/zlib/empty/malformed chunks, and
shorter/longer metadata. Rust compares provider calls, failure text/cause,
metadata, loaded bytes and ordered packet projections. Clearing caller storage
after a successful load verifies Rust ownership. Focused source tests: 3 passed.
The fixture is registered in generate_oracles.py.

Compatibility boundaries remain explicit:

- Counts above i32::MAX return TooManyChunks because FilmPacket's source index is
  signed 32-bit; the native 64-bit host API has a larger integer domain.
- Source positions are usize; negative direct MemoryChunks.Chunk indices have no
  corresponding Rust call. Zero/negative provider counts return Empty.
- Provider errors require Error + Send + Sync + 'static, allowing an owned error
  chain. Borrowed errors are not supported by this API.
- FilmSource owns inflated bytes and copied metadata. It does not preserve an
  original compressed stream as a byte-for-byte archive, nor promise Go slice
  aliasing after load. Film JSON is not a lossless source archive.
- At the initial provider checkpoint directory loading still eagerly read all
  files. The directory lifecycle follow-up below supersedes that implementation;
  identical filesystem error wording remains unclaimed.
- Whole-Film retention, broader callback lifecycle, and complete captured-film
  acceptance remain separate parity gates.

Provider checkpoint validation: full Theater 573 passed, zero failed, 46 ignored
(87.23s); Clippy 41.90s; all-features WASM compilation 17.67s. Formatting, diff,
generator syntax and all 485 pinned hashes pass. Captured tests and a WASM
provider runtime oracle were not run at this checkpoint. Logs:
/private/tmp/halo-provider-{tests,suite,clippy,wasm}.log.

### Directory lifecycle follow-up

FilmDirectorySource now snapshots ordered paths without reading their contents,
implements FilmChunkProvider, and retains each path and underlying io::Error on
failed reads. FilmSource::load_directory aligns metadata to those discovered file
numbers and calls load_from directly. Inflation and indexing therefore occur
between file reads, avoiding the previous eagerly loaded raw-chunk collection.
This is a source-loading parity correction within the existing module.

A 64-case pinned native oracle covers repeated reads, overwrites after discovery,
new files excluded from the snapshot, missing files, matching directories, bounds
errors, and first-error stopping during loading. It records 17 successful loads,
24 missing-file failures, 23 directory-read failures, 288 direct position reads
and 190 loader reads. Rust checks the same bytes, numbers, calls and error
classes, plus path and original I/O error retention. See
../fixtures/source-directory-lifecycle-v41.md for reproduction and scope.

Directory arguments remain literal Rust paths; native filepath.Glob pattern
semantics, enumeration failure behavior and platform-specific I/O error wording
are not reconciled by this change. The provider preserves non-UTF-8 filenames
as operating-system strings, but this oracle uses ASCII names. Filesystem APIs
remain excluded from wasm32. Whole-Film retention and other parity gates remain
open, and the deferred architecture has not begun.

Directory checkpoint validation: four source tests pass (0.28s); full Theater
574 passed, zero failed, 46 ignored (77.64s); Clippy 40.01s; all-features WASM
compilation 6.69s. Formatting, diff, generator syntax and 485 source hashes pass.
Captured-film tests were not rerun. Logs:
/private/tmp/halo-directory-lifecycle-{tests,suite,clippy,wasm}.log.

## Weapon and keyframe-position facade targets

| Native export | Rust implementation | Evidence and boundary |
| --- | --- | --- |
| ScanFilmWeaponShots | scan_weapon_shots_through; source-retaining scan_weapon_shot_reads_through | Explicit 1..=n selection and missing-requested-chunk checks in native_weapon_explicit_ranges; six-film shot corpus. The no-limit convenience call uses the native contiguous prefix. |
| ScanFilmWeaponDamages | scan_weapon_damages_through | Explicit range test plus native_weapon_hit_event_readers and six-film damage/base corpus. WeaponDamageRead additionally retains raw body fields and packet provenance. |
| PairWeaponHits | pair_weapon_hits | native_weapon_hit_pairing_and_distance plus captured statistics. Pairing is derived, not a wire observation. |
| WeaponHitBucketCount | weapon_hit_bucket_count | Derived from the six native distance edges; seven buckets retained in WeaponHitStats. |
| FilmWeaponHitDistance | Composition of build_weapon_hit_tracks, resolve_hit_distance_base and weapon_hit_distance; also used by scan_film_weapon_hits | Native wrapper builds tracks, resolves a base and returns a closure; Rust exposes those steps and stores the tracks/base in FilmWeaponHits. Explicit track ranges and six-film tracks/aggregate evidence cover these operations. Missing map/track evidence does not discard counted hits. |
| DecodeKeyframePositions | scan_keyframe_position_probes | Native pattern and team-split probe, retaining source/comb/vector offsets. 1,024 synthetic cases and six-film corpus; Film.keyframe_position_probes is populated during construction. This output is a spatial heuristic without player identities. |

These are target mappings, not a requirement for identical names or Go closure
representation. Native source-width/host limits and arbitrary caller mutation
contracts remain separate. The keyframe probe's packet walk intentionally differs
from source.Paquets, as it does in the reference: only the first type-2 payload is
used, with its own size/terminator rules. Its time is chunk StartMS, not packet TS.
The mapping does not replace heuristics with recorded facts.

Automatic weapon-hit retention is now part of Film construction; the previous
manual-only gap and validation are recorded in FILM_RETENTION_AUDIT.md. The new
map-constructor corpus check passed all six cases (83.41s), including all 590,758
track samples, source references, distance bases, complete statistics and hit
export roundtrips. The base-constructor check also passed again (21.07s).
Clippy passes (9.34s); no production code changed. See the latest retention audit
checkpoint for precise input and validation scope.

## Stateful frame and march adapter targets

DecodeFrameRecords maps to NativeFilmReader::read_frame_records and the native
frame helpers. The contextual reader retains signed ID settings and native
wrapping/masking; End ignores ID width. Native traversal with a legacy encoding
also wraps IDs, while decode_record_header remains a bounded checked helper.
The 8,640-case native ID oracle, its 1,440 legacy extensions and actual wasm32
execution establish the exercised header/frame domains and explicit cursor
refusals. They do not prove arbitrary signed-cursor continuation.

Native ScanMarchFacts (also reached through the source context) maps to
NativeFilmContext::scan_march_facts and scan_film_march_facts_with_native_config.
Its locator now shares raw contextual policies. MarchCalibration's optional
encoding is a Rust convenience projection; native_config is the authoritative
complete calibrated configuration. Projection failure preserves facts and scores
and reports encoding_error. See march-lazy-policy-validation.json for the
90-case negative/unused-settings oracle and captured validation status.

## Objective/statistics facade reconciliation

All 19 objectives function exports now have an explicit implementation and
native-fixture mapping in public-objectives-targets.json. The inventory is
checked against the pinned facade names; each listed Rust test and fixture
exists. Source/body inspection confirms the wrapper composition and policy
ordering below. This is target reconciliation, not proof of every input or the
remaining parser goal.

| Native target | Rust implementation |
| --- | --- |
| CaptureBurstTimes | scan_source_capture_bursts (objective_source.rs) |
| CountObjectiveFamily | count_objective_family (replay_objective_actions.rs) |
| Extract | extract_source_objective_events_with_roster (objective_source.rs) |
| FlagFilmSignalsFrom | ReplayFlagFilmSignals::from_events (replay_flag_carries.rs) |
| IdentifyNamedEvents | identify_statborg_named_events (statborg_named.rs) |
| IdentifyNamedEventsByRound | identify_statborg_named_events_by_round (statborg_named.rs) |
| NamedEvents | statborg_source_named_events (statborg_source_passes.rs) |
| NamedEventsFrom | statborg_named_events (statborg_named.rs) |
| NetFlagGrabs | count_net_flag_grabs (flag_grabs_net.rs) |
| ObjectiveTypeOf | replay_objective_type (replay_objective_actions.rs) |
| RealRounds | resolve_statborg_rounds(...).real (statborg_rounds.rs) |
| ResolveRoundIdentity | resolve_statborg_round_identity (statborg_identity.rs) |
| RosterFitsStatborg | roster_fits_statborg (statborg_component.rs) |
| SeriesByRound | StatborgComponent::series_by_round / statborg_series_by_round (statborg_component.rs) |
| SeriesTotal | StatborgComponent::series_total / statborg_series_total (statborg_component.rs) |
| SlotIdentityByDeaths | statborg_identity_by_deaths (statborg_identity.rs) |
| SlotIdentityResolved | statborg_source_resolved_identity (statborg_source_passes.rs) |
| StatRecords | scan_source_statborg (statborg_source.rs) |
| StatRecordsCtx | scan_source_statborg_with_diagnostics (statborg_source.rs) |

SeriesByRound filters real rounds, stably orders emissions, selects the strict
or nondecreasing run, then applies the unitary bound. SeriesTotal instead
cumulates rounds, applies the unitary bound, then selects a strict run if
requested. The existing component oracle checks 1,920 queries over 128 cases,
including custom component addresses and team/player selection; it compares
complete round and total maps, not just final scores.

NamedEvents scans source records before mode lookup, including unsupported
modes. The source-pass wrapper preserves source diagnostics before pass-level
diagnostics and additionally retains the source truncation flag. Existing
native fixtures cover 128 source cases, a positive six-event/identified-actor
control, four resolved-identity alternatives, and the whole-frame record-cap
case (33,078 records versus a 33,076 threshold). Ordered records, events, identity
results and diagnostic fields are compared. StatRecordsCtx's context is used by
the native code for logging; Rust exposes the corresponding ordered diagnostics.

Source Extract preserves positional metadata selection and a stateful roster
callback; the 128-case source oracle checks callback order and results. Capture
bursts retain native frame selection and match-time sorting. Family counting
remains case-sensitive; variant classification uses native precedence. The
flag-signal and net-grab mappings retain their complete native result fields.

Focused revalidation: 36 passed, zero failed, one captured test ignored across
statborg, objective, flag, net-grab and capture-burst filters. Logs and exact
counts are in public-objectives-targets.json. No production code or fixtures
changed, so the broader 579-test/WASM/Clippy baseline remains the preceding
checkpoint. No captured objective suite was rerun. Other public targets/types,
remaining native contracts and full-corpus acceptance still need reconciliation.

## Weapon scanner and kill-source target audit

All seven weaponscan/weaponv3 function exports map as follows. The current
native_weapon_patterns test compares complete formula/fire arrays, frame-marker
positions, estimator queries, callback invocation order before deduplication,
custom timestamp ordering and JSON roundtrips. The roster/timing helper test
covers numeric/string identity resolution and first-chunk-wins merging.

| Native export | Rust implementation | Evidence |
| --- | --- | --- |
| FindFramePositions | weapon_frame_marker_positions (weapon_patterns.rs) | native_weapon_patterns |
| ScanFireEventsB5 | scan_weapon_fire_b5 (weapon_patterns.rs) | native_weapon_patterns |
| ScanFormulaA | scan_weapon_formula_a (weapon_patterns.rs) | native_weapon_patterns |
| ScanFormulaANS | scan_weapon_formula_a_nibble (weapon_patterns.rs) | native_weapon_patterns |
| TimestampEstimator | WeaponPatternTimestampEstimator (weapon_patterns.rs) | native_weapon_patterns |
| ResolveBest | resolve_best_player_indices (player_indices.rs) | native_weapon_roster_and_timing_helpers |
| ResolveXuidToPI | resolve_player_indices (player_indices.rs) | native_weapon_roster_and_timing_helpers |

TimestampEstimator here uses A0 7B 42 marker positions and divides the supplied
duration by their count. It is WeaponPatternTimestampEstimator, not the separate
WeaponTimestampEstimator that uses parsed packet timestamps. The latter belongs
to a different native weapon scan path. This distinction matters for callers:
substituting one clock for the other changes observation times.

Kill-source CatalogueSize maps to pinned_kill_damage_catalog().ids.len();
CatalogueProvenance maps to that catalog's provenance. native_damage_catalog_parsing
compares the complete pinned IDs, labels and provenance as well as malformed
catalog cases. ProfilDeDepart maps to kill_replay_starting_native_profile(None),
checked by native_kill_starting_profile_contract. DefaultOptions maps to
KillDecodeOptions::default(), with Carte supplied as the separate map argument.
The original unsigned option-domain gap is superseded by the signed-options
checkpoint below; runtime and source contracts still require separate evidence.

Decode maps to decode_film_kill_sources. This audit identified and corrected its
input-error precedence: source presence and replication packets are checked
before kill-feed selection. The actual native Decode result is now retained in
all 128 existing kill-chunks cases: 10 ErrNoChunk, 24 ErrNoPacket and 94
ErrNoKillFeed. Every old fixture field is unchanged. DecodeError::KillSource
exposes the corresponding typed category and exact native display text.
See ../fixtures/kill-decode-input-errors-v41.md and kill-decode-input-validation.json.

This does not close the entire Decode target: successful complete results,
source selection, option domains and runtime contracts require their own
evidence. The two-film full-result test is being rerun at this checkpoint;
consult the validation record for its terminal state. Source warning/cancellation
behavior is not claimed by the new error fixture. The architecture remains deferred.

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

### Kill-source public result types

All 15 type aliases in the facade's killsource section (including three aliases
from film/types) are mapped in public-kill-types.json. Rust retains FeedTruth as
killer/feed_present on AttributedFilmKill and distributes native Stats across
KillHybridResult and FilmKillSourceResult. assert_native_result reconstructs
both nested native objects and compares the whole result, with ProfilCalibre
compared separately in full; this is stronger than matching only selected counts.
The underlying hybrid, assist, roster, pair and category fixtures supplement the
captured result comparisons. Category names, assignment uniqueness, publication
and path-ratio methods have corresponding Rust implementations.

This is a representation/evidence mapping for produced data, not a proof that
arbitrary hand-constructed negative counters, unknown string Path values or
all native method input domains have been tested. It does not finish the other
facade types, transitive types, source runtime contracts or broader corpus gates.

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

## Remaining facade function mappings

The following entries complete the name-level reconciliation of all 71 function
exports. They identify implementations and existing evidence; they do not close
unverified input domains, transitive methods, callback contracts or corpus gates.

| Native export | Rust implementation | Evidence / boundary |
| --- | --- | --- |
| Lire | fallback_entry | native_fallback_registry_and_counter; unknown names return None |
| NouveauCompteur | FallbackCounter::default | native_fallback_registry_and_counter; Rust receiver cannot be nil |
| Table | fallback_table | native_fallback_registry_and_counter; ordered full catalog |
| Texte | fallback_report_text | native_fallback_registry_and_counter |
| VerifierRegistre | validate_fallback_entries(fallback_table cloned entries) | native_fallback_registry_and_counter; validator sorts native unordered diagnostics |
| DefaultScanFilmOptions | BipedScanOptions::native_defaults, SourceWorldScanOptions and explicit source parameters | Native convenience defaults are separate from Rust Default; signed millisecond isolation versus unsigned microseconds remains a domain distinction. See public-type-rust-map.json ScanFilmOptions. |
| FilmMajorVersionFromHeader | film_major_version_from_header | native_v41_film_key; unknown native zero is Rust None |
| HighlightProfileFromHeader | v41_highlight_profile_from_header | native_v41_film_key; deliberately scoped to v41 |
| LecteurSur | NativeFilmReader::new | native_reader_signed_cursor_sequences and native_reader_cursor_overflow_and_recovery cover signed direct reads; component/frame traversal address limits remain open (READER_CONTRACT.md) |
| NewWorld | FilmWorld::default with registry supplied separately | mutations_generation_views_positions_and_rollback_match_go |
| ParseHighlightEvents | parse_highlight_events | native_highlight_oracle |
| ParseRegistryChunk | parse_registry_chunk | native_registry_parse_outcomes; preserves truncated byte count |
| ProfilDeBalayageParDefaut | NativeScanProfile::default | native_scan_profile_replacement_restoration_and_map_aliases |
| ReadPlayerTable | decode_player_table | native_player_tables_and_failures and native_player_table_corpus |
| RegistryFingerprint | FilmRegistry::fingerprint | fingerprint_edges_match_native; mutable standalone registry recomputes, context caches parsed identity |
| ScanBipedCreations | scan_biped_creations | native_creation_source_and_sparse_bands; accepted native creations always have a participant index |
| ScanBipedPositions | scan_source_quantized_positions / scan_source_world_positions | native_loaded_quantized_positions; source selection and companion tests referenced in public-type-rust-map.json |
| UnknownBuildExpvarPairs | unknown_build_metric_pairs | unknown_build_publication_matches_native |
| LargeurIndexDePlage | position_region_index_width | native_precision_law |

The function-valued variable BuildBipedTracks maps to build_weapon_hit_tracks;
see native_weapon_track_ranges and the six-film track comparisons above.
Its placement under variables in Go does not introduce a second algorithm.

## Public constant snapshot

`halo_rust_public_constants_test.go.txt` emits all 41 exported constants, both
error sentinel messages and the complete 35-entry KnownWeaponHigh32 map from the
pinned native package. `public-constants-v41.json` is registered in the ordinary
source/publication oracle generator. The function-valued BuildBipedTracks is
mapped above rather than serialized.

`native_public_constant_contracts` compares 13 production scalar constants,
NoKillFeed display, both serialized KillReadPath variants, six category-name
lookups using native category values, and three supported stat-table entries.
The complete weapon-family map is independently compared by
native_weapon_roster_and_timing_helpers. This is not a claim that all exported
strings have a corresponding Rust constant: several appear directly in typed
output construction. Those outputs have their existing kill/objective fixtures.

Known representation differences: FilmMajorVersionUnknown is Option absence;
ErrUnknownMapBounds maps to typed MissingCatalog/UnknownMap errors, but Rust
currently uses English display text while the reference uses French sentinel
text and Go-quoted requested names. Classification and retained map name are
covered by native_map_catalog_loading; exact display equivalence is not covered
or claimed. The source-fidelity and runtime acceptance gates remain open.

## Additional types exposed by function signatures

`inventory_signature_types.py` identifies 26 named types used by the 71 public
function signatures but absent from the 50 alias targets. The retained AST
inventory resolves their 156 direct fields (including private/embedded fields).
The three domain types outside film are included, with all 23 defining source
files hashed in public-signature-type-source-hashes.json. This is not a closed
transitive type graph: nested named fields and promoted methods still require
following their definitions. Standard-library Context and Duration contracts
are recorded separately. BuildBipedTracks remains the separately mapped
function-valued variable.

| Native signature type | Rust representation |
| --- | --- |
| damagetag.Provenance | KillDamageProvenance, including both catalog dates/counts |
| domain.ObjectiveEvent | ExtractedObjectiveEvent and ObjectiveEventPlayer; nullable values stay Option |
| fallback.Repli | FallbackEntry; ConditionEffective/Paquet map to effective_condition/source_package |
| grammar.ExpvarPair | Ordered (String, i64) metric pairs |
| grammar.FilmPacket | FilmPacket plus its ordinal in the chunk's native packet sequence; payload source offsets retained |
| grammar.FrameRecord | EntityRecord header, archetype, trace fields/spans, stop diagnostics and source ranges |
| grammar.I0LayoutReport | I0LayoutReport |
| grammar.Lecteur | NativeFilmReader; profile/context/observer setters and signed direct reads; ReadQuantizedVec3 now has a stateful method |
| grammar.PlayerTableReport | PlayerTableReport |
| grammar.WeaponShot | WeaponShot |
| highlightevent.HighlightEvent | NativeHighlightEvent |
| objectives.FlagFilmSignals | ReplayFlagFilmSignals, including is_flag_film |
| objectives.FlagGrabsNetResult | NetFlagGrabs |
| objectives.IdentityStats | StatborgIdentityStats |
| objectives.Roster | FnMut roster callback returning Option team; existing source-selection tests exercise callback ordering |
| objectives.TeamControl | ObjectiveTeamControl |
| playerposition.PlayerPosition | KeyframePositionProbe time/vector/team with additional source offsets |
| profile.FilmIdentity | FilmIdentity; corruption_checks retains ControleDeCorruption |
| profile.HighlightProfile | FilmHighlightProfile; major_version Option distinguishes readable v41 from absent/unreadable header, non-v41 readable headers are rejected |
| source.Source | FilmChunkProvider fallible provider contract |
| types.BipedCreationStats | BipedCreationStats; complete alternate-word histogram projects OtherWord/OtherWordCount through most_common_other |
| types.FlagTrack | FlagGrabTrack with FlagGrabSpan |
| types.Packet | TaggedFilmPacket with signed chunk tag and borrowed payload |
| types.PlayerSlot | PlayerTableSlot and PlayerTableShorts |
| weaponscan.FireEvent | WeaponFireProbe, including FilmIndex5, B5, nullable HitLikely and chunk index |
| weaponscan.FormulaAResult | WeaponFormulaProbe |

These mappings describe field retention and API composition, not arbitrary
caller-input equivalence. Address-sized counts/source offsets and narrowed wire
fields do not imply support for every signed native struct value. The new
stateful quantized-vector method has direct native coverage documented in
../fixtures/reader-quantization-v41.md; other rows retain the field-level evidence
already cited in this audit's source, objective, weapon and type sections.

## Nested declaration graph and provenance method

TRANSITIVE_TYPES.md now records a deterministic traversal from all public type
aliases and function signatures through fields and exported method signatures.
Its 268 nodes include 71 synthetic function signatures; 71 additional native
declarations require reconciliation against existing Rust evidence. All 84 source
hashes match pinned git objects. Dynamic `any` payloads and external behavior
remain explicit limits; this inventory does not establish semantic parity.

The traversal surfaced the already documented Provenance.String gap.
KillSourceProvenance now implements Display, with a 48-case actual-native fixture
covering both published paths, known and opaque origins, and multiplicity
independence. See ../fixtures/kill-provenance-v41.md. The six ability storage
contracts are also reconciled in TYPE_CONTRACTS.md; their existing comparisons
must not be confused with completion of all ability runtime behavior.
