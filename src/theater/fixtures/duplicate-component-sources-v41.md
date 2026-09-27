# Duplicate numbered chunks in component scans

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The biped and inventory duplicate-source fixtures each contain 512 native cases.
Their harnesses reuse the existing source-availability cases and call the actual
native scanners on a loaded Film with metadata numbers 0, 1, 1. Later duplicate
bytes alternate between empty, truncated and equally sized conflicting payloads.
Native FilmChunkAt must keep the first entry. Expected scanner outputs are
identical to the original no-duplicate native fixtures (compared field-for-field).
The default tests and original fixture files remain available unchanged.

- Biped: 114 charge reads, 42 impulses, two grapple reads, 155 equipment lists
  and 53 ability ranks. Camo has negative coverage here, not a positive witness.
- Inventory: 220 inventory records and 152 held-weapon changes.
- Requested chunk sets include present, absent and reordered numbers. The native
  contexts inject cached registry, slot-band, layout and requested chunk lists;
  these are scanner-source tests, not full constructor/profile resolution tests.

Rust uses native anchors from each fixture to exercise the context-aware scanner
APIs. It compares emitted values, counters and packet ordinals, and retains its
existing malformed-anchor and channel JSON checks. The original implementation
failed with Truncated after replacing valid first-chunk bytes with an empty later
duplicate. Numbered byte lookup and packet-ordinal lookup now select the same
first metadata entry for ability, camo, equipment, inventory and held-weapon scans.

Generators: `TestHaloRustBipedDuplicateSources` and
`TestHaloRustInventoryDuplicateSources` in the existing source-availability Go
harnesses. Both are registered in `reference/generate_oracles.py`.
