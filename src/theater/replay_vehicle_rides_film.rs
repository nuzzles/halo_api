//! Vehicle rides directly attested by object-parent-state transitions.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

pub fn replay_vehicle_aim_by_slot(aims: &[BipedAim]) -> BTreeMap<u32, Vec<BipedAim>> {
    vehicle_aim_index(aims, |a| (a.slot, a.timestamp_us))
}
/// Stable grouping of complete cached aim records, without synthetic provenance.
pub fn replay_facts_vehicle_aim_by_slot(
    aims: &[FactsVehicleAim],
) -> BTreeMap<u32, Vec<FactsVehicleAim>> {
    vehicle_aim_index(aims, |a| (a.slot, a.timestamp_us))
}
fn vehicle_aim_index<T: Clone>(
    aims: &[T],
    fields: impl Fn(&T) -> (u32, u64),
) -> BTreeMap<u32, Vec<T>> {
    let mut out = BTreeMap::<u32, Vec<T>>::new();
    for a in aims {
        out.entry(fields(a).0).or_default().push(a.clone());
    }
    for v in out.values_mut() {
        v.sort_by_key(|a| fields(a).1);
    }
    out
}
/// Inclusive episode window, first observation per frame, no interpolation.
/// Input is already grouped and stably sorted by timestamp.
pub fn build_replay_vehicle_ride_aim(
    aims: &[BipedAim],
    start_us: u64,
    end_us: u64,
    origin_us: u64,
    step_us: u64,
    frames: i64,
) -> Vec<ReplayVehicleAim> {
    vehicle_ride_aim_values(
        aims,
        |a| (a.timestamp_us, a.yaw_raw, a.pitch_raw),
        start_us,
        end_us,
        origin_us,
        step_us,
        frames,
    )
}
/// Inclusive cache aim projection with native full-width yaw and pitch arithmetic.
pub fn build_facts_replay_vehicle_ride_aim(
    aims: &[FactsVehicleAim],
    start_us: u64,
    end_us: u64,
    origin_us: u64,
    step_us: u64,
    frames: i64,
) -> Vec<ReplayVehicleAim> {
    vehicle_ride_aim_values(
        aims,
        |a| (a.timestamp_us, a.yaw_raw, a.pitch_raw),
        start_us,
        end_us,
        origin_us,
        step_us,
        frames,
    )
}
fn vehicle_ride_aim_values<T>(
    aims: &[T],
    fields: impl Fn(&T) -> (u64, u32, u32),
    start_us: u64,
    end_us: u64,
    origin_us: u64,
    step_us: u64,
    frames: i64,
) -> Vec<ReplayVehicleAim> {
    let Some(step) = NonZeroU64::new(step_us) else {
        return Vec::new();
    };
    if end_us < start_us {
        return Vec::new();
    }
    // Native sort.Search also has a defined outcome for malformed input order.
    let mut lo = 0;
    let mut hi = aims.len();
    while lo < hi {
        let m = lo + (hi - lo) / 2;
        if fields(&aims[m]).0 >= start_us {
            hi = m
        } else {
            lo = m + 1
        }
    }
    let mut out = Vec::new();
    let mut last = -1;
    for a in &aims[lo..] {
        let (time, yaw, pitch) = fields(a);
        if time > end_us {
            break;
        }
        let t = super::replay_vehicle_shots::frame(time, origin_us, step, frames);
        if t <= last {
            continue;
        }
        last = t;
        let h = super::biped_aim::aim_heading_from_raw(yaw);
        let mut h = ((f64::from(h) * 10.).round() / 10.) as f32;
        if h <= 0. {
            h = 360.;
        }
        let p = super::biped_aim::aim_pitch_from_raw(pitch);
        let mut p = ((f64::from(p) * 10.).round() / 10.) as f32;
        if p == 0. {
            p = 0.;
        }
        out.push(ReplayVehicleAim { t, h, p });
    }
    out
}
pub struct ReplayVehicleRideContext<'a, P = ReplayPlayerPosition, A = BipedAim> {
    pub bipeds: &'a [P],
    pub aim_by_slot: &'a BTreeMap<u32, Vec<A>>,
    pub identity: &'a ReplayIdentityState,
    pub occupancy: &'a [VehicleOccupancy],
    pub lives: &'a [ReplayVehicleLife],
    pub drawable: &'a BTreeSet<EquipmentLifeKey>,
    pub origin_us: u64,
    pub step_us: u64,
    pub frames: i64,
}
impl<P, A> Copy for ReplayVehicleRideContext<'_, P, A> {}
impl<P, A> Clone for ReplayVehicleRideContext<'_, P, A> {
    fn clone(&self) -> Self {
        *self
    }
}
/// Cache inputs retain their own record types; no source records are fabricated.
pub type FactsReplayVehicleRideContext<'a> =
    ReplayVehicleRideContext<'a, FactsBipedPosition, FactsVehicleAim>;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReplayFilmVehicleRides {
    pub rides: BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>>,
    occupants: BTreeMap<EquipmentLifeKey, BTreeSet<u32>>,
    windows: BTreeMap<EquipmentLifeKey, Vec<[i64; 2]>>,
}
impl ReplayFilmVehicleRides {
    /// A life with no explicit occupants supplies no contradictory evidence.
    /// Otherwise an unnamed occupant or any inclusive overlap rejects a fallback.
    pub fn contradicts(&self, key: EquipmentLifeKey, ride: &ReplayVehicleRide) -> bool {
        let Some(occupants) = self.occupants.get(&key).filter(|o| !o.is_empty()) else {
            return false;
        };
        !occupants.contains(&ride.slot)
            || self
                .windows
                .get(&key)
                .is_some_and(|w| w.iter().any(|w| ride.t0 <= w[1] && w[0] <= ride.t1))
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFilmVehicleRideTally {
    pub reads: usize,
    pub named: usize,
    pub outside_life: usize,
    pub not_drawable: usize,
    pub published: usize,
    pub closed_by_read: usize,
    pub closed_by_return: usize,
    pub closed_by_life: usize,
}
/// The first life with matching slot and inclusive time bounds wins. The native
/// attribution does not compare the parent generation carried by the transition.
pub fn replay_vehicle_life_at(
    lives: &[ReplayVehicleLife],
    slot: u32,
    at_us: u64,
) -> Option<EquipmentLifeKey> {
    lives
        .iter()
        .find(|l| l.slot == slot && at_us >= l.lo_us && at_us <= l.hi_us)
        .map(ReplayVehicleLife::key)
}
pub fn build_replay_film_vehicle_rides(
    input: ReplayVehicleRideContext<'_>,
) -> (ReplayFilmVehicleRides, ReplayFilmVehicleRideTally) {
    film_vehicle_ride_values(
        input,
        |p| (p.slot, p.timestamp_us, p.has_world),
        build_replay_vehicle_ride_aim,
    )
}
/// Publish written parent-state rides using cached positions and aim readings.
pub fn build_facts_replay_film_vehicle_rides(
    input: FactsReplayVehicleRideContext<'_>,
) -> (ReplayFilmVehicleRides, ReplayFilmVehicleRideTally) {
    film_vehicle_ride_values(
        input,
        |p| (p.slot, p.timestamp_us, p.has_world),
        build_facts_replay_vehicle_ride_aim,
    )
}
fn film_vehicle_ride_values<P, A>(
    input: ReplayVehicleRideContext<'_, P, A>,
    position: impl Fn(&P) -> (u32, u64, bool),
    aim: impl Fn(&[A], u64, u64, u64, u64, i64) -> Vec<ReplayVehicleAim>,
) -> (ReplayFilmVehicleRides, ReplayFilmVehicleRideTally) {
    let mut out = ReplayFilmVehicleRides::default();
    let mut tally = ReplayFilmVehicleRideTally::default();
    let Some(step) = NonZeroU64::new(input.step_us)
        .filter(|_| !input.occupancy.is_empty() && !input.lives.is_empty())
    else {
        return (out, tally);
    };
    let mut occupancy = BTreeMap::<u32, Vec<u64>>::new();
    for o in input.occupancy {
        occupancy.entry(o.slot).or_default().push(o.timestamp_us);
    }
    for v in occupancy.values_mut() {
        v.sort();
    }
    let mut positions = BTreeMap::<u32, Vec<u64>>::new();
    for (slot, time, has_world) in input.bipeds.iter().map(position) {
        if has_world {
            positions.entry(slot).or_default().push(time);
        }
    }
    for v in positions.values_mut() {
        v.sort();
    }
    for o in input.occupancy.iter().filter(|o| o.attached) {
        tally.reads += 1;
        let Some(key) = replay_vehicle_life_at(input.lives, o.parent_slot, o.timestamp_us) else {
            tally.outside_life += 1;
            continue;
        };
        tally.named += 1;
        if !input.drawable.contains(&key) {
            tally.not_drawable += 1;
            continue;
        }
        let mut end = input
            .lives
            .iter()
            .find(|l| l.key() == key)
            .map_or(0, |l| l.hi_us);
        let mut cause = 0;
        for (source, times) in [(1, occupancy.get(&o.slot)), (2, positions.get(&o.slot))] {
            if let Some(times) = times {
                let i = times.partition_point(|t| *t <= o.timestamp_us);
                if let Some(&t) = times.get(i).filter(|t| **t < end) {
                    end = t;
                    cause = source;
                }
            }
        }
        match cause {
            1 => tally.closed_by_read += 1,
            2 => tally.closed_by_return += 1,
            _ => tally.closed_by_life += 1,
        }
        let project =
            |t| super::replay_vehicle_shots::frame(t, input.origin_us, step, input.frames);
        let t0 = project(o.timestamp_us);
        let t1 = project(end).max(t0);
        let xuid = input.identity.xuid_at(o.slot, o.timestamp_us);
        let ride = ReplayVehicleRide {
            t0,
            t1,
            slot: o.slot,
            xuid: if xuid == 0 {
                String::new()
            } else {
                xuid.to_string()
            },
            seat: o.has_seat.then_some(i64::from(o.seat)),
            src: "film".into(),
            aim: aim(
                input.aim_by_slot.get(&o.slot).map_or(&[], Vec::as_slice),
                o.timestamp_us,
                end,
                input.origin_us,
                input.step_us,
                input.frames,
            ),
        };
        out.occupants.entry(key).or_default().insert(o.slot);
        out.windows.entry(key).or_default().push([t0, t1]);
        out.rides.entry(key).or_default().push(ride);
        tally.published += 1;
    }
    for v in out.rides.values_mut() {
        v.sort_by_key(|r| (r.t0, r.slot));
    }
    (out, tally)
}

/// Enrich published episodes with the first matching parent-state seat reading.
/// Input occupancy order is significant; absent evidence preserves the existing seat.
/// The native tolerance is 2000 divided directly by the supplied clock step.
pub fn assign_replay_vehicle_seats(
    rides: &mut BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>>,
    occupancy: &[VehicleOccupancy],
    origin_us: u64,
    step_us: u64,
    frames: i64,
) -> usize {
    let Some(step) = NonZeroU64::new(step_us) else {
        return 0;
    };
    let tolerance = (2000 / step_us) as i64;
    let mut assigned = 0;
    for (key, list) in rides {
        for ride in list {
            if let Some(seat) = occupancy.iter().find(|o| {
                if !o.attached || !o.has_seat || o.slot != ride.slot || o.parent_slot != key.slot {
                    return false;
                }
                let frame =
                    super::replay_vehicle_shots::frame(o.timestamp_us, origin_us, step, frames);
                frame >= ride.t0.wrapping_sub(tolerance) && frame <= ride.t1
            }) {
                ride.seat = Some(i64::from(seat.seat));
                assigned += 1;
            }
        }
    }
    assigned
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn native_explicit_rides_and_aim() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/vehicle-film-rides-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
        let identity = ReplayIdentityState::from_lives(
            vec![
                IdentityLife {
                    slot: 0,
                    from: 0,
                    to: 900000,
                    xuid: 1,
                    ..Default::default()
                },
                IdentityLife {
                    slot: 1,
                    from: 0,
                    to: 300000,
                    xuid: 2,
                    ..Default::default()
                },
                IdentityLife {
                    slot: 1,
                    from: 400000,
                    to: 900000,
                    xuid: 3,
                    ..Default::default()
                },
            ],
            &[(1, 0), (2, 1), (3, 2)].into(),
        );
        for (i, c) in rows.into_iter().enumerate() {
            let lives: Vec<_> = c["lives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| {
                    let key: EquipmentLifeKey = serde_json::from_value(l["key"].clone()).unwrap();
                    ReplayVehicleLife {
                        slot: key.slot,
                        generation: key.generation,
                        lo_us: l["lo"].as_u64().unwrap(),
                        hi_us: l["hi"].as_u64().unwrap(),
                        ..Default::default()
                    }
                })
                .collect();
            let drawable: BTreeSet<EquipmentLifeKey> =
                serde_json::from_value(c["drawable"].clone()).unwrap();
            let occupancy: Vec<VehicleOccupancy> =
                serde_json::from_value(c["occupancy"].clone()).unwrap();
            let aims: Vec<BipedAim> = serde_json::from_value(c["aims"].clone()).unwrap();
            let aim_by_slot = replay_vehicle_aim_by_slot(&aims);
            let positions: Vec<_> = c["positions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| ReplayPlayerPosition {
                    slot: p["slot"].as_u64().unwrap() as u32,
                    timestamp_us: p["time"].as_u64().unwrap(),
                    x: 0.,
                    y: 0.,
                    z: 0.,
                    has_world: p["world"].as_bool().unwrap(),
                })
                .collect();
            let mut episode_positions = BTreeMap::<u32, Vec<ReplayPlayerPosition>>::new();
            for p in &positions {
                episode_positions.entry(p.slot).or_default().push(p.clone());
            }
            for points in episode_positions.values_mut() {
                points.sort_by_key(|p| p.timestamp_us);
            }
            let episodes = build_replay_vehicle_event_episodes(
                &serde_json::from_value(c["boards"].clone()).unwrap(),
                &serde_json::from_value(c["exits"].clone()).unwrap(),
                &episode_positions,
            );
            let expected_episodes: Vec<ReplayVehicleEpisode> =
                serde_json::from_value(c["episodes"].clone()).unwrap();
            assert_eq!(episodes, expected_episodes, "episodes {i}");
            for probe in c["cover_probes"].as_array().unwrap() {
                assert_eq!(
                    replay_vehicle_episode_covers(
                        &episodes,
                        probe["slot"].as_u64().unwrap() as u32,
                        probe["start"].as_u64().unwrap(),
                        probe["end"].as_u64().unwrap()
                    ),
                    probe["covers"].as_bool().unwrap(),
                    "episode coverage {i}"
                );
            }
            let origin = c["origin"].as_u64().unwrap();
            let step = c["step"].as_u64().unwrap();
            let frames = c["frames"].as_i64().unwrap();
            let read_rides = |value: &Value| -> BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>> {
                value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        (
                            serde_json::from_value(v["key"].clone()).unwrap(),
                            serde_json::from_value(v["rides"].clone()).unwrap(),
                        )
                    })
                    .collect()
            };
            let combined = &c["combined"];
            let combined_lives: Vec<_> = combined["lives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| ReplayVehicleLife {
                    slot: l["slot"].as_u64().unwrap() as u32,
                    generation: l["generation"].as_u64().unwrap() as u32,
                    lo_us: l["lo_us"].as_u64().unwrap(),
                    hi_us: l["hi_us"].as_u64().unwrap(),
                    ..Default::default()
                })
                .collect();
            let combined_positions: Vec<ReplayPlayerPosition> =
                serde_json::from_value(combined["positions"].clone()).unwrap();
            let combined_vehicles: Vec<ReplayPlayerPosition> =
                serde_json::from_value(combined["vehicles"].clone()).unwrap();
            let vehicle_positions: Vec<_> = combined_vehicles
                .into_iter()
                .map(|position| ReplayVehiclePosition {
                    position,
                    companions: Default::default(),
                })
                .collect();
            let events: Vec<VehicleEvent> =
                serde_json::from_value(combined["events"].clone()).unwrap();
            let (combined_rides, combined_tally) =
                build_replay_vehicle_rides(ReplayVehicleRidesContext {
                    film: ReplayVehicleRideContext {
                        bipeds: &combined_positions,
                        aim_by_slot: &aim_by_slot,
                        identity: &identity,
                        occupancy: &occupancy,
                        lives: &combined_lives,
                        drawable: &drawable,
                        origin_us: combined["origin"].as_u64().unwrap(),
                        step_us: combined["step"].as_u64().unwrap(),
                        frames: combined["frames"].as_i64().unwrap(),
                    },
                    events: &events,
                    vehicles: &replay_vehicle_positions_by_slot(&vehicle_positions),
                });
            assert_eq!(
                combined_rides,
                read_rides(&combined["rides"]),
                "combined rides {i}"
            );
            assert_eq!(
                json!(combined_tally),
                combined["tally"],
                "combined tally {i}"
            );
            let ride_logs = super::super::log_test_support::capture_logs(|| combined_tally.log());
            assert_eq!(json!(ride_logs), combined["ride_logs"], "ride logs {i}");
            let publication = &c["publication"];
            let census: WorldObjectKeyframes =
                serde_json::from_value(publication["census"].clone()).unwrap();
            let creations: Vec<EquipmentCreation> =
                serde_json::from_value(publication["creations"].clone()).unwrap();
            let deaths: Vec<ReplayVehicleDeathEvidence> =
                serde_json::from_value(publication["deaths"].clone()).unwrap();
            let output = build_replay_vehicle_publication(
                ReplayVehicleScan {
                    scanned: publication["scanned"].as_bool().unwrap(),
                    keyframes: &census,
                    creations: &creations,
                    positions: &vehicle_positions,
                    events: &events,
                    aims: &aims,
                    deaths: &deaths,
                    occupancy: &occupancy,
                    march_default_retained: publication["default_retained"].as_bool().unwrap(),
                },
                &combined_positions,
                &identity,
                combined["origin"].as_u64().unwrap(),
                combined["step"].as_u64().unwrap(),
                combined["frames"].as_i64().unwrap(),
            );
            let expected: ReplayVehiclePublication =
                serde_json::from_value(publication["output"].clone()).unwrap();
            assert_eq!(output, expected, "publication {i}");
            let mut seats = read_rides(&c["seat_input"]);
            assert_eq!(
                assign_replay_vehicle_seats(&mut seats, &occupancy, origin, step, frames),
                c["seat_count"].as_u64().unwrap() as usize,
                "seat count {i}"
            );
            assert_eq!(seats, read_rides(&c["seat_output"]), "seats {i}");

            let (out, tally) = build_replay_film_vehicle_rides(ReplayVehicleRideContext {
                bipeds: &positions,
                aim_by_slot: &aim_by_slot,
                identity: &identity,
                occupancy: &occupancy,
                lives: &lives,
                drawable: &drawable,
                origin_us: origin,
                step_us: step,
                frames,
            });
            let expected: BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>> = c["rides"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    (
                        serde_json::from_value(v["key"].clone()).unwrap(),
                        serde_json::from_value(v["rides"].clone()).unwrap(),
                    )
                })
                .collect();
            assert_eq!(out.rides, expected, "rides {i}");
            assert_eq!(json!(tally), c["tally"], "tally {i}");
            for p in c["probes"].as_array().unwrap() {
                let key = serde_json::from_value(p["key"].clone()).unwrap();
                let ride = serde_json::from_value(p["ride"].clone()).unwrap();
                assert_eq!(
                    out.contradicts(key, &ride),
                    p["contradicts"].as_bool().unwrap(),
                    "contradicts {i}"
                );
            }
            for p in c["aim_probes"].as_array().unwrap() {
                let aims: Vec<BipedAim> = serde_json::from_value(p["input"].clone()).unwrap();
                let expected: Vec<ReplayVehicleAim> =
                    serde_json::from_value(p["aim"].clone()).unwrap();
                assert_eq!(
                    build_replay_vehicle_ride_aim(
                        &aims,
                        p["start"].as_u64().unwrap(),
                        p["end"].as_u64().unwrap(),
                        origin,
                        step,
                        frames
                    ),
                    expected,
                    "aim {i}"
                );
            }
        }
    }
}
