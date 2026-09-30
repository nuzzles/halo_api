use super::*;

#[test]
fn runtime_owns_shared_canonical_film_and_independent_cursor() {
    let film = Arc::new(
        Film::parse([crate::theater::film::FilmChunk {
            kind: crate::theater::film::ChunkKind::Registry,
            index: None,
            start_ms: None,
            data: [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
        }])
        .unwrap(),
    );
    let mut first = TheaterRuntime::load(film.clone());
    let second = first.clone();
    assert!(std::ptr::eq(first.film(), film.as_ref()));
    assert!(std::ptr::eq(first.film(), second.film()));
    first.advance_to(42);
    assert_eq!(first.timestamp_us(), Some(42));
    assert_eq!(second.timestamp_us(), None);
    first.rewind();
    assert_eq!(first.timestamp_us(), None);
}

fn registry() -> crate::theater::film::FilmChunk {
    crate::theater::film::FilmChunk {
        kind: crate::theater::film::ChunkKind::Registry,
        index: None,
        start_ms: None,
        data: [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
    }
}
fn packet(payload: &[u8], timestamp: u64) -> Vec<u8> {
    [
        9u16.to_le_bytes().as_slice(),
        &[0, 0],
        &(payload.len() as u32).to_le_bytes(),
        &timestamp.to_le_bytes(),
        payload,
    ]
    .concat()
}
fn put(output: &mut [u8], start: usize, data: &[u8]) {
    for (byte, value) in data.iter().enumerate() {
        for bit in 0..8 {
            let at = start + byte * 8 + bit;
            let mask = 1 << (7 - at % 8);
            if value & (1 << (7 - bit)) != 0 {
                output[at / 8] |= mask;
            } else {
                output[at / 8] &= !mask;
            }
        }
    }
}
fn summary_payload(input: &serde_json::Value) -> Vec<u8> {
    // Independently written layout, intentionally unaligned. These literal
    // offsets come from the captured v41 wire fixture, not decoder constants.
    let identity = 37;
    let tail = 14_963;
    let end = 15_443;
    let mut payload = vec![0; (end + 32usize).div_ceil(8)];
    put(&mut payload, 0, &1u32.to_be_bytes());
    put(
        &mut payload,
        identity,
        &input["xuid"].as_u64().unwrap_or(42).to_le_bytes(),
    );
    put(&mut payload, 101, &[0x2d, 0xc0]);
    for (index, unit) in "Nuzzles".encode_utf16().enumerate() {
        put(&mut payload, tail + index * 16, &unit.to_le_bytes());
    }
    put(
        &mut payload,
        tail + 47 * 8,
        &[input["type"].as_u64().unwrap() as u8],
    );
    put(
        &mut payload,
        tail + 48 * 8,
        &(input["ms"].as_u64().unwrap() as u32).to_be_bytes(),
    );
    put(
        &mut payload,
        tail + 55 * 8,
        &[input["flag"].as_u64().unwrap() as u8],
    );
    put(
        &mut payload,
        tail + 59 * 8,
        &[input["metadata"].as_u64().unwrap() as u8],
    );
    put(&mut payload, end, &[0, 0, 0x2e, 0xe0]);
    match input["corrupt"].as_str() {
        Some("marker") => put(&mut payload, end, &[0, 0, 0, 0]),
        Some("utf16") => put(&mut payload, tail, &0xd800u16.to_le_bytes()),
        Some("reserved") => put(&mut payload, tail + 52 * 8, &[1]),
        Some("xuid") => put(&mut payload, identity, &0u64.to_le_bytes()),
        Some("truncated") => {
            payload.pop();
        }
        _ => {}
    }
    payload
}
fn load_summary(payload: &[u8], timestamp: u64) -> TheaterRuntime {
    TheaterRuntime::load(
        Film::parse([
            registry(),
            crate::theater::film::FilmChunk {
                kind: crate::theater::film::ChunkKind::Summary,
                index: None,
                start_ms: None,
                data: packet(payload, timestamp),
            },
        ])
        .unwrap(),
    )
}

#[test]
fn versioned_wire_golden_summaries_include_negative_cases() {
    use medals::Medal;
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/runtime-summaries-v41.json")).unwrap();
    assert_eq!(golden["schema_version"], 1);
    assert_eq!(golden["timestamp_tolerance_us"], 0);
    for case in golden["cases"].as_array().unwrap() {
        let payload = summary_payload(&case["input"]);
        let runtime = load_summary(&payload, 999_999_999);
        let expected = &case["expected"];
        assert_eq!(
            runtime.summary_events().len(),
            usize::from(!expected.is_null()),
            "{}",
            case["name"]
        );
        let report = &runtime.summary_reports()[0];
        assert_eq!(report.declared_events, Some(1));
        assert_eq!(report.count_matches(), Some(!expected.is_null()));
        if expected.is_null() {
            continue;
        }
        let event = &runtime.summary_events()[0];
        assert_eq!(event.actor.gamertag, "Nuzzles");
        assert_eq!(event.actor.xuid, 42);
        assert_eq!(event.actor.roster_link, PlayerLink::Missing);
        assert_eq!(event.timestamp_us, expected["time_us"].as_u64().unwrap());
        assert_eq!(event.order, 0);
        let kind = match event.kind() {
            SummaryKind::Medal => "medal",
            SummaryKind::Kill => "kill",
            SummaryKind::Death => "death",
            SummaryKind::Mode => "mode",
            SummaryKind::Unknown => "unknown",
        };
        assert_eq!(Some(kind), expected["kind"].as_str());
        let medal = match event.payload {
            SummaryPayload::Medal(medal) => medal.name(),
            _ => None,
        };
        assert_eq!(medal, expected["medal"].as_str());
        assert_eq!(event.derivation, SummaryDerivation::GuardedV41Layout);
        assert_eq!(
            event.source,
            SourceRef {
                chunk: 1,
                packet: 0,
                record: RecordRef::Summary(0)
            }
        );
        assert!(matches!(
            runtime.record(event.source),
            Some(Record::Summary(_))
        ));
        assert_eq!(
            runtime
                .query_summaries(SummaryFilter {
                    xuid: Some(42),
                    start_us: Some(event.timestamp_us),
                    end_us: Some(event.timestamp_us),
                    kind: Some(event.kind()),
                    ..Default::default()
                })
                .count(),
            1
        );
        assert_eq!(
            runtime
                .query_summaries(SummaryFilter {
                    xuid: Some(99),
                    ..Default::default()
                })
                .count(),
            0
        );
        assert_eq!(
            runtime
                .query_summaries(SummaryFilter {
                    start_us: Some(event.timestamp_us + 1),
                    end_us: Some(event.timestamp_us),
                    ..Default::default()
                })
                .count(),
            0
        );
        assert_eq!(
            runtime
                .query_summaries(SummaryFilter {
                    medal: Some(Medal::Splatter),
                    ..Default::default()
                })
                .count(),
            usize::from(event.payload == SummaryPayload::Medal(Medal::Splatter))
        );
        if case["name"] == "splatter" {
            assert_eq!(
                event.to_string(),
                "Nuzzles (42) received Splatter at 0:10.000"
            );
        }
    }
}

#[test]
fn summary_order_is_chronological_and_matches_the_complete_event_stream() {
    let input = |ms| serde_json::json!({"type":50,"metadata":156,"flag":1,"ms":ms});
    let first = summary_payload(&input(2000));
    let second = summary_payload(&input(1000));
    let third = summary_payload(&input(1000));
    let runtime = TheaterRuntime::load(
        Film::parse([
            registry(),
            crate::theater::film::FilmChunk {
                kind: crate::theater::film::ChunkKind::Summary,
                index: None,
                start_ms: None,
                data: [
                    packet(&first, 1000000),
                    packet(&second, 1000000),
                    packet(&third, 1000000),
                ]
                .concat(),
            },
        ])
        .unwrap(),
    );
    assert_eq!(
        runtime
            .summary_events()
            .iter()
            .map(|event| (event.timestamp_us, event.source.packet, event.order))
            .collect::<Vec<_>>(),
        vec![(1000000, 1, 2), (1000000, 2, 4), (2000000, 0, 0)]
    );
    for summary in runtime.summary_events() {
        let event = runtime
            .events()
            .iter()
            .find(|event| event.source == summary.source)
            .unwrap();
        assert_eq!(event.order, summary.order);
        assert_eq!(event.provenance, Provenance::DerivedSummary);
    }
}

#[test]
fn summary_reports_keep_missing_counts_and_disagreement_explicit() {
    let mut payload =
        summary_payload(&serde_json::json!({"type":50,"metadata":156,"flag":1,"ms":0}));
    put(&mut payload, 0, &2u32.to_be_bytes());
    let runtime = load_summary(&payload, 0);
    assert_eq!(runtime.summary_events().len(), 1);
    assert_eq!(runtime.summary_reports()[0].count_matches(), Some(false));
    let runtime = load_summary(&[0, 0, 0], 0);
    assert!(runtime.summary_events().is_empty());
    assert_eq!(runtime.summary_reports()[0].declared_events, None);
    assert_eq!(runtime.summary_reports()[0].count_matches(), None);
}

#[test]
fn typed_summary_filters_intersect_actor_kind_medal_and_time() {
    use medals::Medal;
    let inputs = [
        serde_json::json!({"type":50,"metadata":156,"flag":1,"ms":1000,"xuid":42}),
        serde_json::json!({"type":50,"metadata":156,"flag":0,"ms":2000,"xuid":42}),
        serde_json::json!({"type":50,"metadata":156,"flag":1,"ms":2000,"xuid":99}),
        serde_json::json!({"type":50,"metadata":156,"flag":1,"ms":3000,"xuid":42}),
    ];
    let data = inputs
        .iter()
        .flat_map(|input| packet(&summary_payload(input), 0))
        .collect();
    let runtime = TheaterRuntime::load(
        Film::parse([
            registry(),
            crate::theater::film::FilmChunk {
                kind: crate::theater::film::ChunkKind::Summary,
                index: None,
                start_ms: None,
                data,
            },
        ])
        .unwrap(),
    );
    assert_eq!(
        runtime
            .query_summaries(SummaryFilter {
                xuid: Some(42),
                kind: Some(SummaryKind::Medal),
                medal: Some(Medal::Splatter),
                start_us: Some(2000000),
                end_us: Some(3000000)
            })
            .map(|event| event.timestamp_us)
            .collect::<Vec<_>>(),
        vec![3000000]
    );
    assert_eq!(
        runtime
            .query_summaries(SummaryFilter {
                xuid: Some(42),
                kind: Some(SummaryKind::Kill),
                ..Default::default()
            })
            .count(),
        1
    );
    assert_eq!(
        runtime
            .query_summaries(SummaryFilter {
                xuid: Some(42),
                kind: Some(SummaryKind::Kill),
                medal: Some(Medal::Splatter),
                ..Default::default()
            })
            .count(),
        0
    );
    assert_eq!(
        runtime
            .query_summaries(SummaryFilter {
                xuid: Some(99),
                ..Default::default()
            })
            .count(),
        1
    );
}

#[test]
#[ignore = "requires the separately retained v41 corpus and captured service match stats"]
fn captured_v41_summary_semantics() {
    use serde_json::{Value, json};
    use std::io::Read;
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("experiments/films")
        });
    let mut golden = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../fixtures/highlights-corpus-v41.json.zlib")[..],
    )
    .read_to_end(&mut golden)
    .unwrap();
    let golden: Value = serde_json::from_slice(&golden).unwrap();
    let mut compared = 0;
    let mut compared_films = 0;
    for (name, expected) in golden.as_object().unwrap() {
        let Some(expected_events) = expected["events"]
            .as_array()
            .filter(|events| !events.is_empty())
        else {
            continue;
        };
        let directory = root.join(name);
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(directory.join("film.json")).unwrap()).unwrap();
        let mut chunks = Vec::new();
        for chunk in manifest["chunks"].as_array().unwrap() {
            let kind = chunk["chunk_type"].as_i64().unwrap() as i32;
            if !matches!(kind, 1 | 3) {
                continue;
            }
            chunks.push(crate::theater::film::FilmChunk {
                kind: crate::theater::film::ChunkKind::try_from(kind).unwrap(),
                index: chunk["index"].as_i64(),
                start_ms: chunk["start_time_offset_ms"].as_i64(),
                data: std::fs::read(directory.join(chunk["file"].as_str().unwrap())).unwrap(),
            });
        }
        let runtime = TheaterRuntime::load(Film::parse(chunks).unwrap());
        let actual: Vec<_> = runtime
            .summary_events()
            .iter()
            .map(|event| {
                let kind = match event.kind() {
                    SummaryKind::Mode => "mode",
                    SummaryKind::Death => "death",
                    SummaryKind::Kill => "kill",
                    SummaryKind::Medal => "medal",
                    SummaryKind::Unknown => "unknown",
                };
                json!({"XUID":event.actor.xuid,"Gamertag":event.actor.gamertag,"EventType":kind,
                "TypeHint":event.codes.type_code,"IsMedal":event.codes.medal_flag == 1,
                "TimeMS":event.timestamp_us / 1000,"MedalType":event.codes.metadata})
            })
            .collect();
        let mut expected_events = expected_events.clone();
        expected_events.sort_by_key(|event| event["TimeMS"].as_u64().unwrap());
        assert_eq!(
            actual, expected_events,
            "reference semantic events for {name}"
        );
        assert!(
            runtime
                .summary_reports()
                .iter()
                .filter(|report| report.declared_events.is_some())
                .all(|report| report.count_matches() == Some(true)),
            "summary counts for {name}"
        );
        compared += actual.len();
        compared_films += 1;
        if name == "ranked-arena/02-oddball" {
            // Independent Halo service counts; never use parser-generated decoded
            // counts from summary-events-validation.json as expectations.
            let stats: Value = serde_json::from_slice(
                &std::fs::read(directory.join("settings/match-stats.json")).unwrap(),
            )
            .unwrap();
            for player in stats["Players"].as_array().unwrap() {
                let id = player["PlayerId"].as_str().unwrap();
                let Some(xuid) = id.strip_prefix("xuid(").and_then(|id| id.strip_suffix(')'))
                else {
                    continue;
                };
                let xuid: u64 = xuid.parse().unwrap();
                let count = |kind| {
                    runtime
                        .query_summaries(SummaryFilter {
                            xuid: Some(xuid),
                            kind: Some(kind),
                            ..Default::default()
                        })
                        .count() as u64
                };
                let service_count = |field: &str| {
                    player["PlayerTeamStats"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|team| team["Stats"]["CoreStats"][field].as_u64().unwrap())
                        .sum::<u64>()
                };
                let medals: u64 = player["PlayerTeamStats"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|team| team["Stats"]["CoreStats"]["Medals"].as_array().unwrap())
                    .map(|medal| medal["Count"].as_u64().unwrap())
                    .sum();
                assert_eq!(
                    count(SummaryKind::Kill),
                    service_count("Kills"),
                    "service kills for {xuid}"
                );
                assert_eq!(
                    count(SummaryKind::Death),
                    service_count("Deaths"),
                    "service deaths for {xuid}"
                );
                assert_eq!(
                    count(SummaryKind::Medal),
                    medals,
                    "service medals for {xuid}"
                );
            }
        }
    }
    assert!(compared_films > 0);
    assert!(compared > 0);
    eprintln!(
        "verified {compared} pinned reference semantic events across {compared_films} films, plus independent service counts"
    );
}
