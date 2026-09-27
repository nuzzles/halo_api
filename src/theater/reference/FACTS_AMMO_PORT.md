# Native facts-cache ammunition codec

Reference: replay/filmfacts_codec.go encodeAmmo/decodeAmmo and the four/eight-byte
little-endian helpers in replay/filmfacts_flux.go at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

encode_facts_ammo/decode_facts_ammo use the existing KeyframeSlotAmmo storage.
Magazine, reserve and gauge each have their own presence flag. Present zero is
not absence. Overheat and flags are preserved as u32, including the native
narrowing of decoded varints. Gauge values retain all 64 bits, including signed
zero, subnormal values, infinities and NaN payloads. No f32 conversion occurs.

The decoder returns native partial field state when a read fails; callers must
inspect NativeFactsReader::error before accepting a complete record. In particular,
a present magazine/reserve flag followed by a failed varint retains Some(0),
while a truncated gauge does not create a gauge value. Reader errors and offsets
are compared before and after a second decode from the same reader.

The native cache has no gauge-quantum field. Decoding leaves gauge_quantum None;
it must not manufacture an original film quantum from the cached double. Encoding
uses the stored gauge double, matching the native struct's field.

The independent oracle contains 256 source slots covering all eight presence
combinations, zero-valued ammunition, wide flags and gauge edge patterns. Every
source is encoded, then decoded whole, at every truncation point and with each
byte independently flipped: 11,224 cases. Tests compare encoded bytes, all retained
fields, exact gauge bits, both errors and both cursor boundaries. The same fixture
runs through public APIs in the actual WASM harness. See facts-ammo-validation.json.

The transport's eight-byte little-endian use is now implemented and verified.
Other shared section codecs (positions, tracks, creations, world-keyframe state)
and complete cache framing/versioning/assembly remain unported. This is derived
cache compatibility, not a claim of recording re-encoding or full v41 parity.
The Film/ResolvedFilm/playback architecture remains deferred.
