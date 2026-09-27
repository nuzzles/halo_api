use super::*;
use std::{collections::BTreeMap, io::Read};
#[test]
fn native_score_objective_document_stage() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/replay-score-document-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 1536);
    let mut totals = [0; 5];
    for (i, row) in rows.iter().enumerate() {
        let stage = &row["stage"];
        let records: Vec<StatborgRecord> =
            serde_json::from_value(row["full_records"].clone()).unwrap();
        let lines: Vec<StatborgPlayerLine> =
            serde_json::from_value(row["full_lines"].clone()).unwrap();
        let objectives: Vec<StatborgIdentifiedEvent> =
            serde_json::from_value(stage["events"].clone()).unwrap();
        let mut doc: ReplayDocument = serde_json::from_value(stage["before"].clone()).unwrap();
        let coverage = doc.coverage.clone();
        let teams: FilmPlayerTeams = serde_json::from_value(stage["team_input"].clone()).unwrap();
        let indices = PlayerIndexTable::default();
        let slots = BTreeMap::new();
        let ambiguous = Default::default();
        let control = serde_json::from_value(stage["control"].clone()).unwrap();
        let publication =
            ReplayTeamPublication::from_tables(&indices, &slots, &ambiguous, &teams, &control);
        let bridge = serde_json::from_value(stage["bridge"].clone()).unwrap();
        let result = attach_replay_score_and_objectives_to_document(
            &mut doc,
            ReplayScoreDocumentInput {
                score: (!row["absent"].as_bool().unwrap()).then(|| ReplayScoreInput {
                    teams: ReplayTeamScoreInput {
                        records: &records,
                        lines: &lines,
                        team_by_xuid: serde_json::from_value(row["team_by_xuid"].clone()).unwrap(),
                        team_scores: serde_json::from_value(row["team_scores"].clone()).unwrap(),
                        target_score: row["target_score"].as_i64().unwrap(),
                    },
                    truncated: row["truncated"].as_bool().unwrap(),
                    hold_ticks_per_point: row["hold_ticks_per_point"].as_i64().unwrap(),
                }),
                deaths: &[],
                objectives: &objectives,
                objectives_unnamed: stage["unnamed"].as_i64().unwrap(),
                objectives_refused: stage["refused"].as_i64().unwrap(),
            },
            ReplayScoreDocumentContext {
                interval_ms: row["clock"]["interval_ms"].as_i64().unwrap(),
                teams: &publication,
                track_counts: serde_json::from_value(stage["counts"].clone()).unwrap(),
                slot_xuids: &bridge,
            },
        );
        assert_eq!(
            doc,
            serde_json::from_value(stage["after"].clone()).unwrap(),
            "document {i}"
        );
        assert_eq!(doc.coverage, coverage, "coverage envelope {i}");
        assert_eq!(
            result.teams,
            serde_json::from_value(stage["teams"].clone()).unwrap(),
            "teams {i}"
        );
        assert_eq!(
            result.objectives,
            serde_json::from_value(stage["objectives"].clone()).unwrap(),
            "objectives {i}"
        );
        assert_eq!(
            result.score,
            serde_json::from_value(stage["score"].clone()).unwrap(),
            "score {i}"
        );
        assert_eq!(
            result.diagnostics,
            serde_json::from_value::<Vec<StatborgDiagnostic>>(stage["diagnostics"].clone())
                .unwrap(),
            "diagnostics {i}"
        );
        assert_eq!(
            result.round_zero_fallbacks as u64,
            stage["round_zero_fallbacks"].as_u64().unwrap(),
            "fallback {i}"
        );
        assert_eq!(
            result.objectives_without_track as u64,
            stage["without_track"].as_u64().unwrap(),
            "without track {i}"
        );
        assert_eq!(result.clock.origin_ms, doc.content.origin_ms.unwrap_or(0));
        assert_eq!(result.clock.frames, doc.content.frame_count);
        assert_eq!(
            result.clock.interval_ms,
            row["clock"]["interval_ms"].as_i64().unwrap()
        );
        totals[0] += doc.content.objectives.len();
        totals[1] += result.diagnostics.len();
        totals[2] += result.objectives_without_track;
        totals[3] += result.round_zero_fallbacks;
        totals[4] += usize::from(result.score.is_none());
    }
    assert!(totals.iter().all(|&n| n > 0));
    println!("score/objective stage: {totals:?}");
}
