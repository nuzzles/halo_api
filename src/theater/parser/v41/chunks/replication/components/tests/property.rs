use super::*;
use serde_json::Value;

pub(super) fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../fixtures/property-v41/components.json"
    ))
    .unwrap()
}
pub(super) fn disagrees(case: &Case) -> bool {
    case.component == "managed-object-player-masked-property-component"
        && case.bytes()[0] >> 4 >= 12
}
fn oracle() -> Value {
    let o: Value = serde_json::from_str(include_str!(
        "../../../../../../fixtures/property-v41/source-reference.json"
    ))
    .unwrap();
    assert_eq!(o["commit"], "413971bcc485be53a2fa8cf229e170539a5146f8");
    assert_eq!(
        o["blobs"]["components_managed_property.go"],
        "8f7093171e8b48f7feb8c9f85258dd0cce24dee7"
    );
    assert_eq!(
        o["blobs"]["ti13_vecteurs_test.go"],
        "926944ccbe1dca6abdcc14843a774d4fa8e51e70"
    );
    o
}
fn shifted(source: &[u8], shift: usize) -> Vec<u8> {
    let mut data = vec![0; (shift + source.len() * 8).div_ceil(8)];
    for i in 0..source.len() * 8 {
        data[(i + shift) / 8] |= ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
    }
    data
}
#[test]
fn captured_property_vectors_keep_enum_width_and_following_source_bits() {
    let o = oracle();
    let rows = o["captured"].as_array().unwrap();
    assert_eq!(rows.len(), 33);
    let mut conflicts = 0;
    for (i, row) in rows.iter().enumerate() {
        let masked = row["masked"].as_bool().unwrap();
        let width = row["width"].as_u64().unwrap() as usize;
        let case = Case {
            name: format!("captured/{i}"),
            component: if masked {
                "managed-object-player-masked-property-component"
            } else {
                "managed-object-property-component"
            }
            .into(),
            version: 1,
            archetype: 13,
            hex: row["hex"].as_str().unwrap().into(),
            bit_len: 4 + width,
            complete: true,
            snapshot: false,
        };
        if masked && row["tag"].as_u64().unwrap() >= 12 {
            conflicts += 1;
            assert_eq!(row["chained"], true);
        }
        for shift in 0..8 {
            let data = shifted(&case.bytes(), shift);
            let (status, end, fields) = read(&case, &data, shift);
            assert_eq!(
                (status, end),
                (Some(true), (shift + case.bit_len) as i64),
                "{row}"
            );
            assert_eq!(fields[0].raw.low_u64(), row["tag"].as_u64().unwrap());
            assert_eq!(fields.len(), if width == 0 { 1 } else { 2 });
            if width != 0 {
                assert_eq!(fields[1].width, width);
                assert_eq!(fields[1].raw.low_u64(), row["payload"].as_u64().unwrap());
            }
            for f in fields {
                assert_eq!(RawBits::from_source(&data, f.bit, f.width).unwrap(), f.raw);
            }
            // The following source bits are left outside the property's fields.
            assert_eq!(
                RawBits::from_source(&data, end as usize, 16),
                RawBits::from_source(&case.bytes(), case.bit_len, 16)
            );
        }
    }
    assert_eq!(conflicts, 5);
}
#[test]
fn property_components_match_original_instruction_addressed_reader() {
    let oracle = oracle();
    let expected = oracle["components"].as_array().unwrap();
    let cases = cases();
    assert_eq!(cases.len(), 128);
    assert_eq!(cases.len(), expected.len());
    for (case, trace) in cases.iter().zip(expected) {
        assert_eq!(case.name, trace["name"]);
        for shift in 0..8 {
            let data = shifted(&case.bytes(), shift);
            let (status, end, fields) = read(case, &data, shift);
            assert_eq!(status, Some(true));
            assert_eq!(
                end as usize,
                shift + trace["end"].as_u64().unwrap() as usize
            );
            let boundaries: BTreeSet<_> = std::iter::once(shift)
                .chain(fields.iter().map(|f| f.bit + f.width))
                .collect();
            for t in trace["trace"].as_array().unwrap() {
                let at = t[0].as_u64().unwrap() as usize;
                let n = t[1].as_u64().unwrap() as usize;
                assert!(boundaries.contains(&(shift + at)));
                assert!(boundaries.contains(&(shift + at + n)));
            }
            for f in &fields {
                assert_eq!(
                    RawBits::from_source(&data, f.bit, f.width).as_ref(),
                    Some(&f.raw)
                );
            }
            assert_eq!(
                fields,
                serde_json::from_str::<Vec<ComponentField>>(
                    &serde_json::to_string(&fields).unwrap()
                )
                .unwrap()
            );
        }
    }
}
