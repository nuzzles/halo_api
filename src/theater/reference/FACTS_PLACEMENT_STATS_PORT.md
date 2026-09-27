# Placement statistics in the facts cache

Reference: `replay/filmfacts_statsdepose.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`facts_placement_stats.rs` carries all eleven fields of the native placement
statistics, including the full calibration object. All native int fields remain
signed i64 on both host and WASM; no scan flag or zero count removes other data.
Width-pair keys sort by signed lead then signed index; ID keys sort by unsigned
u32. Both map decoders preserve last-write replacement for duplicate keys,
partial map entries after read failures, native count guards (three and two
minimum bytes per entry), and nil versus allocated map results. Empty maps are
encoded with zero count and decode to nil, matching the native cache.

The independently generated oracle compares 13,139 composed complete/truncated
streams from 64 seeded statistics structures. Another 8,769 standalone map cases
exercise duplicate keys, oversized IDs, count guards, malformed varints and
truncation. Comparisons cover writer bytes, every field, map contents, errors and
reader cursor. Signed integer extrema and populated data behind false scan flags
are included. Fixtures are generated directly by the pinned Go functions, not
by the Rust implementation.

These cache DTOs are separate from scan-result types whose existing counters use
usize. Cache integration must perform explicit conversions rather than silently
narrowing values. This shared section does not complete full FilmFacts assembly,
file framing, or all-data parser parity. The architecture phase stays deferred.
