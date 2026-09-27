//! Flag spawn selection and homecoming closures from credited returns and free lives.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub fn replay_flag_spawn_at(spawns: &[ReplayFlagSpawn], x: f32, y: f32) -> Option<usize> {
    let mut best = None;
    let mut distance = 0.01_f64;
    for (i, s) in spawns.iter().enumerate() {
        // Subtract as float32 before promoting, as the native geometry does.
        let dx = f64::from(s.x - x);
        let dy = f64::from(s.y - y);
        let d = dx * dx + dy * dy;
        if d <= distance {
            best = Some(i);
            distance = d;
        }
    }
    best
}
fn first(life: &FreeObjectiveLife) -> (f32, f32) {
    life.pts.first().map_or((0., 0.), |p| (p.x, p.y))
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayFlagSpawnChoice {
    pub spawns: Vec<ReplayFlagSpawn>,
    pub neutral: bool,
    pub neutral_births: usize,
    pub team_births: usize,
}
pub fn choose_replay_flag_spawns(
    spawns: &[ReplayFlagSpawn],
    lives: &[FreeObjectiveLife],
) -> ReplayFlagSpawnChoice {
    let (neutral, teams): (Vec<_>, Vec<_>) = spawns.iter().cloned().partition(|s| s.neutral);
    let births = |spawns: &[ReplayFlagSpawn]| {
        lives
            .iter()
            .filter(|l| {
                let (x, y) = first(l);
                replay_flag_spawn_at(spawns, x, y).is_some()
            })
            .count()
    };
    let neutral_births = births(&neutral);
    let team_births = births(&teams);
    let use_neutral = !neutral.is_empty() && neutral_births >= 3 && neutral_births > team_births;
    ReplayFlagSpawnChoice {
        spawns: if use_neutral { neutral } else { teams },
        neutral: use_neutral,
        neutral_births,
        team_births,
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayFlagHomecoming {
    pub flag: i64,
    pub at: i64,
    pub x: f32,
    pub y: f32,
}
fn flag_of_owner(spawns: &[ReplayFlagSpawn], team: i64) -> Option<usize> {
    if spawns.len() <= 1 {
        return Some(0);
    }
    let mut owned = spawns.iter().enumerate().filter(|(_, s)| s.team == team);
    let (i, _) = owned.next()?;
    if owned.next().is_some() {
        None
    } else {
        Some(i)
    }
}
pub fn replay_flag_returns(
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
    spawns: &[ReplayFlagSpawn],
    teams: &BTreeMap<String, i64>,
) -> Vec<ReplayFlagHomecoming> {
    let mut out: Vec<_> = events
        .iter()
        .filter(|e| e.stat == "flag_returns")
        .map(|e| ReplayFlagHomecoming {
            flag: teams
                .get(identity.at(e.slot, e.time_ms))
                .and_then(|&team| flag_of_owner(spawns, team))
                .map_or(-1, |f| f as i64),
            at: e.time_ms,
            x: 0.,
            y: 0.,
        })
        .collect();
    native_sort::sort_by(&mut out, |a, b| a.at.cmp(&b.at));
    out
}
/// The native caller requires a nonzero replay step for object time conversion.
pub fn replay_flag_object_homecomings(
    lives: &[FreeObjectiveLife],
    spawns: &[ReplayFlagSpawn],
    clock: ReplayMatchClock,
) -> Vec<ReplayFlagHomecoming> {
    let mut out = Vec::new();
    for life in lives {
        let (x, y) = first(life);
        let Some(flag) = replay_flag_spawn_at(spawns, x, y) else {
            continue;
        };
        let frame = flag_frame_of(life.t0_us, clock);
        out.push(ReplayFlagHomecoming {
            flag: flag as i64,
            at: clock.match_ms_of_frame(frame),
            x,
            y,
        });
    }
    native_sort::sort_by(&mut out, |a, b| a.at.cmp(&b.at));
    out
}
pub fn close_replay_flags_by_homecoming(
    raw: &mut [ReplayFlagCarryRaw],
    returns: &[ReplayFlagHomecoming],
    homes: &[ReplayFlagHomecoming],
    spawns: &[ReplayFlagSpawn],
    teams: &BTreeMap<String, i64>,
) {
    let flags: Vec<_> = raw
        .iter()
        .map(|r| replay_flag_of_carrier(spawns, teams, &r.xuid).map_or(-1, |f| f as i64))
        .collect();
    for i in 0..raw.len() {
        let flag = flags[i];
        if flag < 0 {
            continue;
        }
        let inside =
            |h: &&ReplayFlagHomecoming| h.flag == flag && h.at > raw[i].t0 && h.at < raw[i].t1;
        let ret = returns.iter().find(inside).map(|h| h.at);
        let home = homes
            .iter()
            .filter(inside)
            .find(|h| {
                !raw.iter().zip(&flags).any(|(r, &f)| {
                    !r.captured && f != flag && (-1000..=1000).contains(&r.t1.wrapping_sub(h.at))
                })
            })
            .map(|h| h.at);
        match (ret, home) {
            (Some(r), Some(h)) if r <= h => raw[i].close_at(r, ReplayFlagCloser::Return),
            (_, Some(h)) => raw[i].close_at(h, ReplayFlagCloser::Home),
            (Some(r), None) => raw[i].close_at(r, ReplayFlagCloser::Return),
            _ => {}
        }
    }
}
