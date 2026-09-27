//! Ordered native objective-budget warnings. Opt-in observation leaves the legacy
//! budget's serialized value and equality contract unchanged.
use super::{StatborgCounterKey, StatborgEventBudget, StatborgScorePoint};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgDiagnostic {
    pub level: String,
    pub message: String,
    /// Attribute order is the order supplied to native slog, not a sorted map.
    pub attributes: Vec<(String, Value)>,
}

/// Compatibility name for the shared native statborg warning record.
pub type StatborgBudgetDiagnostic = StatborgDiagnostic;

/// Keep one observer for the entire pass: the eight-detail cap spans counters
/// and calls. Summary calls deliberately publish again each time.
#[derive(Debug, Clone, Default)]
pub struct StatborgBudgetDiagnostics {
    pub origin: String,
    detailed_rejections: usize,
    pub records: Vec<StatborgBudgetDiagnostic>,
}
impl StatborgBudgetDiagnostics {
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
            ..Self::default()
        }
    }
    pub fn increment_times(
        &mut self,
        budget: &mut StatborgEventBudget,
        points: &[StatborgScorePoint],
        key: StatborgCounterKey,
    ) -> Vec<i64> {
        budget.increment_times_observed(points, |exhausted, point, unroll, remaining| {
            if !exhausted {
                if self.detailed_rejections >= 8 {
                    return;
                }
                self.detailed_rejections += 1;
            }
            let mut attributes = vec![
                ("passe".into(), json!(self.origin)),
                ("comp".into(), json!(key.component)),
                ("cote".into(), json!(key.side)),
                ("slot".into(), json!(point.slot)),
                ("time_ms".into(), json!(point.time_ms)),
                ("deroulage".into(), json!(unroll)),
            ];
            let message = if exhausted {
                attributes.push(("reste".into(), json!(remaining)));
                attributes.push(("plafond".into(), json!(1_000_000)));
                "objectives: plafond d'evenements du film atteint, deroulage interrompu"
            } else {
                attributes.push(("borne".into(), json!(16)));
                "objectives: deroulage aberrant rejete (dernier rempart memoire)"
            };
            self.records.push(StatborgBudgetDiagnostic {
                level: "WARN".into(),
                message: message.into(),
                attributes,
            });
        })
    }
    pub fn summarize(&mut self, budget: &StatborgEventBudget) {
        if budget.rejected == 0 && !budget.truncated {
            return;
        }
        self.records.push(StatborgBudgetDiagnostic {
            level: "WARN".into(),
            message: "objectives: bornes de deroulage appliquees sur ce film".into(),
            attributes: vec![
                ("passe".into(), json!(self.origin)),
                ("deroulages_rejetes".into(), json!(budget.rejected)),
                ("tronque".into(), json!(budget.truncated)),
                (
                    "evenements_emis".into(),
                    json!(1_000_000 - budget.remaining()),
                ),
                ("plafond".into(), json!(1_000_000)),
            ],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::*;
    use std::collections::BTreeMap;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Call {
        points: Vec<StatborgScorePoint>,
        key: StatborgCounterKey,
        times: Vec<i64>,
    }
    #[derive(Deserialize)]
    struct Case {
        initial: usize,
        calls: Vec<Call>,
        remaining: usize,
        rejected: usize,
        truncated: bool,
        records: Vec<StatborgBudgetDiagnostic>,
    }
    #[derive(Deserialize)]
    struct PassCase {
        records: Vec<StatborgRecord>,
        lines: Vec<StatborgPlayerLine>,
        mode: String,
        events: Vec<StatborgNamedEvent>,
        event_logs: Vec<StatborgBudgetDiagnostic>,
        identity: BTreeMap<i64, String>,
        identity_logs: Vec<StatborgBudgetDiagnostic>,
        cross: BTreeMap<i64, BTreeMap<String, [usize; 2]>>,
        cross_logs: Vec<StatborgBudgetDiagnostic>,
    }
    #[derive(Deserialize)]
    struct ExhaustionCase {
        event_count: usize,
        event_digest: u64,
        event_logs: Vec<StatborgBudgetDiagnostic>,
        identity: BTreeMap<i64, String>,
        identity_logs: Vec<StatborgBudgetDiagnostic>,
        cross: BTreeMap<i64, BTreeMap<String, [usize; 2]>>,
        cross_logs: Vec<StatborgBudgetDiagnostic>,
    }
    #[test]
    fn native_budget_full_pass_exhaustion() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/budget-exhaustion-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let case: ExhaustionCase = serde_json::from_slice(&raw).unwrap();
        // Controlled input shared with the Go harness; expectations come only
        // from its native public passes. This exceeds the real film budget.
        let records: Vec<_> = (0..62503)
            .map(|j| {
                let v = j * 16;
                StatborgRecord {
                    time_ms: j * 10,
                    slot: 10,
                    round: 0,
                    comps: BTreeMap::from([
                        (
                            2,
                            StatborgValue {
                                a: v,
                                b: v,
                                ..Default::default()
                            },
                        ),
                        (
                            3,
                            StatborgValue {
                                a: v,
                                ..Default::default()
                            },
                        ),
                        (
                            12,
                            StatborgValue {
                                a: v,
                                b: v,
                                ..Default::default()
                            },
                        ),
                    ]),
                }
            })
            .collect();
        let (events, logs) = statborg_named_events_with_diagnostics(&records, "flag");
        assert_eq!(events.len(), case.event_count);
        // FNV-1a over every ordered field, matching Go's hash/fnv byte stream.
        let mut digest = 0xcbf29ce484222325u64;
        for e in &events {
            let side = match e.side {
                StatborgSide::A => b'A',
                StatborgSide::B => b'B',
            };
            for byte in e
                .time_ms
                .to_le_bytes()
                .into_iter()
                .chain(e.slot.to_le_bytes())
                .chain(e.comp.to_le_bytes())
                .chain(e.stat.bytes())
                .chain([0, side, 0])
            {
                digest = (digest ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        assert_eq!(digest, case.event_digest);
        assert_eq!(logs, case.event_logs);
        drop(events);
        let lines = [StatborgPlayerLine {
            xuid: "exhaustion".into(),
            kills: 1_000_000,
            deaths: 0,
            assists: 0,
        }];
        let (identity, logs) = statborg_slot_identity_with_diagnostics(&records, &lines);
        assert_eq!(identity.by_slot, case.identity);
        assert_eq!(logs, case.identity_logs);
        let (cross, logs) = statborg_cross_check_named_with_diagnostics(&records, "flag");
        assert_eq!(cross, case.cross);
        assert_eq!(logs, case.cross_logs);
    }
    #[test]
    fn native_budget_pass_oracle() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/budget-passes-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<PassCase> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 64);
        for (i, case) in cases.into_iter().enumerate() {
            let (events, logs) = statborg_named_events_with_diagnostics(&case.records, &case.mode);
            assert_eq!(events, case.events, "events case {i}");
            assert_eq!(logs, case.event_logs, "event logs case {i}");
            assert_eq!(events, statborg_named_events(&case.records, &case.mode));
            let (identity, logs) =
                statborg_slot_identity_with_diagnostics(&case.records, &case.lines);
            assert_eq!(identity.by_slot, case.identity, "identity case {i}");
            assert_eq!(logs, case.identity_logs, "identity logs case {i}");
            assert_eq!(identity, statborg_slot_identity(&case.records, &case.lines));
            let (cross, logs) =
                statborg_cross_check_named_with_diagnostics(&case.records, &case.mode);
            assert_eq!(cross, case.cross, "cross case {i}");
            assert_eq!(logs, case.cross_logs, "cross logs case {i}");
            assert_eq!(cross, statborg_cross_check_named(&case.records, &case.mode));
        }
    }
    #[test]
    fn native_budget_diagnostic_oracle() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/budget-diagnostics-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 64);
        for (i, case) in cases.into_iter().enumerate() {
            let mut budget: StatborgEventBudget = serde_json::from_value(json!({
                "remaining":case.initial, "rejected":0, "truncated":false
            }))
            .unwrap();
            let mut observer = StatborgBudgetDiagnostics::new("rust-budget");
            observer.summarize(&budget);
            assert!(observer.records.is_empty());
            for call in case.calls {
                assert_eq!(
                    observer.increment_times(&mut budget, &call.points, call.key),
                    call.times,
                    "case {i}"
                );
                observer.summarize(&budget);
            }
            observer.summarize(&budget);
            assert_eq!(budget.remaining(), case.remaining, "case {i}");
            assert_eq!(budget.rejected, case.rejected, "case {i}");
            assert_eq!(budget.truncated, case.truncated, "case {i}");
            assert_eq!(observer.records, case.records, "case {i}");
        }
    }
}
