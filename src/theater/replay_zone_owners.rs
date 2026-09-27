//! Zone ownership publication, roster validation and diagnostic agreement counts.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayZoneSpan {
    pub t0: i64,
    pub t1: i64,
    pub owner: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    pub active: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayZonesCoverage {
    pub method: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub roles: String,
    pub catalog: usize,
    pub slots: usize,
    pub paired: usize,
    pub unpaired: usize,
    pub captures: usize,
    pub attributed: usize,
    pub no_position: usize,
    pub outside: usize,
    pub ambiguous_zone: usize,
    pub owner_checked: usize,
    pub owner_agreed: usize,
    pub owner_unpaired: usize,
    pub spans: usize,
    pub hill_periods: usize,
    pub unknown_owner: usize,
    pub letters: usize,
    pub gauge_points: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayZoneState {
    pub zone_ref: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_rank: Option<i64>,
    #[serde(skip_serializing_if = "zone_key_zero")]
    pub key: u32,
    pub spans: Vec<ReplayZoneSpan>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gauge: Vec<ReplayGaugePoint>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gauge_ramps: Vec<ReplayZoneGaugeRamp>,
}
fn zone_key_zero(v: &u32) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayZoneGaugeRamp {
    pub t0: i64,
    pub t1: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capturing_team: Option<i64>,
}
pub fn replay_zone_team_set(teams: &BTreeMap<String, i64>) -> BTreeSet<u64> {
    teams
        .values()
        .filter(|&&t| t >= 0)
        .map(|&t| t as u64)
        .collect()
}
/// The outer Option distinguishes an unknown value from the known neutral owner.
pub fn replay_zone_owner_team(value: u64, teams: &BTreeSet<u64>) -> Option<Option<i64>> {
    if value == 0xffff_ffff {
        Some(None)
    } else if teams.contains(&value) || (teams.is_empty() && value <= 1) {
        Some(Some(value as i64))
    } else {
        None
    }
}
pub fn replay_zone_merge_runs(samples: &[ReplayZoneSample]) -> Vec<ReplayZoneSample> {
    samples
        .iter()
        .enumerate()
        .filter(|(i, s)| *i == 0 || s.v != samples[*i - 1].v)
        .map(|(_, s)| *s)
        .collect()
}
pub fn replay_zone_peak_progress(gauge: &[ReplayZoneSample], t0: i64, t1: i64) -> Option<f32> {
    gauge
        .iter()
        .filter(|s| s.t >= t0 && s.t <= t1)
        .map(|s| s.v)
        .max()
        .map(replay_gauge_progress)
}
pub fn replay_zone_owner_spans(
    owner: &[ReplayZoneSample],
    gauge: &[ReplayZoneSample],
    frames: i64,
    teams: &BTreeSet<u64>,
    coverage: &mut ReplayZonesCoverage,
) -> Vec<ReplayZoneSpan> {
    let groups = replay_zone_merge_runs(owner);
    let mut out = Vec::new();
    for (i, g) in groups.iter().enumerate() {
        let t1 = groups.get(i + 1).map_or(frames - 1, |s| s.t - 1);
        if t1 < g.t {
            continue;
        }
        let Some(team) = replay_zone_owner_team(g.v, teams) else {
            coverage.unknown_owner += 1;
            continue;
        };
        out.push(ReplayZoneSpan {
            t0: g.t,
            t1,
            owner: team,
            progress: replay_zone_peak_progress(gauge, g.t, t1),
            active: false,
        });
    }
    out
}
pub fn check_replay_zone_owner_agreement(
    series: &ReplayZoneSeries,
    owner: &BTreeMap<i64, u32>,
    pairs: &[ReplayZonePair],
    teams: &BTreeMap<String, i64>,
    win: i64,
    coverage: &mut ReplayZonesCoverage,
) {
    for p in pairs {
        let (Some(slot), Some(team)) = (owner.get(&p.zone_ref), teams.get(&p.xuid)) else {
            continue;
        };
        let Some(v) = series
            .owner
            .get(slot)
            .and_then(|ss| replay_zone_value_after(ss, p.t, win))
        else {
            continue;
        };
        if v == 0xffff_ffff {
            continue;
        }
        coverage.owner_checked += 1;
        if v == *team as u64 {
            coverage.owner_agreed += 1;
        }
    }
}
pub fn replay_zone_letter_ranks(
    gauge: &BTreeMap<i64, u32>,
    catalog: i64,
    hill: bool,
) -> BTreeMap<i64, i64> {
    if hill || catalog <= 0 || catalog > 3 || gauge.len() != catalog as usize {
        return BTreeMap::new();
    }
    let mut refs: Vec<_> = gauge.keys().copied().collect();
    refs.sort_by_key(|r| (gauge[r], *r));
    refs.into_iter()
        .enumerate()
        .map(|(i, r)| (r, i as i64))
        .collect()
}
pub fn tally_replay_zone_states(states: &[ReplayZoneState], coverage: &mut ReplayZonesCoverage) {
    for s in states {
        coverage.spans += s.spans.len();
        coverage.gauge_points += s.gauge.len();
        coverage.letters += usize::from(s.letter_rank.is_some());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        owner: Vec<ReplayZoneSample>,
        gauge: Vec<ReplayZoneSample>,
        frames: i64,
        teams: BTreeMap<String, i64>,
        spans: Vec<ReplayZoneSpan>,
        unknown: usize,
        series: ReplayZoneSeries,
        owners: BTreeMap<i64, u32>,
        pairs: Vec<ReplayZonePair>,
        win: i64,
        checked: usize,
        agreed: usize,
        gauges: BTreeMap<i64, u32>,
        catalog: i64,
        hill: bool,
        letters: BTreeMap<i64, i64>,
    }
    #[test]
    fn native_zone_owner_intervals_and_letters() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-zone-owners-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let mut cov = ReplayZonesCoverage::default();
            assert_eq!(
                replay_zone_owner_spans(
                    &c.owner,
                    &c.gauge,
                    c.frames,
                    &replay_zone_team_set(&c.teams),
                    &mut cov
                ),
                c.spans,
                "spans {i}"
            );
            assert_eq!(cov.unknown_owner, c.unknown, "unknown {i}");
            check_replay_zone_owner_agreement(
                &c.series, &c.owners, &c.pairs, &c.teams, c.win, &mut cov,
            );
            assert_eq!(
                (cov.owner_checked, cov.owner_agreed),
                (c.checked, c.agreed),
                "agreement {i}"
            );
            assert_eq!(
                replay_zone_letter_ranks(&c.gauges, c.catalog, c.hill),
                c.letters,
                "letters {i}"
            );
        }
    }
}
