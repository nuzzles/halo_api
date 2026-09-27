# Datum rejection and partial-film retention

Pinned reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
`TestHaloRustDatumLengths` calls actual LireBlocDeDatums for zero-filled payload
lengths 0 through 128. It accepts 62 bytes (one slot) and 104 bytes (two slots),
and rejects the other 127. This is length/refusal coverage; existing datum
oracles separately compare nonzero fields and captured full tables.

The standalone Rust table decoder keeps returning an error for rejected bodies.
Film assembly previously propagated that error, discarding the entire result.
It now retains DatumDecodeFailure with packet identity, raw payload and a Rust
error message. ReplicationStream retains InvalidDatums in packet order. These
are rejected bodies, not decoded tables or unknown packet families. Typed native reasons now distinguish NoEntries, AboveCapacity and Misaligned,
including byte/bit/slot quantities. Their rendered French messages are compared
exactly with the pinned native errors. Older exports without native_error retain
None rather than fabricating a typed reason.

The integration test interleaves all 129 datum candidates with empty valid
replication frames, then uses the public Film constructor with an encoding.
It checks two valid tables, 127 source-addressed failures, all 130 surrounding
frames, native acceptance/slot counts, raw payloads and whole-Film JSON roundtrip.
Disabling coverage retention must not remove these failure records. Invalid
packet envelopes and version/bootstrap errors still fail construction normally.

This changes packet-body failure handling within the current film module. It
neither starts the deferred canonical Film refactor nor claims native decoding
of malformed bytes. Generator registration is in generate_oracles.py.

## Capacity and typed-error extension

Ten additional lengths cover two bytes on either side of the 8,191- and
8,192-slot table sizes. The direct-decoder test checks all 139 native outcomes,
exact error text, every error category and typed-error JSON roundtrips. All
pre-existing length/error/slot expectations were verified unchanged. The public
Film interleaving test stays on the original 129 small lengths and additionally
checks the retained native reasons and messages. Capacity cases do not require
expanding that synthetic Film with megabytes of repeated payloads.

## Recovered-binding continuity

The captured-keyframe regression inserts one refused datum packet from each
native error category between the captured keyframe and a later entity update.
All 123 recovered anchors, the complete final binding table, the original
keyframe result, and the later frame and unparsed tail remain unchanged. The
later packet's source offset shifts by exactly the inserted envelope/body size.
Each refusal retains its raw payload, source offsets and native error message;
the complete replication stream survives a JSON roundtrip.

This is a synthetic insertion into a captured-keyframe sequence, not evidence
that a captured gameplay film contains these corruptions. It verifies Rust's
packet-local refusal contract; the native oracle supplies the refusal reasons,
not an independent native damaged-stream continuation result.
