# Native march context and configuration retention

The native source contract is ScanMarchFacts in internal/grammar/object_deaths.go,
calibrateFrameConfig in object_deaths_calibrate.go, and marchRecordsOf/marchStartOf
in object_deaths_march.go at pin 43a01721e8a02c87c955e175936f0ccf8dd97a81.

NativeFilmContext.scan_march_facts uses the cached registry and numbered source
prefix directly. No-delta scans return before resolving the scan configuration.
With deltas, the scanner uses scan_frame, which deliberately has no observer.
Each width trial walks packets against a fresh chronological timeline. The
per-packet world is restored by walking a copy; successful temporary New/Delete
records affect later reads in that packet, not later packets or other trials.

The retained NativeFrameMetadata contains all frame scalar settings and the
entire native profile, including maps, keyframe framing, MPP widths, movement
range/quantum/descriptors and grammar switches. The snapshot detaches allocated
maps from later live mutations; absent maps remain distinct from allocated empty
maps. This is parser configuration, not recorded wire data. Callback code is not
serialized; observer presence is explicit. The context scanner excludes observers.

FilmMarchFacts.native_config is optional. Old JSON and legacy encoding-only
scans preserve None. A projected FrameEncoding cannot recover every native
profile field, so those callers do not manufacture a complete configuration.
The map-aware Film constructor now passes a full native profile into the native
march scan. KillCalibration.native_profile is initialized from the native starting
settings and updated with the selected handle width during calibration. Replay
installs that inherited profile first, then the map world-object descriptor, and
resolves corruption from the film. Without kill calibration it keeps the native
map-context defaults. The complete chosen settings reach Film.native_march_facts.
Integrated validation passed: four no-inheritance Film constructors (75.51s),
all six inherited-profile films (88.89s), and every new native march configuration
field, fact and counter. The six-film test also checks complete raw kill results
and calibrated native profiles, plus all existing document fields. Earlier
source-context checks below remain complementary coverage.

Current full suite: 549 passed, 46 ignored (94.32s); Clippy, WASM, formatting,
diff checks and all 485 pinned source hashes pass. Logs are listed in PORT_STATUS.md.

Validation uses the existing native oracle output, not expectations generated
by Rust. All 256 march-loaded-v41 cases pass (17.54s), with complete facts,
counters, scores and full native Config comparison. The four captured films in
march-corpus-v41 pass (17.64s, release), covering the source-context entry point,
all Config fields, deaths/occupancy/coverage, calibration and JSON roundtrips.
The detached-map test checks later source edits do not alter saved configuration.
A subsequent v41-only guard test passes (0.02s): no-delta v41 input preserves
absent configuration without converting unused invalid widths, and v75 input
is rejected before the empty-delta return.
Full Theater suite: 548 passed, 46 ignored (94.66s), before the version-only
guard; the subsequent focused guard regression also passes. Final clippy
(9.88s), WASM (14.45s including build-lock wait), formatting and diff checks pass.

This does not implement the deferred Film/ResolvedFilm/playback architecture,
prove arbitrary observer mutation behavior during march calibration, or establish
independently annotated gameplay semantics.
