use super::*;
use std::{collections::BTreeMap, io::Read};
fn unhex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn native_facts_equipment_document_stage() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-equipment-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut totals = [0; 6];
    for (i, row) in rows.iter().enumerate() {
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let coverage = doc.coverage.clone();
        let positions: Vec<_> = row["positions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| FactsBipedPosition {
                timestamp_us: p["time"].as_u64().unwrap(),
                slot: p["slot"].as_u64().unwrap() as u32,
                has_shield: p["has_shield"].as_bool().unwrap(),
                shield_quantum: p["quantum"].as_u64().unwrap() as u8,
                ..Default::default()
            })
            .collect();
        let camo = serde_json::from_value::<Vec<FactsCamoState>>(row["camo"].clone()).unwrap();
        let movement: Vec<_> = row["moves"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| FactsMovementState {
                timestamp_us: r["timestamp"].as_u64().unwrap(),
                slot: r["slot"].as_u64().unwrap() as u32,
                kind: unhex(r["kindHex"].as_str().unwrap()),
                on: r["on"].as_bool().unwrap(),
                ..Default::default()
            })
            .collect();
        let lives = serde_json::from_value::<Vec<IdentityLife>>(row["lives"].clone()).unwrap();
        let identity = ReplayIdentityState::from_lives(lives, &BTreeMap::new());
        let kills =
            serde_json::from_value::<Vec<ReplayEquipmentKill>>(row["kills"].clone()).unwrap();
        let result = assemble_facts_replay_equipment(
            &mut doc,
            FactsReplayEquipmentInput {
                positions: &positions,
                camo: &camo,
                movement: &movement,
                movement_stats: serde_json::from_value(row["stats"].clone()).unwrap(),
                kills: &kills,
                kills_read: row["kill_read"].as_bool().unwrap(),
            },
            FactsReplayEquipmentContext {
                identity: &identity,
                clock: IdentityClock {
                    origin_us: row["origin"].as_u64().unwrap(),
                    step_us: row["step"].as_u64().unwrap(),
                    frame_count: 0,
                },
                interval_ms: row["interval"].as_i64().unwrap(),
            },
        );
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        // JSON publication replaces invalid string bytes; compare native raw
        // strings independently below so JSON collisions cannot hide data loss.
        assert_eq!(
            serde_json::to_value(&doc).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "document JSON {i}"
        );
        assert_eq!(doc.coverage, coverage, "coverage envelope {i}");
        assert_eq!(
            result.closed_by_death,
            serde_json::from_value(row["closed"].clone()).unwrap(),
            "death bounds {i}"
        );
        assert_eq!(
            serde_json::to_value(&result.stances).unwrap(),
            row["stance_cov"],
            "stance coverage {i}"
        );
        let counts: BTreeMap<ReplayByteString, i64> = row["counts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    ReplayByteString(unhex(r[0].as_str().unwrap())),
                    r[1].as_i64().unwrap(),
                )
            })
            .collect();
        assert_eq!(result.stances.by_kind, counts, "raw coverage keys {i}");
        let stances: Vec<_> = row["stance_raw"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| ReplayStance {
                slot: r["slot"].as_u64().unwrap() as u32,
                kind: ReplayByteString(unhex(r["kindHex"].as_str().unwrap())),
                t0: r["t0"].as_i64().unwrap(),
                t1: r["t1"].as_i64().unwrap(),
            })
            .collect();
        assert_eq!(doc.content.stances, stances, "ordered raw stances {i}");
        assert_eq!(
            result.kills_read,
            row["kills_read"].as_bool().unwrap(),
            "kills gate {i}"
        );
        assert_eq!(
            result.diagnostics,
            serde_json::from_value::<Vec<StatborgDiagnostic>>(row["diagnostics"].clone()).unwrap(),
            "diagnostics {i}"
        );
        let coverage_input: Vec<ReplayEquipmentEpisode> =
            serde_json::from_value(row["coverage_input"].clone()).unwrap();
        let late_coverage = build_replay_equipment_coverage(
            &coverage_input,
            doc.content.tracks.as_deref().unwrap_or_default(),
            &result.closed_by_death,
        );
        assert_eq!(
            late_coverage,
            serde_json::from_value(row["coverage_expected"].clone()).unwrap(),
            "late coverage {i}"
        );
        totals[0] += doc.content.equipment_episodes.len();
        totals[1] += doc.content.stances.len();
        totals[2] += doc
            .content
            .equipment_episodes
            .iter()
            .map(|e| e.k + e.a)
            .sum::<usize>();
        totals[3] += result.camo_non_binary;
        totals[4] += result.closed_by_death.len();
        totals[5] += doc
            .content
            .stances
            .iter()
            .filter(|s| std::str::from_utf8(&s.kind.0).is_err())
            .count();
    }
    assert!(totals.iter().all(|&n| n > 0));
    println!("equipment document stage: {totals:?}");
}
