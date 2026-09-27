use super::*;
use serde_json::{Value, json};
use std::{collections::BTreeSet, io::Read};
fn inflate(raw: &[u8]) -> Value {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(raw)
        .read_to_end(&mut bytes)
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
#[test]
fn native_context_equipment_creations() {
    let rows = inflate(include_bytes!(
        "fixtures/context-equipment-creations-v41.json.zlib"
    ));
    let seeds = inflate(include_bytes!(
        "fixtures/context-mpp-calibration-v41.json.zlib"
    ));
    let mut accepted = 0;
    assert_eq!(rows.as_array().unwrap().len(), 64);
    for (case, row) in rows.as_array().unwrap().iter().enumerate() {
        let seed = &seeds[row["seed"].as_u64().unwrap() as usize];
        let mut buffers = vec![vec![0; 16]];
        let mut meta = vec![FilmSourceMetadata {
            index: -1,
            chunk_type: 0,
            start_ms: 0,
        }];
        for c in seed["inputs"].as_array().unwrap() {
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
        let mut profile = NativeScanProfile::default();
        profile.mpp.lead = row["lead"].as_i64().unwrap();
        profile.mpp.index = row["index"].as_i64().unwrap();
        profile.movement.world_object = NativePrecisionDescriptor {
            index_bits: 1,
            axis_bits: [16, 17, 18],
            region: 0,
        };
        context.set_scan_profile(profile).unwrap();
        let band: BTreeSet<u32> = if seed["empty_band"] == true {
            BTreeSet::new()
        } else {
            [512, 513, 515].into()
        };
        let map = FilmMapBounds {
            module: String::new(),
            min: [-100., -200., -300.],
            max: [300., 400., 500.],
            axis_widths: [16, 17, 18],
            region: 0,
            region_index_bits: 1,
        };
        let mut attempts = 0;
        let (out, error) = scan_context_equipment_creations_for_band(
            &context,
            (seed["no_bounds"] != true).then_some(&map),
            &band,
            |a| {
                attempts += 1;
                assert_eq!(meta[a.source.chunk_index as usize].index, a.chunk);
                assert_eq!(a.read.default_state.start_bit, a.bit as i64 + 24);
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
        let facts: Vec<_> = actual.iter().map(FactsEquipmentCreation::from).collect();
        let mut writer = NativeFactsWriter::default();
        encode_facts_creations(&mut writer, &facts);
        // The independent native records validate the projection fields; the codec
        // has its own pinned oracle and is exercised here only for integration.
        assert_eq!(facts.len(), actual.len());
        for (f, c) in facts.iter().zip(&actual) {
            assert_eq!(f.chunk, c.chunk);
            assert_eq!(f.position, [c.x, c.y, c.z]);
            assert_eq!(f.mpp_val, c.mpp_val);
        }
        accepted += actual.len();
    }
    assert_eq!(accepted, 1053);
}
