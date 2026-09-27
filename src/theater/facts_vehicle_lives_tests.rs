use super::*;
use serde::Deserialize;
use std::{io::Read, num::NonZeroU64};
#[derive(Deserialize)]
struct Death {
    #[serde(flatten)]
    evidence: ReplayVehicleDeathEvidence,
    type_index: u32,
}
#[derive(Deserialize)]
struct Probe {
    life: usize,
    origin: u64,
    step: u64,
    frames: i64,
    spawn: u64,
    last: u64,
    bounds: [i64; 3],
    end: String,
    t_end: Option<i64>,
}
#[derive(Deserialize)]
struct Case {
    census: WorldObjectKeyframes,
    deaths: Vec<Death>,
    lives: Vec<ReplayVehicleLife>,
    tally: ReplayVehicleDeathTally,
    probes: Vec<Probe>,
}
#[test]
fn native_facts_vehicle_census_deaths_and_bounds() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-lives-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let census = FactsWorldKeyframes::from(&c.census);
        let deaths: Vec<_> = c
            .deaths
            .into_iter()
            .map(|d| ObjectDeath {
                slot: d.evidence.slot,
                r#gen: d.evidence.generation,
                timestamp_us: d.evidence.timestamp_us,
                tail_desync: d.evidence.tail_desync,
                type_index: d.type_index,
                dead: ObjectDeadState {
                    mort: false,
                    enum_a: 0,
                    enum_b: 0,
                    val_0c: 0,
                    val_0e: 0,
                    has_ref: false,
                    gid_present: false,
                    global_id: 0,
                    val14: 0,
                    val18: 0,
                    src_tag0: 0,
                    src_tag4c: 0,
                },
            })
            .collect();
        let (lives, tally) = build_facts_replay_vehicle_lives(&census, &deaths);
        assert_eq!(lives, c.lives, "lives {i}");
        assert_eq!(tally, c.tally, "tally {i}");
        let (mut assign, _) = build_facts_replay_vehicle_lives(&census, &[]);
        assert_eq!(
            assign_facts_replay_vehicle_deaths(&mut assign, &deaths),
            c.tally,
            "assignment tally {i}"
        );
        assert_eq!(assign, c.lives, "assigned lives {i}");
        for p in c.probes {
            let step = NonZeroU64::new(p.step).unwrap();
            assert_eq!(
                lives[p.life].bounds(p.spawn, p.last, p.origin, step, p.frames),
                p.bounds,
                "bounds {i}/{}",
                p.life
            );
            assert_eq!(
                lives[p.life].end(p.origin, step, p.frames),
                (p.end.as_str(), p.t_end),
                "end {i}/{}",
                p.life
            );
        }
    }
}

