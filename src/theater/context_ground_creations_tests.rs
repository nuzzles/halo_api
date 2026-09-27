use super::*;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};
fn fixture(raw: &[u8]) -> Value {
    let mut b = Vec::new();
    flate2::read::ZlibDecoder::new(raw)
        .read_to_end(&mut b)
        .unwrap();
    serde_json::from_slice(&b).unwrap()
}
#[test]
fn native_context_ground_creations() {
    let rows = fixture(include_bytes!(
        "fixtures/context-ground-creations-v41.json.zlib"
    ));
    let seeds = fixture(include_bytes!(
        "fixtures/ground-creation-profile-v41.json.zlib"
    ));
    assert_eq!(rows.as_array().unwrap().len(), 96);
    let mut accepted = 0;
    let mut ammo = 0;
    for (case, row) in rows.as_array().unwrap().iter().enumerate() {
        let seed = &seeds[row["seed"].as_u64().unwrap() as usize];
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            let h = c["hex"].as_str().unwrap();
            buffers.push(
                (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let mut context = NativeFilmContext::new(Some(&source));
        let mut p = NativeScanProfile::default();
        p.mpp.lead = seed["widths"][0].as_i64().unwrap();
        p.mpp.index = seed["widths"][1].as_i64().unwrap();
        p.movement.world_object = NativePrecisionDescriptor {
            index_bits: seed["index"].as_u64().unwrap(),
            axis_bits: serde_json::from_value(seed["axes"].clone()).unwrap(),
            region: seed["region"].as_u64().unwrap() as u32,
        };
        p.grammar.simulation_complete = seed["simulation"] == true;
        p.grammar.corruption_check = seed["corruption"] == true;
        let widths = |name: &str| {
            NativeSharedWidths::from_map(
                serde_json::from_value::<BTreeMap<String, i64>>(
                    seed["component_widths"][name].clone(),
                )
                .unwrap(),
            )
        };
        p.grammar.calibrated_widths = Some(widths("calibrated"));
        p.grammar.stub_widths = Some(widths("stubs"));
        context.set_scan_profile(p).unwrap();
        let map: FilmMapBounds = serde_json::from_value(seed["map"].clone()).unwrap();
        let band: BTreeSet<u32> = if row["empty_band"] == true {
            BTreeSet::new()
        } else {
            [512].into()
        };
        let mut attempts = 0;
        let (out, error) = scan_context_ground_weapon_creations_for_band(
            &context,
            (row["no_bounds"] != true).then_some(&map),
            &band,
            |a| {
                attempts += 1;
                assert_eq!(a.read.default_state.name, "default-state-42");
                assert_eq!(meta[a.source.chunk_index as usize].index, a.chunk);
            },
        );
        assert_eq!(
            error.map(|e| e.to_string()).unwrap_or_default(),
            row["error"].as_str().unwrap(),
            "error {case}"
        );
        assert_eq!(json!(out.stats), row["stats"], "stats {case}");
        assert_eq!(attempts, out.stats.anchors);
        let actual: Vec<_> = out
            .records
            .iter()
            .map(|r| r.read.creation.as_ref().unwrap().clone())
            .collect();
        let expected: Vec<EquipmentCreation> =
            serde_json::from_value(row["records"].clone()).unwrap();
        assert_eq!(actual, expected, "records {case}");
        for r in &out.records {
            let c = r.read.creation.as_ref().unwrap();
            let trace = r.read.ammo_read.as_ref().unwrap();
            assert_eq!(trace.ammo.is_some(), c.has_ammo);
            if c.has_ammo {
                assert_eq!(trace.ammo.as_ref(), Some(&c.ammo));
                ammo += 1;
            }
        }
        accepted += actual.len();
    }
    assert!(accepted > 0 && ammo > 0);
}
