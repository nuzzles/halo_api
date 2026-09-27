use super::*;
use std::io::Read;

#[test]
fn native_facts_complete_document_composition() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-players-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 128);
    let entry = FactsMapEntry {
        module: b"map".to_vec(),
        axis_widths: [13, 14, 15],
        bounds: [[-10., 100.], [-30., 300.], [-50., 500.]],
        ..Default::default()
    };
    let mut nonempty = 0;
    for (i, row) in rows.iter().enumerate() {
        let hex = row["input"].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|n| u8::from_str_radix(&hex[n..n + 2], 16).unwrap())
            .collect();
        let file = decode_film_facts_file(&bytes, &entry).unwrap();
        let doc = build_facts_replay_document(
            "fixture",
            "halo_infinite",
            &file,
            FactsReplayDocumentOptions {
                frame_interval_ms: row["interval"].as_i64().unwrap(),
                min_points: row["min_points"].as_i64().unwrap(),
                ..Default::default()
            },
        )
        .unwrap();
        let expected: ReplayDocument = serde_json::from_value(row["document"].clone()).unwrap();
        let actual = serde_json::to_value(&doc).unwrap();
        let expected = serde_json::to_value(expected).unwrap();
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&actual[key], value, "document {i} field {key}");
        }
        assert_eq!(
            actual.as_object().unwrap().len(),
            expected.as_object().unwrap().len(),
            "field count {i}"
        );
        nonempty += usize::from(doc.content.frame_count > 0);
    }
    assert_eq!(nonempty, 122);
}

#[test]
fn native_facts_document_caller_precedence() {
    use std::collections::BTreeMap;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-document-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 128);
    let entry = FactsMapEntry {
        module: b"map".to_vec(),
        axis_widths: [13, 14, 15],
        bounds: [[-10., 100.], [-30., 300.], [-50., 500.]],
        ..Default::default()
    };
    let mut totals = [0; 5];
    for (i, row) in rows.iter().enumerate() {
        let hex = row["input"].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|n| u8::from_str_radix(&hex[n..n + 2], 16).unwrap())
            .collect();
        let file = decode_film_facts_file(&bytes, &entry).unwrap();
        let records: Vec<StatborgRecord> = serde_json::from_value(row["records"].clone()).unwrap();
        let identified: Vec<StatborgIdentifiedEvent> =
            serde_json::from_value(row["identified"].clone()).unwrap();
        let identity = match row["identity_mode"].as_u64().unwrap() {
            0 => StatborgRoundIdentity {
                publication: IdentityStatborgPublication::default(),
                starts: Vec::new(),
            },
            1 => StatborgRoundIdentity::from_flat(BTreeMap::from([(10, "100".into())])),
            _ => StatborgRoundIdentity::from_flat(BTreeMap::new()),
        };
        let labels = ReplayLabelCatalog {
            weapons: BTreeMap::from([(
                9,
                ReplayWeaponLabel {
                    en: "Fixture".into(),
                    fr: "Fixture".into(),
                    fx: "ballistic".into(),
                    ..Default::default()
                },
            )]),
            keys: BTreeMap::from([(9, "fixture_weapon".into())]),
            effects: BTreeMap::from([("fixture_weapon".into(), "ballistic".into())]),
            ..Default::default()
        };
        let counter = FallbackCounter::default();
        counter.trigger_n("repli_plafond_grenade_par_defaut", 7);
        let mut doc = None;
        let logs = super::log_test_support::capture_logs(|| {
            doc = Some(
                build_facts_replay_document(
                    "fixture-rich",
                    "halo_infinite",
                    &file,
                    FactsReplayDocumentOptions {
                        frame_interval_ms: row["interval"].as_i64().unwrap(),
                        min_points: row["min_points"].as_i64().unwrap(),
                        statborg_identity: Some(&identity),
                        labels: Some(&labels),
                        objectives: &identified,
                        scoreboard_teams: BTreeMap::from([("100".into(), 1), ("101".into(), 0)]),
                        score: row["score"].as_bool().unwrap().then(|| ReplayScoreInput {
                            teams: ReplayTeamScoreInput {
                                records: &records,
                                target_score: 50,
                                ..Default::default()
                            },
                            hold_ticks_per_point: 10,
                            truncated: row["truncated"].as_bool().unwrap(),
                        }),
                        flag: FactsReplayFlagOptions {
                            scanned: row["flag"].as_bool().unwrap(),
                            records: &records,
                            bursts: &[1000, 2000],
                            identity: Some(&identity),
                            ..Default::default()
                        },
                        vip: FactsReplayVipOptions {
                            scanned: row["vip"].as_bool().unwrap(),
                            records: &records,
                        },
                        skull: FactsReplayCarryOptions {
                            scanned: row["skull"].as_bool().unwrap(),
                            records: &records,
                            identity: Some(&identity),
                        },
                        bomb_arming_scanned: row["bomb"].as_bool().unwrap(),
                        bomb_carry_scanned: row["carry"].as_bool().unwrap(),
                        fallbacks: Some(&counter),
                        ..Default::default()
                    },
                )
                .unwrap(),
            );
        });
        let doc = doc.unwrap();
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
                    let v = &pair[1];
                    let value = if key == "cv" {
                        serde_json::Value::String(format!("{:?}", v.as_f64().unwrap()))
                    } else if v.is_array() || v.is_object() || v.is_null() {
                        serde_json::Value::String(v.to_string())
                    } else {
                        v.clone()
                    };
                    fields.insert(key.into(), value);
                }
                fields.into()
            })
            .collect();
        for (j, (actual, expected)) in logs.iter().zip(&expected_logs).enumerate() {
            assert_eq!(actual, expected, "runtime log {i}:{j}");
        }
        assert_eq!(logs.len(), expected_logs.len(), "runtime log count {i}");
        let expected: ReplayDocument = serde_json::from_value(row["document"].clone()).unwrap();
        let actual = serde_json::to_value(&doc).unwrap();
        let expected = serde_json::to_value(expected).unwrap();
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&actual[key], value, "caller document {i} field {key}");
        }
        assert_eq!(
            actual.as_object().unwrap().len(),
            expected.as_object().unwrap().len(),
            "field count {i}"
        );
        assert_eq!(
            counter.count("repli_plafond_grenade_par_defaut"),
            i as i64 + 8,
            "scan fallback accumulated once {i}"
        );
        totals[0] += doc.content.shots.len();
        totals[1] += doc.content.loadouts.len();
        totals[2] += doc.content.inventory.len();
        totals[3] += doc.content.weapon_changes.len();
        totals[4] += doc.content.objectives.len();
    }
    assert_eq!(totals, [222, 0, 118, 244, 95]);
}
