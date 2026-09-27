use super::*;
use std::io::Read;
#[test]
fn native_facts_pickup_construction() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-pickups-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1024);
    for (i, r) in rows.as_array().unwrap().iter().enumerate() {
        let stats: FactsPickupStats = serde_json::from_value(r["stats"].clone()).unwrap();
        let records: Vec<FactsPickup> = serde_json::from_value(r["input"].clone()).unwrap();
        let clock = serde_json::from_value(r["clock"].clone()).unwrap();
        let equipment = serde_json::from_value(r["equipment"].clone()).unwrap();
        let weapons = serde_json::from_value(r["weapons"].clone()).unwrap();
        let occupant = |slot: u32, ts: u64| {
            if (slot as u64 + ts / 100000).is_multiple_of(3) {
                0
            } else {
                slot as u64 * 100000 + ts / 100000 + 1
            }
        };
        use super::{
            MapSpawnPoint, PickupOriginJudge, PickupOriginPlacement, ReplayPlayerPosition,
        };
        let positions: Vec<_> = records
            .iter()
            .map(|p| ReplayPlayerPosition {
                slot: p.slot,
                timestamp_us: p.timestamp_us,
                x: p.slot as f32,
                y: 0.0,
                z: 0.0,
                has_world: true,
            })
            .collect();
        let judge = PickupOriginJudge::new(
            "established",
            vec![
                MapSpawnPoint {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    kind: "equipment".into(),
                },
                MapSpawnPoint {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                    kind: "grenade".into(),
                },
            ],
            &positions,
            &[
                PickupOriginPlacement {
                    t0: 0,
                    until_max: 8,
                    end: "seen".into(),
                    origin: "dropped".into(),
                    x: 2.0,
                    y: 0.0,
                    z: 0.0,
                },
                PickupOriginPlacement {
                    t0: 2,
                    until_max: 0,
                    end: "open".into(),
                    origin: "dropped".into(),
                    x: 3.0,
                    y: 0.0,
                    z: 0.0,
                },
            ],
        );
        let mut resolve = |slot, ts, frame| judge.resolve(slot, ts, frame);
        let actual = build_facts_replay_pickups(
            &records,
            &stats,
            clock,
            ReplayPickupInputs {
                equipment_families: &equipment,
                weapon_keys: &weapons,
                occupant: if r["occupant"] == true {
                    Some(&occupant)
                } else {
                    None
                },
                origin: if r["origin"] == true {
                    Some(ReplayPickupOrigin {
                        state: &judge.state,
                        catalog_points: judge.catalog_points(),
                        resolve: &mut resolve,
                    })
                } else {
                    None
                },
            },
        );
        let pickups: Vec<ReplayPickup> = serde_json::from_value(r["pickups"].clone()).unwrap();
        let coverage: ReplayPickupCoverage = serde_json::from_value(r["coverage"].clone()).unwrap();
        assert_eq!(actual.pickups, pickups, "pickups {i}");
        assert_eq!(actual.coverage, coverage, "coverage {i}");
    }
}
