use super::*;
use serde_json::Value;
use std::{collections::BTreeMap, io::Read};

fn native_position(e: &Value) -> FactsBipedPosition {
    FactsBipedPosition {
        timestamp_us: e["TimestampUS"].as_u64().unwrap(),
        slot: e["Slot"].as_u64().unwrap() as u32,
        has_world: e["HasWorld"].as_bool().unwrap(),
        quantized: serde_json::from_value(e["Q"].clone()).unwrap(),
        world: ["X", "Y", "Z"].map(|k| e[k].as_f64().unwrap() as f32),
        has_yaw: e["HasYaw"].as_bool().unwrap(),
        yaw_raw: e["YawRaw"].as_u64().unwrap() as u32,
        pitch_raw: e["PitchRaw"].as_u64().unwrap() as u32,
        directions: serde_json::from_value(e.clone()).unwrap(),
        has_body: e["HasBody"].as_bool().unwrap(),
        health: e["Body"]["Health"].as_f64().unwrap() as f32,
        has_shield: e["HasShield"].as_bool().unwrap(),
        shield: e["Shield"]["Shield"].as_f64().unwrap() as f32,
        shield_quantum: e["Shield"]["Q"].as_u64().unwrap() as u8,
    }
}

#[test]
fn native_scanned_positions_to_facts() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/record-mask-hook-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 256);
    let (mut quanta_count, mut world_count, mut error_count) = (0, 0, 0);
    let mut positive = [0; 8];
    for (row_index, row) in rows.iter().enumerate() {
        let hex = row["source_hex"].as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
            .collect();
        let numbers: Vec<i64> = serde_json::from_value(row["source_numbers"].clone()).unwrap();
        let metadata: Vec<_> = numbers
            .into_iter()
            .map(|index| FilmSourceMetadata {
                index,
                chunk_type: 2,
                start_ms: 0,
            })
            .collect();
        let source = FilmSource::load(&[Vec::new(), bytes.clone(), bytes], &metadata).unwrap();
        let flags: BTreeMap<u32, bool> =
            serde_json::from_value(row["source_slots"].clone()).unwrap();
        let band = flags
            .into_iter()
            .filter_map(|(slot, present)| present.then_some(slot))
            .collect();
        let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
        let layout = map.i0_layout();
        let options = BipedScanOptions {
            capture_dirs: row["capture"].as_bool().unwrap(),
            drop_saturated: row["drop"].as_bool().unwrap(),
            isolation_gap_us: row["gap"].as_u64().unwrap(),
            max_speed: row["speed"].as_f64().unwrap(),
            ..BipedScanOptions::native_defaults()
        };
        for case in row["source_cases"].as_array().unwrap() {
            let chunks: Vec<i64> = serde_json::from_value(case["chunks"].clone()).unwrap();
            let dynamic = row["dynamic"].as_bool().unwrap().then_some(2);
            let quantized = scan_source_quantized_position_report_for_band(
                &source, &chunks, &band, &layout, &options, dynamic,
            );
            let context = NativeFilmContext::with_imposed_layout(
                Some(&source),
                Some(&I0Layout {
                    gate_bits: 0,
                    axis_widths: [0; 3],
                    region: 0,
                }),
            );
            let hooks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            context.observation().set_hook(
                NativeHookKind::RecordMask,
                Some({
                    let hooks = hooks.clone();
                    std::sync::Arc::new(move |_| {
                        hooks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    })
                }),
            );
            let world = scan_context_world_positions(
                &context,
                &chunks,
                Some(&band),
                Some(&layout),
                [map.min, map.max],
                &SourceWorldScanOptions {
                    scan: options.clone(),
                    dynamic_forward_level: dynamic,
                    teleport_exemptions: serde_json::from_value(case["exemptions"].clone())
                        .unwrap(),
                },
            );
            assert_eq!(
                hooks.load(std::sync::atomic::Ordering::Relaxed),
                case["world_masks"].as_array().unwrap().len()
            );
            if case["error"] != "" {
                assert!(quantized.is_err());
                assert!(world.is_err());
                error_count += 1;
                continue;
            }
            let quantized = quantized.unwrap();
            let world = world.unwrap();
            let qfacts = facts_from_quantized_position_scan(&quantized, options.capture_dirs);
            let wfacts = facts_from_world_position_scan(&world, options.capture_dirs);
            let expected_q: Vec<_> = case["records"]
                .as_array()
                .unwrap()
                .iter()
                .map(native_position)
                .collect();
            let expected_w: Vec<_> = case["world_records"]
                .as_array()
                .unwrap()
                .iter()
                .map(native_position)
                .collect();
            assert_eq!(
                qfacts, expected_q,
                "quanta row {row_index}, chunks {chunks:?}"
            );
            assert_eq!(
                wfacts, expected_w,
                "world row {row_index}, chunks {chunks:?}"
            );
            quanta_count += qfacts.len();
            world_count += wfacts.len();
            for p in &wfacts {
                let d = &p.directions;
                for (i, present) in [
                    d.has_aim,
                    d.has_vel,
                    p.has_yaw,
                    d.has_aim_b,
                    d.has_roll,
                    d.aim_default,
                    p.has_body,
                    p.has_shield,
                ]
                .into_iter()
                .enumerate()
                {
                    positive[i] += usize::from(present);
                }
            }
            if !options.capture_dirs {
                assert!(
                    qfacts
                        .iter()
                        .all(|p| p.directions == FactsPositionDirections::default())
                );
            }
            // These checks expose cache omissions instead of accidentally treating
            // the derived cache as a lossless representation of the source scan.
            for facts in [&qfacts, &wfacts] {
                let mut writer = NativeFactsWriter::default();
                encode_facts_positions(&mut writer, facts);
                let mut reader = NativeFactsReader::new(writer.bytes());
                let decoded = decode_facts_positions(&mut reader, &layout, [map.min, map.max]);
                assert!(reader.error().is_none());
                assert_eq!(reader.remaining(), 0);
                for (actual, original) in decoded.iter().zip(facts.iter()) {
                    let mut expected = original.clone();
                    expected.directions.mask_bits = 0;
                    if !expected.has_world {
                        expected.quantized = [0; 3];
                    }
                    // HasRoll gates persistence of mode/default even though the
                    // in-memory scan DTO can carry them on a delta path.
                    if !expected.directions.has_roll {
                        expected.directions.fwd_mode = 0;
                        expected.directions.aim_default = false;
                    }
                    assert_eq!(*actual, expected);
                }
            }
        }
    }
    assert_eq!((quanta_count, world_count), (19484, 16914));
    assert!(error_count > 0);
    assert_eq!(positive, [2262, 612, 6678, 4044, 422, 188, 1738, 1738]);
}
