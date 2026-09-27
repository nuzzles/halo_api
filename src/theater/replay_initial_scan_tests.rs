use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_initial_scan_phase() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/initial-scan-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 32);
    let (mut successes, mut logs_count) = (0, 0);
    let mut counts = [0; 5];
    for (i, row) in rows.iter().enumerate() {
        let chunks = row["chunks"].as_array().unwrap();
        let buffers: Vec<_> = chunks
            .iter()
            .map(|c| unhex(c["hex"].as_str().unwrap()))
            .collect();
        let metadata: Vec<_> = chunks
            .iter()
            .map(|c| FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            })
            .collect();
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &metadata).unwrap());
        let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
        let forced = (!row["forced"].is_null()).then(|| I0Layout {
            gate_bits: row["forced"]["GateBits"].as_i64().unwrap(),
            axis_widths: serde_json::from_value(row["forced"]["AxisW"].clone()).unwrap(),
            region: row["forced"]["Region"].as_u64().unwrap() as u32,
        });
        let context =
            NativeFilmContext::for_map(source.as_ref(), Some(&map), forced.as_ref()).unwrap();
        let selection: Vec<i64> = serde_json::from_value(row["selection"].clone()).unwrap();
        let scan = SourceWorldScanOptions {
            scan: BipedScanOptions {
                capture_dirs: false,
                drop_saturated: false,
                isolation_gap_us: 0,
                max_speed: row["speed"].as_f64().unwrap(),
                ..BipedScanOptions::native_defaults()
            },
            dynamic_forward_level: None,
            teleport_exemptions: [(999, vec![123])].into(),
        };
        let mut steps = Vec::new();
        let mut result = None;
        let logs = super::log_test_support::capture_logs(|| {
            result = Some(scan_replay_initial_inputs(
                b"phase",
                &context,
                &map,
                ReplayInitialScanOptions {
                    chunks: &selection,
                    scan: Some(&scan),
                },
                |o| steps.push(o.name()),
            ));
        });
        logs_count += logs.len();
        assert_eq!(json!(logs), row["logs"], "logs {i}");
        assert_eq!(json!(steps), row["steps"], "steps {i}");
        let result = result.unwrap();
        assert_eq!(result.is_err(), row["error"] != "", "outcome {i}");
        let out = match result {
            Ok(out) => out,
            Err(error) => {
                let expected = row["error"].as_str().unwrap();
                let category = if expected.contains("aucun chunk de donnees") {
                    "film chunks"
                } else if expected.contains("aucun slot biped") {
                    "biped slots"
                } else if expected.contains("i0") {
                    "i0 layout"
                } else {
                    panic!("unclassified native error {expected}")
                };
                assert!(
                    matches!(error,ContextPositionScanError::Scan(DecodeError::Missing(s)) if s==category),
                    "error {i}: {error}"
                );
                continue;
            }
        };
        successes += 1;
        let input = &row["input"];
        assert_eq!(json!(out.major_version), input["FilmMajorVersion"]);
        let mut facts = NativeFilmFacts::default();
        out.apply_to_facts(&mut facts);
        let map_entry = FactsMapEntry {
            module: map.module.as_bytes().to_vec(),
            axis_widths: map.axis_widths.map(|w| w as u64),
            region: map.region,
            region_index_bits: map.region_index_bits as u64,
            bounds: std::array::from_fn(|a| [map.min[a], map.max[a]]),
        };
        let captured = capture_film_scan_facts(
            b"phase",
            &context,
            Some(&map_entry),
            &facts,
            &FactsModeGuards::default(),
            None,
            None,
        );
        if row["facts"].is_null() {
            assert!(captured.is_none());
        } else {
            assert_eq!(
                encode_film_facts_file(&captured.unwrap()).unwrap(),
                unhex(row["facts"].as_str().unwrap()),
                "complete captured cache bytes {i}"
            );
        }
        let fire: Vec<FilmFireEvent> =
            serde_json::from_value(input["Fire"].as_array().cloned().unwrap_or_default().into())
                .unwrap();
        assert_eq!(
            out.fire.iter().map(|e| e.event.clone()).collect::<Vec<_>>(),
            fire
        );
        let loadouts: Vec<KeyframeLoadout> = serde_json::from_value(
            input["Loadouts"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into(),
        )
        .unwrap();
        assert_eq!(out.loadouts, loadouts);
        for (p, e) in facts
            .positions
            .iter()
            .zip(input["Positions"].as_array().cloned().unwrap_or_default())
        {
            assert_eq!(p.directions.mask_bits, e["MaskBits"].as_u64().unwrap());
            assert_eq!(p.directions.mask_over, e["MaskOver"].as_bool().unwrap());
        }
        let creations:Vec<_>=out.creations.records.iter().map(|r| {let c=&r.creation;json!({"Slot":c.slot,"Generation":c.generation,"ParticipantIndex":c.participant_index,"HasIndex":true,"Chunk":metadata[r.source.chunk_index as usize].index,"PacketIndex":r.packet_index.unwrap(),"TimestampUS":r.source.timestamp_us,"BitPos":c.start_bit,"Version":c.version,"Representation":c.representation})}).collect();
        assert_eq!(
            creations,
            input["BipedCreations"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        );
        for (j, n) in [
            out.positions.positions.len(),
            out.creations.records.len(),
            out.fire.len(),
            out.loadouts.len(),
            out.translocations.len(),
        ]
        .into_iter()
        .enumerate()
        {
            counts[j] += n;
        }
    }
    assert_eq!(successes, 16);
    assert_eq!(logs_count, 76);
    assert_eq!(counts, [708, 5, 8, 24, 8]);
}
