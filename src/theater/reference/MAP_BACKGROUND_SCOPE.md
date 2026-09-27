# Native map-background scope and calibration contract

Reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81,
replay/map_background.go and replay/map_background_index.go.

## Implemented calibration

MapBackgroundCalibration maps all six fields to map_background_calibration.rs:
f64 meters_per_pixel/origin_x/origin_y, i64 width_px/height_px, and the convention
string. Dimensions do not narrow to a WASM pointer width.

MondeVersPixel maps to world_to_pixel. A nonpositive scale returns (0,0,false).
Otherwise it subtracts and divides in the native order, converts toward zero,
and tests the integer coordinates against the declared dimensions. It does not
subtract half a pixel and does not floor negative fractional coordinates. The
conversion preserves the pinned arm64 behavior for overflow, infinity and NaN;
the actual WASM test checks the same i64 results.

The 3,465-case oracle calls the production method and json.Marshal on the native
calibration. It includes subnormals, signed zero, nonfinite values, signed-width
extremes, dimensions beyond 32 bits and 90 explicit image-edge cases. Finite
calibrations retain all fields and float bits through canonical JSON; nonfinite
calibrations remain usable in memory but reject serialization, as native JSON
does. No null substitution is used to hide a nonfinite coefficient.

## Complete sidecar loading

MapBackground and MapBackgroundStats retain all native metadata fields, nullable
collections, optional statistics, provenance and image names. parse_map_background
implements native input semantics; load_map_background adds host filesystem I/O.
Use these entry points for external sidecars: ordinary DTO deserialization is for
stored/canonical values, not the full native JSON update contract.

MapBackgroundTime retains Unix seconds, nanoseconds and numeric UTC offset. Native
zero time is year 1. The reader matches one-digit hours, comma fractions, truncation
to nanoseconds, null updates and duplicate fields. Escaped timestamp strings are
rejected, as in native time.Time.UnmarshalJSON. Some offsets accepted on input are
refused on publication by both implementations. Host timezone names are not source
data. Nonfinite statistics reject JSON publication.

The pinned native oracle covers 1,031 inputs: 365 accepted, 666 rejected, and three
accepted timestamps that refuse publication. It includes all 109 published JSON
assets, whose hashes are independently checked against pinned Git blobs, plus
schema/type boundaries, duplicate and null updates, timestamp syntax mutations,
and signed-zero behavior. Integer -0 is accepted; -0.0 and -0e0 remain floating
spellings and are rejected for integer fields. Floating negative zero is retained.
The oracle compares all retained fields, float bits and publication/refusal.
See map-background-validation.json and background-asset-hashes.json.

## Background identity index and cache

- MapBackgroundIndex: pure identity lookup, fallback, ambiguity exclusion and
  generic-module refusal are implemented; see BACKGROUND_INDEX_PORT.md.
  Directory loading and cache behavior are implemented and independently tested;
  see BACKGROUND_DIRECTORY_PORT.md.
  NormalizeMapIdentity is not interchangeable with the map-quantization name
  normalizer: explicit Heavies/Ranked variants can have different published assets.

These are external asset contracts. Full sidecar loading does not establish
background index/cache compatibility, browser rendering or all-data film parity. The Film/ResolvedFilm/playback architecture remains deferred.
