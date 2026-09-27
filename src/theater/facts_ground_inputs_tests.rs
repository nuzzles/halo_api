use super::*;
use serde::Deserialize;
use std::io::Read;
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Creation {
    slot: u32,
    #[serde(rename = "Gen")]
    generation: u32,
    #[serde(rename = "TimestampUS")]
    time: u64,
    #[serde(rename = "MPPPresent")]
    present: [bool; 4],
    #[serde(rename = "MPPVal")]
    values: [u64; 4],
    x: f32,
    y: f32,
    z: f32,
    has_ammo: bool,
    ammo: GroundWeaponAmmo,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Point {
    #[serde(rename = "TimestampUS")]
    time: u64,
    chunk: i64,
    x: f32,
    y: f32,
    z: f32,
    at_rest: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Track {
    slot: u32,
    #[serde(rename = "Gen")]
    generation: u32,
    pts: Option<Vec<Point>>,
}
#[derive(Deserialize)]
struct Case {
    creations: Vec<Creation>,
    tracks: Vec<Track>,
    keyframes: WorldObjectKeyframes,
    positions: Vec<ReplayPlayerPosition>,
    objects: Vec<GroundPickupObject>,
    distance_bits: Vec<u64>,
    rejected: GroundObjectRejections,
}
#[test]
fn native_facts_ground_object_assembly() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-ground-objects-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    let families = [
        (1, "family".into()),
        (2, "family".into()),
        (3, "family".into()),
    ]
    .into();
    let objectives = [2].into();
    for (i, c) in cases.into_iter().enumerate() {
        let creations: Vec<_> = c
            .creations
            .into_iter()
            .map(|c| FactsEquipmentCreation {
                slot: c.slot,
                generation: c.generation,
                timestamp_us: c.time,
                position: [c.x, c.y, c.z],
                mpp_present: c.present,
                mpp_val: c.values,
                has_ammo: c.has_ammo,
                ammo: c.ammo,
                ..Default::default()
            })
            .collect();
        let tracks: Vec<_> = c
            .tracks
            .into_iter()
            .map(|t| FactsProjectileTrack {
                slot: t.slot,
                generation: t.generation,
                points: t.pts.map(|points| {
                    points
                        .into_iter()
                        .map(|p| FactsProjectileSample {
                            timestamp_us: p.time,
                            chunk: p.chunk,
                            position: [p.x, p.y, p.z],
                            at_rest: p.at_rest,
                        })
                        .collect()
                }),
            })
            .collect();
        let got = assemble_facts_ground_objects(
            &creations,
            &tracks,
            &FactsWorldKeyframes::from(&c.keyframes),
            &c.positions,
            GroundObjectRule {
                kind: "weapon",
                families: &families,
                objectives: &objectives,
            },
        );
        assert_eq!(got.rejected, c.rejected, "rejections {i}");
        assert_eq!(got.objects, c.objects, "objects {i}");
        assert_eq!(
            got.objects
                .iter()
                .map(|o| o.picker.distance_m.to_bits())
                .collect::<Vec<_>>(),
            c.distance_bits,
            "distance bits {i}"
        );
    }
}

#[test]
fn native_facts_ground_lifetime_resolution() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/ground-lifetimes-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    for (i, r) in rows.into_iter().enumerate() {
        let tracks: Vec<WorldObjectTrack> = serde_json::from_value(r["tracks"].clone()).unwrap();
        let tracks: Vec<_> = tracks.iter().map(FactsProjectileTrack::from).collect();
        let positions: Vec<ReplayPlayerPosition> =
            serde_json::from_value(r["positions"].clone()).unwrap();
        let times: Vec<u64> = serde_json::from_value(r["times"].clone()).unwrap();
        let seen: Vec<u64> = serde_json::from_value(r["seen"].clone()).unwrap();
        let creation = FactsEquipmentCreation {
            timestamp_us: r["birth"].as_u64().unwrap(),
            position: serde_json::from_value(r["pos"].clone()).unwrap(),
            ..Default::default()
        };
        let got = resolve_facts_ground_weapon_pickup(
            &creation,
            r["end"].as_u64().unwrap(),
            r["film_end"].as_u64().unwrap(),
            FactsGroundWeaponResolveInputs {
                keyframe_times: &times,
                seen: &seen,
                tracks: &tracks,
                positions: &positions,
            },
        );
        let mut expected: GroundWeaponPickupResolution =
            serde_json::from_value(r["resolution"].clone()).unwrap();
        expected.picker.distance_m = f64::from_bits(r["picker_bits"].as_u64().unwrap());
        assert_eq!(got, expected, "lifetime {i}");
    }
}
#[derive(Deserialize)]
struct ClockCase {
    objects: Vec<GroundPickupObject>,
    clock: GroundPadClock,
    layer: GroundPadLayer,
}
#[test]
fn native_ground_pad_signed_clock_domain() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/ground-pad-clock-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<ClockCase> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        assert_eq!(
            build_ground_pad_layer(&c.objects, c.clock).unwrap(),
            c.layer,
            "clock {i}"
        );
    }
}

