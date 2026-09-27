# Geometry source scope audit

Pinned source: replay/geometry.go at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

The source mixes external asset ingestion, replay bounds and distance helpers.
The partial-v41 manifest entry retains the remaining 3D distance audit; output
agreement alone does not establish CSV ingestion parity.

## Bounds and publication

- boundsOf, axisGuard/rejects, guardOf, axisValues, sortFloats, rawBounds and
  countRejected map to replay_bounds.rs::replay_bounds and its helpers. The
  constants 200 samples, 12 spreads and 0.5 minimum spread are retained.
- geometryBounds maps to replay_document_types.rs::replay_geometry_bounds:
  centers only, XY, absent for no objects. minf/maxf use ordered comparisons,
  including the native choice of the second argument when comparisons fail.
- Existing fixtures: replay-bounds-v41.json.zlib and document-content-v41.json.
  Focused checks passed: native_bounds_and_outlier_counts (2.04s) and
  native_document_content_schema (0.05s); logs are
  /private/tmp/halo-geometry-{bounds,content}-audit.log.
- FilmReplayDocumentOptions.geometry accepts prepared ReplayMapObject values.
  The new loader's objects can be passed directly to this existing input.

## External ingestion mappings

- LoadGeometry -> replay_geometry_loader.rs::load_replay_geometry. The in-memory
  companion is parse_replay_geometry_csv. Native MapObjectsFile/ObjectTypesFile
  map to REPLAY_MAP_OBJECTS_FILE/REPLAY_OBJECT_TYPES_FILE.
- loadTypeExtents and geomMeasured -> extents and its exact b"ok" predicate.
- readCSV and field -> replay_geometry_csv.rs::read_geometry_csv and
  GeometryCsv::field. Bytes are retained until named-field conversion; invalid
  UTF-8 is not normalized. CSV errors retain category, stage and byte positions.
- parseF32 -> replay_geometry_float.rs::geometry_float. Decimal conversion uses
  Rust's correctly rounded f32 parser after native syntax validation. Hex values
  are rounded directly to f32 with full-tail sticky bits, avoiding double rounding.
- round2 at publication retains native f64 multiplication, half-away rounding,
  division and final f32 conversion.

The catalog is read and parsed before opening the map asset. Missing map files
are accepted only after catalog success; empty files and other I/O errors fail.
The loader preserves accepted order, last duplicate headers/types, filtering and
unknown-type counts. Nonfinite values and signed zeros survive in memory; the
LoadedReplayGeometry serializer rejects nonfinite values rather than writing null.
Native error display text is not emulated, and native nil slices map to empty
vectors. Directory-read I/O failures retain file and OS error rather than the
native CSV header wrapper. See ../fixtures/geometry-loader-v41.md for the native
oracle matrix and geometry-loader-validation.json for current check results.

## Distance call audit

All four native planDist call sites now use replay_plan_distance:

| Native source | Rust source |
| --- | --- |
| grenades.go | replay_grenades.rs |
| vehicle_rides.go | replay_vehicle_rides.rs |
| projectiles.go | replay_projectiles.rs |
| vehicle_relays.go | replay_vehicle_relays.rs |

The formula subtracts in f32, promotes to f64, handles infinity before NaN,
scales by the larger absolute component and explicitly fuses the square-plus-one
step before square root. A native-backed expression probe found the original
std hypot differed in 1,509/4,096 finite pairs; explicit fusion matched all.
The 4,661-case bit oracle and four complete grenade attribution cases are
recorded in ../fixtures/geometry-distance-v41.md. Current correction checks are
in geometry-distance-validation.json.

Eight native dist3 calls map to replay_t0.rs, pickup_origin.rs (two calls share
its origin-match predicate), replay_equipment_origin.rs, ground_weapon_pads.rs,
ground_weapon_lifetimes.rs (two calls), and replay_ground_items.rs. Each subtracts
in f32 before promotion. The square/sum expression matches all 4,096 finite
probe rows. Existing native-backed caller tests cover kickoff detection, pickup
origin, equipment origin, ground-pad clustering, lifetime bounds and ground-item
publication; all passed in the 636-test planar-correction checkpoint.

## Resolution of the derived NaN audit

The earlier two opposite-sign NaN results were initially described as a remaining
port gap. That diagnosis was too strong. verify_geometry_optimization.py now runs
the SAME pinned source and inputs with default optimization and with package-local
`-N -l`. The native parser itself changes the two result bit patterns. All 4,661
planar results, every non-NaN 3D result, and every NaN classification remain equal.
The optimized build also exactly matches the retained native fixture.

See geometry-optimization-audit.json for both bit patterns, Go version and input
rows 4185/4194. This is evidence of compiler-dependent derived arithmetic, not
lost recording data. Existing 3D callers produce the same mathematical result
class; forcing one incidental NaN payload would manufacture a stronger contract
than this native implementation provides. No recording field, source float bits,
opaque payload or quantized value is normalized by this conclusion. Native APIs
that explicitly construct a NaN bit pattern (such as planDist/math.Hypot) remain
covered by their exact-bit tests.

Reproduce with the retained script, the pinned checkout, `--go` pointing to the
reference Go runtime, and `--report` naming an output JSON file. The script verifies
all 485 production hashes, restores the native harness, and never edits fixtures.

Geometry ingestion, bounds, publication and all twelve distance call mappings
are now accounted for. The manifest's ported-v41 status is a source-file contract
assessment, not a claim of full film parity. External catalogs are not recorded
film bytes; the architecture remains deferred in NEXT_PHASE.md.
