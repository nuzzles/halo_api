# Current libfilmshell film-clock oracle

`reference.bin.zlib` records the unchanged packet-framing game ticks and
timestamp-aligned HighlightEvents ticks from
[libfilmshell a13efb323b230c96b59685defbc376720043240f](https://github.com/OpenSpartan/libfilmshell/tree/a13efb323b230c96b59685defbc376720043240f)
for all 32 retained v41 films. The exporter verifies the Git blobs of
`src/container.rs` (`068885d8c5fdc790ea8efd86e5e4e82873ab97f4`),
`src/lives.rs` (`6d2be472947c4c87e9e9727315a1ae6d8f91f928`) and
`src/inflate.rs` (`a8b88897f08ef418146dd72707612f825c3e7b24`). It invokes
the original readers through the research feature. It does not invoke our
parser or patch upstream source.

The deterministic zlib stream contains magic `HCLKv001`, a little-endian film
count, then sorted film rows. A row contains the UTF-8 relative path prefixed
by a little-endian `u16` length, the 32-byte SHA-256 of `film.json`, a
little-endian `u32` frame count, and one `(game_tick, highlight_tick)` pair of
little-endian `u32` values per frame. The ignored local test compares every
pair and checks the count for each film. `verify_source.py` independently checks
the recorded manifest hashes and exact film order.

```sh
python3 src/theater/fixtures/clock-a13efb/export_reference.py \
  /path/to/libfilmshell-at-a13efb experiments/films \
  > /tmp/clock-reference.bin.zlib
cmp /tmp/clock-reference.bin.zlib \
  src/theater/fixtures/clock-a13efb/reference.bin.zlib
python3 src/theater/fixtures/clock-a13efb/verify_source.py experiments/films
cargo test --offline --all-features --lib \
  captured_packet_clock_admission_and_counter_audit -- --ignored --nocapture
```

All 403,465 captured type-0 frames agree with the upstream game and highlight
tick arrays. A counter is readable in 359,692 frames; the remainder follow
the reference's bounded interpolation or per-packet fallback. These are
runtime clock estimates. This fixture does not establish human event timing,
death-to-kill alignment, or the grammar of opaque packet bodies.
