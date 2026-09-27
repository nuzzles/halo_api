use super::*;
use serde_json::Value;
use std::{io::Read, num::NonZeroU64};

#[test]
fn native_facts_inventory_document_stage() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-inventory-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut marked_total = 0;
    for (i, r) in rows.iter().enumerate() {
        let mut doc: ReplayDocument = serde_json::from_value(r["before"].clone()).unwrap();
        let expected: ReplayDocument = serde_json::from_value(r["after"].clone()).unwrap();
        let identity = ReplayIdentityState::from_lives(
            serde_json::from_value(r["lives"].clone()).unwrap(),
            &serde_json::from_value(r["indices"].clone()).unwrap(),
        );
        let inventory: Option<Vec<_>> = r["raw"].as_array().map(|rows| {
            rows.iter()
                .map(|v| FactsKeyframeInventory {
                    timestamp_us: v["TimestampUS"].as_u64().unwrap(),
                    slot: v["Slot"].as_u64().unwrap() as u32,
                    grenades_read: v["GrenadesRead"].as_bool().unwrap(),
                    grenades: serde_json::from_value(v["Grenades"].clone()).unwrap(),
                    selected_grenade_rank: v["SelectedGrenadeRank"].as_i64().unwrap(),
                    drawn_slot: v["DrawnSlot"].as_i64().unwrap(),
                    ammo_candidates: v["AmmoCandidates"].as_i64().unwrap(),
                    ammo_read: v["AmmoRead"].as_bool().unwrap(),
                    ammo: serde_json::from_value(v["Ammo"].clone()).unwrap(),
                    ..Default::default()
                })
                .collect()
        });
        let deltas: Vec<_> = r["deltas"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| FactsInventoryDelta {
                timestamp_us: v["time"].as_u64().unwrap(),
                slot: v["slot"].as_u64().unwrap() as u32,
                grenades: serde_json::from_value(v["counts"].clone()).unwrap(),
                selection_read: v["selection_read"].as_bool().unwrap(),
                selection: v["selection"].as_i64().unwrap(),
                ..Default::default()
            })
            .collect();
        let labels = ReplayLabelCatalog {
            weapons: serde_json::from_value(r["labels"]["weapons"].clone()).unwrap(),
            effects: serde_json::from_value::<Option<_>>(r["labels"]["effects"].clone())
                .unwrap()
                .unwrap_or_default(),
            icons: r["labels"]["icons"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(key, v)| {
                    (
                        key.parse().unwrap(),
                        ReplayWeaponIconRef {
                            url: v["URL"].as_str().unwrap().into(),
                            tinted: v["Tinted"].as_bool().unwrap(),
                        },
                    )
                })
                .collect(),
            ..Default::default()
        };
        let deaths: Vec<IdentityDeath> = serde_json::from_value(r["deaths"].clone()).unwrap();
        let neutral: Vec<ReplayNeutralDeath> =
            serde_json::from_value(r["neutral"].clone()).unwrap();
        let logs = super::log_test_support::capture_logs(|| {
            marked_total += assemble_facts_replay_labels_and_inventory(
                &mut doc,
                FactsReplayInventoryDocumentInput {
                    labels: &labels,
                    neutral_deaths: &neutral,
                    inventory: inventory.as_deref(),
                    deltas: &deltas,
                    ammo_refused: r["refused"].as_bool().unwrap(),
                    deaths: &deaths,
                    identity: &identity,
                    death_offset_ms: r["offset"].as_i64().unwrap(),
                    origin_us: r["origin"].as_u64().unwrap(),
                    step_us: NonZeroU64::new(r["step"].as_u64().unwrap()).unwrap(),
                },
            );
        });
        assert_eq!(doc, expected, "whole stage {i}");
        assert_eq!(
            serde_json::to_value(logs).unwrap(),
            r["logs"],
            "runtime logs {i}"
        );
    }
    assert!(marked_total > 0);
}
