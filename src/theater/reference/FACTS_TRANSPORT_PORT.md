# Derived FilmFacts cache transport

Pinned source: replay/filmfacts_flux.go at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

NativeFactsWriter and NativeFactsReader implement the binary transport used by
LevelUp's derived facts cache. These bytes are not native Theater recording
bytes. This work does not replace Film or start the deferred resolved/playback
architecture, and does not yet implement complete FilmFacts cache containers.

The transport preserves unsigned/signed varints, byte values, exact f32 bits,
length-prefixed arbitrary string bytes and native boolean decoding (only byte 1
is true). Section reads borrow the input slice; a successful empty slice remains
distinct from an error. Failed reads retain the first error and cursor, except
length/count prefixes already consumed before validating the payload. Counts
use the native signed 64-bit domain and minimum-cost guard, including on WASM.

The independent native fixture records 1,120 sequences: 8,206 writer calls and
9,692 read observations, including truncated/mutated inputs, overflowing and
noncanonical varints, exhausted input, signed counts and non-UTF-8 strings.
Writer bytes, reader outputs, f32 bits, source offsets and exact native errors
are compared. Two corrupt native string lengths cause Go slice panics: Rust
instead latches an explicit error at the same source offset. This deliberate
safe refusal is not claimed to reproduce a panic or a decoded string.

The eight-byte gauge use is now implemented; see FACTS_AMMO_PORT.md.
The remaining section schemas, cache file framing/version checks, encoding and
reading FilmFacts/FilmFactsFile, and BuildFromFacts integration remain open.
Existing captured-film parity evidence is separate from this derivative-cache
transport evidence. See facts-transport-validation.json for completed checks.
