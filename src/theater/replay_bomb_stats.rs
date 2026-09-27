//! Bomb statistics and two-pass arming attribution from independently read channels.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayKillReference {
    #[serde(rename = "KillerXUID")]
    pub killer_xuid: u64,
    #[serde(rename = "VictimXUID")]
    pub victim_xuid: u64,
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayBombArming {
    pub t: i64,
    pub time_ms: i64,
    pub start_t: i64,
    pub start_ms: i64,
    pub fuse_ms: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayBombStatsInput {
    pub detonations_read: bool,
    pub objectives: Vec<StatborgIdentifiedEvent>,
    pub carry_read: bool,
    pub carry: ReplayHeldObjectCarry,
    pub kills_read: bool,
    pub kills: Vec<ReplayKillReference>,
    pub armings_read: bool,
    pub armings: Vec<ReplayBombArming>,
    #[serde(rename = "FilmToMatchOffsetMS")]
    pub film_to_match_offset_ms: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayBombPlayerStats {
    pub xuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detonations: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grabs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_as_carrier_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carriers_killed: Option<i64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayBombEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub time_ms: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub xuid: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub actor_source: String,
}
pub fn replay_bomb_event_provenance(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "bomb_armed" => ("navpoint_ring", "exact"),
        "bomb_detonated" => ("statborg", "exact"),
        _ => ("", ""),
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayBombStatsCoverage {
    pub detonations_read: bool,
    pub carry_read: bool,
    pub kills_read: bool,
    pub armings_read: bool,
    pub detonations: usize,
    pub armings: usize,
    pub armings_attributed: usize,
    pub armings_by_drop: usize,
    pub armings_by_active_carry: usize,
    pub armings_no_carrier: usize,
    pub armings_no_bridge: usize,
    pub armings_ambiguous: usize,
    pub periods: usize,
    pub periods_no_bridge: usize,
    pub periods_open: usize,
    pub periods_by_death: usize,
    pub kills: usize,
    pub kills_on_carrier: usize,
    pub players: usize,
}
impl ReplayBombStatsCoverage {
    pub fn arming_balanced(&self) -> bool {
        self.armings_attributed == self.armings_by_drop + self.armings_by_active_carry
            && self.armings_attributed
                + self.armings_no_carrier
                + self.armings_no_bridge
                + self.armings_ambiguous
                == self.armings
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayBombMatchStats {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<ReplayBombPlayerStats>,
    pub coverage: ReplayBombStatsCoverage,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayBombStats {
    pub stats: ReplayBombMatchStats,
    pub events: Vec<ReplayBombEvent>,
}
#[derive(Default)]
struct BombTallies {
    detonations: BTreeMap<String, i64>,
    arms: BTreeMap<String, i64>,
    grabs: BTreeMap<String, i64>,
    seconds: BTreeMap<String, f64>,
    killed: BTreeMap<String, i64>,
}
fn increment(counts: &mut BTreeMap<String, i64>, xuid: &str) {
    let n = counts.entry(xuid.into()).or_default();
    *n = n.wrapping_add(1);
}
/// Drop attribution is completed for every arming before active-carry fallback
/// begins. A fallback must never consume a period needed by a later observed drop.
fn bomb_arms(
    input: &ReplayBombStatsInput,
    cov: &mut ReplayBombStatsCoverage,
) -> (BTreeMap<String, i64>, Vec<ReplayBombEvent>) {
    let mut counts = BTreeMap::new();
    let mut events = Vec::new();
    if !input.armings_read {
        return (counts, events);
    }
    cov.armings = input.armings.len();
    let mut armings = input.armings.clone();
    armings.sort_by_key(|a| a.time_ms);
    let mut candidates: Vec<_> = if input.carry_read {
        input.carry.periods.iter().filter(|p| !p.open).collect()
    } else {
        vec![]
    };
    candidates.sort_by_key(|p| (p.end_ms, p.xuid));
    let mut used = vec![false; candidates.len()];
    let mut verdicts = vec![(None, "", false); armings.len()];
    for (i, a) in armings.iter().enumerate() {
        let at = a.time_ms.wrapping_add(input.film_to_match_offset_ms);
        let mut best = None;
        let mut distance = 0;
        for (j, p) in candidates.iter().enumerate() {
            if used[j] || p.by_death || p.start_ms > at {
                continue;
            }
            let gap = p.end_ms.wrapping_sub(at).wrapping_abs();
            if gap > 2500 {
                continue;
            }
            if best.is_none() || gap < distance {
                best = Some(j);
                distance = gap;
            }
        }
        if let Some(j) = best {
            used[j] = true;
            verdicts[i] = (Some(j), "carry_drop", false);
        }
    }
    for (i, a) in armings.iter().enumerate() {
        if verdicts[i].0.is_some() {
            continue;
        }
        let at = a.time_ms.wrapping_add(input.film_to_match_offset_ms);
        let mut found = None;
        let mut ambiguous = false;
        for (j, p) in candidates.iter().enumerate() {
            if used[j] || p.start_ms > at || p.end_ms < at {
                continue;
            }
            if found.is_some() {
                found = None;
                ambiguous = true;
                break;
            }
            found = Some(j);
        }
        if let Some(j) = found {
            used[j] = true;
        }
        verdicts[i] = (found, "carry_active", ambiguous);
    }
    for (a, (candidate, source, ambiguous)) in armings.iter().zip(verdicts) {
        let mut event = ReplayBombEvent {
            kind: "bomb_armed".into(),
            time_ms: a.time_ms,
            ..Default::default()
        };
        match candidate {
            None if ambiguous => cov.armings_ambiguous += 1,
            None => cov.armings_no_carrier += 1,
            Some(j) if candidates[j].xuid == 0 => cov.armings_no_bridge += 1,
            Some(j) => {
                event.xuid = candidates[j].xuid.to_string();
                event.actor_source = source.into();
                increment(&mut counts, &event.xuid);
                cov.armings_attributed += 1;
                if source == "carry_drop" {
                    cov.armings_by_drop += 1;
                } else {
                    cov.armings_by_active_carry += 1;
                }
            }
        }
        events.push(event);
    }
    (counts, events)
}
pub fn build_replay_bomb_stats(input: &ReplayBombStatsInput) -> ReplayBombStats {
    let mut cov = ReplayBombStatsCoverage {
        detonations_read: input.detonations_read,
        carry_read: input.carry_read,
        kills_read: input.kills_read,
        armings_read: input.armings_read,
        ..Default::default()
    };
    let mut tallies = BombTallies::default();
    let mut events = Vec::new();
    if input.detonations_read {
        for e in &input.objectives {
            if e.event.stat != "bomb_detonations" || e.xuid.is_empty() {
                continue;
            }
            increment(&mut tallies.detonations, &e.xuid);
            events.push(ReplayBombEvent {
                kind: "bomb_detonated".into(),
                time_ms: e.event.time_ms,
                xuid: e.xuid.clone(),
                ..Default::default()
            });
        }
    }
    cov.detonations = events.len();
    let (arms, armed) = bomb_arms(input, &mut cov);
    tallies.arms = arms;
    events.extend(armed);
    events.sort_by(|a, b| (a.time_ms, &a.kind, &a.xuid).cmp(&(b.time_ms, &b.kind, &b.xuid)));
    if input.carry_read {
        cov.periods = input.carry.periods.len();
        for p in &input.carry.periods {
            if p.xuid == 0 {
                cov.periods_no_bridge += 1;
                continue;
            }
            increment(&mut tallies.grabs, &p.xuid.to_string());
            if p.open {
                cov.periods_open += 1;
            }
            if p.by_death {
                cov.periods_by_death += 1;
            }
        }
        // Totals are supplied by reconstruction; never redefine them from clipped periods.
        for (&x, &ms) in &input.carry.carry_ms_by_xuid {
            tallies.seconds.insert(x.to_string(), ms as f64 / 1000.0);
        }
        if input.kills_read {
            cov.kills = input.kills.len();
            for k in &input.kills {
                if k.killer_xuid == 0 || k.killer_xuid == k.victim_xuid {
                    continue;
                }
                if input.carry.periods.iter().any(|p| {
                    p.xuid != 0
                        && p.xuid == k.victim_xuid
                        && k.time_ms >= p.start_ms
                        && k.time_ms <= p.end_ms.wrapping_add(150)
                }) {
                    increment(&mut tallies.killed, &k.killer_xuid.to_string());
                    cov.kills_on_carrier += 1;
                }
            }
        }
    }
    let xuids: BTreeSet<_> = [
        &tallies.detonations,
        &tallies.arms,
        &tallies.grabs,
        &tallies.killed,
    ]
    .into_iter()
    .flat_map(|m| m.keys())
    .chain(tallies.seconds.keys())
    .cloned()
    .collect();
    let players: Vec<_> = xuids
        .into_iter()
        .map(|xuid| ReplayBombPlayerStats {
            detonations: input
                .detonations_read
                .then(|| *tallies.detonations.get(&xuid).unwrap_or(&0)),
            arms: (input.armings_read && input.carry_read)
                .then(|| *tallies.arms.get(&xuid).unwrap_or(&0)),
            grabs: input
                .carry_read
                .then(|| *tallies.grabs.get(&xuid).unwrap_or(&0)),
            time_as_carrier_seconds: input
                .carry_read
                .then(|| *tallies.seconds.get(&xuid).unwrap_or(&0.0)),
            carriers_killed: (input.carry_read && input.kills_read)
                .then(|| *tallies.killed.get(&xuid).unwrap_or(&0)),
            xuid,
        })
        .collect();
    cov.players = players.len();
    ReplayBombStats {
        stats: ReplayBombMatchStats {
            players,
            coverage: cov,
        },
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_complete_bomb_stats_and_arming_attribution() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-bomb-stats-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in fixture["rows"].as_array().unwrap().iter().enumerate() {
            let input: ReplayBombStatsInput = serde_json::from_value(r["input"].clone()).unwrap();
            let actual = build_replay_bomb_stats(&input);
            assert_eq!(
                actual,
                serde_json::from_value::<ReplayBombStats>(r["output"].clone()).unwrap(),
                "bomb stats {i}"
            );
            assert!(actual.stats.coverage.arming_balanced());
            assert_eq!(
                actual
                    .stats
                    .players
                    .iter()
                    .map(|p| p.arms.unwrap_or(0))
                    .sum::<i64>(),
                actual.stats.coverage.armings_attributed as i64
            );
        }
        for p in fixture["provenance"].as_array().unwrap() {
            assert_eq!(
                replay_bomb_event_provenance(p[0].as_str().unwrap()),
                (p[1].as_str().unwrap(), p[2].as_str().unwrap())
            );
        }
    }
}
