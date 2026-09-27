# Native clock origin retained by Film

LevelUp's FilmClockOriginUS reads the first packet of chunk 1. It is independent
of the minimum nonzero timestamp that the legacy Film samples use as their origin.
Film.native_clock_origin now retains ReplayClockOriginRead:

- Read(FilmPacket): the exact first native packet, including a recorded zero
  timestamp, chunk identity, header bytes and payload range.
- MissingChunk: no chunk numbered 1 was supplied.
- MissingPacket: that first chunk-1 entry has no readable native packet.

None on Film means an older export did not retain this observation. The existing
scan_replay_clock_origin API delegates to the same read and preserves its previous
Result and error messages. Duplicate chunk numbers still select the first supplied
entry. The loaded-source clock API remains unchanged.

The Film replay-player builder uses the retained result when present. Older Film
exports retain the original source-read fallback. Identity evidence retains native
nonfatal failure (clock value zero plus error), distinct from a successful zero.

The constructor regression covers a native origin later than the minimum timestamp,
recorded zero, absent chunk and empty chunk. It disables packet-list retention,
checks the retained packet's source boundary, roundtrips complete Film JSON, and
verifies missing-field compatibility. These source bytes are synthetic controlled
inputs. The independent 1,024-case native replay-evidence oracle is also exercised
through the retained-clock path, including 174 native clock failures. The six-film
pre-publication input comparison checks FilmClockOriginUS against the original
native scan output rather than the final replay's translated timeline.
