# Reference object-tag catalogue

`film::catalog::items` holds the pinned libfilmshell object-tag table beside
`film::catalog::weapons` and `film::catalog::maps`. It contains 2,002 asset tags,
40 variant string hashes, 45 parent links and 16 vehicle-part associations. Nine kinds distinguish
weapons, vehicles, equipment, inventory grenades, projectiles, crates, machines,
scenery and controls. Optional names and item classifications remain optional.
Names inherited upstream are preserved; they are not inferred again locally.

```rust
use halo_api::theater::film::catalog::{Catalog, items::TagWeapon};
use halo_api::theater::film::TagId;

let catalog = Catalog;
let tag = TagId(0x2b1824d5);
assert_eq!(TagWeapon::from_tag(catalog, tag), Some(TagWeapon::Br75BattleRifle));
assert_eq!(catalog.tag_info(tag).unwrap().name, Some("BR75 Battle Rifle"));
```

The reference data does not establish a compatible game build. There is no
automatic lookup during Film parsing and no catalogue-based vehicle gate. This supplies reference facts for
future typed runtime views while leaving applicability visible to callers.
`TagId(u32)`, `WeaponVariantId(u32)` and `WeaponId(u64)`
are separate types. No tag-to-weapon-ID conversion or ownership guess is added.
The `TagWeapon` name distinguishes object-tag classification from the existing
weapon family/variant model.

`tags()`, `tag_info()`, `item()`, `weapon_variants()` and
`weapon_variant_name()` expose all entries with deterministic lookup. Vehicle,
equipment, grenade and tag-weapon helpers accept the same `Catalog` value.
Unknown IDs return None without changing the caller's original ID. Relations
are direct recorded catalogue links, not recursively resolved ownership.

The [independent fixture](../fixtures/items-v41/README.md) compares every entry,
relation, name and variant against original upstream lookup results. The pinned
data generator reproduces the local table; namespace, unknown-entry, ordering
and enum/ID serialization tests protect its public representation. This does
not establish which game builds the reference data applies to.
