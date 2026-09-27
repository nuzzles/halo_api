use super::*;
use serde_json::Value;
use std::io::Read;
fn scan(v: &Value) -> FactsWorldObjectScan {
    FactsWorldObjectScan {
        scanned: v["scanned"].as_bool().unwrap(),
        stats: serde_json::from_value(v["stats"].clone()).unwrap(),
        keyframes: FactsWorldKeyframes::from(
            &serde_json::from_value::<WorldObjectKeyframes>(v["keyframes"].clone()).unwrap(),
        ),
        creations: v["creations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FactsEquipmentCreation {
                slot: c["Slot"].as_u64().unwrap() as u32,
                generation: c["Gen"].as_u64().unwrap() as u32,
                timestamp_us: c["TimestampUS"].as_u64().unwrap(),
                position: [
                    c["X"].as_f64().unwrap() as f32,
                    c["Y"].as_f64().unwrap() as f32,
                    c["Z"].as_f64().unwrap() as f32,
                ],
                mpp_present: serde_json::from_value(c["MPPPresent"].clone()).unwrap(),
                mpp_val: serde_json::from_value(c["MPPVal"].clone()).unwrap(),
                has_ammo: c["HasAmmo"].as_bool().unwrap(),
                ammo: serde_json::from_value(c["Ammo"].clone()).unwrap(),
                ..Default::default()
            })
            .collect(),
        tracks: v["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| FactsProjectileTrack {
                slot: t["Slot"].as_u64().unwrap() as u32,
                generation: t["Gen"].as_u64().unwrap() as u32,
                points: t["Pts"].as_array().map(|pts| {
                    pts.iter()
                        .map(|p| FactsProjectileSample {
                            timestamp_us: p["TimestampUS"].as_u64().unwrap(),
                            chunk: p["Chunk"].as_i64().unwrap(),
                            position: [
                                p["X"].as_f64().unwrap() as f32,
                                p["Y"].as_f64().unwrap() as f32,
                                p["Z"].as_f64().unwrap() as f32,
                            ],
                            at_rest: p["AtRest"].as_bool().unwrap(),
                        })
                        .collect()
                }),
            })
            .collect(),
    }
}
fn positions(v: &Value) -> Vec<FactsBipedPosition> {
    serde_json::from_value::<Vec<ReplayPlayerPosition>>(v.clone())
        .unwrap()
        .iter()
        .map(|p| FactsBipedPosition {
            slot: p.slot,
            timestamp_us: p.timestamp_us,
            world: [p.x, p.y, p.z],
            has_world: p.has_world,
            ..Default::default()
        })
        .collect()
}
#[test]
fn native_facts_pickup_pad_document_stage() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-pickup-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut totals = [0; 6];
    for (i, row) in rows.iter().enumerate() {
        let s = &row["stage"];
        let mut doc: ReplayDocument = serde_json::from_value(s["before"].clone()).unwrap();
        let old_shots = doc.coverage.as_ref().unwrap().shots.clone();
        let weapons = scan(&row["weapons"]);
        let powerups = scan(&row["powerups"]);
        let original = positions(&s["original_positions"]);
        let sorted = positions(&s["sorted_positions"]);
        let picks: Vec<FactsPickup> = serde_json::from_value(s["pickup_input"].clone()).unwrap();
        let stats: FactsPickupStats = serde_json::from_value(s["pickup_stats"].clone()).unwrap();
        let changes: Vec<_> = s["weapon_changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FactsWeaponChange {
                timestamp_us: c["timestamp"].as_u64().unwrap(),
                slot: c["slot"].as_u64().unwrap() as u32,
                family: c["family"].as_u64().unwrap() as u32,
                previous: c["previous"].as_u64().unwrap() as u32,
                kind: c["kind_hex"]
                    .as_str()
                    .unwrap()
                    .as_bytes()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                    .collect(),
                ..Default::default()
            })
            .collect();
        let points =
            serde_json::from_value::<Vec<MapSpawnPoint>>(s["spawn_points"].clone()).unwrap();
        let catalog = ReplayEquipmentCatalog {
            families: serde_json::from_value(row["families"].clone()).unwrap(),
            objective_objects: serde_json::from_value(row["objectives"].clone()).unwrap(),
            weapon_keys: serde_json::from_value(s["weapon_keys"].clone()).unwrap(),
            ..Default::default()
        };
        let identity: ReplayIdentityState = serde_json::from_value(s["identity"].clone()).unwrap();
        let clock: GroundPadClock = serde_json::from_value(row["clock"].clone()).unwrap();
        let result = assemble_facts_replay_pickups_and_pads(
            &mut doc,
            FactsReplayPickupDocumentInput {
                weapon_changes: &changes,
                pickups: &picks,
                pickup_stats: &stats,
                positions: &original,
                sorted_positions: &sorted,
                weapons: &weapons,
                powerups: &powerups,
                spawn_points: &points,
                spawn_points_state: s["spawn_state"].as_str().unwrap(),
                catalog: &catalog,
                identity: &identity,
                origin_us: clock.origin_us,
                step_us: clock.step_us,
            },
        );
        assert_eq!(
            doc,
            serde_json::from_value(s["after"].clone()).unwrap(),
            "document {i}"
        );
        assert_eq!(
            result.weapon_objects,
            serde_json::from_value::<Vec<GroundPickupObject>>(s["objects"].clone()).unwrap(),
            "retained objects {i}"
        );
        assert_eq!(
            result.diagnostics,
            serde_json::from_value::<Vec<StatborgDiagnostic>>(s["diagnostics"].clone()).unwrap(),
            "diagnostics {i}"
        );
        assert_eq!(
            s["fallbacks"],
            serde_json::json!([]),
            "native pass has no fallback hits {i}"
        );
        let c = doc.coverage.as_ref().unwrap();
        assert_eq!(c.shots, old_shots);
        totals[0] += doc.content.pickups.len();
        totals[1] += doc.content.weapon_pads.len();
        totals[2] += c.pad_dating.as_ref().unwrap().dated;
        totals[3] += c.pad_dating.as_ref().unwrap().ambiguous;
        totals[4] += c.pickups.as_ref().unwrap().origin_ground;
        totals[5] += c.pickups.as_ref().unwrap().origin_spawner;
    }
    assert!(totals.iter().all(|&n| n > 0));
    println!("pickup/pad document stage: {totals:?}");
}
