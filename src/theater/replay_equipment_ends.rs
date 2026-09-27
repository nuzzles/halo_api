//! Census bounds for equipment display, separate from the last observed movement.
use super::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEquipmentEnd {
    pub until: i64,
    pub until_max: i64,
    pub end: String,
}
fn frame(time: u64, origin: u64, step: u64) -> i64 {
    if time >= origin {
        ((time - origin) / step) as i64
    } else {
        (((origin - time).wrapping_add(step).wrapping_sub(1) / step) as i64).wrapping_neg()
    }
}
fn clamp(t: i64, frames: i64) -> i64 {
    if t < 0 {
        0
    } else if t >= frames {
        frames.wrapping_sub(1)
    } else {
        t
    }
}
/// Return one end per input placement, preserving its index through later filters.
/// A reused slot/generation closes at the next placement of that key. The final
/// census timestamp is included; an unbounded disappearance remains open.
pub fn replay_equipment_ends(
    raw: &[EquipmentPlacement],
    census: &WorldObjectKeyframes,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> Vec<ReplayEquipmentEnd> {
    equipment_end_values(
        raw.iter().map(|p| (p.life, p.t0_us)),
        &census.times_us,
        census
            .seen_us
            .iter()
            .map(|s| {
                (
                    EquipmentLifeKey {
                        slot: s.slot,
                        generation: s.generation,
                    },
                    s.times_us.as_slice(),
                )
            })
            .collect(),
        origin,
        step,
        frames,
    )
}
/// Census disappearance bounds for exact cache lifetimes, in input placement order.
pub fn replay_facts_equipment_ends(
    raw: &[FactsPlacement],
    census: &FactsWorldKeyframes,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> Vec<ReplayEquipmentEnd> {
    equipment_end_values(
        raw.iter().map(|p| {
            (
                EquipmentLifeKey {
                    slot: p.life.slot,
                    generation: p.life.generation,
                },
                p.start_us,
            )
        }),
        &census.times_us,
        census
            .seen_us
            .iter()
            .map(|(&(slot, generation), times)| {
                (EquipmentLifeKey { slot, generation }, times.as_slice())
            })
            .collect(),
        origin,
        step,
        frames,
    )
}
fn equipment_end_values(
    raw: impl ExactSizeIterator<Item = (EquipmentLifeKey, u64)>,
    times: &[u64],
    seen: BTreeMap<EquipmentLifeKey, &[u64]>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> Vec<ReplayEquipmentEnd> {
    let count = raw.len();
    let mut groups = BTreeMap::<EquipmentLifeKey, Vec<(usize, u64)>>::new();
    for (i, (life, timestamp)) in raw.enumerate() {
        groups.entry(life).or_default().push((i, timestamp));
    }
    let film_end = times.last().copied().unwrap_or(0).wrapping_add(1);
    let mut out = vec![
        ReplayEquipmentEnd {
            until: 0,
            until_max: 0,
            end: String::new()
        };
        count
    ];
    for (life, mut group) in groups {
        super::native_sort::sort_by(&mut group, |a, b| a.1.cmp(&b.1));
        for (j, &(i, birth)) in group.iter().enumerate() {
            let life_end = group.get(j + 1).map_or(film_end, |p| p.1);
            let seen =
                ground_weapon_seen_within(seen.get(&life).copied().unwrap_or(&[]), birth, life_end);
            let b = ground_weapon_pickup_bounds(birth, life_end, film_end, times, seen);
            out[i] = if b.never_picked || b.no_later_kf {
                ReplayEquipmentEnd {
                    until: frames.wrapping_sub(1),
                    until_max: frames.wrapping_sub(1),
                    end: "open".into(),
                }
            } else {
                let until = clamp(frame(b.low_us, origin, step.get()), frames);
                let until_max = clamp(frame(b.high_us, origin, step.get()), frames).max(until);
                ReplayEquipmentEnd {
                    until,
                    until_max,
                    end: "seen".into(),
                }
            };
        }
    }
    out
}
