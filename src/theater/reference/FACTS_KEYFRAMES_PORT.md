# Native facts-cache keyframe presence section

Reference: replay/filmfacts_codec.go encodeKeyframes/decodeKeyframes at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

FactsWorldKeyframes retains the complete timestamp list and a map from (slot,
generation) to ordered presence timestamps. Encoding sorts keys by slot then
generation, preserves each timestamp list without deduplication/reordering, and
uses wrapping unsigned deltas. Decoding narrows life keys exactly as native u32
casts do and replaces earlier values when encoded duplicate keys occur.

Native decoding uses an unguarded signed count for the outer timestamps and map,
then count(2) for each presence list. That final guard is intentionally preserved,
although timestamp varints can be one byte. A one-point section encoded as
000101020100 is refused at byte 5; appending one zero byte makes it succeed at
byte 6, leaving the extra byte unread. The 64-case boundary fixture independently
verifies 54 native refusals and 10 successful reads under varying trailing bytes.

Rust avoids native eager allocation from untrusted counts, allowing partial reads
to stop at the same cursor/error without allocating the declared capacity. A
negative outer slice length causes a Go panic; Rust explicitly refuses it at the
same cursor. Negative map capacity is accepted by native Go and is preserved as
an empty map. These distinctions are tested, not inferred from count guards.

The main native oracle has 11,792 cases from 128 source maps, all truncation
points, padded suffixes, duplicate/wide raw life keys and signed count controls.
It compares encoded bytes, complete and partial timestamp/map values, error
messages and source byte offsets. The separate cache DTO does not invent the
live scan's Rust-only band set. Conversion from WorldObjectKeyframes projects
its per-life list into native map semantics.

See facts-keyframes-validation.json for host/WASM results. Creation records,
creation statistics, world scan composition, position sections and complete
facts-cache file framing remain unported. This derivative-cache work does not
establish full v41 film parity or start the deferred architecture refactor.
