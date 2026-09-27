//! Native loaded-film objective entry points. Each call scans statborg once;
//! consumers needing several projections should scan once and use record APIs.
use super::*;
use serde::{Deserialize, Serialize};

/// Retain source truncation explicitly, in addition to the native wrapper value
/// and its ordered source/pass diagnostics. Go's convenience wrappers drop this
/// flag; their underlying StatRecordsCtx supplies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgSourceOutput<T> {
    pub value: T,
    /// The source record cap was reached. Pass-level event-budget exhaustion
    /// is a separate diagnostic and does not change this source-only flag.
    pub truncated: bool,
    pub diagnostics: Vec<StatborgDiagnostic>,
}

fn source_pass<T>(
    source: Option<&FilmSource>,
    pass: impl FnOnce(&[StatborgRecord]) -> (T, Vec<StatborgDiagnostic>),
) -> StatborgSourceOutput<T> {
    // Native convenience wrappers use StatRecords with an empty match ID.
    let (stream, mut diagnostics) = scan_source_statborg_with_diagnostics(source, "");
    let (value, pass_diagnostics) = pass(&stream.records);
    diagnostics.extend(pass_diagnostics);
    StatborgSourceOutput {
        value,
        truncated: stream.truncated,
        diagnostics,
    }
}

/// Native NamedEvents: source scanning precedes even an unsupported mode lookup.
pub fn statborg_source_named_events(
    source: Option<&FilmSource>,
    mode: &str,
) -> StatborgSourceOutput<Vec<StatborgNamedEvent>> {
    source_pass(source, |records| {
        statborg_named_events_with_diagnostics(records, mode)
    })
}

/// Native CrossCheckNamedEvents, retaining source and pass warnings.
pub fn statborg_source_cross_check_named(
    source: Option<&FilmSource>,
    mode: &str,
) -> StatborgSourceOutput<StatborgNamedCrossCheck> {
    source_pass(source, |records| {
        statborg_cross_check_named_with_diagnostics(records, mode)
    })
}

/// Native SlotIdentity, plus the existing Rust pass-budget value and source flag.
pub fn statborg_source_slot_identity(
    source: Option<&FilmSource>,
    lines: &[StatborgPlayerLine],
) -> StatborgSourceOutput<StatborgSlotIdentity> {
    source_pass(source, |records| {
        statborg_slot_identity_with_diagnostics(records, lines)
    })
}

/// Native SlotIdentityResolved: scan once, then compare totals and death-time
/// identity evidence. Source warnings precede totals-pass warnings.
pub fn statborg_source_resolved_identity(
    source: Option<&FilmSource>,
    lines: &[StatborgPlayerLine],
    deaths: &[StatborgDeathInstant],
) -> StatborgSourceOutput<StatborgResolvedIdentity> {
    source_pass(source, |records| {
        resolve_statborg_identity_with_diagnostics(records, lines, deaths)
    })
}

