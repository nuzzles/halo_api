use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn widths(v: &Value) -> FilmMppWidths {
    FilmMppWidths {
        lead: v["Lead"].as_i64().unwrap(),
        index: v["Index"].as_i64().unwrap(),
    }
}
#[test]
fn native_vehicle_scan_phase() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/vehicle-scan-composed-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 128);
    let (mut scanned, mut creations, mut positions) = (0, 0, 0);
    let (mut deaths, mut occupancy, mut aims) = (0, 0, 0);
    for (case, row) in rows.iter().enumerate() {
        assert_eq!(row["panicked"], false, "native panic {case}");
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            buffers.push(unhex(c["hex"].as_str().unwrap()));
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
        let layout = map.i0_layout();
        let mut context = NativeFilmContext::with_imposed_layout(
            Some(&source),
            (row["forced"] == true).then_some(&layout),
        );
        let mut p = NativeScanProfile {
            mpp: widths(&row["inherited"]),
            ..Default::default()
        };
        p.movement.world_object = NativePrecisionDescriptor {
            index_bits: map.region_index_bits.max(1) as u64,
            axis_bits: serde_json::from_value(row["axes"].clone()).unwrap(),
            region: map.region,
        };
        p.grammar.generation_strict = row["strict"] == true;
        p.grammar.simulation_complete = row["simulation"] == true;
        p.grammar.corruption_check = row["corruption"] == true;
        context.set_scan_profile(p).unwrap();
        let mut result = None;
        let mut attempts = 0;
        let logs = super::log_test_support::capture_logs(|| {
            result = Some(scan_replay_vehicle_inputs(
                b"oracle-vehicle",
                &mut context,
                (row["no_bounds"] != true).then_some(&map),
                widths(&row["calibrated"]),
                |a| {
                    attempts += 1;
                    assert_eq!(meta[a.source.chunk_index as usize].index, a.chunk);
                },
            ));
        });
        let logs: Vec<_> = logs
            .into_iter()
            .filter(|v| {
                v["msg"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("vehicules :")
            })
            .collect();
        let out = result.unwrap();
        assert_eq!(json!(logs), row["logs"], "logs {case}");
        assert_eq!(
            context.scan_profile().unwrap().mpp,
            widths(&row["inherited"])
        );
        assert_eq!(row["restored"], true);
        let mut writer = NativeFactsWriter::default();
        encode_facts_vehicle_scan(&mut writer, &out.published);
        assert!(writer.error().is_none());
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "complete vehicle cache {case}"
        );
        assert_eq!(out.published.scanned, row["scanned"] == true);
        assert_eq!(
            attempts,
            out.creations.as_ref().map_or(0, |c| c.stats.anchors)
        );
        let expected = row["positions"].as_array().cloned().unwrap_or_default();
        assert_eq!(out.published.positions.len(), expected.len());
        for (p, e) in out.published.positions.iter().zip(expected) {
            assert_eq!(
                p.world,
                ["X", "Y", "Z"].map(|k| e[k].as_f64().unwrap() as f32),
                "world {case}"
            );
            assert_eq!(p.directions.mask_bits, e["MaskBits"].as_u64().unwrap());
            assert_eq!(p.directions.mask_over, e["MaskOver"] == true);
        }
        if out.published.scanned {
            scanned += 1;
        } else {
            assert_eq!(out.published, FactsVehicleScan::default());
        }
        creations += out.published.creations.len();
        positions += out.published.positions.len();
        deaths += out.published.deaths.deaths.as_ref().map_or(0, Vec::len);
        occupancy += out.published.occupancy.len();
        aims += out.published.aims.len();
    }
    assert!(scanned > 0 && creations > 0 && positions > 0);
    assert!(deaths > 0 && occupancy > 0 && aims > 0);
}
