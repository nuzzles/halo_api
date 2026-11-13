# Film reference catalog

`film::catalog` contains facts looked up *about* identifiers in a recording.
It does not contain the recording's bytes, parsed fields, or reconstructed
match state. Unknown IDs remain representable when no catalogue entry exists.

| Module | Contents |
| --- | --- |
| `maps` | Known map identities, asset associations, build-scoped level tags, coordinate bounds and regions. |
| `items` | Object tags, classifications, names, parent/part relationships and weapon variant hashes. |
| `weapons` | Known 64-bit weapon definitions, names and reference ammo defaults. |
| `medals` | Native medal names, categories, difficulty and reference scores. |
| `options` | Option hashes/names, mode scripts and literal reference defaults. |
| `damage` | Damage-effect tags, causes, relationships and reference amounts. |
| `physics` | Firing/projectile reference values and vehicle model extents (formerly `sandbox`). |

`film::ids` owns raw `WeaponId`, `TagId` and `WeaponVariantId`; the public
`film` root re-exports them. A recorded 64-bit weapon ID is distinct from a
32-bit item tag, and a recorded map UUID or level tag is distinct from a
catalogue map name. Catalogue lookup may label an ID but cannot supply a
missing recorded value or select a parser branch without verified context.

`film` owns source-backed wire records. `parser/v41` owns bit grammar
and build-dependent admission. `runtime` owns gameplay reconstruction. Known
reference data is accessed through `film::catalog`; raw IDs remain at the
`film` root. The `Catalog` value provides access to the single bundled
reference dataset; it does not select a film version or parser layout.
