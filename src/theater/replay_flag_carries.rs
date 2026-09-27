//! Native flag carry evidence, openings and chronological closure stages.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayFlagFilmSignals {
    pub bursts: usize,
    pub captures: usize,
    pub steals: usize,
    pub grabs: usize,
}
impl ReplayFlagFilmSignals {
    pub fn from_events(bursts: &[i64], events: &[StatborgNamedEvent]) -> Self {
        let mut s = Self {
            bursts: bursts.len(),
            ..Default::default()
        };
        for e in events {
            match e.stat.as_str() {
                "flag_captures" => s.captures += 1,
                "flag_steals" => s.steals += 1,
                "flag_grabs" => s.grabs += 1,
                _ => {}
            }
        }
        s
    }
    pub fn is_flag_film(&self) -> bool {
        self.bursts > 0 && self.captures > 0 && self.captures <= self.bursts && self.steals > 0
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayFlagSpawn {
    pub team: i64,
    pub neutral: bool,
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFlagOpening {
    pub slot: i64,
    pub xuid: String,
    pub t0: i64,
    pub steal: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayFlagCloser {
    #[default]
    None,
    Bound,
    Handoff,
    Return,
    Home,
    CarrierKill,
    Object,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayFlagCarryRaw {
    pub xuid: String,
    pub t0: i64,
    pub t1: i64,
    pub steal: bool,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub captured: bool,
    pub closed: bool,
    pub homed: bool,
    pub closed_by: ReplayFlagCloser,
    pub flag_index: i64,
    pub confirmed: bool,
    pub observable: bool,
}
impl ReplayFlagCarryRaw {
    pub fn ends_home(&self) -> bool {
        self.captured || self.homed
    }
    pub fn close_at(&mut self, at: i64, by: ReplayFlagCloser) {
        if at <= self.t0 || at >= self.t1 {
            return;
        }
        self.t1 = at;
        self.closed_by = by;
        // Native carrier-kill evidence only shortens an otherwise open carry.
        self.closed |= by != ReplayFlagCloser::CarrierKill;
        self.captured = false;
        self.homed = matches!(by, ReplayFlagCloser::Return | ReplayFlagCloser::Home);
    }
}
pub fn replay_flag_openings(
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
) -> Vec<ReplayFlagOpening> {
    let mut slots = BTreeMap::<i64, Vec<ReplayFlagOpening>>::new();
    for e in events {
        let steal = e.stat == "flag_steals";
        if !steal && e.stat != "flag_grabs" {
            continue;
        }
        slots.entry(e.slot).or_default().push(ReplayFlagOpening {
            slot: e.slot,
            xuid: identity.at(e.slot, e.time_ms).into(),
            t0: e.time_ms,
            steal,
        });
    }
    let mut out: Vec<ReplayFlagOpening> = Vec::new();
    for (_, mut ops) in slots {
        ops.sort_by_key(|o| o.t0);
        for o in ops {
            if let Some(last) = out
                .last_mut()
                .filter(|last| last.slot == o.slot && o.t0.wrapping_sub(last.t0) <= 250)
            {
                last.steal |= o.steal;
            } else {
                out.push(o);
            }
        }
    }
    out.sort_by_key(|o| (o.t0, o.slot));
    out
}
pub fn bound_replay_flag_carries(
    ops: &[ReplayFlagOpening],
    events: &[StatborgNamedEvent],
    deaths: &[IdentityDeath],
    clock: ReplayMatchClock,
) -> Vec<ReplayFlagCarryRaw> {
    bound_flag_values(
        ops,
        events,
        &deaths
            .iter()
            .map(|d| (d.xuid, d.time_ms))
            .collect::<Vec<_>>(),
        clock,
    )
}

pub(super) fn bound_flag_values(
    ops: &[ReplayFlagOpening],
    events: &[StatborgNamedEvent],
    deaths: &[(u64, i64)],
    clock: ReplayMatchClock,
) -> Vec<ReplayFlagCarryRaw> {
    let end = events
        .iter()
        .map(|e| e.time_ms)
        .chain(deaths.iter().map(|d| d.1))
        .fold(
            clock.match_ms_of_frame(clock.frames.wrapping_sub(1)),
            i64::max,
        )
        .wrapping_add(1);
    let mut next = BTreeMap::new();
    let mut previous = BTreeMap::new();
    for (i, o) in ops.iter().enumerate() {
        if let Some(prev) = previous.insert(o.slot, i) {
            next.insert(prev, o.t0);
        }
    }
    ops.iter()
        .enumerate()
        .map(|(i, o)| {
            let mut r = ReplayFlagCarryRaw {
                xuid: o.xuid.clone(),
                t0: o.t0,
                t1: end,
                steal: o.steal,
                flag_index: -1,
                ..Default::default()
            };
            if let Some(c) = events
                .iter()
                .filter(|e| e.stat == "flag_captures" && e.slot == o.slot && e.time_ms > o.t0)
                .map(|e| e.time_ms)
                .min()
                .filter(|&c| c < r.t1)
            {
                r.t1 = c;
                r.captured = true;
                r.closed_by = ReplayFlagCloser::Bound;
            }
            if let Some(d) = deaths
                .iter()
                .filter(|d| d.0.to_string() == o.xuid && d.1 > o.t0)
                .map(|d| d.1)
                .min()
                .filter(|&d| d < r.t1)
            {
                r.t1 = d;
                r.captured = false;
                r.closed_by = ReplayFlagCloser::Bound;
            }
            if let Some(&n) = next.get(&i).filter(|&&n| n < r.t1) {
                r.t1 = n;
                r.captured = false;
                r.closed_by = ReplayFlagCloser::Bound;
            }
            r.closed = r.closed_by != ReplayFlagCloser::None;
            r
        })
        .collect()
}
pub fn replay_flag_of_carrier(
    spawns: &[ReplayFlagSpawn],
    teams: &BTreeMap<String, i64>,
    xuid: &str,
) -> Option<usize> {
    if spawns.len() <= 1 {
        return Some(0);
    }
    let team = *teams.get(xuid)?;
    if team == -1 {
        return None;
    }
    let mut opponents = spawns.iter().enumerate().filter(|(_, s)| s.team != team);
    let (i, _) = opponents.next()?;
    if opponents.next().is_some() {
        None
    } else {
        Some(i)
    }
}
pub fn close_replay_flags_by_handoff(
    raw: &mut [ReplayFlagCarryRaw],
    ops: &[ReplayFlagOpening],
    spawns: &[ReplayFlagSpawn],
    teams: &BTreeMap<String, i64>,
) -> usize {
    let flags: BTreeMap<_, _> = raw
        .iter()
        .map(|r| {
            (
                r.xuid.clone(),
                replay_flag_of_carrier(spawns, teams, &r.xuid).map_or(-1, |i| i as i64),
            )
        })
        .collect();
    let mut unnamed = 0;
    for r in raw {
        let mine = flags[&r.xuid];
        if mine < 0 {
            unnamed += 1;
            continue;
        }
        // Go's missing map value is zero; retain it for independently supplied ops.
        if let Some(at) = ops
            .iter()
            .filter(|o| {
                o.xuid != r.xuid
                    && o.t0 > r.t0
                    && o.t0 < r.t1
                    && flags.get(&o.xuid).copied().unwrap_or(0) == mine
            })
            .map(|o| o.t0)
            .min()
        {
            r.close_at(at, ReplayFlagCloser::Handoff);
        }
    }
    unnamed
}
pub fn close_replay_flags_by_carrier_kills(
    raw: &mut [ReplayFlagCarryRaw],
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
) -> usize {
    let mut ambiguous = 0;
    for e in events.iter().filter(|e| e.stat == "flag_carriers_killed") {
        let killer = identity.at(e.slot, e.time_ms);
        let mut candidates = raw
            .iter()
            .enumerate()
            .filter(|(_, r)| r.t0 < e.time_ms && e.time_ms < r.t1 && r.xuid != killer)
            .map(|(i, _)| i);
        if let Some(i) = candidates.next() {
            if candidates.next().is_some() {
                ambiguous += 1;
            } else {
                raw[i].close_at(e.time_ms, ReplayFlagCloser::CarrierKill);
            }
        }
    }
    ambiguous
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_flag_openings_and_closures() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-flag-carries-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let get = |key: &str| row[key].clone();
            let events: Vec<StatborgNamedEvent> = serde_json::from_value(get("events")).unwrap();
            let deaths: Vec<IdentityDeath> = serde_json::from_value(get("deaths")).unwrap();
            let clock: ReplayMatchClock = serde_json::from_value(get("clock")).unwrap();
            let spawns: Vec<ReplayFlagSpawn> = serde_json::from_value(get("spawns")).unwrap();
            let teams: BTreeMap<String, i64> = serde_json::from_value(get("teams")).unwrap();
            let by_slot: BTreeMap<i64, String> = serde_json::from_value(get("identity")).unwrap();
            let identity = StatborgRoundIdentity {
                publication: IdentityStatborgPublication {
                    by_round: BTreeMap::from([(0, by_slot)]),
                    ..Default::default()
                },
                starts: Vec::new(),
            };
            let bursts: Vec<i64> = serde_json::from_value(get("bursts")).unwrap();
            let signals = ReplayFlagFilmSignals::from_events(&bursts, &events);
            assert_eq!(
                signals,
                serde_json::from_value(get("signals")).unwrap(),
                "signals {i}"
            );
            assert_eq!(signals.is_flag_film(), row["recognized"].as_bool().unwrap());
            let ops = replay_flag_openings(&events, &identity);
            assert_eq!(
                ops,
                serde_json::from_value::<Vec<ReplayFlagOpening>>(get("openings")).unwrap(),
                "openings {i}"
            );
            let mut raw = bound_replay_flag_carries(&ops, &events, &deaths, clock);
            assert_eq!(
                raw,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("bound")).unwrap(),
                "bound {i}"
            );
            let unnamed = close_replay_flags_by_handoff(&mut raw, &ops, &spawns, &teams);
            assert_eq!(
                unnamed,
                row["unnamed"].as_u64().unwrap() as usize,
                "unnamed {i}"
            );
            assert_eq!(
                raw,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("handoff")).unwrap(),
                "handoff {i}"
            );
            let lives: Vec<FreeObjectiveLife> = serde_json::from_value(get("lives")).unwrap();
            assert_eq!(
                choose_replay_flag_spawns(&spawns, &lives),
                serde_json::from_value(get("choice")).unwrap(),
                "choice {i}"
            );
            let mut home_clock = clock;
            if home_clock.step_us == 0 {
                home_clock.step_us = 100000;
            }
            let homes = replay_flag_object_homecomings(&lives, &spawns, home_clock);
            let returns = replay_flag_returns(&events, &identity, &spawns, &teams);
            assert_eq!(
                homes,
                serde_json::from_value::<Vec<ReplayFlagHomecoming>>(get("homes")).unwrap(),
                "homes {i}"
            );
            assert_eq!(
                returns,
                serde_json::from_value::<Vec<ReplayFlagHomecoming>>(get("returns")).unwrap(),
                "returns {i}"
            );
            close_replay_flags_by_homecoming(&mut raw, &returns, &homes, &spawns, &teams);
            assert_eq!(
                raw,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("homed")).unwrap(),
                "homed {i}"
            );
            let ambiguous = close_replay_flags_by_carrier_kills(&mut raw, &events, &identity);
            assert_eq!(
                ambiguous,
                row["ambiguous"].as_u64().unwrap() as usize,
                "ambiguous {i}"
            );
            assert_eq!(
                raw,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("killed")).unwrap(),
                "kills {i}"
            );
            for c in row["closes"].as_array().unwrap() {
                let mut r: ReplayFlagCarryRaw =
                    serde_json::from_value(c["before"].clone()).unwrap();
                r.close_at(
                    c["at"].as_i64().unwrap(),
                    serde_json::from_value(c["by"].clone()).unwrap(),
                );
                assert_eq!(
                    r,
                    serde_json::from_value(c["after"].clone()).unwrap(),
                    "close {i}"
                );
            }
        }
    }
}
