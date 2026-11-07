# Captured v41 equipment identity

These byte-aligned slices come from Oddball match
`4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73`. They contain one- and two-reference
`unit-equipment-component` readings on owned bipeds, and the prior equipment NEW
defaults for the first referenced object. `evidence.json` records exact source
locations, times, entity IDs, fields and SHA-256 hashes of slices/decompressed
chunks. Bits retain their original byte alignment.

The single reference resolves to entity `1073743267`, whose prior archetype-37
creation records tag `0xa53cd143`. Its human-readable equipment name is unknown.
Reference category-0 base 512 follows LevelUp `413971b`,
`equipement_porte_lecteurs.go`; the tag field follows `equipment_creation.go`.
These are source regression fixtures, not independent video/HUD annotations.

Run `cargo test --locked --offline theater::runtime::replay::equipment`.
