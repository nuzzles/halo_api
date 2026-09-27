use super::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Read;

#[derive(Deserialize)]
struct Case {
    points: Vec<StatborgScorePoint>,
    direct: Vec<StatborgScorePoint>,
    direct_logs: Vec<StatborgDiagnostic>,
    raw: StatborgRoundSeries,
    real: BTreeMap<i64, bool>,
    cumulative: StatborgSlotSeries,
    cumulative_logs: Vec<StatborgDiagnostic>,
    records: Vec<StatborgRecord>,
    total: StatborgSlotSeries,
    total_logs: Vec<StatborgDiagnostic>,
    events: Vec<StatborgNamedEvent>,
    event_logs: Vec<StatborgDiagnostic>,
    identity: BTreeMap<i64, String>,
    identity_logs: Vec<StatborgDiagnostic>,
    cross: StatborgNamedCrossCheck,
    cross_logs: Vec<StatborgDiagnostic>,
}

#[test]
fn native_chronology_warnings() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/chronology-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 128);
    for (i, case) in cases.into_iter().enumerate() {
        let (points, logs) = statborg_chronological_total_with_diagnostics(&case.points);
        assert_eq!(points, case.direct, "direct {i}");
        assert_eq!(logs, case.direct_logs, "direct logs {i}");
        assert_eq!(points, statborg_chronological_total(&case.points));
        let real = case
            .real
            .into_iter()
            .filter_map(|(r, admitted)| admitted.then_some(r))
            .collect();
        let (points, logs) = statborg_cumulate_rounds_with_diagnostics(&case.raw, &real);
        assert_eq!(points, case.cumulative, "cumulative {i}");
        assert_eq!(points, statborg_cumulate_rounds(&case.raw, &real));
        // Each slot invokes ChronologicalTotal once. Native Go's map walk is
        // unspecified across slots, so compare that collection by slot while
        // preserving the exact message and attribute ordering of every record.
        let slot = |d: &StatborgDiagnostic| d.attributes[0].1.as_i64().unwrap();
        assert!(logs.windows(2).all(|w| slot(&w[0]) < slot(&w[1])));
        let mut expected = case.cumulative_logs;
        expected.sort_by_key(slot);
        assert_eq!(logs, expected, "cumulative logs {i}");
        let (total, logs) = KILLS_COMPONENT.series_total_with_diagnostics(&case.records, false);
        assert_eq!(total, case.total, "total {i}");
        assert_eq!(logs, case.total_logs, "total logs {i}");
        assert_eq!(total, KILLS_COMPONENT.series_total(&case.records, false));
        // The record-based cases have one slot, so native publication order is
        // fully specified, including chronology warnings between budget ones.
        let (events, logs) = statborg_named_events_with_diagnostics(&case.records, "flag");
        assert_eq!(events, case.events, "events {i}");
        assert_eq!(logs, case.event_logs, "event logs {i}");
        let (identity, logs) = statborg_slot_identity_with_diagnostics(&case.records, &[]);
        assert_eq!(identity.by_slot, case.identity, "identity {i}");
        assert_eq!(logs, case.identity_logs, "identity logs {i}");
        let (cross, logs) = statborg_cross_check_named_with_diagnostics(&case.records, "flag");
        assert_eq!(cross, case.cross, "cross {i}");
        assert_eq!(logs, case.cross_logs, "cross logs {i}");
    }
}
