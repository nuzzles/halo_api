//! Per-round counter segments and scoreboard-residue identity completion.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub fn statborg_series_by_round(
    records: &[StatborgRecord],
    key: StatborgCounterKey,
    teams: bool,
    strict: bool,
    unitary: bool,
) -> StatborgRoundSeries {
    let real: BTreeSet<_> = resolve_statborg_rounds(records).real.into_iter().collect();
    let mut out = StatborgRoundSeries::new();
    for (slot, rounds) in statborg_raw_series(records, key, teams) {
        for (round, mut points) in rounds {
            if !real.contains(&round) {
                continue;
            }
            points.sort_by_key(|p| p.time_ms);
            let mut kept = statborg_longest_run(&points, strict);
            if unitary {
                kept = statborg_bounded_series(&kept);
            }
            if !kept.is_empty() {
                out.entry(slot).or_default().insert(round, kept);
            }
        }
    }
    out
}
pub fn statborg_series_total(
    records: &[StatborgRecord],
    key: StatborgCounterKey,
    teams: bool,
    strict: bool,
    unitary: bool,
) -> StatborgSlotSeries {
    series_total_impl(records, key, teams, strict, unitary, None)
}
pub fn statborg_series_total_with_diagnostics(
    records: &[StatborgRecord],
    key: StatborgCounterKey,
    teams: bool,
    strict: bool,
    unitary: bool,
) -> (StatborgSlotSeries, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = series_total_impl(records, key, teams, strict, unitary, Some(&mut diagnostics));
    (out, diagnostics)
}
pub(super) fn series_total_impl(
    records: &[StatborgRecord],
    key: StatborgCounterKey,
    teams: bool,
    strict: bool,
    unitary: bool,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> StatborgSlotSeries {
    let real = resolve_statborg_rounds(records).real.into_iter().collect();
    let mut out = super::statborg_series::statborg_cumulate_rounds_observed(
        &statborg_raw_series(records, key, teams),
        &real,
        diagnostics,
    );
    for points in out.values_mut() {
        if unitary {
            *points = statborg_bounded_series(points);
        }
        if strict {
            *points = statborg_longest_run(points, true);
        }
    }
    out
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgKda {
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
}
pub type StatborgRoundSegments = BTreeMap<i64, BTreeMap<i64, StatborgKda>>;
pub fn statborg_round_segments(records: &[StatborgRecord]) -> StatborgRoundSegments {
    let mut out = StatborgRoundSegments::new();
    for (index, key) in [
        StatborgCounterKey {
            component: 2,
            side: StatborgSide::A,
        },
        StatborgCounterKey {
            component: 2,
            side: StatborgSide::B,
        },
        StatborgCounterKey {
            component: 3,
            side: StatborgSide::A,
        },
    ]
    .into_iter()
    .enumerate()
    {
        for (slot, rounds) in statborg_series_by_round(records, key, false, false, true) {
            for (round, points) in rounds {
                let value = points.last().unwrap().value;
                let s = out.entry(round).or_default().entry(slot).or_default();
                match index {
                    0 => s.kills = value,
                    1 => s.deaths = value,
                    _ => s.assists = value,
                }
            }
        }
    }
    out
}
fn segment(segments: &StatborgRoundSegments, round: i64, slot: i64) -> StatborgKda {
    segments
        .get(&round)
        .and_then(|m| m.get(&slot))
        .copied()
        .unwrap_or_default()
}
pub fn statborg_round_residue(
    segments: &StatborgRoundSegments,
    identity: &StatborgRoundIdentity,
    lines: &[StatborgPlayerLine],
    round: i64,
    xuid: &str,
) -> StatborgKda {
    let mut out = lines
        .iter()
        .find(|l| l.xuid == xuid)
        .map(|l| StatborgKda {
            kills: l.kills,
            deaths: l.deaths,
            assists: l.assists,
        })
        .unwrap_or_default();
    for (&r, ids) in &identity.publication.by_round {
        if r == round {
            continue;
        }
        for (&slot, x) in ids {
            if x == xuid {
                let s = segment(segments, r, slot);
                out.kills = out.kills.wrapping_sub(s.kills);
                out.deaths = out.deaths.wrapping_sub(s.deaths);
                out.assists = out.assists.wrapping_sub(s.assists);
            }
        }
    }
    out
}
fn candidates(
    identity: &StatborgRoundIdentity,
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    round: i64,
) -> (Vec<i64>, Vec<String>) {
    let ids = &identity.publication.by_round[&round];
    let unnamed = statborg_emitting_player_slots(records, round)
        .into_iter()
        .filter(|s| ids.get(s).is_none_or(String::is_empty))
        .collect();
    let used: BTreeSet<_> = ids.values().collect();
    let free = lines
        .iter()
        .filter(|l| !used.contains(&l.xuid))
        .map(|l| l.xuid.clone())
        .collect();
    (unnamed, free)
}
impl StatborgRoundIdentity {
    fn completion_copy(&self) -> Self {
        let mut out = self.clone();
        out.publication
            .origins
            .retain(|r, _| out.publication.by_round.contains_key(r));
        for &r in out.publication.by_round.keys() {
            out.publication.origins.entry(r).or_default();
        }
        out
    }
    pub fn completed_by_elimination(
        &self,
        records: &[StatborgRecord],
        lines: &[StatborgPlayerLine],
    ) -> Self {
        if lines.is_empty() || self.publication.by_round.is_empty() {
            return self.clone();
        }
        let segments = statborg_round_segments(records);
        let mut out = self.completion_copy();
        for &round in self.publication.by_round.keys() {
            let (slots, free) = candidates(&out, records, lines, round);
            if slots.len() != 1 || free.len() != 1 {
                continue;
            }
            let slot = slots[0];
            let x = &free[0];
            if segment(&segments, round, slot)
                != statborg_round_residue(&segments, &out, lines, round, x)
            {
                continue;
            }
            out.publication
                .by_round
                .get_mut(&round)
                .unwrap()
                .insert(slot, x.clone());
            out.publication
                .origins
                .get_mut(&round)
                .unwrap()
                .insert(slot, "elimination_roster".into());
        }
        out
    }
    pub fn completed_by_round_residue(
        &self,
        records: &[StatborgRecord],
        lines: &[StatborgPlayerLine],
    ) -> Self {
        if lines.is_empty() || self.publication.by_round.len() <= 1 {
            return self.clone();
        }
        let segments = statborg_round_segments(records);
        let mut out = self.completion_copy();
        for &round in self.publication.by_round.keys() {
            let (slots, free) = candidates(&out, records, lines, round);
            let mut claims = BTreeMap::<i64, Vec<String>>::new();
            let mut uses = BTreeMap::<String, usize>::new();
            for slot in slots {
                let s = segment(&segments, round, slot);
                if s == StatborgKda::default() {
                    continue;
                }
                for x in &free {
                    if statborg_round_residue(&segments, &out, lines, round, x) == s {
                        claims.entry(slot).or_default().push(x.clone());
                        *uses.entry(x.clone()).or_default() += 1;
                    }
                }
            }
            for (slot, xs) in claims {
                if xs.len() != 1 || uses[&xs[0]] != 1 {
                    continue;
                }
                out.publication
                    .by_round
                    .get_mut(&round)
                    .unwrap()
                    .insert(slot, xs[0].clone());
                out.publication
                    .origins
                    .get_mut(&round)
                    .unwrap()
                    .insert(slot, "residu_de_manche".into());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Residue {
        round: i64,
        xuid: String,
        value: StatborgKda,
    }
    #[derive(Deserialize)]
    struct Case {
        key: StatborgCounterKey,
        teams: bool,
        strict: bool,
        unitary: bool,
        round_series: StatborgRoundSeries,
        total_series: BTreeMap<i64, Vec<StatborgScorePoint>>,
        records: Vec<StatborgRecord>,
        lines: Vec<StatborgPlayerLine>,
        identity: StatborgRoundIdentity,
        segments: StatborgRoundSegments,
        residues: Vec<Residue>,
        elimination: StatborgRoundIdentity,
        completion: StatborgRoundIdentity,
    }
    #[test]
    fn native_round_residue_completion() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/statborg-residue-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                statborg_series_by_round(&c.records, c.key, c.teams, c.strict, c.unitary),
                c.round_series,
                "round series {i}"
            );
            assert_eq!(
                statborg_series_total(&c.records, c.key, c.teams, c.strict, c.unitary),
                c.total_series,
                "total series {i}"
            );
            let segments = statborg_round_segments(&c.records);
            assert_eq!(segments, c.segments, "segments {i}");
            for r in c.residues {
                assert_eq!(
                    statborg_round_residue(&segments, &c.identity, &c.lines, r.round, &r.xuid),
                    r.value,
                    "residue {i}"
                );
            }
            assert_eq!(
                c.identity.completed_by_elimination(&c.records, &c.lines),
                c.elimination,
                "elimination {i}"
            );
            assert_eq!(
                c.identity.completed_by_round_residue(&c.records, &c.lines),
                c.completion,
                "completion {i}"
            );
        }
    }
}
