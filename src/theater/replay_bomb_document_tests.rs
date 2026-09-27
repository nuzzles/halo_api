use super::*;
use std::io::Read;

#[test]
fn native_bomb_document_statistics() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/replay-bomb-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2048);
    for (i, row) in rows.iter().enumerate() {
        let input: ReplayBombStatsInput = serde_json::from_value(row["input"].clone()).unwrap();
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        attach_replay_bomb_stats_to_document(
            &mut doc,
            ReplayBombDocumentStatsInput {
                carry_scanned: row["carry_scanned"].as_bool().unwrap(),
                score_read: input.detonations_read,
                objectives: &input.objectives,
                bridge_established: input.carry_read,
                carry: &input.carry,
                kills_read: input.kills_read,
                kills: &input.kills,
                kills_dropped: -123,
                film_clock_origin_us: row["origin"].as_u64().unwrap(),
                death_offset_ms: row["offset"].as_i64().unwrap(),
            },
        );
        assert_eq!(doc, expected, "stats document {i}");
    }
}

#[test]
fn native_bomb_document_armings() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/replay-bomb-arming-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut published = 0;
    let mut suppressed = 0;
    for (i, row) in rows.iter().enumerate() {
        let reads =
            serde_json::from_value::<Vec<NavpointRadialRead>>(row["reads"].clone()).unwrap();
        let clock = serde_json::from_value(row["clock"].clone()).unwrap();
        let scanned = row["scanned"].as_bool().unwrap();
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        let out = attach_replay_bomb_armings_to_document(&mut doc, &reads, scanned, clock);
        assert_eq!(doc, expected, "arming document {i}");
        assert_eq!(
            out.as_ref().map_or(0, |v| v.start_zero_fallbacks) as u64,
            row["fallbacks"].as_u64().unwrap(),
            "arming fallbacks {i}"
        );
        assert_eq!(out.is_some(), scanned);
        if let Some(out) = out {
            published += out.armings.len();
            suppressed += usize::from(out.coverage.suppressed);
        }
    }
    assert!(published > 0 && suppressed > 0);
    println!("{published} published armings; {suppressed} suppressed cases");
}

#[test]
fn native_bomb_document_carries() {
    use std::collections::BTreeMap;
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/replay-bomb-carry-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut published = 0;
    let mut periods = 0;
    for (i, row) in rows.iter().enumerate() {
        let mut lives = vec![
            IdentityLife {
                slot: 10,
                from: 0,
                to: 10000000,
                xuid: 100,
                ..Default::default()
            },
            IdentityLife {
                slot: 11,
                from: 0,
                to: 50000000,
                xuid: 100,
                ..Default::default()
            },
        ];
        if row["recycled"].as_bool().unwrap() {
            lives.push(IdentityLife {
                slot: 10,
                from: 12000000,
                to: 25000000,
                xuid: 101,
                ..Default::default()
            });
        }
        let indices = if row["bridge"].as_bool().unwrap() {
            BTreeMap::from([(100, 0), (101, 1)])
        } else {
            BTreeMap::new()
        };
        let state = ReplayIdentityState::from_lives(lives, &indices);
        let changes: Vec<_> = row["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FactsWeaponChange {
                timestamp_us: c["TimestampUS"].as_u64().unwrap(),
                slot: c["Slot"].as_u64().unwrap() as u32,
                family: c["Family"].as_u64().unwrap() as u32,
                previous: c["Previous"].as_u64().unwrap() as u32,
                kind: vec![0xff, 0x80],
                slot_index: i64::MIN,
            })
            .collect();
        let deaths = serde_json::from_value::<Vec<IdentityDeath>>(row["deaths"].clone()).unwrap();
        let deaths: Vec<_> = deaths
            .iter()
            .map(|d| FactsDeath {
                xuid: d.xuid,
                time_ms: d.time_ms,
                gamertag: vec![0xff, 0x80],
            })
            .collect();
        let deduced: BTreeMap<usize, bool> =
            serde_json::from_value(row["deduced"].clone()).unwrap();
        let deduced = deduced
            .into_iter()
            .filter_map(|(i, b)| b.then_some(i))
            .collect();
        let clock = serde_json::from_value(row["clock"].clone()).unwrap();
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        let raw = attach_facts_replay_bomb_carries_to_document(
            &mut doc,
            FactsReplayBombCarryDocumentInput {
                scanned: row["scanned"].as_bool().unwrap(),
                changes: &changes,
                deaths: &deaths,
                state: &state,
                clock,
                deduced_tracks: &deduced,
            },
        );
        assert_eq!(doc, expected, "carry document {i}");
        assert_eq!(
            serde_json::to_value(&doc.content.tracks).unwrap(),
            row["after"]["tracks"],
            "native track JSON {i}"
        );
        assert_eq!(
            raw,
            serde_json::from_value::<ReplayHeldObjectCarry>(row["raw"].clone()).unwrap(),
            "raw carry {i}"
        );
        published += doc
            .content
            .bomb_carries
            .iter()
            .filter(|c| c.xuid != "seed")
            .count();
        periods += raw.periods.len();
    }
    assert!(published > 0 && periods > 0);
    println!("{published} published carries; {periods} raw periods");
}