#[derive(Deserialize)]
struct PadScan {
    scanned: bool,
    stats: FactsCreationStats,
    creations: Vec<Creation>,
    tracks: Vec<Track>,
    keyframes: WorldObjectKeyframes,
}
impl PadScan {
    fn facts(self) -> FactsWorldObjectScan {
        FactsWorldObjectScan {
            scanned: self.scanned,
            stats: self.stats,
            keyframes: FactsWorldKeyframes::from(&self.keyframes),
            creations: self
                .creations
                .into_iter()
                .map(|c| FactsEquipmentCreation {
                    slot: c.slot,
                    generation: c.generation,
                    timestamp_us: c.time,
                    position: [c.x, c.y, c.z],
                    mpp_present: c.present,
                    mpp_val: c.values,
                    has_ammo: c.has_ammo,
                    ammo: c.ammo,
                    ..Default::default()
                })
                .collect(),
            tracks: self
                .tracks
                .into_iter()
                .map(|t| FactsProjectileTrack {
                    slot: t.slot,
                    generation: t.generation,
                    points: t.pts.map(|pts| {
                        pts.into_iter()
                            .map(|p| FactsProjectileSample {
                                timestamp_us: p.time,
                                chunk: p.chunk,
                                position: [p.x, p.y, p.z],
                                at_rest: p.at_rest,
                            })
                            .collect()
                    }),
                })
                .collect(),
        }
    }
}
#[derive(Deserialize)]
struct PadScanCase {
    weapons: PadScan,
    powerups: PadScan,
    positions: Vec<ReplayPlayerPosition>,
    families: std::collections::BTreeMap<u32, String>,
    objectives: std::collections::BTreeSet<u32>,
    clock: GroundPadClock,
    output: ReplayGroundPads,
}
#[test]
fn native_facts_pad_scans_and_signed_coverage() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-pad-scans-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<PadScanCase> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let positions: Vec<_> = c
            .positions
            .into_iter()
            .map(|p| FactsBipedPosition {
                timestamp_us: p.timestamp_us,
                slot: p.slot,
                has_world: p.has_world,
                world: [p.x, p.y, p.z],
                ..Default::default()
            })
            .collect();
        let catalog = ReplayEquipmentCatalog {
            families: c.families,
            objective_objects: c.objectives,
            ..Default::default()
        };
        let got = build_facts_replay_ground_pads(
            &c.weapons.facts(),
            &c.powerups.facts(),
            &positions,
            c.clock,
            &catalog,
        );
        assert_eq!(got, c.output, "pad scan {i}");
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GroundChange {
    slot: u32,
    time: u64,
    family: u32,
    kind_hex: String,
}
#[derive(Deserialize)]
struct GroundPublication {
    clock: IdentityClock,
    output: ReplayGroundWeapons,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GroundItemsCase {
    objects: Vec<GroundPickupObject>,
    cache_changes: Vec<GroundChange>,
    positions: Vec<ReplayPlayerPosition>,
    publications: Vec<GroundPublication>,
}
#[test]
fn native_facts_ground_items_and_kind_gates() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-ground-items-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<GroundItemsCase> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let changes: Vec<_> = c
            .cache_changes
            .into_iter()
            .map(|c| FactsWeaponChange {
                timestamp_us: c.time,
                slot: c.slot,
                family: c.family,
                kind: c
                    .kind_hex
                    .as_bytes()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                    .collect(),
                ..Default::default()
            })
            .collect();
        for (j, p) in c.publications.into_iter().enumerate() {
            assert_eq!(
                build_facts_replay_ground_weapons(&c.objects, &changes, &c.positions, p.clock),
                p.output,
                "items {i}/{j}"
            );
        }
    }
}

