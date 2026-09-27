use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_equipment_creation_attempts() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/equipment-creation-probe-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 160);
    let map = FilmMapBounds {
        module: String::new(),
        min: [-100., -200., -300.],
        max: [300., 400., 500.],
        axis_widths: [16, 17, 18],
        region: 0,
        region_index_bits: 1,
    };
    let mut count = 0;
    let mut accepted = 0;
    for (case, row) in rows.iter().enumerate() {
        let h = row["hex"].as_str().unwrap();
        let pay: Vec<_> = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect();
        let mut profile = NativeScanProfile::default();
        profile.mpp.lead = row["lead"].as_i64().unwrap();
        profile.mpp.index = row["index"].as_i64().unwrap();
        profile.movement.world_object = NativePrecisionDescriptor {
            index_bits: 1,
            axis_bits: [16, 17, 18],
            region: 0,
        };
        for a in row["attempts"].as_array().unwrap() {
            count += 1;
            let bit = a["bit"].as_u64().unwrap() as usize;
            let out = read_native_equipment_creation(&pay, bit, 31, &map, &profile).unwrap();
            assert_eq!(
                out.creation.is_some(),
                a["ok"].as_bool().unwrap(),
                "admission {case}:{bit}"
            );
            let mut stats = EquipmentCreationStats::default();
            match out.refusal {
                Some(NativeEquipmentCreationRefusal::DefaultOverflow) => stats.overflow = 1,
                Some(NativeEquipmentCreationRefusal::Mask) => stats.mask_bad = 1,
                Some(NativeEquipmentCreationRefusal::Position) => stats.pos_bad = 1,
                None => {}
            }
            assert_eq!(json!(stats), a["stats"], "stats {case}:{bit}");
            if let Some(cre) = out.creation {
                accepted += 1;
                let expected: EquipmentCreation =
                    serde_json::from_value(a["creation"].clone()).unwrap();
                assert_eq!(cre, expected, "creation {case}:{bit}");
                assert_eq!(out.default_state.start_bit, bit as i64 + 24);
                assert_eq!(
                    cre.default_state_bits as i64,
                    out.default_state.end_bit - out.default_state.start_bit
                );
            }
        }
    }
    assert_eq!((count, accepted), (488, 178));
}