#[derive(Deserialize)]
struct RawPosition {
    #[serde(flatten)]
    position: ReplayPlayerPosition,
    #[serde(flatten)]
    directions: FactsPositionDirections,
}
#[derive(Deserialize)]
struct PositionWrapper {
    position: RawPosition,
}
impl PositionWrapper {
    fn facts(self) -> FactsBipedPosition {
        let p = self.position;
        FactsBipedPosition {
            timestamp_us: p.position.timestamp_us,
            slot: p.position.slot,
            has_world: p.position.has_world,
            world: [p.position.x, p.position.y, p.position.z],
            directions: p.directions,
            ..Default::default()
        }
    }
}
#[derive(Deserialize)]
struct SampleCase {
    inputs: Vec<PositionWrapper>,
    grouped: Vec<PositionWrapper>,
    headings: Vec<Option<f32>>,
    life: ReplayVehicleLife,
    origin: u64,
    step: u64,
    frames: i64,
    samples: Vec<ReplayVehicleSample>,
    last: u64,
}
#[derive(Deserialize)]
struct SampleFixture {
    cases: Vec<SampleCase>,
}
#[test]
fn native_facts_vehicle_heading_grouping_and_samples() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-samples-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixture: SampleFixture = serde_json::from_slice(&raw).unwrap();
    assert_eq!(fixture.cases.len(), 1024);
    for (i, c) in fixture.cases.into_iter().enumerate() {
        let inputs: Vec<_> = c.inputs.into_iter().map(PositionWrapper::facts).collect();
        let grouped = replay_facts_vehicle_positions_by_slot(&inputs);
        let positions = grouped.get(&2).map_or(&[][..], Vec::as_slice);
        let expected: Vec<_> = c.grouped.into_iter().map(PositionWrapper::facts).collect();
        assert_eq!(positions, expected, "grouping {i}");
        assert_eq!(
            positions
                .iter()
                .map(replay_facts_vehicle_heading)
                .collect::<Vec<_>>(),
            c.headings,
            "headings {i}"
        );
        assert_eq!(
            build_facts_replay_vehicle_samples(
                positions,
                &c.life,
                c.origin,
                NonZeroU64::new(c.step).unwrap(),
                c.frames
            ),
            (c.samples, c.last),
            "samples {i}"
        );
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrackCase {
    #[serde(flatten)]
    sample: SampleCase,
    spawn_wire: String,
    input_wire: String,
    chosen_wire: String,
    rides: Vec<ReplayVehicleRide>,
    track: Option<ReplayVehicleTrack>,
    drawable: bool,
}
#[derive(Deserialize)]
struct TrackFixture {
    cases: Vec<TrackCase>,
}
fn creation_wire(hex: &str) -> Vec<FactsEquipmentCreation> {
    let bytes: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    let mut reader = NativeFactsReader::new(&bytes);
    let out = decode_facts_creations(&mut reader);
    assert!(reader.error().is_none());
    assert_eq!(reader.offset(), bytes.len());
    out
}
#[test]
fn native_facts_vehicle_spawn_drawable_and_track() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-tracks-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixture: TrackFixture = serde_json::from_slice(&raw).unwrap();
    assert_eq!(fixture.cases.len(), 1024);
    for (i, c) in fixture.cases.into_iter().enumerate() {
        let input = creation_wire(&c.input_wire);
        let spawns = replay_facts_vehicle_spawns_by_life(&input);
        assert_eq!(
            spawns.values().cloned().collect::<Vec<_>>(),
            creation_wire(&c.chosen_wire),
            "complete selected records {i}"
        );
        let positions: Vec<_> = c
            .sample
            .inputs
            .into_iter()
            .map(PositionWrapper::facts)
            .collect();
        let grouped = replay_facts_vehicle_positions_by_slot(&positions);
        let life = c.sample.life;
        let drawable =
            replay_facts_vehicle_drawable_lives(std::slice::from_ref(&life), &spawns, &grouped);
        assert_eq!(drawable.contains(&life.key()), c.drawable, "drawable {i}");
        let spawn = creation_wire(&c.spawn_wire);
        let got = build_facts_replay_vehicle_track(
            &life,
            spawn.first(),
            grouped.get(&2).map_or(&[][..], Vec::as_slice),
            c.rides,
            c.sample.origin,
            NonZeroU64::new(c.sample.step).unwrap(),
            c.sample.frames,
        );
        assert_eq!(got, c.track, "track {i}");
    }
}

#[derive(Deserialize)]
struct AimProbe {
    input: Vec<FactsVehicleAim>,
    start: u64,
    end: u64,
    aim: Vec<ReplayVehicleAim>,
}
#[derive(Deserialize)]
struct AimCase {
    input: Vec<FactsVehicleAim>,
    grouped: std::collections::BTreeMap<u32, Vec<FactsVehicleAim>>,
    origin: u64,
    step: u64,
    frames: i64,
    probes: Vec<AimProbe>,
}
#[test]
fn native_facts_vehicle_rider_aim() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-vehicle-aim-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<AimCase> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        assert_eq!(
            replay_facts_vehicle_aim_by_slot(&c.input),
            c.grouped,
            "grouping {i}"
        );
        assert_eq!(c.probes.len(), 12);
        for (j, p) in c.probes.into_iter().enumerate() {
            assert_eq!(
                build_facts_replay_vehicle_ride_aim(
                    &p.input, p.start, p.end, c.origin, c.step, c.frames
                ),
                p.aim,
                "aim {i}/{j}"
            );
        }
    }
}

