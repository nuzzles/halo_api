//! Native personal-score award labels with explicit ambiguity and compound scores.
use super::{StatborgScorePoint, native_sort};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgAward {
    pub name: String,
    pub category: String,
    pub unit: i64,
    pub count: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgLabelledScore {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    pub slot: i64,
    pub value: i64,
    pub award: String,
    pub category: String,
    pub candidates: Vec<String>,
    pub compound: bool,
}
impl StatborgLabelledScore {
    pub fn resolved(&self) -> bool {
        !self.award.is_empty()
    }
    pub fn describe(&self) -> String {
        if self.resolved() {
            self.award.clone()
        } else if self.candidates.is_empty() {
            "inconnu".into()
        } else {
            format!("l'un de : {}", self.candidates.join(", "))
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgLabelSummary {
    pub total: usize,
    pub resolved: usize,
    pub compound: usize,
    pub unnamed: usize,
    pub by_award: BTreeMap<String, usize>,
}
fn combinations(
    rest: i64,
    units: &[i64],
    start: usize,
    left: usize,
    acc: &mut Vec<i64>,
    out: &mut Vec<Vec<i64>>,
) {
    if left == 0 {
        if rest == 0 {
            out.push(acc.clone());
        }
        return;
    }
    for i in start..units.len() {
        acc.push(units[i]);
        combinations(rest.wrapping_sub(units[i]), units, i, left - 1, acc, out);
        acc.pop();
    }
}
fn decompose(delta: i64, units: &BTreeMap<i64, Vec<&StatborgAward>>) -> (Vec<i64>, bool) {
    if units.contains_key(&delta) {
        return (vec![delta], false);
    }
    let values: Vec<_> = units.keys().rev().copied().collect();
    for n in 2..=3 {
        let mut solutions = Vec::new();
        combinations(delta, &values, 0, n, &mut Vec::new(), &mut solutions);
        if solutions.len() == 1 {
            return (solutions.pop().unwrap(), true);
        }
        if solutions.len() > 1 {
            break;
        }
    }
    (vec![delta], false)
}
/// Positive quota counts admit a unit; they are not decremented as events emit.
/// Exact units win; otherwise only a unique two- or three-part sum is labelled.
pub fn label_statborg_personal_score(
    points: &[StatborgScorePoint],
    quotas: &BTreeMap<i64, Vec<StatborgAward>>,
) -> Vec<StatborgLabelledScore> {
    let mut by_slot = BTreeMap::<i64, Vec<&StatborgScorePoint>>::new();
    for p in points {
        if quotas.contains_key(&p.slot) {
            by_slot.entry(p.slot).or_default().push(p);
        }
    }
    let mut out = Vec::new();
    for (slot, mut points) in by_slot {
        points.sort_by_key(|p| p.time_ms);
        let mut units = BTreeMap::<i64, Vec<&StatborgAward>>::new();
        for a in &quotas[&slot] {
            if a.count > 0 {
                units.entry(a.unit).or_default().push(a);
            }
        }
        for awards in units.values_mut() {
            native_sort::sort_by(awards, |a, b| a.name.cmp(&b.name));
        }
        let mut previous = 0;
        for p in points {
            let delta = p.value.wrapping_sub(previous);
            previous = p.value;
            if delta == 0 {
                continue;
            }
            let (parts, compound) = decompose(delta, &units);
            for value in parts {
                let mut e = StatborgLabelledScore {
                    time_ms: p.time_ms,
                    slot,
                    value,
                    compound,
                    ..Default::default()
                };
                if let Some(candidates) = units.get(&value) {
                    if candidates.len() == 1 {
                        e.award = candidates[0].name.clone();
                        e.category = candidates[0].category.clone();
                    } else {
                        e.candidates = candidates.iter().map(|a| a.name.clone()).collect();
                        if candidates
                            .iter()
                            .all(|a| a.category == candidates[0].category)
                        {
                            e.category = candidates[0].category.clone();
                        }
                    }
                }
                out.push(e);
            }
        }
    }
    out.sort_by_key(|e| (e.time_ms, e.slot));
    out
}
pub fn summarize_statborg_score_labels(events: &[StatborgLabelledScore]) -> StatborgLabelSummary {
    let mut out = StatborgLabelSummary::default();
    for e in events {
        out.total += 1;
        out.compound += usize::from(e.compound);
        if e.resolved() {
            out.resolved += 1;
            *out.by_award.entry(e.award.clone()).or_default() += 1;
        } else if e.candidates.is_empty() {
            out.unnamed += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        points: Vec<StatborgScorePoint>,
        quotas: BTreeMap<i64, Vec<StatborgAward>>,
        events: Vec<StatborgLabelledScore>,
        summary: StatborgLabelSummary,
        descriptions: Vec<String>,
    }
    #[test]
    fn native_personal_score_awards_oracle() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/statborg-awards-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.into_iter().enumerate() {
            let events = label_statborg_personal_score(&row.points, &row.quotas);
            assert_eq!(events, row.events, "events {i}");
            assert_eq!(
                summarize_statborg_score_labels(&events),
                row.summary,
                "summary {i}"
            );
            assert_eq!(
                events
                    .iter()
                    .map(StatborgLabelledScore::describe)
                    .collect::<Vec<_>>(),
                row.descriptions,
                "descriptions {i}"
            );
        }
    }
}
