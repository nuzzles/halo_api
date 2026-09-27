//! Publish one native vehicle life from its creation, position cloud and rides.
use super::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehiclePosition {
    pub position: ReplayPlayerPosition,
    pub companions: BipedCompanions,
}
impl From<&BipedPositionCandidate> for ReplayVehiclePosition {
    fn from(p: &BipedPositionCandidate) -> Self {
        let [x, y, z] = p.record.world;
        Self {
            position: ReplayPlayerPosition {
                slot: p.record.slot,
                timestamp_us: p.source.timestamp_us,
                x,
                y,
                z,
                has_world: true,
            },
            companions: p.record.companions.clone(),
        }
    }
}
/// Ignore positions without world coordinates, then stably sort each slot by time.
pub fn replay_vehicle_positions_by_slot(
    positions: &[ReplayVehiclePosition],
) -> BTreeMap<u32, Vec<ReplayVehiclePosition>> {
    let mut out = BTreeMap::<u32, Vec<ReplayVehiclePosition>>::new();
    for p in positions.iter().filter(|p| p.position.has_world) {
        out.entry(p.position.slot).or_default().push(p.clone());
    }
    for ps in out.values_mut() {
        ps.sort_by_key(|p| p.position.timestamp_us);
    }
    out
}
/// Keep the earliest creation of each life; equal-time re-announcements retain
/// the first record, even when other creation fields differ.
pub fn replay_vehicle_spawns_by_life(
    creations: &[EquipmentCreation],
) -> BTreeMap<EquipmentLifeKey, EquipmentCreation> {
    vehicle_spawn_index(creations, |c| {
        (
            EquipmentLifeKey {
                slot: c.slot,
                generation: c.generation,
            },
            c.timestamp_us,
        )
    })
}
/// Retain the complete earliest cache creation, including signed metadata and masks.
pub fn replay_facts_vehicle_spawns_by_life(
    creations: &[FactsEquipmentCreation],
) -> BTreeMap<EquipmentLifeKey, FactsEquipmentCreation> {
    vehicle_spawn_index(creations, |c| {
        (
            EquipmentLifeKey {
                slot: c.slot,
                generation: c.generation,
            },
            c.timestamp_us,
        )
    })
}
fn vehicle_spawn_index<T: Clone>(
    creations: &[T],
    fields: impl Fn(&T) -> (EquipmentLifeKey, u64),
) -> BTreeMap<EquipmentLifeKey, T> {
    let mut out = BTreeMap::new();
    for c in creations {
        let (key, time) = fields(c);
        if out.get(&key).is_some_and(|p| fields(p).1 <= time) {
            continue;
        }
        out.insert(key, c.clone());
    }
    out
}
/// Position groups must already be sorted by timestamp. Duplicate keys follow
/// the last supplied life, including when that life is not drawable.
pub fn replay_vehicle_drawable_lives(
    lives: &[ReplayVehicleLife],
    spawns: &BTreeMap<EquipmentLifeKey, EquipmentCreation>,
    by_slot: &BTreeMap<u32, Vec<ReplayVehiclePosition>>,
) -> std::collections::BTreeSet<EquipmentLifeKey> {
    vehicle_drawable_values(
        lives,
        |key| spawns.get(&key).is_some_and(|s| s.timestamp_us > 0),
        |life| {
            by_slot
                .get(&life.slot)
                .is_some_and(|ps| vehicle_position_in_window(ps, life, |p| p.position.timestamp_us))
        },
    )
}
pub fn replay_facts_vehicle_drawable_lives(
    lives: &[ReplayVehicleLife],
    spawns: &BTreeMap<EquipmentLifeKey, FactsEquipmentCreation>,
    by_slot: &BTreeMap<u32, Vec<FactsBipedPosition>>,
) -> std::collections::BTreeSet<EquipmentLifeKey> {
    vehicle_drawable_values(
        lives,
        |key| spawns.get(&key).is_some_and(|s| s.timestamp_us > 0),
        |life| {
            by_slot
                .get(&life.slot)
                .is_some_and(|ps| vehicle_position_in_window(ps, life, |p| p.timestamp_us))
        },
    )
}
fn vehicle_position_in_window<T>(
    points: &[T],
    life: &ReplayVehicleLife,
    time: impl Fn(&T) -> u64,
) -> bool {
    let mut lo = 0;
    let mut hi = points.len();
    while lo < hi {
        let m = lo + (hi - lo) / 2;
        if time(&points[m]) >= life.lo_us {
            hi = m;
        } else {
            lo = m + 1;
        }
    }
    points.get(lo).is_some_and(|p| time(p) <= life.hi_us)
}
fn vehicle_drawable_values(
    lives: &[ReplayVehicleLife],
    has_spawn: impl Fn(EquipmentLifeKey) -> bool,
    has_position: impl Fn(&ReplayVehicleLife) -> bool,
) -> std::collections::BTreeSet<EquipmentLifeKey> {
    let mut out = std::collections::BTreeSet::new();
    for life in lives {
        let key = life.key();
        if has_spawn(key) || has_position(life) {
            out.insert(key);
        } else {
            out.remove(&key);
        }
    }
    out
}
/// Prefer the proven config-mode chassis direction, then velocity at >=5m/s.
/// Invalid packed directions refuse the velocity fallback, including its sentinel.
pub fn replay_vehicle_heading(companions: &BipedCompanions) -> Option<f32> {
    vehicle_heading_value(
        companions
            .chassis
            .as_ref()
            .and_then(NativeChassisOrientation::film_heading_degrees),
        companions.velocity,
    )
}
/// Preserve cache presence flags and native saturation of velocity magnitude codes.
pub fn replay_facts_vehicle_heading(position: &FactsBipedPosition) -> Option<f32> {
    let d = &position.directions;
    vehicle_heading_value(
        native_film_chassis_heading(
            d.has_aim.then_some(d.aim_raw),
            d.has_roll.then_some(d.roll_raw),
            d.fwd_mode,
            d.aim_default,
        ),
        d.has_vel.then_some([d.vel_raw, d.vel_scale]),
    )
}
fn vehicle_heading_value(film: Option<f32>, velocity: Option<[u32; 2]>) -> Option<f32> {
    if film.is_some() {
        return film;
    }
    let [code, scale] = velocity?;
    decode_native_direction(code, 19)?;
    let v = decode_native_velocity(u64::from(code), u64::from(scale));
    if f64::from(v[0]).hypot(f64::from(v[1])) < 5. {
        return None;
    }
    let mut h = f64::from(v[1]).atan2(f64::from(v[0])) * 180. / std::f64::consts::PI;
    if h < 0. {
        h += 360.;
    }
    Some(h as f32)
}
/// Provenance counts over all supplied readings, before filtering or decimation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVehicleHeadingSources {
    #[serde(rename = "echantillons")]
    pub samples: i64,
    #[serde(rename = "capDuFilm")]
    pub film: i64,
    #[serde(rename = "capParVelocite")]
    pub velocity: i64,
    #[serde(rename = "sansCap")]
    pub none: i64,
    #[serde(rename = "roulisLuMaisModeNonPublie")]
    pub unpublished_mode: i64,
}
impl ReplayVehicleHeadingSources {
    pub fn log(&self) {
        tracing::info!(
            echantillons = self.samples,
            capDuFilm = self.film,
            capParVelocite = self.velocity,
            sansCap = self.none,
            roulisLuMaisModeNonPublie = self.unpublished_mode,
            "rejeu : source du cap des vehicules"
        );
    }
}
fn vehicle_heading_sources(
    readings: impl Iterator<Item = (bool, bool, bool)>,
) -> ReplayVehicleHeadingSources {
    let mut out = ReplayVehicleHeadingSources::default();
    for (film, heading, roll) in readings {
        out.samples += 1;
        if film {
            out.film += 1;
        } else if heading {
            out.velocity += 1;
            out.unpublished_mode += i64::from(roll);
        } else {
            out.none += 1;
        }
    }
    out
}
/// Count cache observations before world filtering and frame decimation.
pub fn replay_facts_vehicle_heading_sources(
    positions: &[FactsBipedPosition],
) -> ReplayVehicleHeadingSources {
    vehicle_heading_sources(positions.iter().map(|p| {
        let d = &p.directions;
        (
            native_film_chassis_heading(
                d.has_aim.then_some(d.aim_raw),
                d.has_roll.then_some(d.roll_raw),
                d.fwd_mode,
                d.aim_default,
            )
            .is_some(),
            replay_facts_vehicle_heading(p).is_some(),
            d.has_roll,
        )
    }))
}
/// Count heading provenance before frame decimation, including missing headings.
pub(crate) fn log_replay_vehicle_heading_source<'a>(
    companions: impl IntoIterator<Item = &'a BipedCompanions>,
) {
    vehicle_heading_sources(companions.into_iter().map(|c| {
        (
            c.chassis
                .as_ref()
                .and_then(NativeChassisOrientation::film_heading_degrees)
                .is_some(),
            replay_vehicle_heading(c).is_some(),
            c.chassis.as_ref().is_some_and(|c| c.roll.is_some()),
        )
    }))
    .log();
}

