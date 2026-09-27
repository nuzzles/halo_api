use super::*;
use std::{collections::BTreeMap, io::Read, num::NonZeroU64};
fn event(v: &serde_json::Value) -> FactsFireEvent {
    FactsFireEvent {
        timestamp_us: v["TimestampUS"].as_u64().unwrap(),
        film_index: v["FilmIndex"].as_i64().unwrap(),
        weapon_id: v["WeaponID"].as_u64().unwrap(),
        has_aim: v["HasAim"].as_bool().unwrap(),
        aim: serde_json::from_value(v["Aim"].clone()).unwrap(),
    }
}
#[test]
fn native_facts_combat_document_stage() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-combat-stage-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut totals = [0; 5];
    for (i, row) in rows.iter().enumerate() {
        let positions =
            serde_json::from_value::<Vec<ReplayPlayerPosition>>(row["positions"].clone()).unwrap();
        let owners = serde_json::from_value::<BTreeMap<u32, i64>>(row["owners"].clone()).unwrap();
        let events = FactsEvents {
            fire: row["fire"].as_array().unwrap().iter().map(event).collect(),
            loadouts: row["loadouts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| FactsLoadout {
                    timestamp_us: v["TimestampUS"].as_u64().unwrap(),
                    slot: v["Slot"].as_u64().unwrap() as u32,
                    families: serde_json::from_value(v["Families"].clone()).unwrap(),
                })
                .collect(),
            grenades: row["throws"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| FactsGrenadeThrow {
                    timestamp_us: v["TimestampUS"].as_u64().unwrap(),
                    film_index: v["FilmIndex"].as_i64().unwrap(),
                    type_id: v["TypeID"].as_u64().unwrap() as u32,
                })
                .collect(),
            projectiles: row["projectiles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| FactsProjectileTrack {
                    slot: v["Slot"].as_u64().unwrap() as u32,
                    generation: v["Gen"].as_u64().unwrap() as u32,
                    points: v["Pts"].as_array().map(|pts| {
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
        };
        let original = events.clone();
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        let out = assemble_facts_replay_combat(
            &mut doc,
            &events,
            FactsReplayCombatContext {
                positions: &positions,
                owners: &owners,
                origin_us: row["origin"].as_u64().unwrap(),
                step_us: NonZeroU64::new(row["step"].as_u64().unwrap()).unwrap(),
            },
        );
        assert_eq!(doc, expected, "combat document {i}");
        assert_eq!(
            out.shots,
            serde_json::from_value::<ReplayLayerCoverage>(row["shot_cov"].clone()).unwrap(),
            "shot coverage {i}"
        );
        assert_eq!(
            out.grenades,
            serde_json::from_value::<ReplayLayerCoverage>(row["grenade_cov"].clone()).unwrap(),
            "grenade coverage {i}"
        );
        assert_eq!(
            out.projectiles,
            serde_json::from_value::<Option<ReplayProjectileCoverage>>(
                row["projectile_cov"].clone()
            )
            .unwrap(),
            "projectile coverage {i}"
        );
        let orphans: Vec<_> = row["orphans"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| FactsReplayOrphanShot {
                event: event(&o["event"]),
                reason: serde_json::from_value(o["reason"].clone()).unwrap(),
            })
            .collect();
        assert_eq!(out.shot_orphans, orphans, "orphans {i}");
        assert_eq!(events, original, "cache input mutation {i}");
        totals[0] += doc.content.shots.len();
        totals[1] += doc.content.loadouts.len();
        totals[2] += doc.content.projectiles.len();
        totals[3] += doc.content.grenades.len();
        totals[4] += out.projectiles.as_ref().map_or(0, |c| c.truncated);
    }
    assert!(totals.iter().all(|&n| n > 0));
    println!("combat shots/loadouts/projectiles/grenades/truncations: {totals:?}");
}
