use super::*;
use serde_json::json;

#[test]
fn damage_catalogue_matches_every_original_value_relation_and_flag() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/damage-sandbox-v41/reference.json"
    ))
    .unwrap();
    let catalog = Catalog;
    let rows:Vec<_>=catalog.damage_effects().iter().map(|d| {
        assert_eq!(catalog.damage_effect(d.tag),Some(d));
        json!({"tag":d.tag.raw(),"cause":format!("{:?}",d.cause),
            "weapon":d.weapon.map(TagId::raw),"grenade_blast":d.grenade_blast,
            "projectiles":d.projectiles().map(TagId::raw).collect::<Vec<_>>(),
            "amount":d.amount.map(|a|json!({
                "bits":([a.lower,a.upper[0],a.upper[1],a.shield_scale,a.body_scale,a.radius].map(f64::to_bits)),
                "flags":a.flags,"fixed":a.fixed(),"has":(0..32).map(|b|a.has(1<<b)).collect::<Vec<_>>()
            }))})
    }).collect();
    let expected = reference["damage"].as_array().unwrap();
    assert_eq!(rows.len(), expected.len());
    for (actual, expected) in rows.iter().zip(expected) {
        assert_eq!(actual, expected, "damage tag {}", actual["tag"]);
    }
    let (body, shield) = catalog.spartan_vitality();
    assert_eq!(
        json!([[body.to_bits(), shield.to_bits()]]),
        reference["spartan"]
    );
    assert_eq!(
        json!([[
            DamageAmount::CAN_HEADSHOT,
            DamageAmount::SKIPS_SHIELDS,
            DamageAmount::NO_SPILL
        ]]),
        reference["flags"]
    );
    assert!(
        catalog
            .damage_effects()
            .windows(2)
            .all(|w| w[0].tag < w[1].tag)
    );
    for tag in [TagId(0), TagId(u32::MAX)] {
        assert!(catalog.damage_effect(tag).is_none());
    }
    assert!(catalog.damage_effects().iter().any(|d| d.amount.is_none()));
    assert!(
        catalog
            .damage_effects()
            .iter()
            .any(|d| d.projectiles().count() > 1)
    );
    assert!(
        catalog
            .damage_effects()
            .iter()
            .any(|d| d.cause == DamageCause::Unknown)
    );
}
