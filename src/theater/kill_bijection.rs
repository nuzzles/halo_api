//! Native vote initialization, fixed-seed local search and bijection margin.
use super::{KillFeedEvent, KillPacketIdentity, KillRoster, kill_rng::KillRng};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSourceCandidate {
    pub packet: Option<KillPacketIdentity>,
    pub time_ms: i64,
    pub bit: i64,
    pub tag: u32,
    pub victim: i32,
    pub killer: i32,
    pub category: i32,
}
struct Near<'a> {
    candidates: &'a [KillSourceCandidate],
    indices: Vec<Vec<usize>>,
}
impl<'a> Near<'a> {
    fn new(pairs: &[KillFeedEvent], candidates: &'a [KillSourceCandidate]) -> Self {
        Self {
            candidates,
            indices: candidates
                .iter()
                .map(|c| {
                    pairs
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.time_ms.abs_diff(c.time_ms) <= 2500)
                        .map(|(j, _)| j)
                        .collect()
                })
                .collect(),
        }
    }
    fn score(&self, pairs: &[KillFeedEvent], names: &[String], perm: &[i32]) -> i64 {
        let name = |index: i32| {
            usize::try_from(index)
                .ok()
                .and_then(|i| perm.get(i))
                .and_then(|&p| usize::try_from(p).ok())
                .and_then(|p| names.get(p))
        };
        self.candidates
            .iter()
            .zip(&self.indices)
            .filter(|(c, near)| {
                let (Some(v), Some(k)) = (name(c.victim), name(c.killer)) else {
                    return false;
                };
                near.iter()
                    .any(|&j| pairs[j].victim == *v && pairs[j].killer == *k)
            })
            .count() as i64
    }
    fn votes(&self, pairs: &[KillFeedEvent], names: &[String], n: usize) -> Vec<Vec<i64>> {
        let index: BTreeMap<_, _> = names
            .iter()
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();
        let mut out = vec![vec![0; names.len()]; n];
        for (c, near) in self.candidates.iter().zip(&self.indices) {
            for &j in near {
                for (slot, name) in [(c.victim, &pairs[j].victim), (c.killer, &pairs[j].killer)] {
                    if let (Ok(slot), Some(&p)) = (usize::try_from(slot), index.get(name))
                        && slot < n
                    {
                        out[slot][p] += 1;
                    }
                }
            }
        }
        out
    }
    fn refine(
        &self,
        pairs: &[KillFeedEvent],
        names: &[String],
        mut perm: Vec<i32>,
        free: &[i32],
    ) -> (Vec<i32>, i64) {
        let mut best = self.score(pairs, names, &perm);
        loop {
            let mut swap = None;
            let mut score = best;
            for a in 0..free.len() {
                for b in a + 1..free.len() {
                    let (i, j) = (free[a] as usize, free[b] as usize);
                    perm.swap(i, j);
                    let s = self.score(pairs, names, &perm);
                    if s > score {
                        swap = Some((i, j));
                        score = s;
                    }
                    perm.swap(i, j);
                }
            }
            let Some((i, j)) = swap else {
                return (perm, best);
            };
            perm.swap(i, j);
            best = score;
        }
    }
}
fn hungarian(cost: &[Vec<i64>]) -> Vec<usize> {
    let n = cost.len();
    let mut u = vec![0; n + 1];
    let mut v = vec![0; n + 1];
    let mut p = vec![0; n + 1];
    let mut way = vec![0; n + 1];
    for i in 1..=n {
        p[0] = i;
        let mut minv = vec![1i64 << 60; n + 1];
        let mut used = vec![false; n + 1];
        let mut j0 = 0;
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = 1i64 << 60;
            let mut j1 = 0;
            for j in 1..=n {
                if used[j] {
                    continue;
                }
                let cur = cost[i0 - 1][j - 1] - u[i0] - v[j];
                if cur < minv[j] {
                    minv[j] = cur;
                    way[j] = j0;
                }
                if minv[j] < delta {
                    delta = minv[j];
                    j1 = j;
                }
            }
            for j in 0..=n {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    minv[j] -= delta;
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }
    let mut perm = vec![0; n];
    for j in 1..=n {
        if p[j] > 0 {
            perm[p[j] - 1] = j - 1;
        }
    }
    perm
}
/// Resolve the roster permutation and retain independent film-seat vote counters.
/// The native fully-pinned shortcut reports score zero, even for matching events.
/// Returns an error for malformed roster state that would panic in the reference.
pub fn solve_kill_bijection(
    roster: &mut KillRoster,
    pairs: &[KillFeedEvent],
    candidates: &[KillSourceCandidate],
    restarts: usize,
) -> Result<i64, super::DecodeError> {
    let invalid =
        || super::DecodeError::Inconsistent("invalid kill-source roster permutation domain".into());
    let (free, free_names) = roster.free_slots();
    if free.len() > free_names.len()
        || roster.evidence.pins.iter().any(|(&i, &p)| {
            i < 0
                || i as usize >= roster.player_count
                || p < 0
                || p as usize >= roster.evidence.names.len()
        })
    {
        return Err(invalid());
    }
    let near = Near::new(pairs, candidates);
    let votes = near.votes(pairs, &roster.evidence.names, roster.player_count);
    for (i, row) in votes.iter().enumerate() {
        if !roster.evidence.seat_pins.contains(&(i as i32)) {
            continue;
        }
        let Some(&pos) = roster.evidence.pins.get(&(i as i32)) else {
            continue;
        };
        let total: i64 = row.iter().sum();
        let best = row.iter().max().copied().unwrap_or(0);
        if total == 0 {
            roster.table.silent += 1;
        } else if row[pos as usize] == best {
            roster.table.agree += 1;
        } else {
            roster.table.contradict += 1;
        }
    }
    roster.table.inferred = free.len();
    roster.table.free_names = free_names.len();
    let mut pinned = vec![0; roster.player_count];
    for (&i, &p) in &roster.evidence.pins {
        pinned[i as usize] = p;
    }
    if free.is_empty() {
        roster.permutation = pinned;
        return Ok(0);
    }
    let n = free.len();
    let m = free_names.len();
    let k = n.max(m);
    let mut cost = vec![vec![0; k]; k];
    for a in 0..n {
        for b in 0..k {
            cost[a][b] = if b >= m {
                1
            } else {
                -votes[free[a] as usize][free_names[b] as usize]
            };
        }
    }
    let sub = hungarian(&cost);
    let mut perm = pinned.clone();
    for a in 0..n {
        if sub[a] < m {
            perm[free[a] as usize] = free_names[sub[a]];
        }
    }
    let (mut best_perm, mut best) = near.refine(pairs, &roster.evidence.names, perm, &free);
    let mut rng = KillRng::new();
    for _ in 0..restarts {
        let mut perm = pinned.clone();
        let sh = rng.permutation(m);
        for (a, &slot) in free.iter().enumerate() {
            perm[slot as usize] = free_names[sh[a]];
        }
        let (p, s) = near.refine(pairs, &roster.evidence.names, perm, &free);
        if s > best || (s == best && p < best_perm) {
            best_perm = p;
            best = s;
        }
    }
    roster.permutation = best_perm;
    Ok(best)
}
/// Difference between the selected score and the best single free-slot swap.
/// Zero is also returned when there is no alternative swap.
pub fn kill_bijection_margin(
    roster: &KillRoster,
    pairs: &[KillFeedEvent],
    candidates: &[KillSourceCandidate],
    best: i64,
) -> i64 {
    let near = Near::new(pairs, candidates);
    let (free, _) = roster.free_slots();
    let mut cur = roster.permutation.clone();
    let mut second = None;
    for a in 0..free.len() {
        for b in a + 1..free.len() {
            let (i, j) = (free[a] as usize, free[b] as usize);
            if i >= cur.len() || j >= cur.len() {
                continue;
            }
            cur.swap(i, j);
            let score = near.score(pairs, &roster.evidence.names, &cur);
            second = Some(second.map_or(score, |s: i64| s.max(score)));
            cur.swap(i, j);
        }
    }
    second.map_or(0, |second| best - second)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        names: Vec<String>,
        n: usize,
        pins: BTreeMap<i32, i32>,
        seats: BTreeSet<i32>,
        pairs: Vec<KillFeedEvent>,
        candidates: Vec<KillSourceCandidate>,
        restarts: usize,
        permutation: Vec<i32>,
        score: i64,
        margin: i64,
        agree: usize,
        contradict: usize,
        silent: usize,
        inferred: usize,
        free_names: usize,
    }
    #[derive(Deserialize)]
    struct Oracle {
        rows: Vec<Row>,
        rng: Vec<Vec<usize>>,
    }
    #[test]
    fn native_kill_bijection_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-bijection-v41.json.zlib")[..],
        )
        .read_to_end(&mut json)
        .unwrap();
        let oracle: Oracle = serde_json::from_slice(&json).unwrap();
        assert_eq!(oracle.rows.len(), 512);
        let mut rng = KillRng::new();
        for (i, expected) in oracle.rng.into_iter().enumerate() {
            assert_eq!(rng.permutation(i % 33), expected, "rng {i}");
        }
        for (i, row) in oracle.rows.into_iter().enumerate() {
            let mut r = KillRoster {
                evidence: super::super::KillRosterPins {
                    names: row.names,
                    pins: row.pins,
                    seat_pins: row.seats,
                    ..Default::default()
                },
                player_count: row.n,
                humans: row.n,
                ..Default::default()
            };
            let score =
                solve_kill_bijection(&mut r, &row.pairs, &row.candidates, row.restarts).unwrap();
            assert_eq!(score, row.score, "score {i}");
            assert_eq!(r.permutation, row.permutation, "perm {i}");
            assert_eq!(
                kill_bijection_margin(&r, &row.pairs, &row.candidates, score),
                row.margin,
                "margin {i}"
            );
            assert_eq!(
                (
                    r.table.agree,
                    r.table.contradict,
                    r.table.silent,
                    r.table.inferred,
                    r.table.free_names
                ),
                (
                    row.agree,
                    row.contradict,
                    row.silent,
                    row.inferred,
                    row.free_names
                ),
                "counters {i}"
            );
        }
    }
}