#[test]
fn native_facts_vehicle_written_rides() {
    use std::collections::{BTreeMap, BTreeSet};
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-written-rides-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    let identity = ReplayIdentityState::from_lives(
        [(0, 0, 900000, 1), (1, 0, 300000, 2), (1, 400000, 900000, 3)]
            .into_iter()
            .map(|(slot, from, to, xuid)| IdentityLife {
                slot,
                from,
                to,
                xuid,
                ..Default::default()
            })
            .collect(),
        &[(1, 0), (2, 1), (3, 2)].into(),
    );
    for (i, c) in cases.into_iter().enumerate() {
        let lives: Vec<_> = c["lives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                let key: EquipmentLifeKey = serde_json::from_value(v["key"].clone()).unwrap();
                ReplayVehicleLife {
                    slot: key.slot,
                    generation: key.generation,
                    lo_us: v["lo"].as_u64().unwrap(),
                    hi_us: v["hi"].as_u64().unwrap(),
                    ..Default::default()
                }
            })
            .collect();
        let drawable: BTreeSet<EquipmentLifeKey> =
            serde_json::from_value(c["drawable"].clone()).unwrap();
        let occupancy: Vec<VehicleOccupancy> =
            serde_json::from_value(c["occupancy"].clone()).unwrap();
        let aims: Vec<FactsVehicleAim> = serde_json::from_value(c["aims"].clone()).unwrap();
        let aims = replay_facts_vehicle_aim_by_slot(&aims);
        let bipeds: Vec<_> = c["positions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| FactsBipedPosition {
                slot: p["slot"].as_u64().unwrap().try_into().unwrap(),
                timestamp_us: p["time"].as_u64().unwrap(),
                has_world: p["world"].as_bool().unwrap(),
                ..Default::default()
            })
            .collect();
        let (out, tally) = build_facts_replay_film_vehicle_rides(FactsReplayVehicleRideContext {
            bipeds: &bipeds,
            aim_by_slot: &aims,
            identity: &identity,
            occupancy: &occupancy,
            lives: &lives,
            drawable: &drawable,
            origin_us: c["origin"].as_u64().unwrap(),
            step_us: c["step"].as_u64().unwrap(),
            frames: c["frames"].as_i64().unwrap(),
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
        assert_eq!(out.rides, expected, "written rides {i}");
        assert_eq!(
            serde_json::to_value(tally).unwrap(),
            c["tally"],
            "tally {i}"
        );
        for (j, p) in c["probes"].as_array().unwrap().iter().enumerate() {
            assert_eq!(
                out.contradicts(
                    serde_json::from_value(p["key"].clone()).unwrap(),
                    &serde_json::from_value(p["ride"].clone()).unwrap()
                ),
                p["contradicts"].as_bool().unwrap(),
                "contradiction {i}/{j}"
            );
        }
    }
}

