use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

#[test]
fn native_facts_live_objective_document_pass() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-live-objectives-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 128);
    let mut totals = [0; 5];
    for (i, row) in rows.iter().enumerate() {
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        let records =
            serde_json::from_value::<Vec<StatborgRecord>>(row["records"].clone()).unwrap();
        let bursts = serde_json::from_value::<Vec<i64>>(row["bursts"].clone()).unwrap();
        let spawns = serde_json::from_value::<Vec<ReplayFlagSpawn>>(row["spawns"].clone()).unwrap();
        let identified =
            serde_json::from_value::<Vec<StatborgIdentifiedEvent>>(row["identified"].clone())
                .unwrap();
        let deaths = serde_json::from_value::<Vec<IdentityDeath>>(row["deaths"].clone()).unwrap();
        let mut file = NativeFilmFactsFile::default();
        file.facts.header.film_clock_origin_us = 1234567;
        file.facts.queue.deaths = deaths
            .iter()
            .map(|d| FactsDeath {
                xuid: d.xuid,
                time_ms: d.time_ms,
                gamertag: vec![0xff],
            })
            .collect();
        file.facts.weapon_changes = row["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FactsWeaponChange {
                timestamp_us: c["TimestampUS"].as_u64().unwrap(),
                slot: c["Slot"].as_u64().unwrap() as u32,
                family: c["Family"].as_u64().unwrap() as u32,
                previous: c["Previous"].as_u64().unwrap() as u32,
                kind: vec![0xff],
                slot_index: i64::MIN,
            })
            .collect();
        file.mode_guards.flag_gauge = Some(serde_json::from_value(row["gauges"].clone()).unwrap());
        file.mode_guards.flag_gauge_scanned = row["gauge_scanned"].as_bool().unwrap();
        file.mode_guards.zone_reads = file.mode_guards.flag_gauge.clone();
        file.mode_guards.zone_scanned = row["zone_scanned"].as_bool().unwrap();
        file.mode_guards.bomb_reads =
            Some(serde_json::from_value(row["bomb_reads"].clone()).unwrap());
        let identity = match row["identity_mode"].as_u64().unwrap() {
            0 => StatborgRoundIdentity {
                publication: IdentityStatborgPublication::default(),
                starts: Vec::new(),
            },
            1 => StatborgRoundIdentity::from_flat(BTreeMap::from([(10, "100".into())])),
            _ => StatborgRoundIdentity::from_flat(BTreeMap::new()),
        };
        let state = ReplayIdentityState::from_lives(
            vec![IdentityLife {
                slot: 10,
                xuid: 100,
                to: i64::MAX,
                ..Default::default()
            }],
            &BTreeMap::from([(100, 0)]),
        );
        let clock: ReplayMatchClock = serde_json::from_value(row["clock"].clone()).unwrap();
        let mut report = None;
        let logs = super::log_test_support::capture_logs(|| {
            report = Some(assemble_facts_replay_live_objectives(
                &mut doc,
                &file,
                FactsReplayLiveObjectiveOptions {
                    flag: FilmReplayFlagInput {
                        scanned: row["scanned"].as_bool().unwrap(),
                        spawns: &spawns,
                    },
                    flag_records: &records,
                    flag_bursts: &bursts,
                    flag_identity: &identity,
                    flag_return_zone: ReplayFlagReturnZone::HALO_INFINITE,
                    vip_scanned: row["vip_scanned"].as_bool().unwrap(),
                    vip_records: &records,
                    skull_scanned: row["skull_scanned"].as_bool().unwrap(),
                    skull_records: &records,
                    skull_identity: &identity,
                    bomb_carry_scanned: row["carry_scanned"].as_bool().unwrap(),
                    bomb_arming_scanned: row["arming_scanned"].as_bool().unwrap(),
                    score_read: row["score_read"].as_bool().unwrap(),
                    identified_objectives: &identified,
                    kills_read: false,
                    kills: &[],
                    kills_dropped: 0,
                    zone: FilmReplayZoneInput {
                        zones: &[],
                        roles: "",
                        teams: &BTreeMap::new(),
                        hill: false,
                    },
                    catalog: &ReplayEquipmentCatalog::default(),
                },
                FactsReplayLiveObjectiveContext {
                    state: &state,
                    clock,
                    score_clock: ReplayScoreClock {
                        origin_ms: 0,
                        interval_ms: 100,
                        frames: clock.frames,
                    },
                    flag_teams: &BTreeMap::from([("100".into(), 0)]),
                    deduced_tracks: &BTreeSet::new(),
                },
            ));
        });
        let report = report.unwrap();
        let expected_logs: Vec<serde_json::Value> = row["logs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|log| {
                let mut fields = serde_json::Map::new();
                fields.insert("level".into(), log["level"].clone());
                fields.insert("msg".into(), log["message"].clone());
                for pair in log["attributes"].as_array().unwrap() {
                    let key = pair[0].as_str().unwrap();
                    // The capture visitor records floating fields via Debug.
                    // Preserve the numeric value while matching that representation.
                    let value = if key == "cv" {
                        serde_json::Value::String(format!("{:?}", pair[1].as_f64().unwrap()))
                    } else if pair[1].is_array() || pair[1].is_object() {
                        serde_json::Value::String(pair[1].to_string())
                    } else {
                        pair[1].clone()
                    };
                    fields.insert(key.into(), value);
                }
                fields.into()
            })
            .collect();
        assert_eq!(logs, expected_logs, "publication logs {i}");
        assert_eq!(doc, expected, "live objectives document {i}");
        assert!(report.fallbacks.iter().all(|f| f.hits == 0));
        assert!(row["fallbacks"].as_array().unwrap().is_empty());
        totals[0] += doc.content.flag_carries.len();
        totals[1] += doc.content.vip_crown.len();
        totals[2] += doc.content.skull_carries.len();
        totals[3] += doc.content.bomb_carries.len();
        totals[4] += usize::from(doc.content.bomb_stats.is_some());
    }
    assert!(totals.iter().all(|&n| n > 0));
    println!("live objective publications flags/VIP/skull/bomb/stats: {totals:?}");
}
