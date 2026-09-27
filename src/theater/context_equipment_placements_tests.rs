use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_equipment_placements() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-placements-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 48);
    let map = FilmMapBounds {
        module: String::new(),
        min: [-100., -200., -300.],
        max: [300., 400., 500.],
        axis_widths: [16, 17, 18],
        region: 0,
        region_index_bits: 1,
    };
    let mut total = 0;
    let mut scanned = 0;
    for (case, row) in rows.iter().enumerate() {
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
        let mut profile = NativeScanProfile {
            mpp: FilmMppWidths { lead: 11, index: 7 },
            ..Default::default()
        };
        profile.movement.world_object = NativePrecisionDescriptor {
            index_bits: 1,
            axis_bits: [16, 17, 18],
            region: 0,
        };
        context.set_scan_profile(profile).unwrap();
        let mut calibration_attempts = 0;
        let mut creation_attempts = 0;
        let (out, error) = scan_context_equipment_placements(
            &mut context,
            (row["no_bounds"] != true).then_some(&map),
            |o| match o {
                ContextPlacementObservation::Calibration(_) => calibration_attempts += 1,
                ContextPlacementObservation::Creation(_) => creation_attempts += 1,
            },
        );
        assert_eq!(
            error.map(|e| e.to_string()).unwrap_or_default(),
            row["error"].as_str().unwrap(),
            "error {case}"
        );
        let mut expected = row["stats"].clone();
        expected["Calibration"] = row["calibration"].clone();
        let expected: EquipmentPlacementStats = serde_json::from_value(expected).unwrap();
        assert_eq!(out.stats, expected, "stats {case}");
        let placements: Vec<EquipmentPlacement> =
            serde_json::from_value(row["placements"].clone()).unwrap();
        assert_eq!(out.placements, placements, "placements {case}");
        assert_eq!(
            context.scan_profile().unwrap().mpp,
            FilmMppWidths { lead: 11, index: 7 }
        );
        assert_eq!(row["restored"], true);
        assert_eq!(calibration_attempts, out.stats.calibration.anchors * 63);
        if let Some(c) = &out.creations {
            assert_eq!(creation_attempts, c.stats.anchors);
            assert_eq!(
                out.creation_widths,
                Some(FilmMppWidths { lead: 9, index: 5 })
            );
        }
        // These films deliberately leave measured calibration inconclusive.
        // Format27 nevertheless decodes creations; published widths must stay zero.
        if case < 32 {
            assert_eq!(out.stats.calibration.widths, EquipmentMppWidths::default());
        }
        let facts = FactsPlacementStats::from(&out.stats);
        assert_eq!(
            facts.calibration.widths,
            (
                out.stats.calibration.widths.lead as i64,
                out.stats.calibration.widths.index as i64
            )
        );
        assert_eq!(facts.placements, out.placements.len() as i64);
        for p in &out.placements {
            let f = FactsPlacement::from(p);
            assert_eq!(
                json!({"Life":f.life,"T0US":f.start_us,"T1US":f.end_us,"X":f.position[0],"Y":f.position[1],"Z":f.position[2],"GlobalID":f.global_id,"Points":f.points}),
                json!(p)
            );
        }
        total += out.placements.len();
        scanned += usize::from(out.stats.scanned);
    }
    assert_eq!((total, scanned), (48, 30));
}
