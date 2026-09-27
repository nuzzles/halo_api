//! Captured checks of native scan outputs before replay publication/filtering.
use super::*;
use serde_json::{Value, json};

const CHECKED_INPUTS: [&str; 44] = [
    "FilmMajorVersion",
    "Fire",
    "Loadouts",
    "BipedCreations",
    "PlayerTeams",
    "TeamScan",
    "FilmTable",
    "WeaponChanges",
    "MovementStates",
    "MovementStateStats",
    "Grenades",
    "AbilityChargeStats",
    "AbilityCharges",
    "AbilityImpulseStats",
    "AbilityImpulses",
    "GrappleReads",
    "CamoStates",
    "AbilityRanks",
    "PickupStats",
    "Pickups",
    "Inventory",
    "InventoryDeltas",
    "InventoryDeltaAmmoRefused",
    "EquipmentChanges",
    "EquipmentChangeStats",
    "SpawnEvents",
    "SpawnStats",
    "FilmClockOriginUS",
    "ZoomEvents",
    "Positions",
    "Deaths",
    "PlayerIndices",
    "Translocations",
    "Projectiles",
    "Placements",
    "PlacementStats",
    "Pads",
    "Vehicles",
    "FlagMarks",
    "FlagGauge",
    "FlagGaugeScanned",
    "ZoneReads",
    "ZoneScanned",
    "BombReads",
];