#[derive(Deserialize)]
struct RideList {
    key: EquipmentLifeKey,
    rides: Vec<ReplayVehicleRide>,
}
fn ride_map(
    rows: Vec<RideList>,
) -> std::collections::BTreeMap<EquipmentLifeKey, Vec<ReplayVehicleRide>> {
    rows.into_iter().map(|r| (r.key, r.rides)).collect()
}
fn ride_position(p: ReplayPlayerPosition) -> FactsBipedPosition {
    FactsBipedPosition {
        slot: p.slot,
        timestamp_us: p.timestamp_us,
        world: [p.x, p.y, p.z],
        has_world: p.has_world,
        ..Default::default()
    }
}
#[derive(Deserialize)]
struct RideLife {
    slot: u32,
    generation: u32,
    lo_us: u64,
    hi_us: u64,
}
#[derive(Deserialize)]
struct CombinedRideCase {
    origin: u64,
    step: u64,
    frames: i64,
    lives: Vec<RideLife>,
    positions: Vec<ReplayPlayerPosition>,
    vehicles: Vec<ReplayPlayerPosition>,
    events: Vec<FactsVehicleEvent>,
    rides: Vec<RideList>,
    tally: ReplayVehicleRideTally,
}
#[derive(Deserialize)]
struct EpisodePosition {
    slot: u32,
    time: u64,
    world: bool,
}
#[derive(Deserialize)]
struct EpisodeCover {
    slot: u32,
    start: u64,
    end: u64,
    covers: bool,
}
#[derive(Deserialize)]
struct CombinedRideFixture {
    combined: CombinedRideCase,
    boards: std::collections::BTreeMap<u32, Vec<FactsVehicleEvent>>,
    exits: std::collections::BTreeMap<u32, Vec<FactsVehicleEvent>>,
    merged: std::collections::BTreeMap<u32, Vec<FactsVehicleEvent>>,
    episodes: Vec<ReplayVehicleEpisode>,
    cover_probes: Vec<EpisodeCover>,
    positions: Vec<EpisodePosition>,
    drawable: std::collections::BTreeSet<EquipmentLifeKey>,
    occupancy: Vec<VehicleOccupancy>,
    aims: Vec<FactsVehicleAim>,
    seat_input: Vec<RideList>,
    seat_output: Vec<RideList>,
    seat_count: usize,
    origin: u64,
    step: u64,
    frames: i64,
}
#[test]
fn native_facts_vehicle_combined_rides() {
    use std::collections::BTreeMap;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-rides-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<CombinedRideFixture> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    let identity = ReplayIdentityState::from_lives(
        [(0, 0, 900000, 1), (1, 0, 300000, 2), (1, 400000, 900000, 3)]
            .into_iter()
            .map(|(slot, from, to, xuid)| IdentityLife {
                slot,
                from,
                to,
                xuid,
                ..Default::default()
            })
            .collect(),
        &[(1, 0), (2, 1), (3, 2)].into(),
    );
    for (i, c) in cases.into_iter().enumerate() {
        let mut points = BTreeMap::<u32, Vec<ReplayPlayerPosition>>::new();
        for p in c.positions {
            points
                .entry(p.slot)
                .or_default()
                .push(ReplayPlayerPosition {
                    slot: p.slot,
                    timestamp_us: p.time,
                    has_world: p.world,
                    x: 0.,
                    y: 0.,
                    z: 0.,
                });
        }
        for v in points.values_mut() {
            v.sort_by_key(|p| p.timestamp_us);
        }
        let episodes = build_facts_replay_vehicle_event_episodes(&c.boards, &c.exits, &points);
        assert_eq!(episodes, c.episodes, "episodes {i}");
        for (slot, expected) in c.merged {
            let merged = merge_facts_replay_vehicle_events(
                c.boards.get(&slot).map_or(&[], Vec::as_slice),
                c.exits.get(&slot).map_or(&[], Vec::as_slice),
            );
            assert_eq!(merged, expected, "complete merge {i}/{slot}");
            let per_slot: Vec<_> = episodes
                .iter()
                .filter(|e| e.slot == slot)
                .cloned()
                .collect();
            assert_eq!(
                replay_facts_vehicle_episodes_of_occupant(
                    slot,
                    &merged,
                    points.get(&slot).map_or(&[], Vec::as_slice)
                ),
                per_slot,
                "occupant episodes {i}/{slot}"
            );
        }
        for p in c.cover_probes {
            assert_eq!(
                replay_vehicle_episode_covers(&episodes, p.slot, p.start, p.end),
                p.covers,
                "coverage {i}"
            );
        }
        let mut seats = ride_map(c.seat_input);
        assert_eq!(
            assign_replay_vehicle_seats(&mut seats, &c.occupancy, c.origin, c.step, c.frames),
            c.seat_count,
            "seat count {i}"
        );
        assert_eq!(seats, ride_map(c.seat_output), "seats {i}");
        let v = c.combined;
        let lives: Vec<_> = v
            .lives
            .into_iter()
            .map(|l| ReplayVehicleLife {
                slot: l.slot,
                generation: l.generation,
                lo_us: l.lo_us,
                hi_us: l.hi_us,
                ..Default::default()
            })
            .collect();
        let bipeds: Vec<_> = v.positions.into_iter().map(ride_position).collect();
        let vehicles: Vec<_> = v.vehicles.into_iter().map(ride_position).collect();
        let vehicles = replay_facts_vehicle_positions_by_slot(&vehicles);
        let aims = replay_facts_vehicle_aim_by_slot(&c.aims);
        let (rides, tally) = build_facts_replay_vehicle_rides(FactsReplayVehicleRidesContext {
            film: FactsReplayVehicleRideContext {
                bipeds: &bipeds,
                aim_by_slot: &aims,
                identity: &identity,
                occupancy: &c.occupancy,
                lives: &lives,
                drawable: &c.drawable,
                origin_us: v.origin,
                step_us: v.step,
                frames: v.frames,
            },
            events: &v.events,
            vehicles: &vehicles,
        });
        assert_eq!(rides, ride_map(v.rides), "combined rides {i}");
        assert_eq!(tally, v.tally, "combined tally {i}");
    }
}

