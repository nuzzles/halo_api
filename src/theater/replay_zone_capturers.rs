//! LegacyFilm-read capture teams and the explicitly counted completed-ramp fallback.
use super::*;
use std::collections::BTreeSet;
pub fn replay_zone_value_during_ramp(
    samples: &[ReplayZoneSample],
    r: &ReplayZoneRamp,
) -> Option<u64> {
    samples
        .iter()
        .skip_while(|s| s.t < r.t0)
        .take_while(|s| s.t <= r.t_peak)
        .last()
        .map(|s| s.v)
}
pub fn elect_replay_zone_capturer(
    series: &ReplayZoneSeries,
    ramps: &[ReplayZoneRamp],
    owner: &[ReplayZoneSample],
    owner_slot: u32,
    win: i64,
) -> Vec<ReplayZoneSample> {
    let mut best = Vec::new();
    let mut best_n = 0;
    for (&slot, ss) in &series.owner_chained {
        if slot == owner_slot || ss.is_empty() || ss.iter().any(|s| s.v > 7 && s.v != 0xffff_ffff) {
            continue;
        }
        let (mut agree, mut disagree) = (0, 0);
        for r in ramps {
            if replay_gauge_progress(r.top) < 0.95 {
                continue;
            }
            if let (Some(v), Some(expected)) = (
                replay_zone_value_during_ramp(ss, r),
                replay_zone_value_after(owner, r.t_peak, win),
            ) {
                if v == expected {
                    agree += 1;
                } else {
                    disagree += 1;
                }
            }
        }
        if disagree == 0 && agree >= 2 && agree > best_n {
            best = ss.clone();
            best_n = agree;
        }
    }
    best
}
pub fn replay_zone_gauge_ramps(
    ramps: &[ReplayZoneRamp],
    owner: &[ReplayZoneSample],
    capturer: &[ReplayZoneSample],
    teams: &BTreeSet<u64>,
    win: i64,
) -> (Vec<ReplayZoneGaugeRamp>, usize) {
    let mut out = Vec::new();
    let mut inferred = 0;
    for r in ramps {
        let read = replay_zone_value_during_ramp(capturer, r)
            .and_then(|v| replay_zone_owner_team(v, teams));
        let team = if let Some(read) = read {
            read
        } else if replay_gauge_progress(r.top) >= 0.95 {
            let team = replay_zone_value_after(owner, r.t_peak, win)
                .and_then(|v| replay_zone_owner_team(v, teams))
                .flatten();
            if team.is_some() {
                inferred += 1;
            }
            team
        } else {
            None
        };
        out.push(ReplayZoneGaugeRamp {
            t0: r.t0,
            t1: r.t_peak,
            capturing_team: team,
        });
    }
    out.sort_by_key(|r| r.t0);
    (out, inferred)
}
