use super::*;
use std::io::Read;

#[test]
fn native_signed_layer_coverage_and_caller_counters() {
    verify_raw_name_lookup();
    verify_raw_bots();
    verify_nullable_tracks();
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/signed-layer-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let data: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let hex = data["input"].as_str().unwrap();
    let bytes: Vec<_> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let file = decode_film_facts_file(
        &bytes,
        &FactsMapEntry {
            module: b"map".to_vec(),
            axis_widths: [13, 14, 15],
            bounds: [[-10., 100.], [-30., 300.], [-50., 500.]],
            ..Default::default()
        },
    )
    .unwrap();
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 1024);
    let mut negative = 0;
    let mut wide = 0;
    for (i, r) in cases.iter().enumerate() {
        let mut file = file.clone();
        file.facts.queue.player_indices.readings = r["index_readings"].as_i64().unwrap();
        file.facts.queue.player_indices.disagreements = r["index_disagreements"].as_i64().unwrap();
        file.facts.queue.film_table.occupied = r["occupied"].as_i64().unwrap();
        file.facts.queue.film_table.vacant = r["vacant"].as_i64().unwrap();
        file.facts.queue.team_scan.packets = r["team_scan"]["Packets"].as_i64().unwrap();
        file.facts.queue.team_scan.records = r["team_scan"]["Records"].as_i64().unwrap();
        file.facts.queue.team_scan.read = r["team_scan"]["Read"].as_i64().unwrap();
        file.facts.queue.team_scan.unreached = r["team_scan"]["Unreached"].as_i64().unwrap();
        file.facts.queue.team_scan.out_of_domain_index =
            r["team_scan"]["OutOfDomainIndex"].as_i64().unwrap();
        file.facts.queue.team_scan.out_of_domain_value =
            r["team_scan"]["OutOfDomainValue"].as_i64().unwrap();
        file.facts.queue.team_scan.entities = r["team_scan"]["Entities"].as_i64().unwrap();
        file.facts.queue.team_scan.entity_divergences =
            r["team_scan"]["EntityDivergences"].as_i64().unwrap();
        file.facts.queue.team_scan.index_divergences =
            r["team_scan"]["IndexDivergences"].as_i64().unwrap();
        file.facts.queue.team_scan.indices = r["team_scan"]["Indices"].as_i64().unwrap();
        file.facts.queue.team_scan.no_team = r["team_scan"]["NoTeam"].as_i64().unwrap();
        let raw_field = |key: &str| {
            let hex = r[key].as_str().unwrap();
            (0..hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&hex[n..n + 2], 16).unwrap())
                .collect::<Vec<u8>>()
        };
        file.facts.queue.film_table.build = raw_field("build_hex");
        file.facts.queue.film_table.refusal = raw_field("refusal_hex");
        file.facts.queue.team_scan.component = raw_field("component_hex");
        for (seat, name) in file
            .facts
            .queue
            .film_table
            .seats
            .as_mut()
            .unwrap()
            .iter_mut()
            .zip(r["seat_names_hex"].as_array().unwrap())
        {
            let hex = name.as_str().unwrap();
            seat.gamertag = (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect();
        }
        file.facts.queue.deaths = r["deaths"]
            .as_array()
            .unwrap()
            .iter()
            .zip(r["death_names_hex"].as_array().unwrap())
            .map(|(d, name)| {
                let hex = name.as_str().unwrap();
                FactsDeath {
                    xuid: d["XUID"].as_u64().unwrap(),
                    time_ms: d["TimeMS"].as_i64().unwrap(),
                    gamertag: (0..hex.len())
                        .step_by(2)
                        .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                        .collect(),
                }
            })
            .collect();
        let scan = PlayerTeamScanReport::try_from(&file.facts.queue.team_scan).unwrap();
        assert_eq!(scan.component.0, raw_field("component_hex"));
        assert_eq!(
            serde_json::to_value(scan).unwrap(),
            r["team_scan"],
            "team scan {i}"
        );
        let table = ReplayFilmPlayerTable::try_from(&file.facts.queue.film_table).unwrap();
        assert_eq!(table.build.0, raw_field("build_hex"));
        assert_eq!(table.refusal.0, raw_field("refusal_hex"));
        assert_eq!(table.occupied, r["occupied"].as_i64().unwrap());
        assert_eq!(table.vacant, r["vacant"].as_i64().unwrap());
        let indices = PlayerIndexTable::try_from(&file.facts.queue.player_indices).unwrap();
        assert_eq!(
            compose_identity_tables(&table, &indices).coverage.refusal.0,
            raw_field("refusal_hex")
        );
        let coverage: ReplayLayerCoverage = serde_json::from_value(r["coverage"].clone()).unwrap();
        assert_eq!(
            coverage.balanced(),
            r["balanced"].as_bool().unwrap(),
            "balanced {i}"
        );
        assert_eq!(
            coverage.verdict(),
            r["verdict"].as_str().unwrap(),
            "verdict {i}"
        );
        let events: Vec<StatborgIdentifiedEvent> =
            serde_json::from_value(r["events"].clone()).unwrap();
        let unnamed = r["unnamed"].as_i64().unwrap();
        let refused = r["refused"].as_i64().unwrap();
        let clock: ReplayScoreClock = serde_json::from_value(r["clock"].clone()).unwrap();
        let actions = build_replay_objective_actions(&events, unnamed, refused, clock);
        let expected_actions: Vec<ReplayObjectiveAction> = if r["actions"].is_null() {
            Vec::new()
        } else {
            serde_json::from_value(r["actions"].clone()).unwrap()
        };
        assert_eq!(actions.actions, expected_actions, "actions {i}");
        assert_eq!(
            actions.coverage,
            serde_json::from_value(r["action_coverage"].clone()).unwrap(),
            "action coverage {i}"
        );
        negative += usize::from(actions.coverage.available < 0);
        wide += usize::from(actions.coverage.available > i64::from(u32::MAX));
        let mut doc = None;
        let logs = super::log_test_support::capture_logs(|| {
            doc = Some(
                build_facts_replay_document(
                    "signed-layer",
                    "halo_infinite",
                    &file,
                    FactsReplayDocumentOptions {
                        min_points: r["min_points"].as_i64().unwrap(),
                        objectives: &events,
                        objectives_unnamed: unnamed,
                        objectives_refused: refused,
                        ..Default::default()
                    },
                )
                .unwrap(),
            );
        });
        let doc = doc.unwrap();
        let name_hex = |name: &ReplayByteString| {
            name.0
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let roster_names: Vec<_> = doc
            .content
            .roster
            .iter()
            .map(|p| serde_json::json!([p.xuid, name_hex(&p.name)]))
            .collect();
        assert_eq!(
            serde_json::json!(roster_names),
            r["roster_names_hex"],
            "raw roster {i}"
        );
        let identity_names: Vec<_> = doc
            .content
            .identity
            .as_ref()
            .into_iter()
            .flat_map(|v| &v.players)
            .map(|p| serde_json::json!([p.xuid, name_hex(&p.name)]))
            .collect();
        assert_eq!(
            serde_json::json!(identity_names),
            r["identity_names_hex"],
            "raw identity {i}"
        );
        let actual = serde_json::to_value(doc).unwrap();
        let expected: ReplayDocument = serde_json::from_value(r["document"].clone()).unwrap();
        let expected = serde_json::to_value(expected).unwrap();
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&actual[key], value, "document {i} {key}");
        }
        assert_eq!(
            actual.as_object().unwrap().len(),
            expected.as_object().unwrap().len()
        );
        let expected_logs: Vec<serde_json::Value> = r["logs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|log| {
                let mut fields = serde_json::Map::new();
                fields.insert("level".into(), log["level"].clone());
                fields.insert("msg".into(), log["message"].clone());
                for p in log["attributes"].as_array().unwrap() {
                    let key = p[0].as_str().unwrap();
                    let v = &p[1];
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
        for (j, (a, e)) in logs.iter().zip(&expected_logs).enumerate() {
            assert_eq!(a, e, "log {i}:{j}");
        }
        assert_eq!(logs.len(), expected_logs.len(), "log count {i}");
    }
    assert!(negative > 0 && wide > 0);
}

