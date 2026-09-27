use super::*;
use serde_json::{Value, json};
use std::{collections::BTreeSet, io::Read};
fn fixture(raw: &[u8]) -> Value {
    let mut b = Vec::new();
    flate2::read::ZlibDecoder::new(raw)
        .read_to_end(&mut b)
        .unwrap();
    serde_json::from_slice(&b).unwrap()
}
#[test]
fn native_context_vehicle_creations() {
    let rows = fixture(include_bytes!(
        "fixtures/context-vehicle-creations-v41.json.zlib"
    ));
    let seeds = fixture(include_bytes!(
        "fixtures/vehicle-creation-profile-v41.json.zlib"
    ));
    assert_eq!(rows.as_array().unwrap().len(), 128);
    let mut accepted = 0;

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
        let map: FilmMapBounds = serde_json::from_value(seed["map"].clone()).unwrap();
        let layout = map.i0_layout();
        let mut context = NativeFilmContext::with_imposed_layout(
            Some(&source),
            (row["forced"] == true).then_some(&layout),
        );
        let mut p = NativeScanProfile::default();
        p.mpp.lead = seed["widths"][0].as_i64().unwrap();
        p.mpp.index = seed["widths"][1].as_i64().unwrap();
        p.movement.world_object = NativePrecisionDescriptor {
            index_bits: map.region_index_bits.max(1) as u64,
            axis_bits: serde_json::from_value(seed["profile_axes"].clone()).unwrap(),
            region: map.region,
        };
        context.set_scan_profile(p).unwrap();
        let band: BTreeSet<u32> = if row["empty_band"] == true {
            BTreeSet::new()
        } else {
            [512, 514, 515].into()
        };
        let mut attempts = 0;
        let (out, error) = scan_context_vehicle_creations_for_band(
            &context,
            (row["no_bounds"] != true).then_some(&map),
            &band,
            |a| {
                attempts += 1;
                assert_eq!(a.read.default_state.name, "default-state-40");
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
            assert!(r.read.ammo_read.is_none());
            assert_eq!(
                c.after_bit as i64,
                r.read.component_start.unwrap() + layout.total_bits()
            );
        }
        accepted += actual.len();
    }
    assert!(accepted > 0);
}
