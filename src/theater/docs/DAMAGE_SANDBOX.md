# Reference damage and sandbox metadata

`film/catalog/damage` and `film/catalog/physics` sit beside the item, map and weapon
catalogues. Both use the `film::catalog::Catalog` value and the shared 32-bit
`TagId` namespace. The reference data does not establish a compatible game
build. No catalogue lookup runs during canonical parsing.

The complete pinned catalogue contains:

- 468 damage effects: cause, optional weapon, optional numeric amount, flags,
  grenade-blast ambiguity and every associated projectile in original order.
- 151 firing entries: rate, burst count, recovery, optional magazine values and
  optional projectile tag.
- 294 projectile entries: velocity, range and optional impact effect.
- 64 vehicle render-model bounds, with x/y/z minimum and maximum in source order.

```rust
use halo_api::theater::film::catalog::{Catalog, items::TagWeapon};
use halo_api::theater::film::TagId;

let catalog = Catalog;
let fire = catalog.fire_stats(TagId(0x2b1824d5)).unwrap();
let projectile = catalog.projectile_stats(fire.projectile.unwrap()).unwrap();
let effect = catalog.damage_effect(projectile.impact.unwrap()).unwrap();
assert_eq!(TagWeapon::from_tag(catalog, effect.weapon.unwrap()),
           Some(TagWeapon::Br75BattleRifle));
```

Full ordered tables are available through `damage_effects()`,
`firing_catalogue()`, `projectile_catalogue()` and `vehicle_models()`. Lookups
return None for unknown tags. Optional amounts and relationships are not replaced
with zeros or guessed owners. Multiple projectile sources stay multiple.

`DamageAmount::fixed()` and flag helpers reproduce upstream's metadata rules.
They do not establish a hit's actual damage. Reference Spartan vitality is
available explicitly through `spartan_vitality()` and does not initialize runtime
players. Magazine values do not replace observed inventory. Vehicle bounds are
render-model extents, not collision shapes, poses or the missing vehicle physics
type needed to decode some snapshots.

The [independent fixture](../fixtures/damage-sandbox-v41/README.md) compares every
row and floating-point bit pattern against original upstream lookups. Precision
is exactly that of the upstream f64 constants; no re-quantization, f32 conversion,
distance falloff, damage estimate or build compatibility is inferred.
