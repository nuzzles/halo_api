use super::*;
use std::{collections::BTreeMap, io::Read};
fn hex_bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn native_facts_grapple_placement_document_stage() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-placement-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut totals = [0; 4];
    for (i, row) in rows.iter().enumerate() {
        let stage = &row["stage"];
        let mut doc: ReplayDocument = serde_json::from_value(stage["before"].clone()).unwrap();
        let before = doc.clone();
        let clock: IdentityClock = serde_json::from_value(row["clock"].clone()).unwrap();
        let map: Option<FilmMapBounds> = serde_json::from_value(stage["map"].clone()).unwrap();
        let grapple: Vec<FactsGrappleRead> =
            serde_json::from_value(stage["grapple"].clone()).unwrap();
        let samples: Vec<ReplayPositionSample> =
            serde_json::from_value(row["positions"].clone()).unwrap();
        let positions: Vec<_> = samples
            .iter()
            .map(|p| FactsBipedPosition {
                slot: p.position.slot,
                timestamp_us: p.position.timestamp_us,
                world: [p.position.x, p.position.y, p.position.z],
                has_world: p.position.has_world,
                has_yaw: p.aim.is_some(),
                yaw_raw: p.aim.unwrap_or_default()[0],
                pitch_raw: p.aim.unwrap_or_default()[1],
                ..Default::default()
            })
            .collect();
        let raw: Vec<_> = row["raw"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| FactsPlacement {
                start_us: p["T0US"].as_u64().unwrap(),
                end_us: p["T1US"].as_u64().unwrap(),
                life: serde_json::from_value(p["Life"].clone()).unwrap(),
                global_id: p["GlobalID"].as_u64().unwrap() as u32,
                points: p["Points"].as_i64().unwrap(),
                position: [
                    p["X"].as_f64().unwrap() as f32,
                    p["Y"].as_f64().unwrap() as f32,
                    p["Z"].as_f64().unwrap() as f32,
                ],
            })
            .collect();
        let spawns: Vec<_> = row["spawns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| FactsSpawnEvent {
                timestamp_us: s["time"].as_u64().unwrap(),
                spawned: FactsLifeKey {
                    slot: s["slot"].as_u64().unwrap() as u32,
                    generation: s["generation"].as_u64().unwrap() as u32,
                },
                spawned_valid: s["valid"].as_bool().unwrap(),
                ..Default::default()
            })
            .collect();
        let changes: Vec<_> = row["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FactsEquipmentChange {
                timestamp_us: c["time"].as_u64().unwrap(),
                slot: c["slot"].as_u64().unwrap() as u32,
                kind: hex_bytes(c["kindHex"].as_str().unwrap()),
                ..Default::default()
            })
            .collect();
        let lives: Vec<IdentityLife> = serde_json::from_value(row["lives"].clone()).unwrap();
        let census = FactsWorldKeyframes::from(
            &serde_json::from_value::<WorldObjectKeyframes>(row["census"].clone()).unwrap(),
        );
        let s = &stage["stats"];
        let stats = FactsPlacementStats {
            scanned: s["scanned"].as_bool().unwrap(),
            calibration: FactsMppCalibration {
                widths: serde_json::from_value(s["widths"].clone()).unwrap(),
                ..Default::default()
            },
            lives: s["lives"].as_i64().unwrap(),
            anchors: s["anchors"].as_i64().unwrap(),
            confirmed: s["confirmed"].as_i64().unwrap(),
            ..Default::default()
        };
        let families: BTreeMap<u32, String> =
            serde_json::from_value(stage["families"].clone()).unwrap();
        let pieces = replay_equipment_catalog().spawned_piece_ids;
        let result = assemble_facts_replay_grapple_and_placements(
            &mut doc,
            FactsReplayPlacementDocumentInput {
                grapple: &grapple,
                map: map.as_ref(),
                origin_us: clock.origin_us,
                step_us: clock.step_us,
                placements: FactsReplayEquipmentPlacementInput {
                    raw: &raw,
                    stats: &stats,
                    positions: &positions,
                    census: &census,
                    spawns: &spawns,
                    spawn_lists: stage["spawn_lists"].as_i64().unwrap(),
                    lives: &lives,
                    changes: &changes,
                    families: &families,
                    spawned_piece_ids: &pieces,
                },
            },
        );
        assert_eq!(
            doc,
            serde_json::from_value(stage["after"].clone()).unwrap(),
            "document {i}"
        );
        assert_eq!(
            result.diagnostics,
            serde_json::from_value::<Vec<StatborgDiagnostic>>(stage["diagnostics"].clone())
                .unwrap(),
            "diagnostics {i}"
        );
        assert_eq!(
            result.manifest_fallbacks as u64,
            stage["fallbacks"].as_u64().unwrap(),
            "fallbacks {i}"
        );
        if map.is_none() {
            assert_eq!(doc.content.grapple_lines, before.content.grapple_lines);
            assert_eq!(
                doc.coverage.as_ref().unwrap().grapple,
                before.coverage.as_ref().unwrap().grapple
            );
            totals[3] += 1;
        }
        assert_eq!(
            doc.coverage.as_ref().unwrap().shots,
            before.coverage.as_ref().unwrap().shots
        );
        totals[0] += doc.content.equipment_placements.len();
        totals[1] += doc.content.grapple_lines.len();
        totals[2] += result.manifest_fallbacks;
    }
    assert!(totals.iter().all(|&n| n > 0));
    println!("grapple/placement document stage: {totals:?}");
}
