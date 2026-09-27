# Shared cached positions and direction fields

Reference: replay/filmfacts_codec.go encode/decodePositionSection and
replay/filmfacts_directions.go at 43a01721e8a02c87c955e175936f0ccf8dd97a81.

FactsBipedPosition/FactsPositionDirections implement the shared biped/vehicle
cache projection. The slot table preserves first-occurrence order. Timestamp
and per-slot position deltas wrap in the native integer domains. Position
quanta, primary aim, directions, velocity, chassis roll/mode/default-direction,
secondary aim, flags and health/shield values retain their native presence gates.
World coordinates are recomputed from the supplied f32 map bounds and I0 widths
using the native f64 midpoint calculation and final f32 conversion.

The direction decoder can update an existing value. It resets the flag byte's
booleans, but leaves gated raw values unchanged when the corresponding gate is
absent, matching native mutation semantics. FwdMode is masked to two bits only
when roll is present. MaskBits is not encoded and remains untouched by this
standalone decoder; a newly decoded position starts it at zero.

The native cache intentionally omits Chunk/PacketIndex, MaskBits, body vitality
quantum/flags and shield regeneration/auxiliary fields. It also omits values
behind false presence flags, including quanta when HasWorld is false. The codec
must not be used as lossless storage for the richer film decoder's output. The
cache DTO models the stored projection and does not fabricate omitted data.

The independent native oracle contains 8,122 position cases and 17,709 seeded
direction cases, including full streams, every truncation point, all direction
flag bytes and mode combinations, interleaved slots, arbitrary raw quanta, signed
zero/nonfinite float representations, and invalid slot-table controls. It compares
encoded bytes, retained values, computed world float bits, errors and cursors.
The malformed empty-slot case also checks the native slot diagnostic overwriting
an earlier varint error. Negative slot indices are safely refused instead of
reproducing native index panics, at the same consumed offset.

See facts-positions-validation.json. Shared cache sections are implemented, but
remaining channel codecs, complete cache files/version gates and integration
remain open. Full v41 parser parity is incomplete. The Film/ResolvedFilm/playback
architecture remains deferred.
