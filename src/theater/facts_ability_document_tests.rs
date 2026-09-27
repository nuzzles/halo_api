use super::*;
use serde_json::Value;
use std::{io::Read, num::NonZeroU64};

#[test]
fn native_facts_ability_document_stages() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-ability-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut totals = [0; 5];
    for (i, r) in rows.iter().enumerate() {
        let s = &r["stage"];
        let mut doc: ReplayDocument = serde_json::from_value(s["before"].clone()).unwrap();
        let middle: ReplayDocument = serde_json::from_value(s["middle"].clone()).unwrap();
        let after: ReplayDocument = serde_json::from_value(s["after"].clone()).unwrap();
        let expected_palette: Option<ReplayAbilityPalette> =
            serde_json::from_value(s["palette"].clone()).unwrap();
        let ranks: Vec<_> = r["ranks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| FactsAbilityRank {
                timestamp_us: v["time"].as_u64().unwrap(),
                slot: v["slot"].as_u64().unwrap() as u32,
                rank: v["rank"].as_i64().unwrap(),
            })
            .collect();
        let inventory: Vec<_> = r["inventory"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| FactsKeyframeInventory {
                timestamp_us: v["TimestampUS"].as_u64().unwrap(),
                slot: v["Slot"].as_u64().unwrap() as u32,
                ability_rank: v["AbilityRank"].as_i64().unwrap(),
                ..Default::default()
            })
            .collect();
        let equipment: Vec<_> = s["equipment"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| FactsEquipmentChange {
                timestamp_us: v["time"].as_u64().unwrap(),
                slot: v["slot"].as_u64().unwrap() as u32,
                rank: v["rank"].as_i64().unwrap(),
                previous: v["previous"].as_i64().unwrap(),
                gap: v["gap"].as_i64().unwrap(),
                recovered: v["recovered"].as_bool().unwrap(),
                kind: v["kind_hex"]
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
        let equipment_stats: FactsEquipmentChangeStats =
            serde_json::from_value(s["equipment_stats"].clone()).unwrap();
        let translocations: Vec<_> = s["translocations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| FactsTranslocation {
                timestamp_us: v["TimestampUS"].as_u64().unwrap(),
                slot: v["Slot"].as_u64().unwrap() as u32,
                has_positions: v["HasPositions"].as_bool().unwrap(),
                from: serde_json::from_value(v["From"].clone()).unwrap(),
                to: serde_json::from_value(v["To"].clone()).unwrap(),
            })
            .collect();
        let lives: Vec<_> = r["lives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| IdentityLife {
                slot: v["slot"].as_u64().unwrap() as u32,
                from: v["from"].as_i64().unwrap(),
                to: v["to"].as_i64().unwrap(),
                ..Default::default()
            })
            .collect();
        let impulses: Vec<FactsAbilityImpulse> =
            serde_json::from_value::<Option<_>>(r["impulses"].clone())
                .unwrap()
                .unwrap_or_default();
        let impulse_stats: FactsAbilityImpulseStats =
            serde_json::from_value(r["impulse_stats"].clone()).unwrap();
        let charges: Vec<_> = r["reads"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| FactsAbilityCharge {
                timestamp_us: v["time"].as_u64().unwrap(),
                slot: v["slot"].as_u64().unwrap() as u32,
                charges: v["charges"].as_i64().unwrap(),
                emplacement: v["emplacement"].as_i64().unwrap(),
                low: v["low"].as_i64().unwrap(),
            })
            .collect();
        let charge_stats: FactsAbilityChargeStats =
            serde_json::from_value(s["charge_stats"].clone()).unwrap();
        let measured = serde_json::from_value(r["measured"].clone()).unwrap();
        let labels = ReplayLabelCatalog {
            abilities: serde_json::from_value(r["palettes"].clone()).unwrap(),
            grenades: vec![ReplayLabel {
                en: "Frag".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let origin_us = r["origin"].as_u64().unwrap();
        let step_us = s["step"].as_u64().unwrap();
        let logs = super::log_test_support::capture_logs(|| {
            let palette = assemble_facts_replay_abilities_and_translocations(
                &mut doc,
                FactsReplayAbilityDocumentInput {
                    labels: &labels,
                    ranks: &ranks,
                    inventory: &inventory,
                    equipment_changes: &equipment,
                    equipment_stats: &equipment_stats,
                    translocations: &translocations,
                    origin_us,
                    step_us: NonZeroU64::new(step_us).unwrap(),
                },
            );
            assert_eq!(doc, middle, "palette stage {i}");
            assert_eq!(palette, expected_palette, "selected palette {i}");
            assemble_facts_replay_ability_actions(
                &mut doc,
                FactsReplayAbilityActionDocumentInput {
                    ranks: &ranks,
                    lives: &lives,
                    palette: palette.as_ref(),
                    impulse_families: &measured,
                    charge_families: &measured,
                    impulses: &impulses,
                    impulse_stats: &impulse_stats,
                    charges: &charges,
                    charge_stats: &charge_stats,
                    origin_us,
                    step_us,
                },
            );
        });
        assert_eq!(doc, after, "actions stage {i}");
        for (total, count) in totals.iter_mut().zip([
            doc.content.abilities.len(),
            doc.content.ability_impulses.len(),
            doc.content.ability_charges.len(),
            doc.content.translocations.len(),
            doc.content.equipment_changes.len(),
        ]) {
            *total += count;
        }

        assert_eq!(
            serde_json::to_value(logs).unwrap(),
            s["logs"],
            "runtime logs {i}"
        );
    }
    assert_eq!(totals, [7503, 256, 256, 1285, 2078]);
}