#[derive(Deserialize)]
struct ObjectivePublication {
    scanned: bool,
    clock: IdentityClock,
    output: ReplayObjectiveObjects,
}
#[derive(Deserialize)]
struct ObjectiveCase {
    creations: Vec<Creation>,
    tracks: Vec<Track>,
    objective_labels: std::collections::BTreeMap<u32, ReplayLabel>,
    objective_families: std::collections::BTreeMap<u32, String>,
    free_lives: Vec<FreeObjectiveLife>,
    objective_publications: Vec<ObjectivePublication>,
}
#[test]
fn native_facts_free_objectives_and_publication() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-ground-objects-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<ObjectiveCase> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let creations: Vec<_> = c
            .creations
            .into_iter()
            .map(|c| FactsEquipmentCreation {
                slot: c.slot,
                generation: c.generation,
                timestamp_us: c.time,
                position: [c.x, c.y, c.z],
                mpp_present: c.present,
                mpp_val: c.values,
                has_ammo: c.has_ammo,
                ammo: c.ammo,
                ..Default::default()
            })
            .collect();
        let tracks: Vec<_> = c
            .tracks
            .into_iter()
            .map(|t| FactsProjectileTrack {
                slot: t.slot,
                generation: t.generation,
                points: t.pts.map(|ps| {
                    ps.into_iter()
                        .map(|p| FactsProjectileSample {
                            timestamp_us: p.time,
                            chunk: p.chunk,
                            position: [p.x, p.y, p.z],
                            at_rest: p.at_rest,
                        })
                        .collect()
                }),
            })
            .collect();
        assert_eq!(
            free_facts_objective_lives(true, &creations, &tracks, &c.objective_labels),
            c.free_lives,
            "free objective lives {i}"
        );
        assert!(
            free_facts_objective_lives(false, &creations, &tracks, &c.objective_labels).is_empty(),
            "unscanned {i}"
        );
        assert_eq!(c.objective_publications.len(), 5);
        for (j, p) in c.objective_publications.into_iter().enumerate() {
            assert_eq!(
                build_facts_replay_objective_objects(
                    FactsReplayObjectiveInput {
                        scanned: p.scanned,
                        creations: &creations,
                        tracks: &tracks,
                        labels: &c.objective_labels,
                        families: &c.objective_families
                    },
                    p.clock
                ),
                p.output,
                "objective publication {i}/{j}"
            );
        }
    }
}

#[test]
fn native_facts_flag_markers_and_carries() {
    use std::collections::BTreeMap;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/replay-flag-geometry-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let get = |name: &str| c[name].clone();
        let marks: FactsCarrierMarkScan = serde_json::from_value(get("marks")).unwrap();
        let bridge: BTreeMap<u32, u64> = serde_json::from_value(get("bridge")).unwrap();
        let clock: ReplayMatchClock = serde_json::from_value(get("clock")).unwrap();
        let mut assigned: Vec<ReplayFlagCarryRaw> =
            serde_json::from_value(get("assigned")).unwrap();
        mark_facts_replay_flag_carries(&mut assigned, &marks, &bridge, clock.death_offset_ms);
        assert_eq!(
            assigned,
            serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("marked")).unwrap(),
            "marker confirmation {i}"
        );
        let tracks: Vec<ReplayTrack> = serde_json::from_value(get("tracks")).unwrap();
        let ambiguous: BTreeMap<u32, bool> = serde_json::from_value(get("ambiguous")).unwrap();
        let spawns: Vec<ReplayFlagSpawn> =
            serde_json::from_value(get("assignment_spawns")).unwrap();
        let teams: BTreeMap<String, i64> = serde_json::from_value(get("assignment_teams")).unwrap();
        let lives: Vec<FreeObjectiveLife> = serde_json::from_value(get("lives")).unwrap();
        let events: Vec<StatborgNamedEvent> = serde_json::from_value(get("full_events")).unwrap();
        let by_slot: BTreeMap<i64, String> = serde_json::from_value(get("full_identity")).unwrap();
        let identity = StatborgRoundIdentity {
            publication: IdentityStatborgPublication {
                by_round: BTreeMap::from([(0, by_slot)]),
                ..Default::default()
            },
            starts: Vec::new(),
        };
        for (free, prefix) in [(&lives[..], "full"), (&[][..], "empty_free")] {
            let got = build_facts_replay_flags(
                FactsReplayFlagScan {
                    scanned: c["full_scanned"].as_bool().unwrap(),
                    signals: serde_json::from_value(get("full_signals")).unwrap(),
                    events: &events,
                    identity: &identity,
                    teams: &teams,
                    marks: &marks,
                    spawns: &spawns,
                    free,
                },
                ReplayFlagContext {
                    clock,
                    tracks: &tracks,
                    deaths: &[],
                    bridge: &bridge,
                    ambiguous_slots: &ambiguous,
                },
            );
            assert_eq!(
                got.carries,
                serde_json::from_value::<Vec<ReplayFlagCarry>>(get(&format!("{prefix}_carries")))
                    .unwrap(),
                "carries {i}/{prefix}"
            );
            assert_eq!(
                got.coverage,
                serde_json::from_value(get(&format!("{prefix}_cov"))).unwrap(),
                "coverage {i}/{prefix}"
            );
            assert_eq!(
                got.tracks_without_bridge as u64,
                c[format!("{prefix}_no_bridge")].as_u64().unwrap(),
                "bridge fallback {i}/{prefix}"
            );
            assert_eq!(
                got.drops_using_pickup as u64,
                c[format!("{prefix}_drop_pickup")].as_u64().unwrap(),
                "pickup fallback {i}/{prefix}"
            );
        }
    }
}
