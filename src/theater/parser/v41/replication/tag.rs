//! Add known game reference data while projecting a recorded object tag.

use crate::theater::film::TagId;
use crate::theater::film::catalog::Catalog;
use crate::theater::film::catalog::damage::DamageCause as CatalogDamageCause;
use crate::theater::film::catalog::items::{Grenade, Item, TagCategory};
use crate::theater::film::replication::{
    DamageAmount, DamageCause, DamageReference, FiringReference, ProjectileReference,
    ReplicationValue, TagKind, TagLink, TagReference,
};

pub(super) fn value(id: TagId) -> ReplicationValue {
    let catalog = Catalog;
    let kind = match catalog.tag_info(id) {
        Some(info) => match (info.kind, info.item) {
            (TagCategory::Weapon, Some(Item::Weapon(item))) => TagKind::Weapon(item),
            (TagCategory::Vehicle, Some(Item::Vehicle(item))) => TagKind::Vehicle(item),
            (TagCategory::Equipment, Some(Item::Equipment(item))) => TagKind::Equipment(item),
            (TagCategory::Grenade, Some(Item::Grenade(item))) => TagKind::Grenade(item),
            (TagCategory::Projectile, _) => TagKind::Projectile,
            (TagCategory::Crate, _) => TagKind::Crate,
            (TagCategory::Machine, _) => TagKind::Machine,
            (TagCategory::Scenery, _) => TagKind::Scenery,
            (TagCategory::Control, _) => TagKind::Control,
            _ => TagKind::Unknown,
        },
        None => TagKind::Unknown,
    };
    let firing = catalog.fire_stats(id).map(|stats| FiringReference {
        rounds_per_second: stats.rounds_per_second,
        burst: stats.burst,
        recovery: stats.recovery,
        magazine: stats.magazine,
        projectile: stats
            .projectile
            .map(|id| projectile_reference(id).map_or(TagLink::Unknown(id), TagLink::Known)),
    });
    let reference = firing
        .map(TagReference::Firing)
        .or_else(|| projectile_reference(id).map(TagReference::Projectile))
        .or_else(|| damage_reference(id).map(TagReference::Damage))
        .or_else(|| catalog.vehicle_bounds(id).map(TagReference::VehicleBounds))
        .map(Box::new);
    ReplicationValue::Tag {
        id,
        kind,
        reference,
    }
}

fn projectile_reference(id: TagId) -> Option<ProjectileReference> {
    let stats = Catalog.projectile_stats(id)?;
    Some(ProjectileReference {
        tag: id,
        velocity: stats.velocity,
        range: stats.range,
        impact: stats
            .impact
            .map(|id| damage_reference(id).map_or(TagLink::Unknown(id), TagLink::Known)),
    })
}

fn damage_reference(id: TagId) -> Option<DamageReference> {
    let effect = Catalog.damage_effect(id)?;
    let cause = match effect.cause {
        CatalogDamageCause::Weapon => DamageCause::Weapon {
            weapon: effect.weapon,
        },
        CatalogDamageCause::Melee => DamageCause::Melee {
            weapon: effect.weapon,
        },
        CatalogDamageCause::Grenade => {
            let source = effect
                .weapon
                .expect("catalogued grenade effect has a source tag");
            DamageCause::Grenade {
                kind: grenade_kind(source),
                source,
            }
        }
        CatalogDamageCause::Equipment => DamageCause::Equipment,
        CatalogDamageCause::Vehicle => DamageCause::Vehicle,
        CatalogDamageCause::Collision => DamageCause::Collision,
        CatalogDamageCause::World => DamageCause::World,
        CatalogDamageCause::Explosion => DamageCause::Explosion,
        _ => DamageCause::Unknown,
    };
    Some(DamageReference {
        tag: id,
        cause,
        amount: effect.amount.map(|amount| DamageAmount {
            lower: amount.lower,
            upper: amount.upper,
            shield_scale: amount.shield_scale,
            body_scale: amount.body_scale,
            flags: amount.flags,
            radius: amount.radius,
        }),
        grenade_blast: effect.grenade_blast,
        projectiles: effect.projectiles().collect(),
    })
}

fn grenade_kind(source: TagId) -> Grenade {
    let info = Catalog
        .tag_info(source)
        .expect("catalogued grenade source has a tag entry");
    if let Some(Item::Grenade(kind)) = info.item {
        return kind;
    }
    // Thrown grenades are projectile tags, so they have a catalogue name but
    // no inventory-grenade Item value.
    [
        Grenade::Frag,
        Grenade::Plasma,
        Grenade::Dynamo,
        Grenade::Spike,
    ]
    .into_iter()
    .find(|kind| Some(kind.name()) == info.name)
    .expect("catalogued grenade projectile names a known grenade type")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn firing_reference(id: TagId) -> FiringReference {
        let ReplicationValue::Tag {
            reference: Some(reference),
            ..
        } = value(id)
        else {
            panic!("expected a firing tag reference");
        };
        let TagReference::Firing(firing) = *reference else {
            panic!("expected firing details");
        };
        firing
    }

    #[test]
    fn projectile_link_distinguishes_unknown_from_absent() {
        let firing = firing_reference(TagId(0x4007_310f));
        assert_eq!(
            firing.projectile,
            Some(TagLink::Unknown(TagId(0xa6ef_800c)))
        );

        let firing_without_projectile = firing_reference(TagId(0x0000_a4bc));
        assert_eq!(firing_without_projectile.projectile, None);
    }

    #[test]
    fn damage_causes_keep_associated_tags_without_treating_blasts_as_causes() {
        let grenade = damage_reference(TagId(0x0001_62eb)).unwrap();
        assert_eq!(
            grenade.cause,
            DamageCause::Grenade {
                kind: Grenade::Frag,
                source: TagId(0x0001_62ee)
            }
        );
        assert!(grenade.grenade_blast);
        assert_eq!(grenade.projectiles.len(), 2);

        let weapon_blast = damage_reference(TagId(0x64ab_85c4)).unwrap();
        assert_eq!(weapon_blast.cause, DamageCause::Weapon { weapon: None });
        assert!(weapon_blast.grenade_blast);

        let melee = damage_reference(TagId(0x0000_27ce)).unwrap();
        assert_eq!(
            melee.cause,
            DamageCause::Melee {
                weapon: Some(TagId(0x0000_ae3c))
            }
        );

        for (effect, kind, source) in [
            (0x0000_b160, Grenade::Frag, 0x0000_ae3e),
            (0x0000_d614, Grenade::Plasma, 0x0000_d5ff),
            (0x5925_5106, Grenade::Dynamo, 0xbf82_fc4b),
            (0xc868_1ecf, Grenade::Spike, 0x5729_76be),
        ] {
            assert_eq!(
                damage_reference(TagId(effect)).unwrap().cause,
                DamageCause::Grenade {
                    kind,
                    source: TagId(source)
                }
            );
        }

        let grenade_effects: Vec<_> = Catalog
            .damage_effects()
            .iter()
            .filter(|effect| effect.cause == CatalogDamageCause::Grenade)
            .collect();
        assert_eq!(grenade_effects.len(), 27);
        for effect in grenade_effects {
            let source = effect.weapon.expect("grenade source tag");
            assert!(matches!(
                damage_reference(effect.tag).unwrap().cause,
                DamageCause::Grenade { source: projected, .. } if projected == source
            ));
        }
    }
}
