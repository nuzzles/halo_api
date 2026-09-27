//! Object possession reconstructed from held-family transitions.
use super::{FactsDeath, IdentityDeath};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const HELD_OBJECT_OPEN_END_MS: i64 = 1 << 30;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayHeldObjectEvent {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    pub slot: u32,
    #[serde(rename = "XUID")]
    pub xuid: u64,
    pub pickup: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayHeldObjectPeriod {
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "XUID")]
    pub xuid: u64,
    #[serde(rename = "DebutMS")]
    pub start_ms: i64,
    #[serde(rename = "FinMS")]
    pub end_ms: i64,
    #[serde(rename = "FinParMort")]
    pub by_death: bool,
    #[serde(rename = "Ouverte")]
    pub open: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayHeldObjectCarry {
    #[serde(rename = "Events")]
    pub events: Vec<ReplayHeldObjectEvent>,
    #[serde(rename = "Periods")]
    pub periods: Vec<ReplayHeldObjectPeriod>,
    #[serde(rename = "CarryMSByXUID")]
    pub carry_ms_by_xuid: BTreeMap<u64, i64>,
}
/// Resolve identity at each transition, including recycled slots. Input event
/// identities are deliberately replaced. Deaths retain their supplied order.
/// Native drop transitions close directly; death lookup is used only at the
/// next pickup or at the open-end sentinel.
pub fn build_replay_held_object_carry(
    events: &[ReplayHeldObjectEvent],
    occupant: Option<&dyn Fn(u32, i64) -> u64>,
    deaths: &[IdentityDeath],
) -> ReplayHeldObjectCarry {
    held_object_carry(events, occupant, deaths.iter().map(|d| (d.xuid, d.time_ms)))
}

/// Cached deaths need no text conversion: only identity and time are consumed.
pub fn build_facts_replay_held_object_carry(
    events: &[ReplayHeldObjectEvent],
    occupant: Option<&dyn Fn(u32, i64) -> u64>,
    deaths: &[FactsDeath],
) -> ReplayHeldObjectCarry {
    held_object_carry(events, occupant, deaths.iter().map(|d| (d.xuid, d.time_ms)))
}

