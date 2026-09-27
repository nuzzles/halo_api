# Cached movement states and stance intervals

`build_facts_replay_stances` feeds the same life/death-aware episode accumulator
as recording-backed stance publication. It groups and sorts by original native
kind bytes and stable timestamp order. Progress, chunk, and packet index remain
in the cache DTO; the native stance publisher does not consume those fields.

`ReplayStance`, `ReplayStanceCoverage`, and `ReplayStances` retain their default
String kind API, with a cache specialization using `ReplayByteString`. The latter
retains bytewise identity, including distinct invalid UTF-8 kinds. JSON rendering
replaces each invalid byte with U+FFFD, matching Go encoding/json string values.
Distinct map keys may therefore render identically. JSON roundtrips cannot recover
those identities; preserve the in-memory structure or original cache. This is
string-value parity, not a claim of byte-identical serde JSON escaping.

Coverage uses i64 counters and u64 axis widths on host and WASM. Dropped readings
increment with native signed wrapping. Empty input, absent tracks, and zero step
retain initial coverage and bypass folding. Native cache encoding omits
JumpEpisodes and JumpsDerived, so decoded-cache publication leaves both zero.
Outer document attachment must still honor the native scanned gates.

The pinned Go fixture generator is halo_rust_facts_stances_test.go.txt. Its 1,024
cases exercise invalid UTF-8 collisions, empty/unknown kinds, stable ties, wide
timestamps, signed counter extrema, dropped overflow, map widths, and lifetime
closure. Expectations include raw hexadecimal kind identities and separate raw
map counts; comparing JSON alone would miss identity loss. Original stance
publication fixtures remain regression coverage. See facts-movement-inputs-validation.json
for checks actually completed.

Full cache document assembly, placements/pads/vehicles/objectives, scan capture,
fallback restoration, legacy domain reconciliation, and the final parity audit
remain pending. This adapter does not complete all-data v41 parity or start the
Film/ResolvedFilm/playback architecture.
