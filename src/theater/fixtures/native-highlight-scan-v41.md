# Native highlight scan integration

Reference commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Source: `internal/grammar/highlight_events.go`, especially scanHighlightEvents,
parseEventAtBit and decodeEventBytes.

NativeFilmChunk.highlight_scan runs on already decompressed footer chunks. The
reference's bitwise XUID/prefix search and 20,000-bit tail window are independent
of the existing summary packet reader's captured fixed offset. The two results
remain separate. Neither candidate scan establishes a canonical record partition.
No roster association, inferred actions, sorting by timestamp, or playback state
is constructed.

The shared scan retains each admitted identity candidate, its source bit, prefix,
XUID and window end, including candidates that yield no event. Ordered tail
attempts include too-early markers (no 60-byte body), rejected event layouts and
the first accepted layout. Full 60-byte bodies preserve raw UTF16 units, string
tails, unrecognized hints and unused bytes. All coordinates are chunk-relative.
The existing parse_highlight_events API shares the scan with tracing disabled
and retains its original plaintext/zlib behavior. The new native reader does not
attempt to decompress its already-decompressed source again.

The existing independent highlights-v41 oracle verifies identical selected events
across alignments and transport cases. New checks independently extract source
bits for identities, markers and every retained body byte, and round-trip JSON.
A controlled input checks missing tails, too-early markers, rejected layouts and
unused-byte retention before the first accepted tail. The native-entry corpus
compares events directly against highlights-corpus-v41.json.zlib across all 32
films and verifies retained body bytes against each original footer chunk.

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib native_highlight -- --nocapture
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_ -- --include-ignored --nocapture
```

## Raw objective-footer fields

The reference's `internal/facts/objectives/film.go` exposes raw slot (byte 36)
and team (byte 37), in addition to the timestamp and XUID. Those fields now live
on NativeHighlightIdentityRead.objective_fields. This projection shares the
fixed-block decoder with scan_objective_footer, but preserves source order.
Only the FIRST tail marker may supply these fields: a too-early or non-mode
first marker suppresses them even if the highlight scanner later accepts a mode
layout. This is a measured difference between reference readers, not a fallback.

The independent objective-extract-v41 fixture verifies this projection (sorting
only a test copy to match the reference's published order). Controlled cases
cover later-mode rejection and retain raw slot/team values 254/255 without roster
filtering. The native-entry corpus verifies the raw bytes at the selected first
tail. No mode-specific action, capture, team identity, or player relationship is
inferred. Old exports deserialize with absent objective_fields.
