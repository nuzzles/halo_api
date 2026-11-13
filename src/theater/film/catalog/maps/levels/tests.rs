use super::*;
#[test]
fn level_catalogue_matches_all_upstream_float_words_and_widths() {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/levels-v41/catalogue.json"
    ))
    .unwrap();
    assert_eq!(source["build"], BUILD);
    assert!(BUILD.starts_with(&format!("{BUILD_ID}.")));
    assert!(BUILD.ends_with(&format!(" {BRANCH}")));
    let rows = source["levels"].as_array().unwrap();
    assert_eq!(rows.len(), all().len());
    assert_eq!(rows.len(), 33);
    let mut regions = 0;
    for (expected, level) in rows.iter().zip(all()) {
        assert_eq!(expected["tag"], level.tag);
        assert_eq!(expected["module"], level.module);
        assert_eq!(expected["display_name"].as_str(), level.display_name);
        let api_bytes: [u8; 16] = expected["api_id"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let api_id = uuid::Uuid::from_bytes_le(api_bytes);
        assert_eq!(level.api_id, Some(api_id));
        assert_eq!(by_api_id(api_id), Some(level));
        assert_eq!(expected["structure_tag"], level.structure_tag);
        assert_eq!(expected["index_bits"], level.index_bits);
        assert_eq!(resolve(BUILD, level.tag), Some(level));
        assert!(resolve("HI_1_13_0", level.tag).is_none());
        assert!(resolve("269225.26.04.08.1618 HI_1_12_0", level.tag).is_none());
        for (actual, expected) in level.regions().zip(expected["regions"].as_array().unwrap()) {
            regions += 1;
            assert_eq!(
                actual.bounds.region,
                expected["index"].as_u64().unwrap() as u32
            );
            assert_eq!(
                serde_json::json!(actual.bounds.min.map(f32::to_bits)),
                expected["min_bits"]
            );
            assert_eq!(
                serde_json::json!(actual.bounds.max.map(f32::to_bits)),
                expected["max_bits"]
            );
            assert_eq!(serde_json::json!(actual.axis_bits), expected["axis_bits"]);
            assert_eq!(actual.bounds.axis_bits(), Some(actual.axis_bits));
            assert_eq!(level.region(actual.bounds.region), Some(actual));
            assert!(actual.bounds.region < (1 << level.index_bits));
            let raw = [0; 3];
            let pos = actual.bounds.dequantize(raw, actual.axis_bits).unwrap();
            for (axis, coordinate) in pos.iter().enumerate() {
                let expected = f64::from(actual.bounds.min[axis])
                    + 0.5
                        * (f64::from(actual.bounds.max[axis]) - f64::from(actual.bounds.min[axis]))
                        / (1u64 << actual.axis_bits[axis]) as f64;
                assert_eq!(*coordinate, expected);
            }
        }
        assert!(level.region(u32::MAX).is_none());
    }
    assert_eq!(regions, 65);
    assert_eq!(by_api_id(uuid::Uuid::nil()), None);
}