fn verify_raw_name_lookup() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/raw-name-lookup-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 169);
    let decode = |v: &serde_json::Value| {
        let hex = v.as_str().unwrap();
        (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect::<Vec<u8>>()
    };
    let encode = |v: &ReplayByteString| v.0.iter().map(|b| format!("{b:02x}")).collect::<String>();
    for (i, row) in rows.as_array().unwrap().iter().enumerate() {
        let deaths: Vec<_> = row["deaths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| IdentityDeath {
                xuid: d[0].as_u64().unwrap(),
                gamertag: ReplayByteString(decode(&d[1])),
                time_ms: 0,
            })
            .collect();
        let index = replay_kill_identity_index(&deaths);
        let expected: Vec<_> = index
            .iter()
            .map(|(k, v)| serde_json::json!([encode(k), v]))
            .collect();
        assert_eq!(serde_json::json!(expected), row["index"], "name index {i}");
        for q in row["queries"].as_array().unwrap() {
            assert_eq!(
                serde_json::json!(resolve_replay_kill_identity(decode(&q[0]), &index)),
                q[1],
                "name lookup {i}"
            );
        }
    }
}

fn verify_raw_bots() {
    use std::collections::{BTreeMap, BTreeSet};
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/raw-bots-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 81);
    let decode = |v: &serde_json::Value| {
        let hex = v.as_str().unwrap();
        ReplayByteString(
            (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect(),
        )
    };
    let encode = |s: &ReplayByteString| s.0.iter().map(|b| format!("{b:02x}")).collect::<String>();
    for (i, row) in rows.as_array().unwrap().iter().enumerate() {
        let bots: Vec<_> = row["bots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| IdentityBot {
                film_index: b[0].as_i64().unwrap(),
                name: decode(&b[1]),
                bot_id: b[2].as_i64().unwrap(),
            })
            .collect();
        let (names, shared) = identity_bot_names_by_seat(&bots);
        assert_eq!(
            shared,
            row["shared"].as_u64().unwrap() as usize,
            "bot shared {i}"
        );
        let names: Vec<_> = names
            .iter()
            .map(|(k, v)| serde_json::json!([k, encode(v)]))
            .collect();
        assert_eq!(serde_json::json!(names), row["seats"], "bot seats {i}");
        let owners = BTreeMap::from([(1, 0), (2, 1), (3, 2), (4, 0)]);
        let mut tracks: Vec<_> = [
            (1, 0, 10, ""),
            (2, 12, 20, ""),
            (3, 25, 30, ""),
            (4, 0, 30, "100"),
        ]
        .into_iter()
        .map(|(slot, start_frame, end_frame, xuid)| ReplayTrackIdentity {
            slot,
            start_frame,
            end_frame,
            xuid: xuid.into(),
            ..Default::default()
        })
        .collect();
        name_bot_identity_tracks(&mut tracks, &owners, &bots);
        assert_eq!(
            serde_json::json!(tracks.iter().map(|t| encode(&t.bot)).collect::<Vec<_>>()),
            row["tracks"],
            "bot tracks {i}"
        );
        let indices = PlayerIndexTable {
            by_xuid: BTreeMap::from([(100, 3)]),
            ..Default::default()
        };
        let teams = FilmPlayerTeams::default();
        let ambiguous = BTreeSet::new();
        let control = BTreeMap::new();
        let publication =
            ReplayTeamPublication::from_tables(&indices, &owners, &ambiguous, &teams, &control);
        let roster = publication.roster(&indices, &BTreeMap::new(), &bots);
        let roster: Vec<_> = roster
            .iter()
            .map(|p| serde_json::json!([p.film_index, p.xuid, encode(&p.name), p.bot, p.bid]))
            .collect();
        assert_eq!(
            serde_json::json!(roster),
            row["roster"],
            "raw bot roster {i}"
        );
        let published: Vec<_> = tracks
            .iter()
            .map(|t| ReplayTrack {
                slot: t.slot,
                start_frame: t.start_frame,
                end_frame: t.end_frame,
                xuid: t.xuid.clone(),
                bot: t.bot.clone(),
                ..Default::default()
            })
            .collect();
        let presence: Vec<_> = replay_presence_envelopes(&published)
            .iter()
            .map(|(k, v)| serde_json::json!([encode(k), v]))
            .collect();
        assert_eq!(
            serde_json::json!(presence),
            row["presence"],
            "raw bot presence {i}"
        );
        let mut chain: Vec<_> = [(5, 0, 10), (6, 220, 230)]
            .into_iter()
            .map(|(slot, start_frame, end_frame)| ReplayTrackIdentity {
                slot,
                start_frame,
                end_frame,
                ..Default::default()
            })
            .collect();
        attribute_identity_successions(
            &mut chain,
            &[IdentitySuccession {
                bot_name: bots[0].name.clone(),
                film_index: 0,
                switch_match_ms: 0,
            }],
            0,
            100000,
            0,
            row["matches"].as_u64().unwrap() as usize,
            &[],
        );
        assert_eq!(
            serde_json::json!(chain.iter().map(|t| encode(&t.bot)).collect::<Vec<_>>()),
            row["chain"],
            "raw bot succession {i}"
        );
    }
}

fn verify_nullable_tracks() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/nullable-tracks-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 64);
    for (i, row) in rows.as_array().unwrap().iter().enumerate() {
        let tracks: Vec<ReplayTrack> = serde_json::from_value(row["tracks"].clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&tracks).unwrap(),
            row["tracks"],
            "native track JSON {i}"
        );
        for (track, kind) in tracks.iter().zip(row["kinds"].as_array().unwrap()) {
            let kind = kind.as_u64().unwrap();
            assert_eq!(track.points.is_none(), kind == 0);
            assert_eq!(track.points().len(), kind.saturating_sub(1) as usize);
        }
        let (bounds, rejected) = replay_bounds(&tracks);
        assert_eq!(
            bounds,
            serde_json::from_value(row["bounds"].clone()).unwrap(),
            "nullable bounds {i}"
        );
        assert_eq!(rejected, row["rejected"].as_u64().unwrap() as usize);
        assert_eq!(
            serde_json::to_value(&tracks).unwrap(),
            row["tracks"],
            "unmodified tracks {i}"
        );
    }
}
