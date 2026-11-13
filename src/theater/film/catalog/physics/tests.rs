use super::*;
use serde_json::json;

#[test]
fn sandbox_catalogue_matches_original_f64_bits_and_optional_relationships() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/damage-sandbox-v41/reference.json"
    ))
    .unwrap();
    let c = Catalog;
    let fire: Vec<_> = c
        .firing_catalogue()
        .iter()
        .map(|f| {
            assert_eq!(c.fire_stats(f.tag), Some(f));
            json!({"tag":f.tag.raw(),"bits":[f.rounds_per_second.to_bits(),f.recovery.to_bits()],
            "burst":f.burst,"magazine":f.magazine,"projectile":f.projectile.map(TagId::raw)})
        })
        .collect();
    assert_eq!(json!(fire), reference["fire"]);
    let projectiles:Vec<_>=c.projectile_catalogue().iter().map(|p| {
        assert_eq!(c.projectile_stats(p.tag),Some(p));
        json!({"tag":p.tag.raw(),"bits":[p.velocity.to_bits(),p.range.to_bits()],"impact":p.impact.map(TagId::raw)})
    }).collect();
    assert_eq!(json!(projectiles), reference["projectile"]);
    let bounds: Vec<_> = c
        .vehicle_models()
        .iter()
        .map(|&(tag, values)| {
            assert_eq!(c.vehicle_bounds(tag), Some(values));
            json!([tag.raw(), values.map(f64::to_bits)])
        })
        .collect();
    assert_eq!(json!(bounds), reference["bounds"]);
    assert!(c.firing_catalogue().windows(2).all(|w| w[0].tag < w[1].tag));
    assert!(
        c.projectile_catalogue()
            .windows(2)
            .all(|w| w[0].tag < w[1].tag)
    );
    assert!(c.vehicle_models().windows(2).all(|w| w[0].0 < w[1].0));
    for tag in [TagId(0), TagId(u32::MAX)] {
        assert!(c.fire_stats(tag).is_none());
        assert!(c.projectile_stats(tag).is_none());
        assert!(c.vehicle_bounds(tag).is_none());
    }
    assert!(c.firing_catalogue().iter().any(|r| r.magazine.is_none()));
    assert!(c.firing_catalogue().iter().any(|r| r.projectile.is_none()));
    assert!(c.projectile_catalogue().iter().any(|r| r.impact.is_none()));
}