#[test]
fn native_facts_vehicle_publication() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-publication-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    let identity = ReplayIdentityState::from_lives(
        [(0, 0, 900000, 1), (1, 0, 300000, 2), (1, 400000, 900000, 3)]
            .into_iter()
            .map(|(slot, from, to, xuid)| IdentityLife {
                slot,
                from,
                to,
                xuid,
                ..Default::default()
            })
            .collect(),
        &[(1, 0), (2, 1), (3, 2)].into(),
    );
    for (i, c) in cases.into_iter().enumerate() {
        let wire = c["scan_wire"].as_str().unwrap();
        let bytes: Vec<_> = wire
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect();
        let mut reader = NativeFactsReader::new(&bytes);
        let scan = decode_facts_vehicle_scan(
            &mut reader,
            &I0Layout {
                axis_widths: [8, 8, 8],
                gate_bits: 0,
                region: 0,
            },
            [[0.; 3], [128.; 3]],
        );
        assert!(reader.error().is_none(), "scan {i}: {:?}", reader.error());
        assert_eq!(reader.offset(), bytes.len(), "scan boundary {i}");
        let v = &c["combined"];
        let bipeds: Vec<ReplayPlayerPosition> =
            serde_json::from_value(v["positions"].clone()).unwrap();
        let bipeds: Vec<_> = bipeds.into_iter().map(ride_position).collect();
        let got = build_facts_replay_vehicle_publication(
            &scan,
            &bipeds,
            &identity,
            v["origin"].as_u64().unwrap(),
            v["step"].as_u64().unwrap(),
            v["frames"].as_i64().unwrap(),
        );
        let expected: ReplayVehiclePublication =
            serde_json::from_value(c["publication"]["output"].clone()).unwrap();
        assert_eq!(got, expected, "full cache publication {i}");
    }
}

#[test]
fn native_facts_vehicle_heading_sources() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-tracks-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.iter().enumerate() {
        let inputs: Vec<PositionWrapper> = serde_json::from_value(c["inputs"].clone()).unwrap();
        let inputs: Vec<_> = inputs.into_iter().map(PositionWrapper::facts).collect();
        let expected: ReplayVehicleHeadingSources =
            serde_json::from_value(c["heading_log"].clone()).unwrap();
        assert_eq!(
            replay_facts_vehicle_heading_sources(&inputs),
            expected,
            "heading provenance {i}"
        );
    }
}
