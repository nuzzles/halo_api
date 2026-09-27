//! Native candidate voting and plateau refinement for the match-to-film death clock.
use super::{IdentityDeath, IdentityLife};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityDeathClock {
    pub offset_ms: i64,
    pub matched: usize,
    pub runner_up: usize,
}
/// LegacyFilm time = match time + offset. Empty evidence produces zero counts.
/// The match-count refinement consumes deaths in input order, as in the native
/// clock calibration; it is distinct from the final distance-sorted death join.
pub fn calibrate_identity_death_clock(
    lives: &[IdentityLife],
    deaths: &[IdentityDeath],
) -> IdentityDeathClock {
    let ends: Vec<_> = lives.iter().map(|l| l.to / 1000).collect();
    if ends.is_empty() || deaths.is_empty() {
        return IdentityDeathClock::default();
    }
    let (mut best_offset, mut best_n, mut second_n) = (0_i64, -1_i64, 0_i64);
    for candidate in vote_offsets(&ends, deaths) {
        let (offset, n) = refine_offset(&ends, deaths, candidate);
        let n = n as i64;
        if best_n >= 0 && offset.wrapping_sub(best_offset).wrapping_abs() <= 300 {
            if n > best_n {
                best_offset = offset;
                best_n = n;
            }
            continue;
        }
        if n > best_n {
            second_n = best_n;
            best_n = n;
            best_offset = offset;
        } else if n > second_n {
            second_n = n;
        }
    }
    if best_n < 0 {
        return IdentityDeathClock::default();
    }
    IdentityDeathClock {
        offset_ms: best_offset,
        matched: best_n as usize,
        runner_up: second_n.max(0) as usize,
    }
}
fn pivot_bins(pivots: &[i64], others: &[i64], pivot_is_end: bool) -> [BTreeMap<i64, usize>; 2] {
    let mut grids = [BTreeMap::new(), BTreeMap::new()];
    for &p in pivots {
        let mut seen = [BTreeSet::new(), BTreeSet::new()];
        for &a in others {
            let diff = if pivot_is_end {
                p.wrapping_sub(a)
            } else {
                a.wrapping_sub(p)
            };
            for (g, bin) in [diff.div_euclid(150), diff.wrapping_add(75).div_euclid(150)]
                .into_iter()
                .enumerate()
            {
                if seen[g].insert(bin) {
                    *grids[g].entry(bin).or_default() += 1;
                }
            }
        }
    }
    grids
}
fn vote_offsets(ends: &[i64], deaths: &[IdentityDeath]) -> Vec<i64> {
    let times: Vec<_> = deaths.iter().map(|d| d.time_ms).collect();
    let by_death = pivot_bins(&times, ends, false);
    let by_end = pivot_bins(ends, &times, true);
    let mut bins = Vec::new();
    for g in 0..2 {
        for (&bin, &n) in &by_death[g] {
            let center = bin
                .wrapping_mul(150)
                .wrapping_add(75)
                .wrapping_sub(g as i64 * 75);
            bins.push((center, n.min(by_end[g].get(&bin).copied().unwrap_or(0))));
        }
    }
    bins.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut out: Vec<i64> = Vec::new();
    for (center, _) in bins {
        if out
            .iter()
            .any(|&c| center.wrapping_sub(c).wrapping_abs() <= 150)
        {
            continue;
        }
        out.push(center);
        if out.len() == 3 {
            break;
        }
    }
    out
}
fn count_matches(ends: &[i64], deaths: &[IdentityDeath], offset: i64) -> usize {
    let mut used = vec![false; ends.len()];
    let mut count = 0;
    for d in deaths {
        let target = d.time_ms.wrapping_add(offset);
        let mut best = None;
        let mut distance = 151;
        for (i, &end) in ends.iter().enumerate() {
            if used[i] {
                continue;
            }
            let delta = end.wrapping_sub(target).wrapping_abs();
            if delta < distance {
                distance = delta;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            used[i] = true;
            count += 1;
        }
    }
    count
}
fn refine_offset(ends: &[i64], deaths: &[IdentityDeath], around: i64) -> (i64, usize) {
    let anchor = *ends.iter().min().expect("nonempty life ends");
    let mut start = around.wrapping_sub(300);
    let remainder = start.wrapping_sub(anchor).rem_euclid(10);
    if remainder != 0 {
        start = start.wrapping_add(10 - remainder);
    }
    let mut best = None;
    let mut plateau = Vec::new();
    let mut offset = start;
    while offset <= around.wrapping_add(300) {
        let n = count_matches(ends, deaths, offset);
        if best.is_none_or(|b| n > b) {
            best = Some(n);
            plateau.clear();
            plateau.push(offset);
        } else if best == Some(n) {
            plateau.push(offset);
        }
        let Some(next) = offset.checked_add(10) else {
            break;
        };
        offset = next;
    }
    (
        plateau.get(plateau.len() / 2).copied().unwrap_or(start),
        best.unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        ends: Vec<i64>,
        deaths: Vec<IdentityDeath>,
        candidates: Vec<i64>,
        refined: Vec<Refined>,
        output: IdentityDeathClock,
    }
    #[derive(Deserialize)]
    struct Refined {
        offset: i64,
        matched: usize,
        at_candidate: usize,
    }
    #[test]
    fn native_death_clock_calibration() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-death-clock-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            assert_eq!(
                vote_offsets(&c.ends, &c.deaths),
                c.candidates,
                "candidates {i}"
            );
            for (&candidate, r) in c.candidates.iter().zip(c.refined) {
                assert_eq!(
                    count_matches(&c.ends, &c.deaths, candidate),
                    r.at_candidate,
                    "candidate count {i}"
                );
                assert_eq!(
                    refine_offset(&c.ends, &c.deaths, candidate),
                    (r.offset, r.matched),
                    "refinement {i}"
                );
            }
            let lives: Vec<_> = c
                .ends
                .into_iter()
                .enumerate()
                .map(|(slot, to)| IdentityLife {
                    slot: slot as u32,
                    to: to * 1000,
                    ..Default::default()
                })
                .collect();
            assert_eq!(
                calibrate_identity_death_clock(&lives, &c.deaths),
                c.output,
                "calibration {i}"
            );
        }
    }
}
