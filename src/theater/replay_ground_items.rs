//! Individual moving ground weapons, ending at a matched pickup or census bound.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayGroundAmmo {
    pub mag: u32,
    pub res: u32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayGroundWeapon {
    pub t0: i64,
    pub t1: i64,
    pub t1max: i64,
    pub x: f32,
    pub y: f32,
    #[serde(default, skip_serializing_if = "ground_zero_f")]
    pub z: f32,
    pub w: String,
    pub origin: String,
    pub dropper: i64,
    pub end: String,
    pub picker: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ammo: Option<ReplayGroundAmmo>,
}
fn ground_zero_f(v: &f32) -> bool {
    *v == 0.
}
fn ground_zero(v: &usize) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayGroundWeaponCoverage {
    pub objects: usize,
    pub published: usize,
    pub at_rest: usize,
    pub dropper_named: usize,
    pub takes_total: usize,
    pub pickup_linked: usize,
    pub end_pickup: usize,
    pub end_seen: usize,
    pub end_open: usize,
    #[serde(default, skip_serializing_if = "ground_zero")]
    pub ammo_read: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayGroundWeapons {
    pub weapons: Vec<ReplayGroundWeapon>,
    pub coverage: ReplayGroundWeaponCoverage,
}
fn actor_at(positions: &[&ReplayPlayerPosition], time: u64) -> Option<[f32; 3]> {
    let i = positions.partition_point(|p| p.timestamp_us < time);
    let mut best = None;
    let mut gap = 250_001;
    for j in [i.checked_sub(1), Some(i)].into_iter().flatten() {
        if let Some(p) = positions.get(j) {
            let d = p.timestamp_us.abs_diff(time);
            if d < gap {
                best = Some([p.x, p.y, p.z]);
                gap = d;
            }
        }
    }
    best
}
fn frame(time: u64, clock: IdentityClock) -> i64 {
    if time < clock.origin_us {
        return 0;
    }
    let t = ((time - clock.origin_us) / clock.step_us) as i64;
    if t < 0 {
        0
    } else if t >= clock.frame_count {
        clock.frame_count.wrapping_sub(1)
    } else {
        t
    }
}
/// Positions must be sorted by time. Taken/swapped events consume one matching
/// family object, within its observed lifetime and strictly closer than 1.5m.
/// Equal actor-time gaps favor the preceding sample; equal object distances favor
/// input order. At-rest objects remain in the pad layer instead.
pub fn build_replay_ground_weapons(
    objects: &[GroundPickupObject],
    changes: &[HeldWeaponChange],
    positions: &[ReplayPlayerPosition],
    clock: IdentityClock,
) -> ReplayGroundWeapons {
    build_ground_item_values(
        objects,
        changes
            .iter()
            .filter(|c| {
                matches!(
                    c.kind,
                    HeldWeaponChangeKind::Taken | HeldWeaponChangeKind::Swapped
                )
            })
            .map(|c| GroundTake {
                slot: c.slot,
                timestamp_us: c.timestamp_us,
                family: c.family,
            }),
        positions,
        clock,
    )
}
/// Cache matching only consumes exactly recorded taken/swapped kinds.
/// Unknown kinds remain in the cache and do not acquire pickup semantics.
pub fn build_facts_replay_ground_weapons(
    objects: &[GroundPickupObject],
    changes: &[FactsWeaponChange],
    positions: &[ReplayPlayerPosition],
    clock: IdentityClock,
) -> ReplayGroundWeapons {
    build_ground_item_values(
        objects,
        changes
            .iter()
            .filter(|c| matches!(c.kind.as_slice(), b"taken" | b"swapped"))
            .map(|c| GroundTake {
                slot: c.slot,
                timestamp_us: c.timestamp_us,
                family: c.family,
            }),
        positions,
        clock,
    )
}
struct GroundTake {
    slot: u32,
    timestamp_us: u64,
    family: u32,
}
fn build_ground_item_values(
    objects: &[GroundPickupObject],
    changes: impl Iterator<Item = GroundTake>,
    positions: &[ReplayPlayerPosition],
    clock: IdentityClock,
) -> ReplayGroundWeapons {
    let mut out = ReplayGroundWeapons {
        coverage: ReplayGroundWeaponCoverage {
            objects: objects.len(),
            ..Default::default()
        },
        ..Default::default()
    };
    if objects.is_empty() || clock.step_us == 0 {
        return out;
    }
    let moving: Vec<_> = objects.iter().filter(|o| o.appearance.has_delta).collect();
    out.coverage.at_rest = objects.len() - moving.len();
    let mut by_slot = BTreeMap::<u32, Vec<&ReplayPlayerPosition>>::new();
    for p in positions.iter().filter(|p| p.has_world) {
        by_slot.entry(p.slot).or_default().push(p);
    }
    let mut takes: Vec<_> = changes.collect();
    takes.sort_by_key(|c| c.timestamp_us);
    out.coverage.takes_total = takes.len();
    let mut picks = vec![None; moving.len()];
    for c in takes {
        if c.family == u32::MAX {
            continue;
        }
        let Some(actor) = actor_at(
            by_slot.get(&c.slot).map_or(&[][..], Vec::as_slice),
            c.timestamp_us,
        ) else {
            continue;
        };
        let mut best = None;
        let mut distance = 1.5;
        for (i, o) in moving.iter().enumerate() {
            if picks[i].is_some()
                || o.family_id != c.family
                || c.timestamp_us < o.appearance.timestamp_us
                || c.timestamp_us > o.bounds.high_us
            {
                continue;
            }
            let dx = f64::from(actor[0] - o.position[0]);
            let dy = f64::from(actor[1] - o.position[1]);
            let dz = f64::from(actor[2] - o.position[2]);
            let d = dz.mul_add(dz, dx.mul_add(dx, dy * dy)).sqrt();
            if d < distance {
                best = Some(i);
                distance = d;
            }
        }
        if let Some(i) = best {
            picks[i] = Some((c.slot, c.timestamp_us));
            out.coverage.pickup_linked += 1;
        }
    }
    for (o, pick) in moving.into_iter().zip(picks) {
        let t0 = frame(o.appearance.timestamp_us, clock);
        let (t1, t1max, end, picker) = if let Some((slot, time)) = pick {
            out.coverage.end_pickup += 1;
            let t = frame(time, clock);
            (t, t, "pickup", i64::from(slot))
        } else if o.bounds.never_picked || o.bounds.no_later_kf {
            out.coverage.end_open += 1;
            let t = clock.frame_count.wrapping_sub(1);
            (t, t, "open", -1)
        } else {
            out.coverage.end_seen += 1;
            (
                frame(o.bounds.low_us, clock),
                frame(o.bounds.high_us, clock),
                "seen",
                -1,
            )
        };
        let t1 = t1.max(t0);
        let t1max = t1max.max(t1);
        out.coverage.dropper_named += usize::from(o.dropper_slot >= 0);
        out.coverage.ammo_read += usize::from(o.has_ammo);
        out.weapons.push(ReplayGroundWeapon {
            t0,
            t1,
            t1max,
            x: o.position[0],
            y: o.position[1],
            z: o.position[2],
            w: format!("{:08x}", o.family_id),
            origin: o.appearance.class.clone(),
            dropper: o.dropper_slot,
            end: end.into(),
            picker,
            ammo: o.has_ammo.then_some(ReplayGroundAmmo {
                mag: o.ammo.mag,
                res: o.ammo.res,
            }),
        });
    }
    out.coverage.published = out.weapons.len();
    super::native_sort::sort_by(&mut out.weapons, |a, b| (a.t0, &a.w).cmp(&(b.t0, &b.w)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Publication {
        clock: IdentityClock,
        output: ReplayGroundWeapons,
    }
    #[derive(Deserialize)]
    struct Case {
        objects: Vec<GroundPickupObject>,
        positions: Vec<ReplayPlayerPosition>,
        changes: Vec<HeldWeaponChange>,
        publications: Vec<Publication>,
    }
    #[test]
    fn native_ground_weapon_publication() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/ground-objects-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            for (j, p) in c.publications.into_iter().enumerate() {
                assert_eq!(
                    build_replay_ground_weapons(&c.objects, &c.changes, &c.positions, p.clock),
                    p.output,
                    "case {i}/{j}"
                );
            }
        }
    }
}
