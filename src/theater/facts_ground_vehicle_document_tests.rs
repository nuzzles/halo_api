use super::*;
use serde_json::Value;
use std::{collections::BTreeMap, io::Read};

#[test]
fn native_facts_ground_vehicle_document_stage() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-ground-vehicle-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let identity = ReplayIdentityState::from_lives(
        [(0, 0, 900000, 1), (1, 0, 300000, 2), (1, 400000, 900000, 3)]
            .into_iter()
            .map(|(slot, from, to, xuid)| IdentityLife {
                slot,
                from,
                to,
                xuid,
                ..Default::default()
            })
            .collect(),
        &[(1, 0), (2, 1), (3, 2)].into(),
    );
    let mut recovered = 0;
    for (i, row) in rows.iter().enumerate() {
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        let bytes: Vec<_> = row["scan_wire"]
            .as_str()
            .unwrap()
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect();
        let mut reader = NativeFactsReader::new(&bytes);
        let scan = decode_facts_vehicle_scan(
            &mut reader,
            &I0Layout {
                axis_widths: [8, 8, 8],
                gate_bits: 0,
                region: 0,
            },
            [[0.; 3], [128.; 3]],
        );
        assert!(reader.error().is_none(), "scan {i}: {:?}", reader.error());
        assert_eq!(reader.offset(), bytes.len(), "scan boundary {i}");
        let positions: Vec<ReplayPlayerPosition> =
            serde_json::from_value(row["positions"].clone()).unwrap();
        let positions: Vec<_> = positions
            .iter()
            .map(|p| FactsBipedPosition {
                slot: p.slot,
                timestamp_us: p.timestamp_us,
                world: [p.x, p.y, p.z],
                has_world: p.has_world,
                ..Default::default()
            })
            .collect();
        let objects: Vec<GroundPickupObject> =
            serde_json::from_value(row["objects"].clone()).unwrap();
        let changes: Vec<_> = row["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FactsWeaponChange {
                timestamp_us: c["TimestampUS"].as_u64().unwrap(),
                slot: c["Slot"].as_u64().unwrap() as u32,
                family: c["Family"].as_u64().unwrap() as u32,
                previous: c["Previous"].as_u64().unwrap() as u32,
                kind: c["Kind"].as_str().unwrap().as_bytes().to_vec(),
                ..Default::default()
            })
            .collect();
        let orphans: Vec<_> = row["orphans"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| {
                let e = &o["event"];
                FactsReplayOrphanShot {
                    event: FactsFireEvent {
                        timestamp_us: e["TimestampUS"].as_u64().unwrap(),
                        film_index: e["FilmIndex"].as_i64().unwrap(),
                        weapon_id: e["WeaponID"].as_u64().unwrap(),
                        has_aim: e["HasAim"].as_bool().unwrap(),
                        aim: serde_json::from_value(e["Aim"].clone()).unwrap(),
                    },
                    reason: serde_json::from_value(o["reason"].clone()).unwrap(),
                }
            })
            .collect();
        let mut fallbacks = BTreeMap::new();
        let logs = super::log_test_support::capture_logs(|| {
            fallbacks = assemble_facts_replay_ground_and_vehicles(
                &mut doc,
                FactsReplayGroundVehicleDocumentInput {
                    weapon_objects: &objects,
                    weapon_changes: &changes,
                    sorted_positions: &positions,
                    vehicles: &scan,
                    identity: &identity,
                    shot_orphans: &orphans,
                    origin_us: row["origin"].as_u64().unwrap(),
                    step_us: row["step"].as_u64().unwrap(),
                },
            );
        });
        assert_eq!(doc, expected, "whole stage {i}");
        assert_eq!(
            serde_json::to_value(fallbacks).unwrap(),
            row["fallbacks"],
            "fallbacks {i}"
        );
        assert_eq!(
            serde_json::to_value(logs).unwrap(),
            row["logs"],
            "ordered runtime logs {i}"
        );
        recovered += doc.content.shots.len() - 1;
    }
    assert_eq!(recovered, 1446);
}
