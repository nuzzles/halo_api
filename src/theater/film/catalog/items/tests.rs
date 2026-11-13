use super::*;
use serde_json::json;

#[test]
fn entire_reference_catalogue_matches_original_lookups_and_relations() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/items-v41/reference.json")).unwrap();
    let catalog = Catalog;
    let rows: Vec<_> = catalog
        .tags()
        .iter()
        .map(|t| {
            assert_eq!(catalog.tag_info(t.tag), Some(t));
            assert_eq!(catalog.item(t.tag), t.item);
            match t.item {
                Some(Item::Weapon(w)) => assert_eq!(TagWeapon::from_tag(catalog, t.tag), Some(w)),
                Some(Item::Vehicle(v)) => assert_eq!(Vehicle::from_tag(catalog, t.tag), Some(v)),
                Some(Item::Equipment(e)) => {
                    assert_eq!(Equipment::from_tag(catalog, t.tag), Some(e))
                }
                Some(Item::Grenade(g)) => assert_eq!(Grenade::from_tag(catalog, t.tag), Some(g)),
                None => {}
            }
            json!({"tag":t.tag.raw(),"kind":format!("{:?}",t.kind),"name":t.name,
            "item":t.item.map(|i|format!("{i:?}")),"item_name":t.item.map(Item::name),
            "parent":t.parent.map(TagId::raw),"part_of":t.part_of.map(TagId::raw),
            "power_weapon":t.power_weapon})
        })
        .collect();
    assert_eq!(json!(rows), reference["tags"]);
    let variants: Vec<_> = catalog
        .weapon_variants()
        .map(|(id, name)| {
            assert_eq!(catalog.weapon_variant_name(id), Some(name));
            json!([id.raw(), name])
        })
        .collect();
    assert_eq!(json!(variants), reference["variants"]);
    let grenades: Vec<_> = (0..=5)
        .map(|i| {
            json!([
                i,
                Grenade::from_type(i).map(|g| json!([g.type_index(), g.name()]))
            ])
        })
        .collect();
    assert_eq!(json!(grenades), reference["grenades"]);
}

#[test]
fn tag_variant_and_weapon_namespaces_and_unknowns_remain_distinct() {
    let catalog = Catalog;
    assert!(catalog.tags().windows(2).all(|w| w[0].tag < w[1].tag));
    let variants: Vec<_> = catalog.weapon_variants().collect();
    assert!(variants.windows(2).all(|w| w[0].0 < w[1].0));
    assert_eq!(catalog.tag_info(TagId(1)), None);
    assert_eq!(catalog.tag_info(TagId(u32::MAX)), None);
    assert_eq!(catalog.weapon_variant_name(WeaponVariantId(1)), None);
    assert_eq!(
        catalog.weapon_variant_name(WeaponVariantId(0x42c9679f)),
        Some("default")
    );
    let tag = TagId(0x2b1824d5);
    assert_eq!(
        TagWeapon::from_tag(catalog, tag),
        Some(TagWeapon::Br75BattleRifle)
    );
    assert_eq!(Vehicle::from_tag(catalog, tag), None);
    assert_eq!(Equipment::from_tag(catalog, tag), None);
    assert_eq!(Grenade::from_tag(catalog, tag), None);
    let weapon = crate::theater::film::WeaponId::from_parts(tag.raw(), 0x42c9679f);
    assert_ne!(u64::from(tag.raw()), weapon.raw());
    for id in [TagId(0), tag, TagId(u32::MAX)] {
        assert_eq!(serde_json::from_value::<TagId>(json!(id)).unwrap(), id);
        assert_eq!(format!("{id}").len(), 8);
    }
    for t in catalog.tags() {
        if let Some(item) = t.item {
            assert_eq!(serde_json::from_value::<Item>(json!(item)).unwrap(), item);
        }
    }
    assert!(
        catalog
            .tags()
            .iter()
            .any(|t| t.name.is_none() && t.item.is_none())
    );
    assert!(catalog.tags().iter().any(|t| t.parent.is_some()));
    assert!(catalog.tags().iter().any(|t| t.part_of.is_some()));
}
