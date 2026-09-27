use super::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Read};
#[test]
fn native_ground_ammo_traversal() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/native-ground-ammo-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 512);
    let mut successful = 0;
    let mut after_ammo = 0;
    for (case, row) in rows.iter().enumerate() {
        let h = row["hex"].as_str().unwrap();
        let payload: Vec<_> = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect();
        let mut profile = NativeScanProfile::default();
        profile.grammar.simulation_complete = row["simulation"] == true;
        profile.grammar.corruption_check = row["corruption"] == true;
        let map = |name: &str| {
            serde_json::from_value::<BTreeMap<String, i64>>(row["widths"][name].clone()).unwrap()
        };
        profile.grammar.calibrated_widths = Some(NativeSharedWidths::from_map(map("calibrated")));
        profile.grammar.stub_widths = Some(NativeSharedWidths::from_map(map("stubs")));
        let names: Vec<String> = serde_json::from_value(row["names"].clone()).unwrap();
        let arch = FilmArchetype {
            index: 42,
            levels: vec![0; names.len()],
            components: names,
        };
        let mask: Vec<usize> = serde_json::from_value(row["mask"].clone()).unwrap();
        let out = read_native_ground_weapon_ammo(
            &payload,
            row["start"].as_i64().unwrap(),
            &mask,
            &arch,
            &NativeReaderContext {
                profile,
                observer: None,
            },
        )
        .unwrap();
        assert_eq!(
            out.ammo.is_some(),
            row["ok"].as_bool().unwrap(),
            "admission {case}"
        );
        if let Some(ammo) = &out.ammo {
            assert_eq!(json!(ammo), row["ammo"], "ammo {case}");
            successful += 1;
        }
        assert_eq!(json!(out.end_bit), row["end"], "cursor {case}");
        let trace: Vec<_> = out
            .components
            .iter()
            .map(|c| json!({"index":c.index,"name":c.name,"ported":c.ported,"start":c.start_bit}))
            .collect();
        let expected = if row["trace"].is_null() {
            json!([])
        } else {
            row["trace"].clone()
        };
        assert_eq!(json!(trace), expected, "trace {case}");
        for c in &out.components {
            if c.index > 20 {
                after_ammo += 1;
            }
            if let Some(read) = &c.read {
                assert_eq!(read.start_bit, c.start_bit);
            }
            for f in &c.corruption_fields {
                assert!(f.width == 1 || f.width == 32);
            }
        }
        assert_eq!(
            out.desync_at,
            out.components.iter().find(|c| !c.ported).map(|c| c.index)
        );
    }
    assert!(successful > 0);
    assert!(after_ammo > 0);
}