pub(super) fn held_object_carry(
    events: &[ReplayHeldObjectEvent],
    occupant: Option<&dyn Fn(u32, i64) -> u64>,
    deaths: impl Iterator<Item = (u64, i64)>,
) -> ReplayHeldObjectCarry {
    let mut events = events.to_vec();
    events.sort_by_key(|e| e.time_ms);
    let mut deaths_by_xuid = BTreeMap::<u64, Vec<i64>>::new();
    for (xuid, time_ms) in deaths {
        deaths_by_xuid.entry(xuid).or_default().push(time_ms);
    }
    for e in &mut events {
        e.xuid = occupant.map_or(0, |f| f(e.slot, e.time_ms));
    }
    let first_death = |p: &ReplayHeldObjectPeriod, before: i64| {
        if p.xuid != 0
            && let Some(time) = deaths_by_xuid
                .get(&p.xuid)
                .and_then(|times| times.iter().find(|&&t| t >= p.start_ms && t <= before))
        {
            return (*time, true);
        }
        (before, false)
    };
    let mut periods = Vec::<ReplayHeldObjectPeriod>::new();
    let mut open: Option<usize> = None;
    for e in &events {
        if !e.pickup {
            if let Some(i) = open.filter(|&i| periods[i].slot == e.slot) {
                periods[i].end_ms = e.time_ms;
                periods[i].by_death = false;
                periods[i].open = false;
                open = None;
            }
            continue;
        }
        if let Some(i) = open {
            let (time, death) = first_death(&periods[i], e.time_ms);
            periods[i].end_ms = time;
            periods[i].by_death = death;
            periods[i].open = false;
        }
        periods.push(ReplayHeldObjectPeriod {
            slot: e.slot,
            xuid: e.xuid,
            start_ms: e.time_ms,
            end_ms: 0,
            by_death: false,
            open: true,
        });
        open = Some(periods.len() - 1);
    }
    if let Some(i) = open {
        let (time, death) = first_death(&periods[i], HELD_OBJECT_OPEN_END_MS);
        periods[i].end_ms = time;
        periods[i].by_death = death;
        periods[i].open = !death;
    }
    let mut carry_ms_by_xuid = BTreeMap::<u64, i64>::new();
    for p in &periods {
        if p.xuid != 0 && !p.open {
            let total = carry_ms_by_xuid.entry(p.xuid).or_default();
            *total = total.wrapping_add(p.end_ms.wrapping_sub(p.start_ms));
        }
    }
    ReplayHeldObjectCarry {
        events,
        periods,
        carry_ms_by_xuid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::*;
    use std::io::Read;
    #[test]
    fn native_held_object_and_bomb_carries() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-held-object-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let events =
                serde_json::from_value::<Vec<ReplayHeldObjectEvent>>(r["events"].clone()).unwrap();
            let deaths = serde_json::from_value::<Vec<IdentityDeath>>(r["deaths"].clone()).unwrap();
            let occupant = |slot: u32, ms: i64| {
                if slot == 14 {
                    0
                } else {
                    (100 + (i64::from(slot) + ms / 10000) % 4) as u64
                }
            };
            let bridge = r["bridge"].as_bool().unwrap();
            let carry = build_replay_held_object_carry(
                &events,
                if bridge { Some(&occupant) } else { None },
                &deaths,
            );
            assert_eq!(
                carry,
                serde_json::from_value::<ReplayHeldObjectCarry>(r["output"].clone()).unwrap(),
                "held carry {i}"
            );
            let cached_deaths: Vec<_> = deaths
                .iter()
                .map(|d| FactsDeath {
                    xuid: d.xuid,
                    time_ms: d.time_ms,
                    gamertag: vec![0xff, 0, 0x80],
                })
                .collect();
            assert_eq!(
                build_facts_replay_held_object_carry(
                    &events,
                    if bridge { Some(&occupant) } else { None },
                    &cached_deaths,
                ),
                carry,
                "cached held carry {i}"
            );
            let clock: ReplayMatchClock = serde_json::from_value(r["clock"].clone()).unwrap();
            let presence = ReplayCarrierPresence {
                named: BTreeMap::from([
                    (
                        "100".into(),
                        vec![
                            ReplayPresenceSpan { t0: 10, t1: 80 },
                            ReplayPresenceSpan { t0: 100, t1: 180 },
                        ],
                    ),
                    ("101".into(), vec![ReplayPresenceSpan { t0: 200, t1: 500 }]),
                ]),
                unnamed: if r["unknown"].as_bool().unwrap() {
                    vec![ReplayPresenceSpan { t0: 300, t1: 400 }]
                } else {
                    vec![]
                },
            };
            let bombs = build_replay_bomb_carries(&carry, clock, &presence);
            assert_eq!(
                bombs,
                serde_json::from_value::<ReplayBombCarries>(r["bombs"].clone()).unwrap(),
                "bomb carry {i}"
            );
            assert!(bombs.coverage.as_ref().unwrap().balanced());
            let mut change_json = r["changes"].clone();
            // Synthetic native structs omit the classification that filtering never reads.
            for c in change_json.as_array_mut().unwrap() {
                c["Kind"] = serde_json::json!("taken");
            }
            let changes: Vec<HeldWeaponChange> = serde_json::from_value(change_json).unwrap();
            let cached_changes: Vec<_> = changes
                .iter()
                .map(|ch| FactsWeaponChange {
                    timestamp_us: ch.timestamp_us,
                    slot: ch.slot,
                    family: ch.family,
                    previous: ch.previous,
                    slot_index: i64::MIN,
                    kind: vec![0xff, 0, 0x80],
                })
                .collect();
            assert_eq!(
                replay_facts_bomb_held_events(&cached_changes, clock.death_offset_ms),
                serde_json::from_value::<Vec<ReplayHeldObjectEvent>>(r["filtered"].clone())
                    .unwrap(),
                "cached bomb filter {i}"
            );
            assert_eq!(
                replay_bomb_held_events(&changes, clock.death_offset_ms),
                serde_json::from_value::<Vec<ReplayHeldObjectEvent>>(r["filtered"].clone())
                    .unwrap(),
                "bomb filter {i}"
            );
        }
    }
}
