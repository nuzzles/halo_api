# Native facts-cache projectile tracks

Reference: replay/filmfacts_codec.go encodeTracks/decodeTracks at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

FactsProjectileTrack/FactsProjectileSample retain slot, generation, ordered samples,
wrapping delta timestamps, all three f32 bit patterns, rest flags and signed 64-bit
chunk numbers. The cache's signed machine-word chunk domain exceeds the i32 chunk
field of existing WorldObjectSample. A separate cache DTO preserves it on host and
WASM; conversion from existing WorldObjectTrack widens safely. Decoding does not
narrow or fabricate a native-film source span from a cached sample.

The encoder preserves track and sample order, resets the timestamp delta for each
track and uses wrapping subtraction. Decreasing times are not sorted or repaired.
The decoder uses native count(3)/count(2) guards and retains partial tracks and
samples when a later field fails. Point lists with no decoded samples remain None,
matching the native nil list. The top-level decoded list is allocated empty when
there are no tracks. Inspect the reader error before accepting a complete section.

The independent oracle uses 96 source arrays with empty/populated tracks, random
identities, decreasing timestamps, f32 edge patterns and chunk values including
-1, 2^32, i64::MIN and i64::MAX. Every whole stream, truncation point and single-byte
flip yields 23,464 cases. Tests compare writer bytes, every retained field and float
bit, both decoded passes, errors and cursor boundaries. See facts-tracks-validation.json.

This adds the shared cached-track codec used by projectiles and world objects.
Position sections, creation records/stats, keyframe maps, world scan composition
and full facts-cache containers remain pending. A 64-case native probe confirms that the keyframe decoder's count(2) guard
rejects 54 standalone/short-tail cases with one-byte timestamp varints. A one-point
section fails by itself and succeeds with one trailing byte. See
facts-keyframe-count-audit.json; Rust now preserves this behavior; see FACTS_KEYFRAMES_PORT.md.

This is native derivative-cache compatibility. Full v41 parsing parity and the
requested Film/ResolvedFilm/playback architecture are not completed by this codec;
the architecture remains deferred.
