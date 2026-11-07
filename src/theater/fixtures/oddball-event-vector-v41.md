# Captured event-vector continuation

`oddball-event-vector-v41.bin` is the unchanged 835-byte packet payload at
decompressed chunk 52, byte 754329 of the bundled Oddball match
`4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73`.
SHA-256: `611e85069e9fe86f4a24663b8854b6fd8beed68563b558e4c9e27818ad394cf8`.

The contextual event list starts at bit 1. Its codes are
`82,15,82,82,82,82,82,82,82,82,15,15`; its terminator is bit 1998.
The eight property-7 vectors use a default-region guard, a one-bit region index,
and 18/18/15-bit axes. The first vector occupies bits 304..356 and contains
`97243,129662,16630`. No dynamic-position leading precision flag or finite tail
is present. Reading either would break the following event boundaries.

With the supplied Recharge context, sequential entity decoding begins at 1999:
biped NEW `1073742562` ends at 3688 (matching player/control index 5), weapon NEW
`1073745545` ends at 4115, and a previously bound DELTA ends at 4148, followed by
the entity terminator. The CLI integration test checks the recovered biped's
source, generation and life timestamp, 1034948531 microseconds.

Reference layout sources at LevelUp `d61443ef59268ad734355db8e9974f68db5ca6d0`:
`.ai/V7.5/film_re/NOTE_PLAYERGAMEEVENT_0XE9_2026-09-01.md` identifies property 7
as runtime-dependent; `grammar/components_position_i0.go` documents the
absolute payload's region/axes grammar (under
`apps/go-api/internal/games/halo_infinite/film/internal/`). The quantized event
branch is a contextual interpretation validated by this continuation, not a
claim that the reference implements it or that all event-vector modes are known.
Default canonical parsing still stops at bit 304 without that context.

The historical spawn read agrees at this source but is regression evidence,
not independent video validation. Other regions, default-region vectors and
unknown runtime modes remain unsupported. Tests retain the complete captured
packet, test every byte-truncated event prefix, and mutate both region guards.
