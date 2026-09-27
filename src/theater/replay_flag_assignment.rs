//! Chronological flag assignment under the native own-team invariant.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFlagAssignmentCoverage {
    pub own_flag_refused: usize,
    pub unresolved: usize,
    pub assigned_by_play: usize,
}
#[derive(Clone, Copy)]
struct GroundEvent {
    at: i64,
    rank: u8,
    index: usize,
}
struct Ground {
    positions: Vec<Option<(f32, f32)>>,
    in_play: Vec<bool>,
}
impl Ground {
    fn home(&mut self, f: usize) {
        if f < self.positions.len() {
            self.positions[f] = None;
            self.in_play[f] = false;
        }
    }
    fn choose(
        &self,
        r: &ReplayFlagCarryRaw,
        spawns: &[ReplayFlagSpawn],
        accept: impl Fn(usize) -> bool,
    ) -> (Option<usize>, bool) {
        if !r.steal {
            let mut best = None;
            let mut distance = 64.;
            for (i, p) in self.positions.iter().enumerate() {
                if !accept(i) {
                    continue;
                }
                if let Some((x, y)) = p {
                    let d = flag_sq_distance(*x, *y, r.x0, r.y0);
                    if d <= distance {
                        best = Some(i);
                        distance = d;
                    }
                }
            }
            if best.is_some() {
                return (best, false);
            }
            let mut playing = self
                .in_play
                .iter()
                .enumerate()
                .filter(|(i, v)| **v && accept(*i));
            if let Some((i, _)) = playing.next()
                && playing.next().is_none()
            {
                return (Some(i), true);
            }
        }
        let mut best = None;
        let mut distance = 0.;
        for (i, s) in spawns.iter().enumerate() {
            if !accept(i) {
                continue;
            }
            let d = flag_sq_distance(s.x, s.y, r.x0, r.y0);
            if best.is_none() || d < distance {
                best = Some(i);
                distance = d;
            }
        }
        (best, false)
    }
}
pub fn assign_replay_flags(
    raw: &mut [ReplayFlagCarryRaw],
    spawns: &[ReplayFlagSpawn],
    teams: &BTreeMap<String, i64>,
    return_times: &[i64],
    homes: &[ReplayFlagHomecoming],
) -> ReplayFlagAssignmentCoverage {
    let mut cov = ReplayFlagAssignmentCoverage::default();
    if spawns.is_empty() {
        for r in raw {
            r.flag_index = 0;
        }
        return cov;
    }
    let mut ground = Ground {
        positions: vec![None; spawns.len()],
        in_play: vec![false; spawns.len()],
    };
    let mut events = Vec::new();
    for (i, r) in raw.iter().enumerate() {
        events.push(GroundEvent {
            at: r.t0,
            rank: 3,
            index: i,
        });
        events.push(GroundEvent {
            at: r.t1,
            rank: 0,
            index: i,
        });
    }
    for &at in return_times {
        events.push(GroundEvent {
            at,
            rank: 1,
            index: 0,
        });
    }
    for h in homes {
        events.push(GroundEvent {
            at: h.at,
            rank: 2,
            index: h.flag as usize,
        });
    }
    events.sort_by_key(|e| (e.at, e.rank));
    for e in events {
        match e.rank {
            0 => {
                let r = &raw[e.index];
                let f = r.flag_index as usize;
                if f >= spawns.len() {
                    continue;
                }
                if r.ends_home() {
                    ground.home(f);
                } else {
                    ground.positions[f] = Some((r.x1, r.y1));
                }
            }
            1 => {
                let mut dropped = ground
                    .positions
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.is_some());
                if let Some((i, _)) = dropped.next()
                    && dropped.next().is_none()
                {
                    ground.home(i);
                }
            }
            2 => ground.home(e.index),
            _ => {
                let r = &mut raw[e.index];
                let team = teams.get(&r.xuid).copied();
                let own = |i: usize| team.is_some_and(|t| t != -1 && spawns[i].team == t);
                let (mut f, mut elimination) = ground.choose(r, spawns, |_| true);
                if f.is_some_and(own) {
                    cov.own_flag_refused += 1;
                    (f, elimination) = ground.choose(r, spawns, |i| !own(i));
                }
                r.flag_index = f.map_or(-1, |f| f as i64);
                if let Some(f) = f {
                    if elimination {
                        cov.assigned_by_play += 1;
                    }
                    ground.positions[f] = None;
                    ground.in_play[f] = true;
                } else {
                    cov.unresolved += 1;
                }
            }
        }
    }
    cov
}
