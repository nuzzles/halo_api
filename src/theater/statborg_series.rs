//! Native per-round counter series and bounded counter-to-event derivation.
use super::{StatborgRecord, resolve_statborg_round_bounds, resolve_statborg_rounds};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgScorePoint {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    pub slot: i64,
    pub value: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatborgSide {
    A,
    B,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgCounterKey {
    pub component: i64,
    pub side: StatborgSide,
}
pub type StatborgRoundSeries = BTreeMap<i64, BTreeMap<i64, Vec<StatborgScorePoint>>>;
pub fn statborg_raw_series(
    records: &[StatborgRecord],
    key: StatborgCounterKey,
    teams: bool,
) -> StatborgRoundSeries {
    let bounds = resolve_statborg_round_bounds(records);
    let mut raw = StatborgRoundSeries::new();
    for r in records {
        if (r.slot <= 8) != teams || bounds.excludes(r) {
            continue;
        }
        let Some(v) = r.comps.get(&key.component) else {
            continue;
        };
        let value = match key.side {
            StatborgSide::A => v.a,
            StatborgSide::B => v.b,
        };
        if value < 0 || (key.component == 0 && !super::statborg_mode_score_in_domain(v)) {
            continue;
        }
        raw.entry(r.slot)
            .or_default()
            .entry(r.round)
            .or_default()
            .push(StatborgScorePoint {
                time_ms: r.time_ms,
                slot: r.slot,
                value,
            });
    }
    raw
}
/// Reconstruct the native longest nondecreasing run, including its tie choice.
pub fn statborg_longest_run(
    points: &[StatborgScorePoint],
    strict: bool,
) -> Vec<StatborgScorePoint> {
    let mut tails: Vec<usize> = Vec::new();
    let mut prev = vec![None; points.len()];
    for (i, p) in points.iter().enumerate() {
        let at = tails.partition_point(|&j| {
            if strict {
                points[j].value < p.value
            } else {
                points[j].value <= p.value
            }
        });
        if at > 0 {
            prev[i] = Some(tails[at - 1]);
        }
        if at == tails.len() {
            tails.push(i);
        } else {
            tails[at] = i;
        }
    }
    let mut out = Vec::with_capacity(tails.len());
    let mut at = tails.last().copied();
    while let Some(i) = at {
        out.push(points[i].clone());
        at = prev[i];
    }
    out.reverse();
    out
}
/// Drop backward timestamps without sorting a cumulative round sequence.
pub fn statborg_chronological_total(points: &[StatborgScorePoint]) -> Vec<StatborgScorePoint> {
    statborg_chronological_total_observed(points, None)
}
/// Preserve the native warning for a backward-time drop. Equal timestamps and
/// a negative first timestamp are valid; the first point always survives.
pub fn statborg_chronological_total_with_diagnostics(
    points: &[StatborgScorePoint],
) -> (Vec<StatborgScorePoint>, Vec<super::StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = statborg_chronological_total_observed(points, Some(&mut diagnostics));
    (out, diagnostics)
}
pub(super) fn statborg_chronological_total_observed(
    points: &[StatborgScorePoint],
    diagnostics: Option<&mut Vec<super::StatborgDiagnostic>>,
) -> Vec<StatborgScorePoint> {
    let mut out: Vec<StatborgScorePoint> = Vec::new();
    let mut first_backward = None;
    for p in points {
        if out.last().is_none_or(|last| p.time_ms >= last.time_ms) {
            out.push(p.clone());
        } else {
            first_backward.get_or_insert(p.time_ms);
        }
    }
    if let (Some(time), Some(diagnostics)) = (first_backward, diagnostics) {
        use serde_json::json;
        diagnostics.push(super::StatborgDiagnostic {
            level: "WARN".into(),
            message: "objectives: serie cumulee NON CHRONOLOGIQUE — points ecartes".into(),
            attributes: vec![
                ("slot".into(), json!(points[0].slot)),
                ("ecartes".into(), json!(points.len() - out.len())),
                ("retenus".into(), json!(out.len())),
                ("premierRecul".into(), json!(time)),
            ],
        });
    }
    out
}
/// Cumulative series indexed by native slot.
pub type StatborgSlotSeries = BTreeMap<i64, Vec<StatborgScorePoint>>;
pub fn statborg_cumulate_rounds(
    raw: &StatborgRoundSeries,
    real: &BTreeSet<i64>,
) -> StatborgSlotSeries {
    statborg_cumulate_rounds_observed(raw, real, None)
}
/// Native Go iterates slot maps without a specified order. Rust emits per-slot
/// chronology warnings in ascending slot order; within each slot order is exact.
pub fn statborg_cumulate_rounds_with_diagnostics(
    raw: &StatborgRoundSeries,
    real: &BTreeSet<i64>,
) -> (StatborgSlotSeries, Vec<super::StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = statborg_cumulate_rounds_observed(raw, real, Some(&mut diagnostics));
    (out, diagnostics)
}
pub(super) fn statborg_cumulate_rounds_observed(
    raw: &StatborgRoundSeries,
    real: &BTreeSet<i64>,
    mut diagnostics: Option<&mut Vec<super::StatborgDiagnostic>>,
) -> StatborgSlotSeries {
    let mut out = BTreeMap::new();
    for (&slot, rounds) in raw {
        let mut offset = 0i64;
        let mut series = Vec::new();
        for (round, points) in rounds {
            if !real.contains(round) {
                continue;
            }
            let mut sorted = points.clone();
            sorted.sort_by_key(|p| p.time_ms);
            let kept = statborg_longest_run(&sorted, false);
            for p in &kept {
                series.push(StatborgScorePoint {
                    time_ms: p.time_ms,
                    slot,
                    value: p.value.wrapping_add(offset),
                });
            }
            if let Some(last) = kept.last() {
                offset = offset.wrapping_add(last.value);
            }
        }
        let series = statborg_chronological_total_observed(&series, diagnostics.as_deref_mut());
        if !series.is_empty() {
            out.insert(slot, series);
        }
    }
    out
}
pub fn statborg_series_by_slot(
    records: &[StatborgRecord],
    key: StatborgCounterKey,
) -> StatborgSlotSeries {
    statborg_series_by_slot_observed(records, key, None)
}
pub(super) fn statborg_series_by_slot_observed(
    records: &[StatborgRecord],
    key: StatborgCounterKey,
    diagnostics: Option<&mut Vec<super::StatborgDiagnostic>>,
) -> StatborgSlotSeries {
    statborg_cumulate_rounds_observed(
        &statborg_raw_series(records, key, false),
        &resolve_statborg_rounds(records).real.into_iter().collect(),
        diagnostics,
    )
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgBoundedStep {
    pub point: StatborgScorePoint,
    pub unroll: i64,
    pub kept: i64,
}
pub fn statborg_bound_steps(points: &[StatborgScorePoint]) -> Vec<StatborgBoundedStep> {
    let mut previous = 0i64;
    points
        .iter()
        .map(|p| {
            let delta = p.value.wrapping_sub(previous);
            let mut s = StatborgBoundedStep {
                point: p.clone(),
                unroll: 0,
                kept: 0,
            };
            if delta > 0 {
                previous = p.value;
                s.unroll = delta;
                if delta <= 16 {
                    s.kept = delta;
                }
            }
            s
        })
        .collect()
}
pub fn statborg_bounded_series(points: &[StatborgScorePoint]) -> Vec<StatborgScorePoint> {
    let mut total = 0i64;
    statborg_bound_steps(points)
        .into_iter()
        .map(|s| {
            total = total.wrapping_add(s.kept);
            StatborgScorePoint {
                value: total,
                ..s.point
            }
        })
        .collect()
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgEventBudget {
    remaining: usize,
    pub rejected: usize,
    pub truncated: bool,
}
impl Default for StatborgEventBudget {
    fn default() -> Self {
        Self {
            remaining: 1_000_000,
            rejected: 0,
            truncated: false,
        }
    }
}
impl StatborgEventBudget {
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    /// An increment that exceeds the remaining budget is not partially emitted.
    /// Later calls remain silent once truncation has been reported.
    pub fn increment_times(&mut self, points: &[StatborgScorePoint]) -> Vec<i64> {
        self.increment_times_observed(points, |_, _, _, _| {})
    }
    pub(super) fn increment_times_observed(
        &mut self,
        points: &[StatborgScorePoint],
        mut observe: impl FnMut(bool, &StatborgScorePoint, i64, usize),
    ) -> Vec<i64> {
        let mut out = Vec::new();
        if self.truncated {
            return out;
        }
        for s in statborg_bound_steps(points) {
            if s.unroll > 0 && s.kept == 0 {
                self.rejected += 1;
                observe(false, &s.point, s.unroll, self.remaining);
            } else if s.kept > 0 {
                if s.kept as usize > self.remaining {
                    self.truncated = true;
                    observe(true, &s.point, s.kept, self.remaining);
                    break;
                }
                self.remaining -= s.kept as usize;
                out.extend(std::iter::repeat_n(s.point.time_ms, s.kept as usize));
            }
        }
        out
    }
    pub(super) fn increment_times_diagnostic(
        &mut self,
        points: &[StatborgScorePoint],
        key: StatborgCounterKey,
        diagnostics: Option<&mut super::StatborgBudgetDiagnostics>,
    ) -> Vec<i64> {
        match diagnostics {
            Some(d) => d.increment_times(self, points, key),
            None => self.increment_times(points),
        }
    }
    pub fn counts_by_slot(
        &mut self,
        records: &[StatborgRecord],
        key: StatborgCounterKey,
    ) -> BTreeMap<i64, usize> {
        self.counts_by_slot_diagnostic(records, key, None)
    }
    pub(super) fn counts_by_slot_diagnostic(
        &mut self,
        records: &[StatborgRecord],
        key: StatborgCounterKey,
        mut diagnostics: Option<&mut super::StatborgBudgetDiagnostics>,
    ) -> BTreeMap<i64, usize> {
        statborg_series_by_slot_observed(
            records,
            key,
            diagnostics.as_deref_mut().map(|d| &mut d.records),
        )
        .into_iter()
        .map(|(slot, p)| {
            (
                slot,
                self.increment_times_diagnostic(&p, key, diagnostics.as_deref_mut())
                    .len(),
            )
        })
        .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgPlayerLine {
    #[serde(rename = "XUID")]
    pub xuid: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgSlotIdentity {
    pub by_slot: BTreeMap<i64, String>,
    pub budget: StatborgEventBudget,
}
/// Match the complete K/D/A triplet, then refuse every identity claimed by
/// multiple slots. All three counters share the native one-million-event budget.
pub fn statborg_slot_identity(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
) -> StatborgSlotIdentity {
    slot_identity_impl(records, lines, None)
}
/// Native slot-identity output and ordered budget publications for this pass.
pub fn statborg_slot_identity_with_diagnostics(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
) -> (StatborgSlotIdentity, Vec<super::StatborgBudgetDiagnostic>) {
    let mut diagnostics = super::StatborgBudgetDiagnostics::new("slot_identity");
    let out = slot_identity_impl(records, lines, Some(&mut diagnostics));
    (out, diagnostics.records)
}
fn slot_identity_impl(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    mut diagnostics: Option<&mut super::StatborgBudgetDiagnostics>,
) -> StatborgSlotIdentity {
    let mut budget = StatborgEventBudget::default();
    let kills = budget.counts_by_slot_diagnostic(
        records,
        StatborgCounterKey {
            component: 2,
            side: StatborgSide::A,
        },
        diagnostics.as_deref_mut(),
    );
    let deaths = budget.counts_by_slot_diagnostic(
        records,
        StatborgCounterKey {
            component: 2,
            side: StatborgSide::B,
        },
        diagnostics.as_deref_mut(),
    );
    let assists = budget.counts_by_slot_diagnostic(
        records,
        StatborgCounterKey {
            component: 3,
            side: StatborgSide::A,
        },
        diagnostics.as_deref_mut(),
    );
    if let Some(d) = diagnostics {
        d.summarize(&budget);
    }
    let mut claims = BTreeMap::<String, Vec<i64>>::new();
    for (slot, k) in kills {
        let matches: Vec<_> = lines
            .iter()
            .filter(|l| {
                l.kills == k as i64
                    && l.deaths == deaths.get(&slot).copied().unwrap_or(0) as i64
                    && l.assists == assists.get(&slot).copied().unwrap_or(0) as i64
            })
            .collect();
        if matches.len() == 1 {
            claims
                .entry(matches[0].xuid.clone())
                .or_default()
                .push(slot);
        }
    }
    let by_slot = claims
        .into_iter()
        .filter_map(|(x, slots)| {
            if slots.len() == 1 {
                Some((slots[0], x))
            } else {
                None
            }
        })
        .collect();
    StatborgSlotIdentity { by_slot, budget }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        lines: Vec<StatborgPlayerLine>,
        identity: BTreeMap<i64, String>,
        records: Vec<StatborgRecord>,
        key: StatborgCounterKey,
        teams: bool,
        raw: StatborgRoundSeries,
        series: BTreeMap<i64, Vec<StatborgScorePoint>>,
        points: Vec<StatborgScorePoint>,
        strict: bool,
        longest: Vec<StatborgScorePoint>,
        chronological: Vec<StatborgScorePoint>,
        steps: Vec<StatborgBoundedStep>,
        bounded: Vec<StatborgScorePoint>,
        initial_budget: usize,
        times: Vec<i64>,
        counts: BTreeMap<i64, usize>,
        budget: StatborgEventBudget,
    }
    #[test]
    fn native_counter_series_and_event_budget() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/statborg-series-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                statborg_slot_identity(&c.records, &c.lines).by_slot,
                c.identity,
                "identity {i}"
            );
            assert_eq!(
                statborg_raw_series(&c.records, c.key, c.teams),
                c.raw,
                "raw {i}"
            );
            assert_eq!(
                statborg_series_by_slot(&c.records, c.key),
                c.series,
                "series {i}"
            );
            assert_eq!(
                statborg_longest_run(&c.points, c.strict),
                c.longest,
                "longest {i}"
            );
            assert_eq!(
                statborg_chronological_total(&c.points),
                c.chronological,
                "chronological {i}"
            );
            assert_eq!(statborg_bound_steps(&c.points), c.steps, "steps {i}");
            assert_eq!(statborg_bounded_series(&c.points), c.bounded, "bounded {i}");
            let mut budget = StatborgEventBudget {
                remaining: c.initial_budget,
                ..Default::default()
            };
            assert_eq!(budget.increment_times(&c.points), c.times, "times {i}");
            assert_eq!(
                budget.counts_by_slot(&c.records, c.key),
                c.counts,
                "counts {i}"
            );
            assert_eq!(budget, c.budget, "budget {i}");
        }
    }
}
