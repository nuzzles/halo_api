# Complete native facts-file container

`facts_file.rs` composes LEVELUPFILMFACTS codec 1 / schema 3, the existing header,
REPLAYINPUTS25 blob, mode guards and all four JSON payloads. This is native derived
cache parity, not canonical recording fidelity or byte-for-byte film re-encoding.

Decode checks header versions and the cooking key. Revision freshness remains an
explicit `FactsFileHeader::usable` caller decision. Sections are processed in order;
unknown IDs are skipped, absent sections are permitted, and duplicate JSON sections
update persistent native backing state. Section 1 replaces blob facts and guards;
it permits trailing bytes after guards, exactly as the reference does. Every failed
file decode rejects the assembled value. Negative body offsets which cause native
indexing panics are safely refused. Typed header/cooking/blob errors are retained.

Encode writes all five sections in native order. A failed embedded blob or JSON
encoder returns no file bytes. This differs deliberately from the blob API, whose
native contract exposes partial bytes on error. Mode guards have a separate field
in NativeFilmFactsFile because the native blob excludes those FilmInputs fields.

The native full-file oracle covers 2,919 cases: all prefixes of a valid file and
its input section, header-only files, mismatched cooking keys, known/unknown IDs,
invalid JSON, repeated JSON backing state, duplicate input sections, permitted
trailing input bytes, malformed section lengths, and header offset overflow.
Six writer failure controls check NaN/infinities in blob and killsource JSON and
native nil-byte results. Exact normalized bytes complement the independently
validated component decoders; this is not an independently annotated action suite.

See `facts-file-validation.json` for actual platform results. Full parser parity
still needs cache/scan DTO conversion, BuildFromFacts, integration and final source,
field, runtime, export and captured-film reconciliation. The architectural phase
in NEXT_PHASE.md remains deferred.