pub(super) fn read_selected_inputs(path: &std::path::Path) -> Value {
    struct Inputs;
    impl<'de> serde::de::Visitor<'de> for Inputs {
        type Value = Value;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("complete native FilmInputs object")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
            let mut selected = serde_json::Map::new();
            let mut keys = std::collections::BTreeSet::new();
            while let Some(key) = map.next_key::<String>()? {
                assert!(keys.insert(key.clone()), "duplicate native input key");
                if CHECKED_INPUTS.contains(&key.as_str()) {
                    selected.insert(key, map.next_value()?);
                } else {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
            assert_eq!(keys.len(), 44, "review native FilmInputs schema change");
            assert_eq!(selected.len(), CHECKED_INPUTS.len());
            Ok(Value::Object(selected))
        }
    }
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(flate2::read::ZlibDecoder::new(file));
    let mut decoder = serde_json::Deserializer::from_reader(reader);
    let value = serde::de::Deserializer::deserialize_map(&mut decoder, Inputs).unwrap();
    decoder.end().unwrap();
    value
}

// Normalize native nil collections only where Rust has an actual empty collection.
// No records, keys, field values or ordering are removed from either side.
pub(super) fn normalize_empty(expected: &mut Value, actual: &Value) {
    match (expected, actual) {
        (e @ Value::Null, Value::Array(a)) if a.is_empty() => *e = json!([]),
        (e @ Value::Null, Value::Object(a)) if a.is_empty() => *e = json!({}),
        (Value::Array(e), Value::Array(a)) => {
            for (e, a) in e.iter_mut().zip(a) {
                normalize_empty(e, a);
            }
        }
        (Value::Object(e), Value::Object(a)) => {
            for (key, value) in e {
                if let Some(actual) = a.get(key) {
                    normalize_empty(value, actual);
                }
            }
        }
        _ => {}
    }
}

pub(super) fn first_difference(actual: &Value, expected: &Value, path: &str) -> String {
    match (actual, expected) {
        (Value::Array(a), Value::Array(e)) => {
            if a.len() != e.len() {
                return format!("{path}: lengths {} != {}", a.len(), e.len());
            }
            for (i, (a, e)) in a.iter().zip(e).enumerate() {
                if a != e {
                    return first_difference(a, e, &format!("{path}[{i}]"));
                }
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            if a.keys().ne(e.keys()) {
                return format!("{path}: object keys differ");
            }
            for (key, a) in a {
                if a != &e[key] {
                    return first_difference(a, &e[key], &format!("{path}.{key}"));
                }
            }
        }
        _ => return format!("{path}: {actual} != {expected}"),
    }
    format!("{path}: no leaf difference")
}

pub(super) fn assert_native_scan_inputs(
    film: &LegacyFilm,
    expected: &Value,
    folder: &str,
    flag_requested: bool,
) {
    let mut checked = Vec::new();
    let mut check = |key: &str, actual: Value| {
        let mut native = expected
            .get(key)
            .unwrap_or_else(|| panic!("missing native {key}"))
            .clone();
        normalize_empty(&mut native, &actual);
        if key == "Fire" {
            // Aim is explicitly float32 in both implementations; Go's JSON uses
            // shortest float32 text while serde_json exposes its f64 conversion.
            for row in native.as_array_mut().unwrap() {
                for axis in row["Aim"].as_array_mut().unwrap() {
                    *axis = json!(axis.as_f64().unwrap() as f32);
                }
            }
        }
        if key == "Vehicles" {
            super::vehicle_input_retention_tests::normalize_vehicle(&mut native, &actual);
        }
        if key == "Projectiles" {
            normalize_track_floats(&mut native);
        }
        if key == "Placements" {
            normalize_xyz(&mut native);
        }
        if key == "Pads" {
            for kind in ["Weapons", "Powerups"] {
                normalize_xyz(&mut native[kind]["Creations"]);
                normalize_track_floats(&mut native[kind]["Tracks"]);
            }
        }
        if key == "Translocations" {
            for row in native.as_array_mut().unwrap() {
                for name in ["From", "To"] {
                    for axis in row[name].as_array_mut().unwrap() {
                        *axis = json!(axis.as_f64().unwrap() as f32);
                    }
                }
            }
        }
        if key == "Positions" {
            // Native position/vitality fields are float32. Compare exactly after
            // decoding that declared type, including integer-spelled zeros.
            for row in native.as_array_mut().unwrap() {
                for key in ["X", "Y", "Z"] {
                    row[key] = json!(row[key].as_f64().unwrap() as f32);
                }
                for (outer, inner) in [("Body", "Health"), ("Shield", "Shield")] {
                    row[outer][inner] = json!(row[outer][inner].as_f64().unwrap() as f32);
                }
            }
        }
        if key == "Inventory" {
            // Gauge is float64, including native JSON's integer spelling of zero.
            for row in native.as_array_mut().unwrap() {
                for ammo in row["Ammo"].as_array_mut().unwrap() {
                    if let Some(gauge) = ammo["Gauge"].as_f64() {
                        ammo["Gauge"] = json!(gauge);
                    }
                }
            }
        }
        if actual != native {
            panic!(
                "native input {folder}/{key}: {}",
                first_difference(&actual, &native, "$")
            );
        }
        checked.push(key.to_owned());
    };
    let identity = film
        .native_identity_inputs
        .as_ref()
        .expect("identity inputs retained");
    check("Deaths", json!(identity.deaths));
    check("PlayerIndices", json!(identity.player_indices().0));
    assert!(film.native_translocations_scanned);
    check(
        "Translocations",
        json!(
            film.native_translocations
                .iter()
                .map(|r| {
                    let positions = r.read.event.positions();
                    let [from, to] = positions.unwrap_or([[0.0; 3]; 2]);
                    json!({"TimestampUS":r.source.timestamp_us,"Slot":r.read.event.slot,
            "HasPositions":positions.is_some(),"From":from,"To":to})
                })
                .collect::<Vec<_>>()
        ),
    );
    check(
        "Projectiles",
        json!(
            film.native_projectiles
                .as_ref()
                .map(|s| &s.tracks)
                .cloned()
                .unwrap_or_default()
        ),
    );
    let placements = film.equipment_placements.as_ref();
    check(
        "Placements",
        json!(
            placements
                .map(|s| &s.placements)
                .cloned()
                .unwrap_or_default()
        ),
    );
    let stats = placements.map(|s| s.stats.clone()).unwrap_or_default();
    let mut stats_json = json!(stats);
    let mut widths = stats.calibration.by_widths.clone();
    // The native fixture orders structured map keys by their Go JSON encoding,
    // whose field order here is Lead,Index (not alphabetical or numeric order).
    widths.sort_by_key(|w| {
        format!(
            "{{\"Lead\":{},\"Index\":{}}}",
            w.widths.lead, w.widths.index
        )
    });
    stats_json["Calibration"]["ByWidths"] = json!(widths);
    check("PlacementStats", stats_json);
    check(
        "Pads",
        json!({"Weapons":native_pad_input(film, 42),"Powerups":native_pad_input(film, 37)}),
    );
    check(
        "Vehicles",
        super::vehicle_input_retention_tests::native_vehicle_input(film),
    );
    // The reference six-film harness supplies no zone catalog or bomb input.
    // Flag input is requested by its manifest and still requires native signals.
    let flag = flag_requested
        && super::replay_flags_film::flag_named_inputs(
            &film.statborg.records,
            &film.capture_bursts_ms,
            None,
        )
        .0
        .is_flag_film();
    let objective = native_objective_inputs(film, flag, false, false);
    for key in [
        "FlagMarks",
        "FlagGauge",
        "FlagGaugeScanned",
        "ZoneReads",
        "ZoneScanned",
        "BombReads",
    ] {
        check(key, objective[key].clone());
    }
    check("FilmMajorVersion", json!(film.registry.major_version));
    check("Fire", json!(film.fire_events));
    check("Loadouts", json!(film.keyframe_loadouts));
    check(
        "BipedCreations",
        json!(film.biped_creations.records.iter().map(|r| {
        let c = &r.creation;
        json!({"Slot":c.slot,"Generation":c.generation,"ParticipantIndex":c.participant_index,
            "HasIndex":true,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
            "TimestampUS":r.source.timestamp_us,"BitPos":c.start_bit,"Version":c.version,
            "Representation":c.representation})
    }).collect::<Vec<_>>()),
    );
    let teams = film.player_teams.as_ref().expect("team scan retained");
    check("PlayerTeams", json!(teams.by_index));
    check("TeamScan", json!(teams.report));
    check(
        "FilmTable",
        json!(
            film.player_table
                .as_ref()
                .map(ReplayFilmPlayerTable::from_decoded)
                .unwrap_or_else(|| ReplayFilmPlayerTable {
                    refusal: "sans_section".into(),
                    ..Default::default()
                })
        ),
    );
    let weapons = film
        .weapon_changes
        .as_ref()
        .expect("weapon changes retained");
    check("WeaponChanges", json!(weapons.records));
    let movement = film.movement_states.as_ref().expect("movement retained");
    check("MovementStates", json!(movement.records));
    check("MovementStateStats", json!(movement.stats));
    check(
        "Grenades",
        json!(
            film.grenade_throws
                .as_ref()
                .expect("grenade scan retained")
                .records
        ),
    );
    let charges = film.ability_charges.as_ref().expect("charge scan retained");
    check("AbilityChargeStats", json!(charges.stats));
    check("AbilityCharges", json!(charges.records.iter().map(|r| json!({
        "Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
        "TimestampUS":r.source.timestamp_us,"Emplacement":r.emplacement,"Charges":r.charges,"Low":r.low,
    })).collect::<Vec<_>>()));
    let abilities = film.ability_states.as_ref().expect("ability scan retained");
    check("AbilityImpulseStats", json!(abilities.impulse_stats));
    check(
        "AbilityImpulses",
        json!(abilities.impulses.iter().map(|r| json!({
        "Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
        "TimestampUS":r.source.timestamp_us,"Predicted":r.predicted,
    })).collect::<Vec<_>>()),
    );
    check(
        "GrappleReads",
        json!(abilities.grapple.iter().map(|r| json!({
        "Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
        "TimestampUS":r.source.timestamp_us,"Heavy":r.heavy,"PosQ":r.position_quantized,
    })).collect::<Vec<_>>()),
    );
    let channels = film
        .biped_channels
        .as_ref()
        .expect("biped channels retained");
    check(
        "CamoStates",
        json!(channels.camo_states().map(|r| json!({
        "Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
        "TimestampUS":r.source.timestamp_us,"Q":r.quantum,
    })).collect::<Vec<_>>()),
    );
    check(
        "AbilityRanks",
        json!(channels.ability_ranks().map(|r| json!({
        "Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
        "TimestampUS":r.source.timestamp_us,"Counter":r.counter,"Rank":r.rank.unwrap(),
    })).collect::<Vec<_>>()),
    );
    let pickups = &film.native_pickups;
    let st = &pickups.stats;
    check(
        "PickupStats",
        json!({"Packets":st.packets,"Type9":st.type_9,"Type8":st.type_8,
        "OtherType":st.other_type,"Published":st.published,"MultiEvent":st.multi_event,
        "RefusedNoRef":st.refused_no_ref,"RefusedNoCatalog":st.refused_no_catalog,
        "RefusedOffBand":st.refused_off_band,"UnexpectedWideRef":st.unexpected_wide_ref}),
    );
    check("Pickups", json!(pickups.attempts.iter().filter(|r| r.published).map(|r| json!({
        "TimestampUS":r.source.timestamp_us,"Chunk":r.source.chunk_index,"Slot":r.read.slot().unwrap(),
        "CatalogID":r.read.catalog_id.unwrap(),"Class":r.read.class.unwrap(),
    })).collect::<Vec<_>>()));
    check(
        "FilmClockOriginUS",
        json!(
            film.native_clock_origin
                .as_ref()
                .expect("native clock retained")
                .timestamp_us()
                .unwrap_or(0)
        ),
    );
    check(
        "ZoomEvents",
        json!(
            film.native_zoom_events
                .iter()
                .map(|r| json!({
                    "Slot":r.read.slot,"Level":r.read.level,"TimestampUS":r.source.timestamp_us,
                }))
                .collect::<Vec<_>>()
        ),
    );
    let inventory = film
        .keyframe_inventory
        .as_ref()
        .expect("inventory retained");
    check("Inventory", json!(inventory.records.iter().map(|r| json!({
        "TimestampUS":r.timestamp_us,"Chunk":r.chunk,"PacketIndex":r.packet_index,"Slot":r.slot,
        "Grenades":r.grenades,"GrenadesRead":r.grenades_read,"GrenadesByPosition":r.grenades_by_position,
        "SelectedGrenadeRank":r.selected_grenade_rank,"AbilityRank":r.ability_rank,
        "Ammo":r.ammo.iter().map(|a| json!({"Mag":a.mag,"Res":a.res,"Gauge":a.gauge,
            "Overheat":a.overheat,"Flags":a.flags})).collect::<Vec<_>>(),
        "AmmoRead":r.ammo_read,"DrawnSlot":r.drawn_slot,"AmmoCandidates":r.ammo_candidates,
    })).collect::<Vec<_>>()));
    let delta = film
        .inventory_deltas
        .as_ref()
        .expect("inventory deltas retained");
    check("InventoryDeltaAmmoRefused", json!(delta.stats.ammo_refused));
    check("InventoryDeltas", json!(delta.records.iter().map(|r| json!({
        "Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
        "TimestampUS":r.source.timestamp_us,"Grenades":r.grenades,
        "SelRead":r.selection.is_some(),"Sel":r.selection.as_ref().map_or(0,|s|s.rank.map_or(-1,i32::from)),
        "Mask":r.selection.as_ref().map_or(0,|s|s.mask),
        "Ammo":r.ammo.iter().map(|a|json!({"WeaponSlot":a.weapon_slot,"Mag":a.magazine,
            "FracQ":a.fraction_quantum,"Res":a.reserve})).collect::<Vec<_>>(),
    })).collect::<Vec<_>>()));
    let changes = film
        .equipment_changes
        .as_ref()
        .expect("equipment changes retained");
    check("EquipmentChanges", json!(changes.assembly.records.iter().map(|r| json!({
        "TimestampUS":r.source.timestamp_us,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.unwrap(),
        "Slot":r.slot,"Counter":r.counter,"Rank":r.rank.map_or(-1,i32::from),
        "Previous":r.previous.map_or(-1,i32::from),"Kind":r.kind,"Recovered":r.recovered,"Gap":r.gap,
    })).collect::<Vec<_>>()));
    let a = &changes.assembly.stats;
    let w = &changes.walk;
    check(
        "EquipmentChangeStats",
        json!({"Walk":{"Records":w.records,"WithI48":w.with_component,
        "Read":w.read,"Unread":w.unread,"Gated":w.gated},
        "Lives":a.lives,"Repeats":a.repeats,"CounterJumps":a.counter_jumps,"MissedEstimate":a.missed_estimate,
        "LivesFirstOffSpec":a.lives_first_off_spec,"Spawned":a.spawned,"Taken":a.taken,"Spent":a.spent,"Recovered":a.recovered}),
    );
    let spawned = &film.equipment_spawns;
    let st = &spawned.stats;
    check(
        "SpawnStats",
        json!({"Chunks":st.chunks,"Packets":st.packets,"Lists":st.lists,"Events":st.events,
        "WithSpawned":st.with_spawned,"WithSource":st.with_source,"Ref2":st.reference_2}),
    );
    check("SpawnEvents", json!(spawned.records.iter().map(|r| {
        let life = |value:Option<ObjectLife>|json!({"Slot":value.map_or(0,|v|v.slot),"Gen":value.map_or(0,|v|v.generation)});
        json!({"Chunk":r.packet.chunk_index,"PacketIndex":r.packet_index.unwrap(),"TimestampUS":r.packet.timestamp_us,
            "Spawned":life(r.spawned),"SpawnedValid":r.spawned.is_some(),"Source":life(r.source),
            "SourceValid":r.source.is_some(),"Ref2Present":r.reference_2_present})
    }).collect::<Vec<_>>()));
    check(
        "Positions",
        json!(
            film.biped_positions
                .as_ref()
                .expect("positions retained")
                .accepted()
                .map(native_position)
                .collect::<Vec<_>>()
        ),
    );
    // Keep this count explicit: a final-document match does not silently turn
    // the remaining native inputs into verified intermediate-field retention.
    assert_eq!(checked.len(), CHECKED_INPUTS.len());
}

// Project every exported native BipedPosition field, including zero values for
// absent companions. Rejected candidates remain retained separately in LegacyFilm.
pub(super) fn native_position(p: &BipedPositionCandidate) -> Value {
    let r = &p.record;
    let c = &r.companions;
    let orientation = c.chassis.clone().unwrap_or_default();
    let [vel, scale] = c.velocity.unwrap_or_default();
    let [yaw, pitch] = c.aim.unwrap_or_default();
    let [yaw_b, pitch_b] = c.aim_b.unwrap_or_default();
    let (mask, over) = r.compact_component_mask();
    let b = c.body.as_ref();
    let s = c.shield.as_ref();
    let bf = b.map_or([false; 3], |b| b.flags);
    let sf = s.map_or([false; 4], |s| s.flags);
    let regen = s.map_or([None; 2], |s| s.regen);
    let body = json!({"Q":b.map_or(0, |b| b.quantum),"Health":b.map_or(0.0, |b| b.health),
            "F5c":bf[0],"F5d":bf[1],"F5e":bf[2]});
    let shield = json!({"Q":s.map_or(0, |s| s.quantum),"Shield":s.map_or(0.0, |s| s.shield),
            "RegenPresent":s.is_some_and(|s|s.regen_present),"HasRegen0":regen[0].is_some(),
            "HasRegen1":regen[1].is_some(),"Regen0":regen[0].unwrap_or(0),"Regen1":regen[1].unwrap_or(0),
            "Block64":s.map_or(0, |s| s.block_64),"F66":sf[0],"F67":sf[1],"F69":sf[2],"F68":sf[3]});
    json!({
        "Slot":r.slot,"Chunk":p.source.chunk_index,"PacketIndex":p.packet_index.unwrap(),
        "TimestampUS":p.source.timestamp_us,"X":r.world[0],"Y":r.world[1],"Z":r.world[2],
        "HasWorld":true,"Q":r.quantized,"HasAim":c.forward.is_some(),"AimRaw":c.forward.unwrap_or(0),
        "HasRoll":orientation.roll.is_some(),"RollRaw":orientation.roll.unwrap_or(0),
        "FwdMode":orientation.mode,"AimDefault":orientation.direction_default,
        "HasVel":c.velocity.is_some(),"VelRaw":vel,"VelScale":scale,
        "HasYaw":c.aim.is_some(),"YawRaw":yaw,"PitchRaw":pitch,"AimFlag0":c.aim_flag_0,"AimFlag1":c.aim_flag_1,
        "HasAimB":c.aim_b.is_some(),"YawRawB":yaw_b,"PitchRawB":pitch_b,"AimFlag2":c.aim_flag_2,
        "MaskBits":mask,"MaskOver":over,"HasBody":b.is_some(),"HasShield":s.is_some(),
        "Body":body,"Shield":shield
    })
}

pub(super) fn normalize_xyz(rows: &mut Value) {
    if let Some(rows) = rows.as_array_mut() {
        for row in rows {
            for name in ["X", "Y", "Z"] {
                row[name] = json!(row[name].as_f64().unwrap() as f32);
            }
        }
    }
}
fn normalize_track_floats(tracks: &mut Value) {
    if let Some(tracks) = tracks.as_array_mut() {
        for track in tracks {
            normalize_xyz(&mut track["Pts"]);
        }
    }
}
fn native_pad_input(film: &LegacyFilm, archetype: u32) -> Value {
    let creations = if archetype == 42 {
        film.ground_weapon_creations.as_ref()
    } else {
        film.equipment_pad_creations.as_ref()
    };
    let tracks = film.ground_object_tracks.get(&archetype);
    let census = film
        .world_object_keyframes
        .get(&archetype)
        .cloned()
        .unwrap_or_default();
    json!({
        "Scanned":creations.is_some() && tracks.is_some(),
        "Creations":creations.map(|s| s.records.clone()).unwrap_or_default(),
        "Stats":creations.map(|s| s.stats.clone()).unwrap_or_default(),
        "Tracks":tracks.map(|s| s.tracks.clone()).unwrap_or_default(),
        "Keyframes":native_census(&census)
    })
}

pub(super) fn native_census(census: &WorldObjectKeyframes) -> Value {
    let mut seen = census.seen_us.iter().collect::<Vec<_>>();
    seen.sort_by_key(|s| format!("{{\"Slot\":{},\"Gen\":{}}}", s.slot, s.generation));
    let seen: Vec<_> = seen
        .into_iter()
        .map(|s| {
            json!({
                "key":{"Slot":s.slot,"Gen":s.generation},"value":s.times_us,
            })
        })
        .collect();
    json!({"Band":census.band.iter().map(|slot| (slot.to_string(),true)).collect::<std::collections::BTreeMap<_,_>>(),
        "TimesUS":census.times_us,"SeenUS":seen})
}

pub(super) fn native_objective_inputs(
    film: &LegacyFilm,
    flag: bool,
    zones: bool,
    bomb: bool,
) -> Value {
    let managed = replay_managed_property_reads(film);
    json!({
        "FlagMarks":if flag { film.carrier_marks.clone().unwrap_or_default() } else { CarrierMarkScan::default() },
        "FlagGauge":if flag { managed } else { &[] },"FlagGaugeScanned":flag,
        "ZoneReads":if zones { managed } else { &[] },"ZoneScanned":zones,
        "BombReads":if bomb { super::replay_bomb_armings::replay_bomb_radial_reads(film) } else { &[] },
    })
}
pub(super) fn assert_objective_variants(film: &LegacyFilm, row: &Value) {
    for variant in row["variants"].as_array().unwrap() {
        let enabled = variant["enabled"].as_bool().unwrap();
        let actual = native_objective_inputs(film, enabled, enabled, enabled);
        let mut expected = variant["inputs"].clone();
        normalize_empty(&mut expected, &actual);
        assert!(
            actual == expected,
            "{}: {}",
            row["folder"],
            first_difference(&actual, &expected, "$objectives")
        );
    }
}
