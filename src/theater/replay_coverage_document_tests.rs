use super::*;
use std::io::Read;
#[test]
fn native_coverage_document_stage() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/coverage-document-v41.json.zlib")[..])
        .read_to_end(&mut bytes)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut totals = [0; 5];
    for (i, row) in rows.iter().enumerate() {
        let mut doc: ReplayDocument = serde_json::from_value(row["before"].clone()).unwrap();
        let v = &row["input"];
        let own = &row["own"];
        let counter = |key: &str| own[key].as_u64().unwrap() as usize;
        let registry = IdentityRegistryOutput {
            owners: IdentityOwnerOutput {
                state: serde_json::from_value(row["state"].clone()).unwrap(),
                from_deaths: counter("FromDeaths"),
                deaths_named: counter("DeathsNamed"),
                lives_total: counter("LivesTotal"),
                index_readings: own["IndexReadings"].as_i64().unwrap(),
                index_disagreements: own["IndexDisagreements"].as_i64().unwrap(),
                clock: IdentityDeathClock {
                    offset_ms: own["DeathOffsetMS"].as_i64().unwrap(),
                    matched: counter("DeathOffsetMatches"),
                    runner_up: counter("DeathOffsetRunnerUp"),
                },
                ..Default::default()
            },
            tables: compose_identity_tables(
                &ReplayFilmPlayerTable::default(),
                &PlayerIndexTable::default(),
            ),
            scoreboard: Default::default(),
        };
        let naming = serde_json::from_value(row["naming"].clone()).unwrap();
        let closed = serde_json::from_value(row["closed"].clone()).unwrap();
        let identity: Option<ReplayDecoderIdentity> =
            serde_json::from_value(row["identity"].clone()).unwrap();
        let diagnostics = assemble_replay_document_coverage(
            &mut doc,
            ReplayCoverageDocumentInput {
                shots: serde_json::from_value(v["shots"].clone()).unwrap(),
                grenades: serde_json::from_value(v["grenades"].clone()).unwrap(),
                objectives: serde_json::from_value(v["objectives"].clone()).unwrap(),
                score: serde_json::from_value(v["score"].clone()).unwrap(),
                projectiles: serde_json::from_value(v["projectiles"].clone()).unwrap(),
                tracks: serde_json::from_value(v["tracks"].clone()).unwrap(),
                teams: serde_json::from_value(v["teams"].clone()).unwrap(),
                stances: serde_json::from_value(v["stances"].clone()).unwrap(),
                seats: serde_json::from_value(v["seats"].clone()).unwrap(),
                film_major_version: serde_json::from_value(v["major"].clone()).unwrap(),
                kills_read: v["kills_read"].as_bool().unwrap(),
                deaths_paths: serde_json::from_value(v["paths"].clone()).unwrap(),
            },
            ReplayCoverageDocumentContext {
                registry: &registry,
                naming: &naming,
                closed_by_death: &closed,
                decoder_identity: identity.as_ref(),
                interval_ms: row["interval"].as_i64().unwrap(),
            },
        );
        let expected: ReplayDocument = serde_json::from_value(row["after"].clone()).unwrap();
        assert_eq!(doc, expected, "whole document {i}");
        assert_eq!(
            diagnostics,
            serde_json::from_value::<Vec<StatborgDiagnostic>>(row["diagnostics"].clone()).unwrap(),
            "diagnostics {i}"
        );
        let c = doc.coverage.as_ref().unwrap();
        assert!(c.decoder.is_some());
        assert!(!c.verdict.contains_key("seed"));
        totals[0] += usize::from(c.t0_film.as_ref().is_some_and(|t| t.detected));
        totals[1] +=
            usize::from(doc.content.origin_ms.is_none() && doc.content.t0_film_ms == Some(999));
        totals[2] +=
            usize::from(!c.equipment.as_ref().unwrap().kills_read && c.deaths_paths.is_some());
        totals[3] += diagnostics.len();
        totals[4] += usize::from(
            doc.content.origin_ms == Some(i64::MAX)
                && doc.content.t0_film_ms.is_some_and(|t| t < 0),
        );
    }
    assert!(totals.iter().all(|&n| n > 0));
    println!("coverage document stage: {totals:?}");
}