/// Native SlotIdentityFromDeaths. Retain source warnings and truncation even
/// though the death-only pass has no additional diagnostic publications.
pub fn statborg_source_identity_by_deaths(
    source: Option<&FilmSource>,
    deaths: &[StatborgDeathInstant],
) -> StatborgSourceOutput<std::collections::BTreeMap<i64, String>> {
    source_pass(source, |records| {
        (statborg_identity_by_deaths(records, deaths), Vec::new())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Expected {
        lines: Vec<StatborgPlayerLine>,
        named: Vec<StatborgNamedEvent>,
        named_logs: Vec<StatborgDiagnostic>,
        cross: StatborgNamedCrossCheck,
        cross_logs: Vec<StatborgDiagnostic>,
        identity: BTreeMap<i64, String>,
        identity_logs: Vec<StatborgDiagnostic>,
        resolved: StatborgResolvedIdentity,
        resolved_logs: Vec<StatborgDiagnostic>,
        stream: FilmStatborgStream,
    }
    #[derive(Deserialize)]
    struct Case {
        chunks: Vec<String>,
        metadata: Vec<FilmSourceMetadata>,
        mode: String,
    }
    fn fixture<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_end(&mut raw)
            .unwrap();
        serde_json::from_slice(&raw).unwrap()
    }
    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    fn check(source: Option<&FilmSource>, mode: &str, expected: Expected) {
        let named = statborg_source_named_events(source, mode);
        assert_eq!(named.value, expected.named);
        assert_eq!(named.diagnostics, expected.named_logs);
        assert_eq!(named.truncated, expected.stream.truncated);
        let cross = statborg_source_cross_check_named(source, mode);
        assert_eq!(cross.value, expected.cross);
        assert_eq!(cross.diagnostics, expected.cross_logs);
        assert_eq!(cross.truncated, expected.stream.truncated);
        let identity = statborg_source_slot_identity(source, &expected.lines);
        assert_eq!(identity.value.by_slot, expected.identity);
        assert_eq!(identity.diagnostics, expected.identity_logs);
        assert_eq!(identity.truncated, expected.stream.truncated);
        let resolved = statborg_source_resolved_identity(source, &expected.lines, &[]);
        assert_eq!(resolved.value, expected.resolved);
        assert_eq!(resolved.diagnostics, expected.resolved_logs);
        assert_eq!(resolved.truncated, expected.stream.truncated);
    }
    #[test]
    fn native_loaded_resolved_death_identity() {
        let rows: Vec<serde_json::Value> = fixture(include_bytes!(
            "fixtures/statborg-source-resolved-v41.json.zlib"
        ));
        assert_eq!(rows.len(), 4);
        let mut death_results = 0;
        for row in rows {
            let source = FilmSource::load(
                &[hex(row["chunk"].as_str().unwrap())],
                &[FilmSourceMetadata {
                    index: 44,
                    chunk_type: 2,
                    start_ms: 500,
                }],
            )
            .unwrap();
            let lines: Vec<StatborgPlayerLine> =
                serde_json::from_value(row["lines"].clone()).unwrap();
            let deaths: Vec<StatborgDeathInstant> =
                serde_json::from_value(row["deaths"].clone()).unwrap();
            let actual = statborg_source_resolved_identity(Some(&source), &lines, &deaths);
            let expected: StatborgResolvedIdentity =
                serde_json::from_value(row["resolved"].clone()).unwrap();
            assert_eq!(actual.value, expected);
            assert_eq!(
                actual.diagnostics,
                serde_json::from_value::<Vec<StatborgDiagnostic>>(row["logs"].clone()).unwrap()
            );
            assert!(!actual.truncated);
            let death_only = statborg_source_identity_by_deaths(Some(&source), &deaths);
            assert_eq!(
                death_only.value,
                serde_json::from_value::<BTreeMap<i64, String>>(row["death_only"].clone()).unwrap()
            );
            assert_eq!(
                death_only.diagnostics,
                serde_json::from_value::<Vec<StatborgDiagnostic>>(row["death_logs"].clone())
                    .unwrap()
            );
            assert!(!death_only.truncated);
            assert_eq!(
                serde_json::from_value::<StatborgSourceOutput<BTreeMap<i64, String>>>(
                    serde_json::to_value(&death_only).unwrap()
                )
                .unwrap(),
                death_only
            );
            death_results += usize::from(actual.value.stats.source == "deaths");
        }
        assert_eq!(death_results, 1);
    }

    #[test]
    fn native_loaded_objective_passes() {
        let cases: Vec<serde_json::Value> =
            fixture(include_bytes!("fixtures/statborg-source-v41.json.zlib"));
        assert_eq!(cases.len(), 128);
        for raw in cases {
            let case: Case = serde_json::from_value(raw.clone()).unwrap();
            let expected: Expected = serde_json::from_value(raw).unwrap();
            let chunks: Vec<_> = case.chunks.iter().map(|s| hex(s)).collect();
            let source =
                (!chunks.is_empty()).then(|| FilmSource::load(&chunks, &case.metadata).unwrap());
            check(source.as_ref(), &case.mode, expected);
        }
    }
    #[test]
    fn native_loaded_objective_positive_control() {
        let rows: Vec<serde_json::Value> = fixture(include_bytes!(
            "fixtures/statborg-source-positive-v41.json.zlib"
        ));
        assert_eq!(rows.len(), 1);
        for raw in rows {
            let case: Case = serde_json::from_value(raw.clone()).unwrap();
            let expected: Expected = serde_json::from_value(raw).unwrap();
            assert_eq!(expected.named.len(), 6);
            assert_eq!(
                expected.identity.get(&10).map(String::as_str),
                Some("named-player")
            );
            assert_eq!(expected.cross[&10].len(), 2);
            let chunks: Vec<_> = case.chunks.iter().map(|s| hex(s)).collect();
            let source = FilmSource::load(&chunks, &case.metadata).unwrap();
            check(Some(&source), &case.mode, expected);
        }
    }
    #[test]
    fn native_loaded_objective_pass_truncation_order() {
        let raw: serde_json::Value = fixture(include_bytes!(
            "fixtures/statborg-source-limit-v41.json.zlib"
        ));
        let payload = hex(raw["payload"].as_str().unwrap());
        let expected: Expected = serde_json::from_value(raw).unwrap();
        let packet = |timestamp: u64| {
            let mut data = vec![0; 16];
            data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
            data[8..16].copy_from_slice(&timestamp.to_le_bytes());
            data.extend_from_slice(&payload);
            data
        };
        let many: Vec<_> = (0..11026).flat_map(|j| packet(j * 1000)).collect();
        let metadata = [
            FilmSourceMetadata {
                index: 18,
                chunk_type: 2,
                start_ms: 400,
            },
            FilmSourceMetadata {
                index: -7,
                chunk_type: 3,
                start_ms: -200,
            },
            FilmSourceMetadata {
                index: 99,
                chunk_type: 2,
                start_ms: 90000,
            },
        ];
        let source = FilmSource::load(&[packet(1000), many, packet(1000)], &metadata).unwrap();
        assert!(expected.stream.truncated);
        assert_eq!(expected.named_logs.len(), 5);
        check(Some(&source), "flag", expected);
    }
}
