# Native type-12 bot metadata

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`,
`internal/facts/killsource/botmeta.go` (not internal/grammar).

`NativeFilmPacket.bot_metadata_read` is populated for non-footer type-12 packets,
including short/refused payloads. It retains the big-endian count at bits 0..32.
An unavailable count or count above 64 prevents scanning, exactly as loadBotMeta;
a zero count still permits the reference's name scan.

Name candidates use the shared reference-compatible scanner. Names must be
terminated printable-ASCII UTF16BE, length 4..48; slot and ID are read at fixed
backward offsets and must satisfy the reference's bounds. There is no fixed
entry stride. Each candidate retains raw name units including the terminator,
slot/ID coordinates and name end. Duplicate-name candidates remain present;
selected_by_reference labels the first occurrence that scanBotEntries publishes.
No cross-packet deduplication, player attribution, entity lifetime or aggregate
bot count is built. Candidates are not a canonical packet partition. Rejected
scan regions and unparsed bytes survive in the enclosing source chunk.

Validation uses the existing independent 512-case bot-metadata oracle and captured
bot-corpus oracle. Selected candidates match the former's per-packet outputs
when the count admits scanning. Independent bit extraction checks candidate raw
values and name units, including repeated-name candidates. Admission tests cover
missing/zero/invalid counts, other packet types, footer packets, JSON round trips,
and old exports. The native-entry corpus counts integrated reads/candidates.

Reproduce:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib bot_metadata -- --nocapture
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_ -- --include-ignored --nocapture
```

The earlier native-entry audit's claim that the pinned production reference had
no type-12 reader was incorrect: it missed this reader in killsource. Its limited
heuristic parsing does not prove a complete type-12 record grammar. The payload
therefore still has an Opaque primary body alongside the explicit native read.
