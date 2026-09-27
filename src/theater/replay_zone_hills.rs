//! Hill designation, geometric activation and ownership; no invented capture gauge.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplayHillFallbacks {
    pub whole_period_votes: usize,
    pub open_owner_tail: usize,
}
#[derive(Clone)]
struct Period {
    t0: i64,
    t1: i64,
    zone: Option<usize>,
    top: Option<u64>,
}
fn votes(
    zones: &[ObjectiveZone],
    points: &BTreeMap<i64, Vec<ReplayPoint>>,
    t0: i64,
    t1: i64,
) -> BTreeMap<usize, usize> {
    let mut out = BTreeMap::new();
    if t0 > t1 {
        return out;
    }
    for ps in points.range(t0..=t1).map(|(_, ps)| ps) {
        for p in ps {
            let pos = ObjectiveVec3 {
                x: p.x as f64,
                y: p.y as f64,
                z: p.z as f64,
            };
            let mut best = f64::INFINITY;
            let mut hit = None;
            let mut n = 0;
            for z in zones {
                let d = z.volume.distance_to(pos);
                if d < best {
                    best = d;
                    hit = Some(z.spatial_rank);
                    n = 1;
                } else if d == best {
                    n += 1;
                }
            }
            if n == 1 && best <= 5. {
                *out.entry(hit.unwrap()).or_default() += 1;
            }
        }
    }
    out
}
fn clear_modal(votes: &BTreeMap<usize, usize>) -> Option<usize> {
    let mut best = None;
    let mut n = 0;
    let mut second = 0;
    for (&r, &v) in votes {
        if v > n {
            second = n;
            best = Some(r);
            n = v;
        } else {
            second = second.max(v);
        }
    }
    if n > second { best } else { None }
}
fn designator(series: &ReplayZoneSeries) -> Option<(u32, Vec<i64>, i64)> {
    let mut best: Option<(u32, Vec<i64>, i64)> = None;
    for (&slot, ss) in &series.desig {
        if series.owner.get(&slot.wrapping_add(1)).map_or(0, Vec::len) < 2 {
            continue;
        }
        let changes: Vec<_> = replay_zone_merge_runs(ss).iter().map(|s| s.t).collect();
        if changes.is_empty() || best.as_ref().is_some_and(|b| changes.len() <= b.1.len()) {
            continue;
        }
        let mut first = changes[0];
        for k in 1..=3 {
            for map in [&series.owner, &series.gauge] {
                if let Some(s) = map.get(&slot.wrapping_add(k)).and_then(|ss| ss.first()) {
                    first = first.min(s.t);
                }
            }
        }
        best = Some((slot, changes, first));
    }
    best
}
fn states(
    periods: &[Period],
    owner: &[ReplayZoneSample],
    teams: &BTreeSet<u64>,
    cov: &mut ReplayZonesCoverage,
    fb: &mut ReplayHillFallbacks,
) -> Vec<ReplayZoneState> {
    let groups = replay_zone_merge_runs(owner);
    let mut runs = Vec::new();
    for (i, g) in groups.iter().enumerate() {
        let t1 = if let Some(next) = groups.get(i + 1) {
            next.t - 1
        } else {
            fb.open_owner_tail += 1;
            i64::MAX
        };
        if t1 < g.t {
            continue;
        }
        let Some(team) = replay_zone_owner_team(g.v, teams) else {
            cov.unknown_owner += 1;
            continue;
        };
        runs.push((g.t, t1, team));
    }
    let mut by_ref = BTreeMap::<usize, Vec<ReplayZoneSpan>>::new();
    for p in periods {
        if p.t1 < p.t0 {
            continue;
        }
        let mut cursor = p.t0;
        let mut spans = Vec::new();
        for &(r0, r1, team) in &runs {
            let (t0, t1) = (r0.max(p.t0), r1.min(p.t1));
            if t0 > t1 {
                continue;
            }
            if t0 > cursor {
                spans.push(ReplayZoneSpan {
                    t0: cursor,
                    t1: t0 - 1,
                    active: true,
                    ..Default::default()
                });
            }
            spans.push(ReplayZoneSpan {
                t0,
                t1,
                owner: team,
                active: true,
                ..Default::default()
            });
            cursor = t1 + 1;
        }
        if cursor <= p.t1 {
            spans.push(ReplayZoneSpan {
                t0: cursor,
                t1: p.t1,
                active: true,
                ..Default::default()
            });
        }
        if spans.len() == 1 {
            spans[0].progress = p.top.map(replay_gauge_progress);
        }
        by_ref.entry(p.zone.unwrap_or(0)).or_default().extend(spans);
    }
    by_ref
        .into_iter()
        .map(|(zone, mut spans)| {
            spans.sort_by_key(|s| s.t0);
            ReplayZoneState {
                zone_ref: zone as i64,
                spans,
                ..Default::default()
            }
        })
        .collect()
}
pub fn build_replay_hill_states(
    zones: &[ObjectiveZone],
    series: &ReplayZoneSeries,
    teams: &BTreeSet<u64>,
    tracks: &[ReplayTrack],
    frames: i64,
    cov: &mut ReplayZonesCoverage,
) -> (Vec<ReplayZoneState>, ReplayHillFallbacks) {
    let mut fb = ReplayHillFallbacks::default();
    let mut points = BTreeMap::<i64, Vec<ReplayPoint>>::new();
    for tr in tracks {
        for p in tr.points() {
            points.entry(p.t).or_default().push(p.clone());
        }
    }
    let mut ramps = series.ramps();
    let output = if let Some((slot, changes, first)) = designator(series) {
        cov.method = "designator+geometry".into();
        let mut bounds = vec![first];
        bounds.extend(changes);
        let mut periods = Vec::new();
        for (i, &t0) in bounds.iter().enumerate() {
            let t1 = bounds.get(i + 1).map_or(frames - 1, |t| t - 1);
            if t1 >= t0 {
                periods.push(Period {
                    t0,
                    t1,
                    zone: None,
                    top: None,
                });
            }
        }
        cov.hill_periods = periods.len();
        let mut kept = Vec::new();
        for mut p in periods {
            let mut v = BTreeMap::new();
            for r in &ramps {
                if r.t_peak < p.t0 || r.t0 > p.t1 {
                    continue;
                }
                for (zone, n) in votes(zones, &points, r.t0.max(p.t0), r.t_peak.min(p.t1)) {
                    *v.entry(zone).or_default() += n;
                }
                p.top = Some(p.top.map_or(r.top, |q| q.max(r.top)));
            }
            if v.is_empty() {
                fb.whole_period_votes += 1;
                v = votes(zones, &points, p.t0, p.t1);
            }
            p.zone = clear_modal(&v);
            if p.zone.is_none() {
                cov.unpaired += 1;
            } else {
                kept.push(p);
            }
        }
        states(
            &kept,
            series
                .owner
                .get(&slot.wrapping_add(1))
                .map_or(&[], Vec::as_slice),
            teams,
            cov,
            &mut fb,
        )
    } else {
        cov.method = "positions+geometry".into();
        ramps.sort_by_key(|r| r.t0);
        if ramps.is_empty() {
            return (Vec::new(), fb);
        }
        let mut periods: Vec<Period> = Vec::new();
        for r in ramps {
            let zone = clear_modal(&votes(zones, &points, r.t0, r.t_peak));
            if zone.is_none() {
                cov.unpaired += 1;
                continue;
            }
            while let Some(last) = periods.last_mut() {
                last.t1 = r.t0 - 1;
                if last.t1 >= last.t0 {
                    break;
                }
                periods.pop();
            }
            if let Some(last) = periods.last_mut().filter(|p| p.zone == zone) {
                last.t1 = last.t1.max(r.t_peak);
                last.top = Some(last.top.unwrap_or(0).max(r.top));
            } else {
                periods.push(Period {
                    t0: r.t0,
                    t1: r.t_peak,
                    zone,
                    top: Some(r.top),
                });
            }
        }
        if let Some(last) = periods.last_mut() {
            last.t1 = last.t1.max(frames - 1);
        }
        cov.hill_periods = periods.len();
        states(&periods, &[], &BTreeSet::new(), cov, &mut fb)
    };
    cov.paired = output.len();
    tally_replay_zone_states(&output, cov);
    (output, fb)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        entry: MapObjectivesEntry,
        series: ReplayZoneSeries,
        tracks: Vec<ReplayTrack>,
        teams: BTreeMap<String, i64>,
        frames: i64,
        states: Vec<ReplayZoneState>,
        coverage: ReplayZonesCoverage,
        fallbacks: ReplayHillFallbacks,
    }
    #[test]
    fn native_complete_hill_states() {
        let mut b = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-zone-hills-v41.json.zlib")[..],
        )
        .read_to_end(&mut b)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&b).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let mut cov = ReplayZonesCoverage::default();
            let (s, f) = build_replay_hill_states(
                &c.entry.zones_of_role("hill").zones,
                &c.series,
                &replay_zone_team_set(&c.teams),
                &c.tracks,
                c.frames,
                &mut cov,
            );
            assert_eq!(s, c.states, "states {i}");
            assert_eq!(cov, c.coverage, "coverage {i}");
            assert_eq!(f, c.fallbacks, "fallbacks {i}");
        }
    }
}
