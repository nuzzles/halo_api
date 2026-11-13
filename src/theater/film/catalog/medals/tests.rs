use super::*;
use serde_json::json;

#[test]
fn medal_catalogue_matches_original_api_including_holes_and_all_metadata() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/game-catalogues-v41/reference.json"
    ))
    .unwrap();
    let catalog = Catalog;
    let rows: Vec<_> = catalog
        .medals()
        .map(|medal| {
            assert_eq!(catalog.medal(medal.index()), Some(medal));
            assert_eq!(
                serde_json::from_value::<Medal>(json!(medal)).unwrap(),
                medal
            );
            json!({
                "index": medal.index().raw(), "variant": format!("{medal:?}"),
                "name": medal.name(), "kind": format!("{:?}", medal.kind()),
                "difficulty": format!("{:?}", medal.difficulty()),
                "personal_score": medal.personal_score(), "sorting_weight": medal.sorting_weight(),
            })
        })
        .collect();
    assert_eq!(rows.len(), 157);
    assert_eq!(json!(rows), reference["medals"]);
    let indices: std::collections::BTreeSet<_> = reference["medals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["index"].as_u64().unwrap() as u32)
        .collect();
    for index in (0..=u32::from(u16::MAX)).chain([u32::MAX]) {
        assert_eq!(
            catalog.medal(MedalIndex(index)).is_some(),
            indices.contains(&index),
            "{index}"
        );
    }
    assert_eq!(catalog.medal(MedalIndex(14)), None);
    let double = catalog.medal(MedalIndex(0)).unwrap();
    assert_eq!(double.personal_score(), 50);
    assert_eq!(double.sorting_weight(), 100);
    assert_eq!(catalog.medal(MedalIndex(8)), Some(Medal::Killionaire));
    assert_eq!(
        serde_json::from_value::<MedalIndex>(json!(u32::MAX)).unwrap(),
        MedalIndex(u32::MAX)
    );
}
