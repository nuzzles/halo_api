# New event-body traces at f050371

162 independently authored event lists cover codes 2, 8, 12, 13, 14, 80, 104,
105 and 106. Code 117, also added upstream, already has local native coverage.
Each fixture includes a following zoom event and list terminator. Presence
combinations, zero/max words, all networked-AI forms, default/indexed points and
three outer-reference forms are retained. Reserved numeric codes remain raw;
a complete read is not proof that every value occurs during gameplay.

The exporter runs original libfilmshell `events.rs` at
`f050371a6cda83b9887eeb83a1884336fce28e53`, blob
`4fd94a1480341fa5faf52fd6f481672bb833740f`. Only primitive read/skip tracing and
the test harness are added to a disposable archive. It shares the original
event exporter, resolves cached compatible dependencies after removing only
the disposable Cargo.lock, and disables incremental compilation. Neither
source checkout is modified. The old 1,494-case oracle at 18067987 regenerates
byte-identically with that exporter.

Default widths are 18 bits at precision 12 and 22 at precision 16. The explicit
indexed test layout uses one region bit and [15,15,17] widths at precision 16,
[11,11,13] at precision 12. These test widths are not an independent map catalogue.
Local unknown-region/missing-context refusal is intentionally stricter than
upstream's fallback-to-play-space behavior.

The source tests compare every primitive boundary at eight alignments, exact
raw scalars, record/list endpoints, following events and serialization. The
existing tests additionally run every-bit truncation, invalid/missing position
context and public Film/frame/entity/control integration on these new fixtures.
The old six references-only fixtures for 14/104/106 retain their original oracle;
their reference prefixes are still compared, while these new complete bodies
supersede the old pin's body-refusal expectation.

```sh
python3 src/theater/fixtures/events-f050371-v41/wire_cases.py
python3 src/theater/fixtures/events-f050371-v41/export_reference.py /path/to/libfilmshell > /tmp/new-events-reference.json
cmp /tmp/new-events-reference.json src/theater/fixtures/events-f050371-v41/reference.json
cargo test --all-features --config build.incremental=false frame::events::tests
```

See docs/LIBFILMSHELL_UPDATE_F050371.md and docs/VALIDATION.md for the wider
source audit and captured validation. No historical oracle was regenerated to
hide a mismatch.
