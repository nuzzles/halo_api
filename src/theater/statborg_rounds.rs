//! Native statborg round admission and consensus boundaries over decoded records.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgValue {
    pub a: i64,
    pub b: i64,
    pub c: i64,
    pub d: i64,
    pub has_c: bool,
    pub has_d: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgRecord {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    pub slot: i64,
    pub round: i64,
    pub comps: BTreeMap<i64, StatborgValue>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgRoundsDecision {
    pub written: Vec<i64>,
    pub real: Vec<i64>,
    pub contradicted: Vec<i64>,
    pub contradicted_records: usize,
    pub decreed: bool,
}
impl StatborgRoundsDecision {
    /// Native RealSet projection; repeated entries collapse without changing
    /// the retained ordered decision vectors.
    pub fn real_set(&self) -> BTreeSet<i64> {
        self.real.iter().copied().collect()
    }
}
/// Native diagnostic threshold, not a record-filtering rule.
pub const STATBORG_OUTLIERS_NOMINAL_MAX: usize = 27;
/// Native thresholds for real-round admission.
pub const STATBORG_MIN_ROUND_RUN: usize = 3;
pub const STATBORG_MIN_ROUND_RECORDS: usize = 25;
pub const STATBORG_MIN_ROUND_RECORD_SHARE: usize = 10;
pub const STATBORG_MAX_ROUND: i64 = 7;
pub const STATBORG_MAX_EMPTY_ROUND_RUN: usize = 1;

/// Longest strictly increasing mode-score run per round, across player and team
/// slots. Both A and B must be in the native mode-score domain.
pub fn statborg_mode_score_runs_by_round(records: &[StatborgRecord]) -> BTreeMap<i64, usize> {
    let mut series = BTreeMap::<(i64, i64), Vec<(i64, i64)>>::new();
    for r in records {
        if let Some(v) = r
            .comps
            .get(&0)
            .filter(|v| super::statborg_mode_score_in_domain(v))
        {
            series
                .entry((r.slot, r.round))
                .or_default()
                .push((r.time_ms, v.a));
        }
    }
    let mut runs = BTreeMap::<i64, usize>::new();
    for ((_, round), mut points) in series {
        points.sort_by_key(|p| p.0);
        let mut tails = Vec::new();
        for (_, value) in points {
            let i = tails.partition_point(|&v| v < value);
            if i == tails.len() {
                tails.push(value);
            } else {
                tails[i] = value;
            }
        }
        if !tails.is_empty() {
            let length = runs.entry(round).or_default();
            *length = (*length).max(tails.len());
        }
    }
    runs
}
/// Written rounds include every record, including team records and rounds the
/// later contiguity decision will reject.
pub fn statborg_present_rounds(records: &[StatborgRecord]) -> BTreeSet<i64> {
    records.iter().map(|r| r.round).collect()
}
/// The absolute and relative thresholds apply only to player-slot records.
pub fn statborg_material_rounds(records: &[StatborgRecord]) -> BTreeSet<i64> {
    let mut counts = BTreeMap::<i64, usize>::new();
    for r in records.iter().filter(|r| r.slot > 8) {
        *counts.entry(r.round).or_default() += 1;
    }
    let reference = counts.values().copied().max().unwrap_or(0);
    counts
        .into_iter()
        .filter_map(|(round, n)| {
            (n >= STATBORG_MIN_ROUND_RECORDS
                && n * 100 >= reference * STATBORG_MIN_ROUND_RECORD_SHARE)
                .then_some(round)
        })
        .collect()
}
pub fn statborg_has_round_after(
    runs: &BTreeMap<i64, usize>,
    material: &BTreeSet<i64>,
    round: i64,
) -> bool {
    runs.iter()
        .any(|(&r, &n)| r > round && r <= STATBORG_MAX_ROUND && n >= STATBORG_MIN_ROUND_RUN)
        || material
            .iter()
            .any(|&r| r > round && r <= STATBORG_MAX_ROUND)
}
/// Return the native real-round set and whether round zero was decreed as fallback.
pub fn statborg_contiguous_rounds(
    runs: &BTreeMap<i64, usize>,
    material: &BTreeSet<i64>,
    present: &BTreeSet<i64>,
) -> (BTreeSet<i64>, bool) {
    let mut real = BTreeSet::new();
    let mut gap = 0;
    let mut seen = false;
    for round in 0..=STATBORG_MAX_ROUND {
        if runs.get(&round).copied().unwrap_or(0) >= STATBORG_MIN_ROUND_RUN
            || material.contains(&round)
        {
            gap = 0;
            seen = true;
            real.insert(round);
            continue;
        }
        gap += 1;
        if gap > STATBORG_MAX_EMPTY_ROUND_RUN
            || (seen && !present.contains(&round))
            || !statborg_has_round_after(runs, material, round)
        {
            break;
        }
        real.insert(round);
    }
    let decreed = real.is_empty();
    if decreed {
        real.insert(0);
    }
    (real, decreed)
}
pub fn resolve_statborg_rounds(records: &[StatborgRecord]) -> StatborgRoundsDecision {
    let runs = statborg_mode_score_runs_by_round(records);
    let material = statborg_material_rounds(records);
    let written = statborg_present_rounds(records);
    let (real, decreed) = statborg_contiguous_rounds(&runs, &material, &written);
    let admitted: BTreeSet<_> = material
        .into_iter()
        .chain(
            runs.into_iter()
                .filter_map(|(r, n)| (n >= STATBORG_MIN_ROUND_RUN).then_some(r)),
        )
        .collect();
    let contradicted: BTreeSet<_> = written
        .intersection(&admitted)
        .filter(|r| !real.contains(r))
        .copied()
        .collect();
    StatborgRoundsDecision {
        contradicted_records: records
            .iter()
            .filter(|r| contradicted.contains(&r.round))
            .count(),
        written: written.into_iter().collect(),
        real: real.into_iter().collect(),
        contradicted: contradicted.into_iter().collect(),
        decreed,
    }
}
/// Emitting player slots, including unresolved identities, in native slot order.
pub fn statborg_emitting_player_slots(records: &[StatborgRecord], round: i64) -> BTreeSet<i64> {
    records
        .iter()
        .filter(|r| r.round == round && r.slot > 8 && r.comps.contains_key(&2))
        .map(|r| r.slot)
        .collect()
}
const OPEN_FROM: i64 = -(1 << 62);
const OPEN_TO: i64 = (1 << 62) - 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgRoundSpan {
    pub from_ms: i64,
    pub to_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgKeptSegment {
    pub slot: i64,
    pub round: i64,
    #[serde(rename = "FromMS")]
    pub from_ms: i64,
    #[serde(rename = "ToMS")]
    pub to_ms: i64,
    #[serde(rename = "GapMS")]
    pub gap_ms: i64,
    pub records: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgRoundBounds {
    pub by_round: BTreeMap<i64, StatborgRoundSpan>,
    pub kept: Vec<StatborgKeptSegment>,
}
impl StatborgRoundBounds {
    /// Count only excluded records, preserving the native kept-segment exemptions.
    pub fn outliers(&self, records: &[StatborgRecord]) -> usize {
        records.iter().filter(|r| self.excludes(r)).count()
    }
    pub fn starts(&self) -> Vec<i64> {
        let mut v: Vec<_> = self
            .by_round
            .values()
            .filter(|s| s.from_ms != OPEN_FROM)
            .map(|s| s.from_ms)
            .collect();
        v.sort_unstable();
        v
    }
    pub fn posed(&self) -> bool {
        !self.by_round.is_empty()
    }
    pub fn excludes(&self, r: &StatborgRecord) -> bool {
        self.by_round.get(&r.round).is_some_and(|s| {
            !self
                .kept
                .iter()
                .any(|k| k.slot == r.slot && k.round == r.round)
                && (r.time_ms < s.from_ms || r.time_ms >= s.to_ms)
        })
    }
}
fn median(mut v: Vec<i64>) -> i64 {
    v.sort_unstable();
    v[(v.len() - 1) / 2]
}
fn round_marks(records: &[StatborgRecord]) -> Vec<(i64, i64, i64)> {
    let real: BTreeSet<_> = resolve_statborg_rounds(records).real.into_iter().collect();
    let mut starts = BTreeMap::<i64, BTreeMap<i64, i64>>::new();
    let mut times = BTreeMap::<i64, Vec<i64>>::new();
    for r in records.iter().filter(|r| real.contains(&r.round)) {
        starts
            .entry(r.round)
            .or_default()
            .entry(r.slot)
            .and_modify(|t| *t = (*t).min(r.time_ms))
            .or_insert(r.time_ms);
        times.entry(r.round).or_default().push(r.time_ms);
    }
    let max = starts.values().map(BTreeMap::len).max().unwrap_or(0);
    starts
        .into_iter()
        .filter(|(_, s)| s.len() * 2 > max)
        .map(|(r, s)| {
            (
                r,
                median(s.into_values().collect()),
                median(times.remove(&r).unwrap()),
            )
        })
        .collect()
}
pub fn statborg_round_starts_ms(records: &[StatborgRecord]) -> BTreeMap<i64, i64> {
    round_marks(records)
        .into_iter()
        .map(|(r, t, _)| (r, t))
        .collect()
}
pub fn resolve_statborg_round_bounds(records: &[StatborgRecord]) -> StatborgRoundBounds {
    let marks = round_marks(records);
    let mut out = StatborgRoundBounds::default();
    if marks.len() < 2 || marks.windows(2).any(|w| w[0].1 >= w[1].1) {
        return out;
    }
    for &(r, _, _) in &marks {
        out.by_round.insert(
            r,
            StatborgRoundSpan {
                from_ms: OPEN_FROM,
                to_ms: OPEN_TO,
            },
        );
    }
    for w in marks.windows(2) {
        let (prev, next) = (w[0], w[1]);
        if prev.2 >= next.1 || next.2 < next.1 {
            continue;
        }
        out.by_round.get_mut(&prev.0).unwrap().to_ms = next.1;
        out.by_round.get_mut(&next.0).unwrap().from_ms = next.1;
    }
    let mut groups = BTreeMap::<(i64, i64), Vec<i64>>::new();
    for r in records {
        if out.by_round.contains_key(&r.round) {
            groups.entry((r.slot, r.round)).or_default().push(r.time_ms);
        }
    }
    for ((slot, round), ts) in groups {
        let s = &out.by_round[&round];
        if ts.iter().any(|&t| t >= s.from_ms && t < s.to_ms) {
            continue;
        }
        let from = *ts.iter().min().unwrap();
        let to = *ts.iter().max().unwrap();
        let gap = if to < s.from_ms && s.from_ms != OPEN_FROM {
            s.from_ms.wrapping_sub(to)
        } else if from >= s.to_ms && s.to_ms != OPEN_TO {
            from.wrapping_sub(s.to_ms)
        } else {
            0
        };
        out.kept.push(StatborgKeptSegment {
            slot,
            round,
            from_ms: from,
            to_ms: to,
            gap_ms: gap,
            records: ts.len(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct HelperDecision {
        runs: BTreeMap<i64, usize>,
        material: BTreeMap<i64, bool>,
        present: BTreeMap<i64, bool>,
        real: BTreeMap<i64, bool>,
        decreed: bool,
    }
    fn true_keys(map: BTreeMap<i64, bool>) -> BTreeSet<i64> {
        map.into_iter()
            .filter_map(|(k, v)| v.then_some(k))
            .collect()
    }
    #[test]
    fn native_round_admission_helpers() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/round-helpers-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(
            fixture["constants"],
            serde_json::json!([
                STATBORG_MIN_ROUND_RUN,
                STATBORG_MIN_ROUND_RECORDS,
                STATBORG_MIN_ROUND_RECORD_SHARE,
                STATBORG_MAX_ROUND,
                STATBORG_MAX_EMPTY_ROUND_RUN
            ])
        );
        assert_eq!(fixture["cases"].as_array().unwrap().len(), 128);
        for (i, row) in fixture["cases"].as_array().unwrap().iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(row["records"].clone()).unwrap();
            let expected: HelperDecision = serde_json::from_value(row.clone()).unwrap();
            let runs = statborg_mode_score_runs_by_round(&records);
            let material = statborg_material_rounds(&records);
            let present = statborg_present_rounds(&records);
            assert_eq!(runs, expected.runs, "runs {i}");
            assert_eq!(material, true_keys(expected.material), "material {i}");
            assert_eq!(present, true_keys(expected.present), "present {i}");
            assert_eq!(
                statborg_contiguous_rounds(&runs, &material, &present),
                (true_keys(expected.real), expected.decreed),
                "decision {i}"
            );
            for p in row["probes"].as_array().unwrap() {
                assert_eq!(
                    statborg_has_round_after(&runs, &material, p["round"].as_i64().unwrap()),
                    p["after"].as_bool().unwrap(),
                    "after {i}"
                );
            }
        }
        let decisions: Vec<HelperDecision> =
            serde_json::from_value(fixture["decisions"].clone()).unwrap();
        assert_eq!(decisions.len(), 256);
        for (i, d) in decisions.into_iter().enumerate() {
            assert_eq!(
                statborg_contiguous_rounds(&d.runs, &true_keys(d.material), &true_keys(d.present)),
                (true_keys(d.real), d.decreed),
                "supplied decision {i}"
            );
        }
    }
    #[derive(Deserialize)]
    struct Case {
        records: Vec<StatborgRecord>,
        decision: StatborgRoundsDecision,
        bounds: StatborgRoundBounds,
        starts: Vec<i64>,
        round_starts: BTreeMap<i64, i64>,
        posed: bool,
        excluded: Vec<bool>,
        outliers: usize,
        real_set: BTreeMap<i64, bool>,
        outliers_nominal_max: usize,
    }
    #[test]
    fn native_round_decisions_and_boundaries() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/statborg-rounds-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                resolve_statborg_rounds(&c.records),
                c.decision,
                "decision {i}"
            );
            assert_eq!(
                statborg_round_starts_ms(&c.records),
                c.round_starts,
                "round starts {i}"
            );
            let b = resolve_statborg_round_bounds(&c.records);
            assert_eq!(b.outliers(&c.records), c.outliers, "outliers {i}");
            assert_eq!(
                c.decision.real_set(),
                c.real_set.into_keys().collect(),
                "real set {i}"
            );
            assert_eq!(STATBORG_OUTLIERS_NOMINAL_MAX, c.outliers_nominal_max);
            assert_eq!(b, c.bounds, "bounds {i}");
            assert_eq!(b.starts(), c.starts, "starts {i}");
            assert_eq!(b.posed(), c.posed, "posed {i}");
            assert_eq!(
                c.records.iter().map(|r| b.excludes(r)).collect::<Vec<_>>(),
                c.excluded,
                "excluded {i}"
            );
        }
    }
}