fn round(v: f32, scale: f64) -> f32 {
    ((f64::from(v) * scale).round() / scale) as f32
}
/// One sample per frame within the inclusive life window. Heading and last-seen
/// time update even for readings skipped by frame decimation.
pub fn build_replay_vehicle_samples(
    positions: &[ReplayVehiclePosition],
    life: &ReplayVehicleLife,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> (Vec<ReplayVehicleSample>, u64) {
    vehicle_sample_values(
        positions.iter().map(|p| {
            (
                p.position.timestamp_us,
                [p.position.x, p.position.y, p.position.z],
                replay_vehicle_heading(&p.companions),
            )
        }),
        life,
        origin,
        step,
        frames,
    )
}
/// World-position filtering and stable slot grouping, retaining complete cache rows.
pub fn replay_facts_vehicle_positions_by_slot(
    positions: &[FactsBipedPosition],
) -> BTreeMap<u32, Vec<FactsBipedPosition>> {
    let mut out = BTreeMap::<u32, Vec<FactsBipedPosition>>::new();
    for p in positions.iter().filter(|p| p.has_world) {
        out.entry(p.slot).or_default().push(p.clone());
    }
    for points in out.values_mut() {
        points.sort_by_key(|p| p.timestamp_us);
    }
    out
}
/// Positions are already grouped and sorted by the preceding native stage.
pub fn build_facts_replay_vehicle_samples(
    positions: &[FactsBipedPosition],
    life: &ReplayVehicleLife,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> (Vec<ReplayVehicleSample>, u64) {
    vehicle_sample_values(
        positions
            .iter()
            .map(|p| (p.timestamp_us, p.world, replay_facts_vehicle_heading(p))),
        life,
        origin,
        step,
        frames,
    )
}
fn vehicle_sample_values(
    positions: impl Iterator<Item = (u64, [f32; 3], Option<f32>)>,
    life: &ReplayVehicleLife,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> (Vec<ReplayVehicleSample>, u64) {
    let mut out = Vec::new();
    let mut last_frame = -1_i64;
    let mut last_seen = 0;
    let mut heading = None;
    for (time, world, current_heading) in positions {
        if time < life.lo_us || time > life.hi_us {
            continue;
        }
        if let Some(h) = current_heading {
            heading = Some(h);
        }
        last_seen = time;
        let t = super::replay_vehicle_shots::frame(time, origin, step, frames);
        if last_frame >= 0 && t.wrapping_sub(last_frame) < 1 {
            continue;
        }
        last_frame = t;
        let h = heading
            .map(|h| {
                let h = round(h, 10.);
                if h <= 0. { 360. } else { h }
            })
            .unwrap_or(0.);
        out.push(ReplayVehicleSample {
            t,
            x: round(world[0], 100.),
            y: round(world[1], 100.),
            z: round(world[2], 100.),
            h,
        });
    }
    (out, last_seen)
}
/// Assemble a life after its position grouping and ride attribution. No spawn and
/// no samples means no published track. Spawn timestamp zero means absent in Go.
pub fn build_replay_vehicle_track(
    life: &ReplayVehicleLife,
    spawn: Option<&EquipmentCreation>,
    positions: &[ReplayVehiclePosition],
    rides: Vec<ReplayVehicleRide>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> Option<ReplayVehicleTrack> {
    publish_vehicle_track(
        life,
        spawn.map(|s| VehicleTrackSpawn {
            timestamp_us: s.timestamp_us,
            world: [s.x, s.y, s.z],
            chassis: s.mpp_present[1].then_some(s.mpp_val[1] as u32),
        }),
        build_replay_vehicle_samples(positions, life, origin, step, frames),
        rides,
        origin,
        step,
        frames,
    )
}
pub fn build_facts_replay_vehicle_track(
    life: &ReplayVehicleLife,
    spawn: Option<&FactsEquipmentCreation>,
    positions: &[FactsBipedPosition],
    rides: Vec<ReplayVehicleRide>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> Option<ReplayVehicleTrack> {
    publish_vehicle_track(
        life,
        spawn.map(|s| VehicleTrackSpawn {
            timestamp_us: s.timestamp_us,
            world: s.position,
            chassis: s.mpp_present[1].then_some(s.mpp_val[1] as u32),
        }),
        build_facts_replay_vehicle_samples(positions, life, origin, step, frames),
        rides,
        origin,
        step,
        frames,
    )
}
struct VehicleTrackSpawn {
    timestamp_us: u64,
    world: [f32; 3],
    chassis: Option<u32>,
}
fn publish_vehicle_track(
    life: &ReplayVehicleLife,
    spawn: Option<VehicleTrackSpawn>,
    samples: (Vec<ReplayVehicleSample>, u64),
    rides: Vec<ReplayVehicleRide>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) -> Option<ReplayVehicleTrack> {
    let (samples, last) = samples;
    let spawn = spawn.filter(|s| s.timestamp_us > 0);
    if spawn.is_none() && samples.is_empty() {
        return None;
    }
    let (end, t_end) = life.end(origin, step, frames);
    let [t0, t1, t1_max] = life.bounds(
        spawn.as_ref().map_or(0, |s| s.timestamp_us),
        last,
        origin,
        step,
        frames,
    );
    let mut track = ReplayVehicleTrack {
        slot: life.slot,
        r#gen: life.generation,
        t0,
        t1,
        t1_max,
        end: end.into(),
        t_end,
        samples,
        ..Default::default()
    };
    if let Some(s) = spawn {
        track.spawn = Some(ReplayVehicleSpawn {
            x: round(s.world[0], 100.),
            y: round(s.world[1], 100.),
            z: round(s.world[2], 100.),
            h: None,
        });
        if let Some(chassis) = s.chassis {
            track.chassis = format!("{chassis:08x}");
            track.family = replay_vehicle_family(chassis).into();
        }
    }
    if replay_vehicle_family_is_rideable(&track.family) {
        track.rides = clamp_replay_vehicle_rides(rides, t0, t1_max);
    }
    Some(track)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Family {
        family: String,
        rideable: bool,
        formatted: String,
    }
    #[derive(Deserialize)]
    struct Oracle {
        cases: Vec<Case>,
        families: BTreeMap<u32, Family>,
    }
    #[derive(Deserialize)]
    struct Case {
        #[serde(rename = "spawnInputs")]
        spawn_inputs: Vec<EquipmentCreation>,
        spawns: Vec<EquipmentCreation>,
        drawable: bool,
        coverage: ReplayVehicleCoverage,
        cycle_tracks: Vec<ReplayVehicleTrack>,
        cycle_step: u64,
        cycles: Vec<ReplayVehicleCycle>,
        cycle_coverage: ReplayVehicleCoverage,
        life: ReplayVehicleLife,
        spawn: EquipmentCreation,
        inputs: Vec<ReplayVehiclePosition>,
        grouped: Vec<ReplayVehiclePosition>,
        headings: Vec<Option<f32>>,
        heading_log: serde_json::Value,
        rides: Vec<ReplayVehicleRide>,
        origin: u64,
        step: u64,
        frames: i64,
        samples: Vec<ReplayVehicleSample>,
        last: u64,
        track: Option<ReplayVehicleTrack>,
    }
    #[test]
    fn native_vehicle_track_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-vehicle-tracks-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let oracle: Oracle = serde_json::from_slice(&raw).unwrap();
        for (id, f) in oracle.families {
            assert_eq!(replay_vehicle_family(id), f.family);
            assert_eq!(replay_vehicle_family_is_rideable(&f.family), f.rideable);
            assert_eq!(format!("{id:08x}"), f.formatted);
        }
        for (i, c) in oracle.cases.into_iter().enumerate() {
            assert_eq!(
                replay_vehicle_spawns_by_life(&c.spawn_inputs)
                    .into_values()
                    .collect::<Vec<_>>(),
                c.spawns,
                "spawns {i}"
            );
            assert_eq!(
                replay_vehicle_drawable_lives(
                    std::slice::from_ref(&c.life),
                    &replay_vehicle_spawns_by_life(&c.spawn_inputs),
                    &replay_vehicle_positions_by_slot(&c.inputs)
                )
                .contains(&c.life.key()),
                c.drawable,
                "drawable {i}"
            );
            let heading_log = super::super::log_test_support::capture_log(|| {
                log_replay_vehicle_heading_source(c.inputs.iter().map(|p| &p.companions));
            });
            assert_eq!(heading_log, c.heading_log, "heading source {i}");
            let positions = replay_vehicle_positions_by_slot(&c.inputs)
                .remove(&2)
                .unwrap_or_default();
            assert_eq!(positions, c.grouped, "grouping {i}");
            for (p, h) in positions.iter().zip(c.headings) {
                assert_eq!(replay_vehicle_heading(&p.companions), h, "heading {i}");
            }
            let step = NonZeroU64::new(c.step).unwrap();
            let (samples, last) =
                build_replay_vehicle_samples(&positions, &c.life, c.origin, step, c.frames);
            assert_eq!(samples, c.samples, "samples {i}");
            assert_eq!(last, c.last, "last {i}");
            let track = build_replay_vehicle_track(
                &c.life,
                Some(&c.spawn),
                &positions,
                c.rides,
                c.origin,
                step,
                c.frames,
            );
            assert_eq!(track, c.track, "track {i}");
            let mut coverage = ReplayVehicleCoverage {
                scanned: true,
                ..Default::default()
            };
            tally_replay_vehicle_coverage(&track.into_iter().collect::<Vec<_>>(), &mut coverage);
            assert_eq!(coverage, c.coverage, "coverage {i}");
            let mut cycle_coverage = ReplayVehicleCoverage::default();
            let cycles =
                build_replay_vehicle_cycles(&c.cycle_tracks, c.cycle_step, &mut cycle_coverage);
            assert_eq!(cycles, c.cycles, "cycles {i}");
            assert_eq!(cycle_coverage, c.cycle_coverage, "cycle coverage {i}");
        }
    }
}
