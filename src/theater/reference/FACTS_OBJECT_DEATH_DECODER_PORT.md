# Object-death facts cache decoder

Reference: `replay/filmfacts_mortsdobjet.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`decode_facts_object_deaths` reads the native length-prefixed JSON projection.
It retains every supported death, statistic and frame/profile field with native
integer widths and exact decoded float32 bits. Empty charges and failed charges
return the native zero projection, not production-default scan settings. Framing
errors remain sticky; JSON errors replace the reader error with the native prefix.

The decoder uses an ordered syntax representation and schema-directed updates.
It validates the entire JSON syntax before type conversion, preserving native
error precedence even when a type mismatch precedes malformed unknown fields.
Repeated struct fields merge; null leaves scalar/struct values unchanged, clears
maps/slices, and map values start from zero. Slice backing cells survive shorter
repeated fields until decoding completes. Empty slices clear backing storage.
Fixed arrays ignore excess elements and zero missing trailing elements. Native
case-insensitive field matching, UTF-8/surrogate substitution, numeric map-key
conversion, first type errors, signed zero and numeric ranges are retained.

The syntax parser stores nodes and nesting state in flat vectors. Both traversal
and cleanup avoid recursion through untrusted JSON. It accepts Go's 10,000-container
limit and rejects the next container with the native diagnostic. Unknown JSON
fields are ignored as in the native cache reader; this is not a canonical Film
representation or a promise to preserve opaque native recording bytes here.

## Evidence

- 6,450 typed/framing cases across 219 charges, including 64 completely populated
  native charges generated from the actual Go types, duplicate-field/backing
  controls, missing/null fields, scalar limits and map-key conversion.
- 3,783 independent syntax cases: every byte in fourteen syntax contexts, inner
  JSON truncations, syntax/type error precedence and nesting boundary controls.
- Assertions compare complete values, all seven profile float bit patterns,
  error strings and the final source cursor with the pinned native decoder.
- Writer coverage remains separate: 2,304 complete writer cases plus 68,608
  float32 formatting cases. See FACTS_OBJECT_DEATH_WRITER_PORT.md.

The codec's DTO preserves native int/uint domains on WASM. Conversion to existing
scan/replay types and full cache assembly remain integration work; neither this
section nor passing reference cases completes all-data v41 parser parity. The
Film/ResolvedFilm/playback architecture remains deferred.
