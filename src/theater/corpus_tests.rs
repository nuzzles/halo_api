//! Optional local-corpus regression. Captured unit fixtures remain self-contained.
use super::*;
use crate::clients::hi::models::{FilmChunk, FilmChunkData};
use std::{fs, path::Path};

#[test]
#[ignore = "diagnostic frontier report over downloaded v41 keyframes; not a parity assertion"]
fn local_keyframe_frontiers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films");
    let mut stops = std::collections::BTreeMap::<String, usize>::new();
    let mut tables = 0;
    let mut complete_records = 0;
    for group in fs::read_dir(root).unwrap().flatten() {
        if !group.path().is_dir() {
            continue;
        }
        for film in fs::read_dir(group.path()).unwrap().flatten() {
            let path = film.path();
            if !path.join("film.json").exists() {
                continue;
            }
            let registry =
                parse_registry(&fs::read(path.join("chunk-000-type-1.bin")).unwrap()).unwrap();
            let identity = decode_film_identity(
                &fs::read(path.join("chunk-000-type-1.bin")).unwrap(),
                &registry,
            )
            .unwrap()
            .unwrap();
            let meta: serde_json::Value =
                serde_json::from_slice(&fs::read(path.join("film.json")).unwrap()).unwrap();
            for c in meta["chunks"].as_array().unwrap() {
                if c["chunk_type"] != 2 {
                    continue;
                }
                let data = fs::read(path.join(c["file"].as_str().unwrap())).unwrap();
                let mut offset = 0;
                while offset < data.len() {
                    let kind = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
                    let size = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap())
                        as usize;
                    if kind == 2 {
                        let encoding = FrameEncoding {
                            keyframe_layout: Default::default(),
                            keyframe_simulation_complete: None,
                            native_id_low_bits: None,
                            component_widths: Default::default(),
                            new_record: Default::default(),
                            position_capture: None,
                            ids: RecordIdLayout {
                                low_bits: 13,
                                base: 0,
                            },
                            mpp_widths: [9, 5],
                            position: None,
                            extra_fields: false,
                            corruption_check: identity.corruption_checks,
                        };
                        let table = decode_keyframe_table(
                            &data[offset + 16..offset + 16 + size],
                            &registry,
                            &encoding,
                            &mut EntityBindings::default(),
                        );
                        tables += 1;
                        complete_records += table
                            .records
                            .iter()
                            .filter(|r| r.stop == KeyframeStop::Complete)
                            .count();
                        *stops.entry(format!("{:?}", table.stop)).or_default() += 1;
                    }
                    offset += 16 + size;
                }
            }
        }
    }
    println!("tables={tables}, complete_records={complete_records}, stops={stops:#?}");
}

#[test]
#[ignore = "requires downloaded experiments/films and pre-existing decoded-film.json files"]
fn local_v41_corpus_preserves_existing_observations() {
    use std::io::Read;
    let mut oracle = String::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/event-heads-levelup-v41.json.zlib")[..],
    )
    .read_to_string(&mut oracle)
    .unwrap();
    let oracle: Vec<serde_json::Value> = serde_json::from_str(&oracle).unwrap();
    let mut bot_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bot-corpus-v41.json.zlib")[..])
        .read_to_end(&mut bot_raw)
        .unwrap();
    let bot_rows: Vec<serde_json::Value> = serde_json::from_slice(&bot_raw).unwrap();
    let mut statborg_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/statborg-corpus-v41.json.zlib")[..])
        .read_to_end(&mut statborg_raw)
        .unwrap();
    let expected_statborg: std::collections::BTreeMap<String, FilmStatborgStream> =
        serde_json::from_slice(&statborg_raw).unwrap();
    let mut team_raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/player-teams-corpus-v41.json.zlib")[..],
    )
    .read_to_end(&mut team_raw)
    .unwrap();
    let expected_teams: std::collections::BTreeMap<String, FilmPlayerTeams> =
        serde_json::from_slice(&team_raw).unwrap();
    let mut highlight_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/highlights-corpus-v41.json.zlib")[..])
        .read_to_end(&mut highlight_raw)
        .unwrap();
    let expected_highlights: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_slice(&highlight_raw).unwrap();
    let expected_bots: std::collections::BTreeMap<String, FilmBotMetadata> = bot_rows
        .into_iter()
        .map(|row| {
            (
                row["folder"].as_str().unwrap().into(),
                serde_json::from_value(row["output"].clone()).unwrap(),
            )
        })
        .collect();
    let mut expected_heads = std::collections::BTreeMap::<String, Vec<DecodedHeadEvent>>::new();
    for row in oracle {
        let source = row["source"].as_str().unwrap();
        if source == "synthetic" {
            continue;
        }
        let parent = Path::new(source).parent().unwrap();
        let key = format!(
            "{}/{}",
            parent
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            parent.file_name().unwrap().to_str().unwrap()
        );
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        expected_heads
            .entry(key)
            .or_default()
            .push(decode_packet_head_event(&data).unwrap());
    }
    let mut roster_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/roster-v41.json.zlib")[..])
        .read_to_end(&mut roster_raw)
        .unwrap();
    let roster_rows: Vec<serde_json::Value> = serde_json::from_slice(&roster_raw).unwrap();
    let mut expected_rosters = std::collections::BTreeMap::<String, Vec<serde_json::Value>>::new();
    for row in roster_rows {
        let source = row["source"].as_str().unwrap();
        if !source.starts_with('/') {
            continue;
        }
        let parent = Path::new(source).parent().unwrap();
        let key = format!(
            "{}/{}",
            parent
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            parent.file_name().unwrap().to_str().unwrap()
        );
        let chunk = Path::new(source)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split('-')
            .nth(1)
            .unwrap()
            .parse::<i32>()
            .unwrap();
        let entries = if row["entries"].is_null() {
            serde_json::json!([])
        } else {
            row["entries"].clone()
        };
        expected_rosters.entry(key).or_default().push(super::roster_updates::tests::normalize(serde_json::json!({"chunk":chunk,"offset":row["payload_offset"],"time":row["timestamp_us"],"size":row["payload_size"],"entries":entries,"report":row["report"]})));
    }
    let mut fire_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/fire-d61443e-v41.json.zlib")[..])
        .read_to_end(&mut fire_raw)
        .unwrap();
    let fire_rows: Vec<serde_json::Value> = serde_json::from_slice(&fire_raw).unwrap();
    let mut expected_fire = std::collections::BTreeMap::<String, Vec<FilmFireEvent>>::new();
    for row in fire_rows {
        let source = row["source"].as_str().unwrap();
        if !source.contains("/experiments/films/") || row["valid"] != true {
            continue;
        }
        let parent = Path::new(source).parent().unwrap();
        let key = format!(
            "{}/{}",
            parent
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            parent.file_name().unwrap().to_str().unwrap()
        );
        expected_fire
            .entry(key)
            .or_default()
            .push(serde_json::from_value(row["event"].clone()).unwrap());
    }
    let mut grenade_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/grenade-v41.json.zlib")[..])
        .read_to_end(&mut grenade_raw)
        .unwrap();
    let grenade_rows: Vec<serde_json::Value> = serde_json::from_slice(&grenade_raw).unwrap();
    let mut expected_grenades = std::collections::BTreeMap::<String, GrenadeThrowStream>::new();
    for row in grenade_rows {
        let source = row["source"].as_str().unwrap();
        if source == "generated" {
            continue;
        }
        let parent = Path::new(source).parent().unwrap();
        let key = format!(
            "{}/{}",
            parent
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            parent.file_name().unwrap().to_str().unwrap()
        );
        let stream = expected_grenades
            .entry(key)
            .or_insert_with(|| GrenadeThrowStream {
                grammar: serde_json::from_value(row["grammar"].clone()).unwrap(),
                records: Vec::new(),
                stats: Default::default(),
            });
        let records: Vec<FilmGrenadeThrow> = serde_json::from_value(if row["events"].is_null() {
            serde_json::json!([])
        } else {
            row["events"].clone()
        })
        .unwrap();
        stream.records.extend(records);
        let stats: GrenadeThrowStats = serde_json::from_value(row["stats"].clone()).unwrap();
        stream.stats.patterns += stats.patterns;
        stream.stats.other_archetypes += stats.other_archetypes;
        stream.stats.rejected_known_ids += stats.rejected_known_ids;
        stream.stats.indeterminate_archetype += stats.indeterminate_archetype;
        stream.stats.published += stats.published;
    }
    let mut census_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/world-census-v41.json.zlib")[..])
        .read_to_end(&mut census_raw)
        .unwrap();
    let census_rows: Vec<serde_json::Value> = serde_json::from_slice(&census_raw).unwrap();
    let mut expected_census = std::collections::BTreeMap::<
        String,
        std::collections::BTreeMap<u32, WorldObjectKeyframes>,
    >::new();
    for row in census_rows {
        let folder = row["folder"].as_str().unwrap();
        if folder == "generated" {
            continue;
        }
        let path = Path::new(folder);
        let key = format!(
            "{}/{}",
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            path.file_name().unwrap().to_str().unwrap()
        );
        expected_census.insert(key, serde_json::from_value(row["census"].clone()).unwrap());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films");
    let mut folders = Vec::new();
    for group in fs::read_dir(root).unwrap().flatten() {
        if !group.path().is_dir() {
            continue;
        }
        for film in fs::read_dir(group.path()).unwrap().flatten() {
            if film.path().join("film.json").exists() {
                folders.push(film.path());
            }
        }
    }
    folders.sort();
    assert_eq!(folders.len(), 32);
    for path in folders {
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<FilmChunkData> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: c["decompressed_size"].as_i64().unwrap(),
                    file_relative_path: c["file_relative_path"].as_str().unwrap().into(),
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                },
                data: fs::read(path.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let previous: LegacyFilm = serde_json::from_reader(std::io::BufReader::new(
            fs::File::open(path.join("decoded-film.json")).unwrap(),
        ))
        .unwrap();
        let actual = LegacyFilm::try_from_chunks(
            &chunks,
            DecodeOptions {
                major_version: 41,
                match_id: previous.match_id.clone(),
                duration_us: Some(previous.duration_us),
                retain_coverage: false,
            },
        )
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(actual.registry.archetypes.len(), 50, "{}", path.display());
        assert!(actual.identity.is_some(), "{}", path.display());
        let key = format!(
            "{}/{}",
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            path.file_name().unwrap().to_str().unwrap()
        );
        assert_eq!(actual.statborg, expected_statborg[&key], "statborg {key}");
        let native = actual.native_highlights.as_ref().unwrap();
        let expected = &expected_highlights[&key];
        assert_eq!(
            native.chunk_index,
            expected["index"].as_i64().unwrap() as i32
        );
        let events: Option<Vec<NativeHighlightEvent>> =
            serde_json::from_value(expected["events"].clone()).unwrap();
        assert_eq!(
            native.events,
            events.unwrap_or_default(),
            "native highlights {key}"
        );
        let deaths = native.deaths();
        assert_eq!(deaths.is_err(), expected["error"].as_bool().unwrap());
        if let Ok(deaths) = deaths {
            let expected_deaths: Vec<IdentityDeath> =
                serde_json::from_value(expected["deaths"].clone()).unwrap();
            assert_eq!(deaths, expected_deaths, "native deaths {key}");
        }
        let mut accepted_teams = actual.player_teams.clone();
        if let Some(teams) = &mut accepted_teams {
            teams.attempts.clear();
        }
        assert_eq!(
            accepted_teams.as_ref(),
            Some(&expected_teams[&key]),
            "teams {key}"
        );
        assert_eq!(
            actual.bot_metadata, expected_bots[&key],
            "bot metadata {key}"
        );
        assert_eq!(
            actual.fire_events,
            expected_fire.remove(&key).unwrap_or_default(),
            "native fire: {}",
            path.display()
        );
        let expected = expected_grenades
            .remove(&key)
            .unwrap_or_else(|| GrenadeThrowStream {
                grammar: GrenadeGrammar {
                    projectile_archetype: 41,
                    resolved_by_name: true,
                    known_build: true,
                },
                records: Vec::new(),
                stats: Default::default(),
            });
        assert_eq!(
            actual.grenade_throws.as_ref(),
            Some(&expected),
            "native grenades: {}",
            path.display()
        );
        assert_eq!(
            actual.world_object_keyframes,
            expected_census.remove(&key).unwrap(),
            "native world-object census: {}",
            path.display()
        );
        let native_rosters:Vec<_>=actual.roster_updates.as_ref().expect("known corpus build").iter().map(|u|super::roster_updates::tests::normalize(serde_json::json!({"chunk":u.source.chunk_index,"offset":u.source.payload_offset,"time":u.source.timestamp_us,"size":u.source.payload_size,"entries":u.roster.entries,"report":u.roster.report}))).collect();
        assert_eq!(
            native_rosters,
            expected_rosters.remove(&key).unwrap_or_default(),
            "native roster updates: {}",
            path.display()
        );
        let native_heads: Vec<_> = actual
            .head_events
            .iter()
            .filter(|head| matches!(head.event.kind, 8 | 9 | 21 | 22 | 103))
            .map(|head| head.event.clone())
            .collect();
        assert_eq!(
            native_heads,
            expected_heads.remove(&key).unwrap_or_default(),
            "native head events: {}",
            path.display()
        );
        assert_eq!(
            actual.players,
            previous.players,
            "player observations: {}",
            path.display()
        );
        assert_eq!(actual.clocks, previous.clocks, "clocks: {}", path.display());
        assert_eq!(
            actual.projectiles,
            previous.projectiles,
            "projectiles: {}",
            path.display()
        );
        assert_eq!(
            actual.summary_events,
            previous.summary_events,
            "events: {}",
            path.display()
        );
        println!("preserved {}", path.display());
    }
    assert!(expected_heads.is_empty());
    assert!(expected_rosters.is_empty());
    assert!(expected_fire.is_empty());
    assert!(expected_grenades.is_empty());
    assert!(expected_census.is_empty());
}

#[test]
#[ignore = "requires four downloaded films; compares actual reference positions and companion fields"]
fn biped_position_scan_matches_production_films() {
    use std::io::Read;
    let mut json = String::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/gameplay-levelup-v41.json.zlib")[..])
        .read_to_string(&mut json)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let mut ground_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/ground-creations-v41.json.zlib")[..])
        .read_to_end(&mut ground_raw)
        .unwrap();
    let ground_rows: Vec<serde_json::Value> = serde_json::from_slice(&ground_raw).unwrap();
    let mut ground_kf_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/ground-keyframes-v41.json.zlib")[..])
        .read_to_end(&mut ground_kf_raw)
        .unwrap();
    let ground_kf_rows: serde_json::Value = serde_json::from_slice(&ground_kf_raw).unwrap();
    let mut object_raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/ground-objects-corpus-v41.json.zlib")[..],
    )
    .read_to_end(&mut object_raw)
    .unwrap();
    let object_rows: Vec<serde_json::Value> = serde_json::from_slice(&object_raw).unwrap();
    for row in rows {
        let folder = row["folder"].as_str().unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/films")
            .join(folder);
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: c["decompressed_size"].as_i64().unwrap(),
                    file_relative_path: c["file_relative_path"].as_str().unwrap().into(),
                },
                data: fs::read(path.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
        let equipment_registry = decode_registry(&chunks).unwrap();
        let ground_row = ground_rows.iter().find(|r| r["folder"] == folder).unwrap();
        let expected_kf: Vec<KeyframeGroundWeapon> = serde_json::from_value(
            ground_kf_rows["films"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["folder"] == folder)
                .unwrap()["records"]
                .clone(),
        )
        .unwrap();
        assert_eq!(
            scan_keyframe_ground_weapons(&chunks).unwrap(),
            expected_kf,
            "ground keyframes {folder}"
        );
        let ground_band = world_object_slot_band(&chunks, 42).unwrap();
        let ground = scan_ground_weapon_creations_for_band(
            &chunks,
            &map,
            &ground_band,
            &equipment_registry,
            [9, 5],
            ground_row["corruption"].as_bool().unwrap(),
        )
        .unwrap();
        let ground_expected: Vec<EquipmentCreation> =
            serde_json::from_value(ground_row["records"].clone()).unwrap();
        assert_eq!(ground.records, ground_expected, "ground creations {folder}");
        let ground_stats: EquipmentCreationStats =
            serde_json::from_value(ground_row["stats"].clone()).unwrap();
        assert_eq!(ground.stats, ground_stats, "ground creation stats {folder}");
        let object_row = object_rows.iter().find(|r| r["folder"] == folder).unwrap();
        let families: std::collections::BTreeMap<u32, String> =
            serde_json::from_value(object_row["families"].clone()).unwrap();
        assert_eq!(&families, v41_weapon_families(), "ground family catalog");
        let mut player_positions: Vec<ReplayPlayerPosition> =
            serde_json::from_value(if row["inputs"]["Positions"].is_null() {
                serde_json::json!([])
            } else {
                row["inputs"]["Positions"].clone()
            })
            .unwrap();
        player_positions.sort_by_key(|p| p.timestamp_us);
        let ground_tracks = scan_world_object_tracks_for_band(&chunks, &map, &ground_band).unwrap();
        let ground_kf = scan_world_object_keyframes(&chunks, 42);
        let assembly = assemble_ground_objects(
            &ground.records,
            &ground_tracks.tracks,
            &ground_kf,
            &player_positions,
            GroundObjectRule {
                kind: "weapon",
                families: &families,
                objectives: &Default::default(),
            },
        );
        let mut expected_objects: Vec<GroundPickupObject> =
            serde_json::from_value(object_row["objects"].clone()).unwrap();
        for (o, bits) in expected_objects
            .iter_mut()
            .zip(object_row["distance_bits"].as_array().unwrap())
        {
            o.picker.distance_m = f64::from_bits(bits.as_u64().unwrap());
        }
        assert_eq!(
            assembly.objects, expected_objects,
            "ground assembly {folder}"
        );
        let expected_rejected: GroundObjectRejections =
            serde_json::from_value(object_row["rejected"].clone()).unwrap();
        assert_eq!(
            assembly.rejected, expected_rejected,
            "ground rejections {folder}"
        );
        let equipment_band = world_object_slot_band(&chunks, 37).unwrap();
        let placements =
            scan_equipment_placements_for_band(&chunks, &map, &equipment_band, &equipment_registry)
                .unwrap();
        let expected_placements: Vec<EquipmentPlacement> =
            serde_json::from_value(if row["inputs"]["Placements"].is_null() {
                serde_json::json!([])
            } else {
                row["inputs"]["Placements"].clone()
            })
            .unwrap();
        assert_eq!(
            placements.placements, expected_placements,
            "equipment placements {folder}"
        );
        let expected_stats: EquipmentPlacementStats =
            serde_json::from_value(row["inputs"]["PlacementStats"].clone()).unwrap();
        assert_eq!(placements.stats, expected_stats, "placement stats {folder}");
        let expected_projectiles: Vec<WorldObjectTrack> =
            serde_json::from_value(if row["inputs"]["Projectiles"].is_null() {
                serde_json::json!([])
            } else {
                row["inputs"]["Projectiles"].clone()
            })
            .unwrap();
        let actual_projectiles = match scan_world_object_tracks(&chunks, &map, 41) {
            Ok(stream) => stream.tracks,
            Err(DecodeError::Missing("world object slot band")) => Vec::new(),
            Err(e) => panic!("projectiles {folder}: {e}"),
        };
        assert_eq!(
            actual_projectiles.len(),
            expected_projectiles.len(),
            "projectile count {folder}"
        );
        for (index, (a, b)) in actual_projectiles
            .iter()
            .zip(&expected_projectiles)
            .enumerate()
        {
            assert_eq!(a, b, "projectile {folder} track {index}");
        }
        println!("{folder}: {} projectile tracks", actual_projectiles.len());
        let teleports = scan_translocator_events(&chunks, Some(&map)).unwrap();
        let loadouts = scan_keyframe_loadouts(&chunks).unwrap();
        assert_eq!(
            serde_json::json!(loadouts),
            row["inputs"]["Loadouts"],
            "loadouts {folder}"
        );
        println!("{folder}: {} keyframe loadouts", loadouts.len());
        let inventory = scan_keyframe_inventory(&chunks, 0).unwrap();
        let mut expected_inventory: Vec<KeyframeInventory> =
            serde_json::from_value(row["inputs"]["Inventory"].clone()).unwrap();
        for (inv, bits) in expected_inventory
            .iter_mut()
            .zip(row["keyframe_inventory_gauge_bits"].as_array().unwrap())
        {
            for (slot, value) in inv.ammo.iter_mut().zip(bits.as_array().unwrap()) {
                slot.gauge = value.as_u64().map(f64::from_bits);
                slot.gauge_quantum = slot.gauge.map(|g| (g * 4095.).round() as u16);
            }
        }
        assert_eq!(
            inventory.records.len(),
            expected_inventory.len(),
            "keyframe inventory count {folder}"
        );
        for (index, (actual, expected)) in inventory
            .records
            .iter()
            .zip(&expected_inventory)
            .enumerate()
        {
            assert_eq!(
                actual, expected,
                "keyframe inventory {folder} record {index}"
            );
        }
        assert_eq!(
            serde_json::json!(inventory.stats),
            row["keyframe_inventory_stats"],
            "keyframe inventory stats {folder}"
        );
        assert!(inventory.default_grenade_max);
        println!(
            "{folder}: {} keyframe inventory records",
            inventory.records.len()
        );
        let heads = scan_packet_head_events(&chunks).unwrap();
        let stream =
            scan_biped_positions(&chunks, &map, BipedScanOptions::default(), &teleports).unwrap();
        let expected_fire: Vec<FilmFireEvent> =
            serde_json::from_value(if row["inputs"]["Fire"].is_null() {
                serde_json::json!([])
            } else {
                row["inputs"]["Fire"].clone()
            })
            .unwrap();
        assert_eq!(
            scan_fire_events(&chunks).unwrap(),
            expected_fire,
            "production fire: {folder}"
        );
        let expected_grenades: Vec<FilmGrenadeThrow> =
            serde_json::from_value(if row["inputs"]["Grenades"].is_null() {
                serde_json::json!([])
            } else {
                row["inputs"]["Grenades"].clone()
            })
            .unwrap();
        let grenade_registry = decode_registry(&chunks).unwrap();
        let native_grenades = scan_grenade_throws(&chunks, Some(&grenade_registry), None).unwrap();
        assert_eq!(
            native_grenades.records, expected_grenades,
            "production grenades: {folder}"
        );
        let weapon_changes = scan_held_weapon_changes(
            &chunks,
            &decode_registry(&chunks).unwrap(),
            &stream,
            &map.position_encoding(),
            &loadouts,
        )
        .unwrap();
        let expected_changes = row["inputs"]["WeaponChanges"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            serde_json::json!(weapon_changes.records),
            serde_json::json!(expected_changes),
            "weapon changes {folder}"
        );
        assert_eq!(
            serde_json::json!(weapon_changes.stats),
            row["weapon_stats"],
            "weapon stats {folder}"
        );
        println!("{folder}: {} weapon changes", weapon_changes.records.len());
        let expected = row["inputs"]["Positions"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let actual: Vec<_> = stream.accepted().collect();
        let pickups = biped_pickups_from_heads(&heads, stream.slot_band);
        let expected_pickups = row["inputs"]["Pickups"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            pickups.records.len(),
            expected_pickups.len(),
            "pickups {folder}"
        );
        for (a, e) in pickups.records.iter().zip(&expected_pickups) {
            assert_eq!(
                serde_json::json!({"Slot":a.slot,"TimestampUS":a.source.timestamp_us,"Chunk":a.source.chunk_index,"CatalogID":a.catalog_id,"Class":a.class}),
                *e
            );
        }
        let s = &pickups.stats;
        assert_eq!(s.truncated, 0);
        assert_eq!(
            serde_json::json!({"Packets":s.packets,"Type9":s.type_9,"Type8":s.type_8,"OtherType":0,"Published":s.published,"MultiEvent":s.multi_event,"RefusedNoRef":s.refused_no_ref,"RefusedNoCatalog":s.refused_no_catalog,"RefusedOffBand":s.refused_off_band,"UnexpectedWideRef":s.unexpected_wide_ref}),
            row["inputs"]["PickupStats"]
        );
        let zoom = biped_zoom_from_heads(&heads);
        let expected_zoom = row["inputs"]["ZoomEvents"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(zoom.len(), expected_zoom.len(), "zoom {folder}");
        for (a, e) in zoom.iter().zip(&expected_zoom) {
            assert_eq!(
                serde_json::json!({"Slot":a.slot,"TimestampUS":a.source.timestamp_us,"Level":a.level}),
                *e
            );
        }
        println!(
            "{folder}: {} pickups, {} zoom events",
            pickups.records.len(),
            zoom.len()
        );
        assert_eq!(
            actual.len(),
            expected.len(),
            "{folder}, candidates={}",
            stream.candidates.len()
        );
        let mut ordinals = std::collections::BTreeMap::new();
        let mut counts = std::collections::BTreeMap::<i32, usize>::new();
        for p in super::packets::index(&chunks).unwrap() {
            let n = counts.entry(p.chunk_index).or_default();
            ordinals.insert((p.chunk_index, p.payload_offset), *n);
            *n += 1;
        }
        let creations = scan_biped_creations(&chunks).unwrap();
        let registry = decode_registry(&chunks).unwrap();
        let inventory =
            scan_inventory_deltas(&chunks, &registry, &stream, &map.position_encoding()).unwrap();
        assert_inventory_parity(&inventory, &row, folder);
        println!(
            "{folder}: {} inventory deltas, ammo_refused={}",
            inventory.records.len(),
            inventory.stats.ammo_refused
        );
        let channels =
            assert_biped_gameplay_parity(&chunks, &registry, &stream, &map, &row, folder);
        let equipment =
            scan_equipment_changes(&chunks, &registry, &stream, &channels, &map).unwrap();
        let equipment_records: Vec<_> = equipment.assembly.records.iter().map(|a| serde_json::json!({"TimestampUS":a.source.timestamp_us,"Chunk":a.source.chunk_index,"PacketIndex":a.packet_index.expect("loaded equipment packet ordinal"),"Slot":a.slot,"Counter":a.counter,"Rank":a.rank.map_or(-1,i32::from),"Previous":a.previous.map_or(-1,i32::from),"Kind":a.kind,"Recovered":a.recovered,"Gap":a.gap})).collect();
        assert_eq!(
            equipment_records,
            row["inputs"]["EquipmentChanges"]
                .as_array()
                .cloned()
                .unwrap_or_default(),
            "equipment {folder}"
        );
        let s = &equipment.assembly.stats;
        let w = &equipment.walk;
        assert_eq!(
            serde_json::json!({"Lives":s.lives,"Repeats":s.repeats,"CounterJumps":s.counter_jumps,"MissedEstimate":s.missed_estimate,"LivesFirstOffSpec":s.lives_first_off_spec,"Spawned":s.spawned,"Taken":s.taken,"Spent":s.spent,"Recovered":s.recovered,"Walk":{"Records":w.records,"WithI48":w.with_component,"Read":w.read,"Unread":w.unread,"Gated":w.gated}}),
            row["inputs"]["EquipmentChangeStats"],
            "equipment stats {folder}"
        );
        println!(
            "{folder}: {} equipment changes, {} recovered",
            equipment_records.len(),
            s.recovered
        );
        let spawns = equipment_spawns_from_heads(&chunks, &heads).unwrap();
        let expected_spawns = row["inputs"]["SpawnEvents"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            spawns.records.len(),
            expected_spawns.len(),
            "spawn events {folder}"
        );
        for (a, e) in spawns.records.iter().zip(&expected_spawns) {
            let life = |v: Option<ObjectLife>| {
                v.map_or(
                    serde_json::json!({"Slot":0,"Gen":0}),
                    |v| serde_json::json!({"Slot":v.slot,"Gen":v.generation}),
                )
            };
            assert_eq!(
                serde_json::json!({"Chunk":a.packet.chunk_index,"PacketIndex":a.packet_index.expect("loaded spawn packet ordinal"),"TimestampUS":a.packet.timestamp_us,"Source":life(a.source),"SourceValid":a.source.is_some(),"Spawned":life(a.spawned),"SpawnedValid":a.spawned.is_some(),"Ref2Present":a.reference_2_present}),
                *e
            );
        }
        let s = &spawns.stats;
        assert_eq!(s.truncated, 0);
        assert_eq!(
            serde_json::json!({"Chunks":s.chunks,"Packets":s.packets,"Lists":s.lists,"Events":s.events,"WithSpawned":s.with_spawned,"WithSource":s.with_source,"Ref2":s.reference_2}),
            row["inputs"]["SpawnStats"]
        );
        println!("{folder}: {} equipment spawn events", spawns.records.len());
        let expected_creations = row["inputs"]["BipedCreations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            creations.records.len(),
            expected_creations.len(),
            "creation count {folder}"
        );
        for (a, e) in creations.records.iter().zip(&expected_creations) {
            let c = &a.creation;
            assert_eq!(
                serde_json::json!({"Slot":c.slot,"Generation":c.generation,"ParticipantIndex":c.participant_index,"HasIndex":true,"Chunk":a.source.chunk_index,"PacketIndex":a.packet_index.expect("loaded creation ordinal"),"TimestampUS":a.source.timestamp_us,"BitPos":c.start_bit,"Version":c.version,"Representation":c.representation}),
                *e,
                "creation {folder}"
            );
        }
        println!("{folder}: {} biped creations", creations.records.len());
        for (a, e) in actual.iter().zip(&expected) {
            super::biped_capture::tests::assert_native(&a.record.companions, e, e);
            let mask_bits = a
                .record
                .component_indices
                .iter()
                .fold(0u64, |m, &i| m | (1u64 << i));
            assert_eq!(mask_bits, e["MaskBits"].as_u64().unwrap());
            assert_eq!(e["MaskOver"], false);
            assert_eq!(u64::from(a.record.slot), e["Slot"].as_u64().unwrap());
            assert_eq!(a.source.timestamp_us, e["TimestampUS"].as_u64().unwrap());
            assert_eq!(a.source.chunk_index as i64, e["Chunk"].as_i64().unwrap());
            assert_eq!(
                ordinals[&(a.source.chunk_index, a.source.payload_offset)] as u64,
                e["PacketIndex"].as_u64().unwrap()
            );
            for (i, name) in ["X", "Y", "Z"].iter().enumerate() {
                assert_eq!(
                    u64::from(a.record.quantized[i]),
                    e["Q"][i].as_u64().unwrap()
                );
                assert_eq!(
                    a.record.world[i].to_bits(),
                    (e[name].as_f64().unwrap() as f32).to_bits()
                );
            }
        }
        println!(
            "{folder}: {} accepted of {} candidates; band={:?}",
            actual.len(),
            stream.candidates.len(),
            stream.slot_band
        );
        println!(
            "companions: aim={}, velocity={}, body={}, shield={}",
            actual
                .iter()
                .filter(|p| p.record.companions.aim.is_some())
                .count(),
            actual
                .iter()
                .filter(|p| p.record.companions.velocity.is_some())
                .count(),
            actual
                .iter()
                .filter(|p| p.record.companions.body.is_some())
                .count(),
            actual
                .iter()
                .filter(|p| p.record.companions.shield.is_some())
                .count()
        );
    }
}

#[test]
#[ignore = "requires four captured films; production movement transition and counter parity"]
fn movement_scan_matches_production_films() {
    use std::io::Read;
    let mut json = String::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/gameplay-levelup-v41.json.zlib")[..])
        .read_to_string(&mut json)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let mut failures = vec![];
    for row in rows {
        let folder = row["folder"].as_str().unwrap();
        let trace_folder = std::env::var("HALO_TRACE_FOLDER").ok();
        if trace_folder
            .as_ref()
            .is_some_and(|selected| selected != folder)
        {
            continue;
        }
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/films")
            .join(folder);
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: c["decompressed_size"].as_i64().unwrap(),
                    file_relative_path: c["file_relative_path"].as_str().unwrap().into(),
                },
                data: fs::read(path.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();

        let registry = decode_registry(&chunks).unwrap();
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 13,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: Some(map.position_encoding()),
            extra_fields: false,
            corruption_check: false,
        };
        let native_trace: std::collections::BTreeMap<(i32, usize), serde_json::Value> =
            if trace_folder.is_some() {
                let rows: Vec<serde_json::Value> = serde_json::from_slice(
                    &fs::read("/private/tmp/halo-movement-trace.json").unwrap(),
                )
                .unwrap();
                rows.into_iter()
                    .map(|r| {
                        (
                            (
                                r["chunk"].as_i64().unwrap() as i32,
                                r["packet"].as_u64().unwrap() as usize,
                            ),
                            r,
                        )
                    })
                    .collect()
            } else {
                Default::default()
            };
        let movement = super::movement_states::scan_movement_states_observed(&chunks, &registry, &encoding, |source, index, frame| {
            if trace_folder.is_none() {return;}
            let native = &native_trace[&(source.chunk_index,index)];
            let records=native["records"].as_array().map(Vec::as_slice).unwrap_or_default();
            let expected:Vec<_>=records.iter().map(|r|(r["Type"].as_u64().unwrap(),r["ID"].as_u64().unwrap(),r["TypeIndex"].as_u64().unwrap(),r["Trace"]["EndBit"].as_u64().unwrap(),r["DesyncAt"] == -1)).collect();
            let actual:Vec<_>=frame.records.iter().map(|r|(match r.header.kind {RecordKind::New=>1,RecordKind::Delete=>2,_=>3},u64::from(r.header.id.unwrap()),u64::from(r.archetype.unwrap_or(0)),if r.header.kind==RecordKind::Delete{0}else{r.end_bit as u64},r.stop==EntityViewStop::Complete)).collect();
            let components_match = frame.records.iter().zip(records).all(|(actual, expected)| {
                let components = expected["Trace"]["Comps"].as_array().map(Vec::as_slice).unwrap_or_default();
                actual.attempts.len() == components.len() && actual.attempts.iter().zip(components).all(|(a, e)| {
                    a.span.index as u64 == e["Index"].as_u64().unwrap()
                        && a.span.name == e["Name"].as_str().unwrap()
                        && a.span.start_bit as u64 == e["StartBit"].as_u64().unwrap()
                        && a.status == e["Ported"].as_bool()
                })
            });
            if actual != expected || !components_match {
                fs::write("/private/tmp/halo-movement-first-diff.json", serde_json::to_vec_pretty(&serde_json::json!({"native":native,"rust":frame})).unwrap()).unwrap();
                panic!("frame {}:{index}: native end {}, Rust end {}; last Rust stop {:?}. Full trace: /private/tmp/halo-movement-first-diff.json", source.chunk_index, native["end"], frame.end_bit, frame.records.last().map(|r| &r.stop));
            }
        }).unwrap();
        println!("{folder}: movement {:?}", movement.stats);
        let expected: Vec<MovementStateRead> =
            serde_json::from_value(row["inputs"]["MovementStates"].clone()).unwrap();
        let expected_stats: MovementStateStats =
            serde_json::from_value(row["inputs"]["MovementStateStats"].clone()).unwrap();
        if movement.stats != expected_stats || movement.records != expected {
            println!("expected: {expected_stats:?}");
            failures.push(folder.to_string());
        }
    }
    assert!(failures.is_empty(), "movement mismatch: {failures:?}");
}

#[test]
#[ignore = "requires downloaded v41 film footer chunks"]
fn local_native_highlights_and_deaths() {
    use std::{collections::BTreeMap, io::Read};
    #[derive(serde::Deserialize)]
    struct Case {
        events: Option<Vec<NativeHighlightEvent>>,
        deaths: Option<Vec<IdentityDeath>>,
        error: bool,
        index: i32,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/highlights-corpus-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: BTreeMap<String, Case> = serde_json::from_slice(&raw).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films");
    let mut events_count = 0;
    let mut deaths_count = 0;
    for (name, case) in cases {
        let dir = root.join(&name);
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                let index = c["index"].as_i64().unwrap() as i32;
                FilmChunkData {
                    metadata: FilmChunk {
                        index,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: 0,
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: if index == case.index {
                        fs::read(dir.join(c["file"].as_str().unwrap())).unwrap()
                    } else {
                        Vec::new()
                    },
                }
            })
            .collect();
        let data = &chunks
            .iter()
            .find(|c| c.metadata.index == case.index)
            .unwrap()
            .data;
        let events = parse_highlight_events(data, 41).unwrap();
        events_count += events.len();
        assert_eq!(events, case.events.unwrap_or_default(), "{name}");
        let deaths = scan_film_deaths(&chunks, 41);
        assert_eq!(deaths.is_err(), case.error, "{name}");
        if let Ok(deaths) = deaths {
            deaths_count += deaths.len();
            assert_eq!(deaths, case.deaths.unwrap_or_default(), "{name}");
        }
    }
    println!("native highlights={events_count}, deaths={deaths_count}");
}

#[test]
#[ignore = "requires six downloaded films; compares native player, combat and vehicle assembly from film bytes"]
fn local_film_player_assembly() {
    use std::io::Read;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/film-players-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_film_assembly(rows);
}

#[test]
#[ignore = "requires the hour-long raid and its generated native oracle"]
fn local_film_raid_assembly() {
    use std::io::Read;
    let path = std::env::var_os("HALO_FILM_RAID_ORACLE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/theater/fixtures/film-raid-v41.json.zlib")
        });
    let bytes = fs::read(&path)
        .expect("generate the raid oracle with reference/generate_oracles.py --include-raid");
    let raw = if path.extension().is_some_and(|x| x == "json") {
        bytes
    } else {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(bytes.as_slice())
            .read_to_end(&mut raw)
            .unwrap();
        raw
    };
    assert_film_assembly(serde_json::from_slice(&raw).unwrap());
}

fn assert_film_assembly(rows: Vec<serde_json::Value>) {
    for row in rows {
        let folder = row["folder"].as_str().unwrap();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/films")
            .join(folder);
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: 0,
                    file_relative_path: String::new(),
                },
                data: fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let map_name = match folder {
            "maps/01-bazaar-idle" => "bazaar",
            "maps/02-aquarius" => "aquarius",
            "appearance/01-cadet-blue" | "appearance/02-cadet-brick" => "high ground",
            _ => "recharge",
        };
        let ids = RecordIdLayout {
            low_bits: 13,
            base: 0,
        };
        let film = if folder == "raids/01-hour-long-raid" {
            LegacyFilm::try_from_chunks_with_map_bounds(
                &chunks,
                DecodeOptions::v41(),
                "Facility Aetheria",
                serde_json::from_value(row["map"].clone()).unwrap(),
                ids,
            )
        } else {
            LegacyFilm::try_from_chunks_with_map(&chunks, DecodeOptions::v41(), map_name, ids)
        }
        .unwrap();
        // Exercise the portable LegacyFilm contract before comparing native assembly.
        // Expectations still come from the pinned native oracle, not this round trip.
        let portable = serde_json::to_vec(&film).unwrap();
        let restored: LegacyFilm = serde_json::from_slice(&portable)
            .unwrap_or_else(|error| panic!("portable Film {folder}: {error}"));
        assert_eq!(film, restored, "portable Film fields {folder}");
        if let Some(expected) = row.get("radial_scan") {
            let expected: NavpointRadialScan = serde_json::from_value(expected.clone()).unwrap();
            let explicit = LegacyFilm::try_from_chunks_with_encoding(
                &chunks,
                DecodeOptions::v41(),
                film.replication.as_ref().unwrap().encoding.clone(),
            )
            .unwrap();
            let explicit_restored: LegacyFilm =
                serde_json::from_slice(&serde_json::to_vec(&explicit).unwrap()).unwrap();
            assert_eq!(
                explicit_restored, explicit,
                "explicit portable Film {folder}"
            );
            let packets: std::collections::BTreeMap<_, _> = chunks
                .iter()
                .map(|c| {
                    (
                        c.metadata.index,
                        super::fire_events::native_chunk_packets(c),
                    )
                })
                .collect();
            for candidate in [&restored, &explicit_restored] {
                assert!(
                    candidate.navpoint_radial_error.is_none(),
                    "navpoint error {folder}"
                );
                let actual = candidate.navpoint_radial.as_ref().unwrap();
                if !expected.reads.is_empty() {
                    assert!(
                        !actual.attempts.is_empty(),
                        "positive retained trace {folder}"
                    );
                }
                let mut published = actual.clone();
                published.attempts.clear();
                assert_eq!(published, expected, "native navpoint Film data {folder}");
                for a in &actual.attempts {
                    let source = a.source.unwrap();
                    assert_eq!(
                        packets[&source.chunk_index][a.packet_index.unwrap()],
                        source
                    );
                }
            }
            assert_eq!(
                explicit_restored.navpoint_radial, restored.navpoint_radial,
                "complete navpoint trace across constructors {folder}"
            );
        }

        let result =
            build_film_replay_players(&restored, &chunks, FilmReplayPlayerOptions::default())
                .unwrap();
        if row["empty"] == true {
            assert!(result.is_none(), "{folder}");
            continue;
        }
        let result = result.unwrap();
        assert_eq!(
            result.clock,
            serde_json::from_value(row["clock"].clone()).unwrap(),
            "clock {folder}"
        );
        assert_eq!(
            result.origin_ms,
            serde_json::from_value(row["origin_ms"].clone()).unwrap(),
            "origin {folder}"
        );
        assert_eq!(result.duration_ms, row["duration_ms"].as_i64().unwrap());
        assert_eq!(
            result.frame_interval_ms,
            row["frame_interval_ms"].as_i64().unwrap()
        );
        let expected: Vec<ReplayTrack> = serde_json::from_value(row["tracks"].clone()).unwrap();
        assert_eq!(
            result.players.publication.tracks, expected,
            "tracks {folder}"
        );
        assert_eq!(
            result.players.bounds,
            serde_json::from_value(row["bounds"].clone()).unwrap(),
            "bounds {folder}"
        );
        assert_eq!(
            result.players.roster,
            serde_json::from_value::<Vec<ReplayRosterEntry>>(row["roster"].clone()).unwrap(),
            "roster {folder}"
        );
        assert_eq!(
            result.players.identity,
            serde_json::from_value(row["identity"].clone()).unwrap(),
            "identity {folder}"
        );
        assert_eq!(
            result.players.teams,
            serde_json::from_value(row["teams"].clone()).unwrap(),
            "teams {folder}"
        );
        assert_eq!(
            result.players.seats,
            serde_json::from_value(row["seats"].clone()).unwrap(),
            "seats {folder}"
        );
        let vehicles = build_film_replay_vehicles(&film, &result).unwrap();
        let expected_vehicles: ReplayVehiclePublication =
            serde_json::from_value(row["vehicles"].clone()).unwrap();
        assert_eq!(vehicles, expected_vehicles, "vehicles {folder}");
        let facts = film.native_vehicles.as_ref().unwrap();
        let scan = serde_json::json!({"positions": facts.positions.as_ref().map_or(0, |p| p.accepted().count()), "creations": facts.creations.as_ref().map_or(0, |c| c.records.len()), "aims": facts.aims.len(), "events": facts.events.len()});
        assert_eq!(scan, row["vehicle_scan"], "vehicle scan {folder}");
        eprintln!(
            "vehicles {folder}: {} tracks, {} rides",
            vehicles.tracks.len(),
            vehicles.coverage.rides
        );
        // The raid snapshot predates ability publication; its next regeneration
        // adds this layer. The six-film fixture requires it now.
        if folder != "raids/01-hour-long-raid" || row.get("ability_layer").is_some() {
            let abilities = build_film_replay_abilities(&film, &result).unwrap();
            assert_eq!(
                abilities,
                serde_json::from_value::<FilmReplayAbilities>(row["ability_layer"].clone())
                    .unwrap(),
                "ability publication {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("pickup_layer").is_some() {
            assert_eq!(
                build_film_replay_weapon_changes(&film, &result),
                serde_json::from_value::<ReplayWeaponChanges>(row["weapon_change_layer"].clone())
                    .unwrap(),
                "weapon changes {folder}"
            );
            let placements = build_film_replay_equipment_placements(&film, &result);
            assert_eq!(
                build_film_replay_pickups(
                    &film,
                    &result,
                    &placements,
                    FilmReplayPickupOptions::default()
                ),
                serde_json::from_value::<ReplayPickupOutput>(row["pickup_layer"].clone()).unwrap(),
                "pickup publication {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("skull_layer").is_some() {
            assert_eq!(
                build_film_replay_skull_carries(
                    &film,
                    &result,
                    folder == "ranked-arena/02-oddball"
                ),
                serde_json::from_value::<ReplaySkullCarries>(row["skull_layer"].clone()).unwrap(),
                "skull publication {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("carrier_marks").is_some() {
            assert_eq!(
                film.carrier_marks.as_ref(),
                Some(
                    &serde_json::from_value::<CarrierMarkScan>(row["carrier_marks"].clone())
                        .unwrap()
                ),
                "carrier marker scan {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("objective_object_layer").is_some() {
            assert_eq!(
                build_film_replay_objective_objects(&film, &result),
                serde_json::from_value::<ReplayObjectiveObjects>(
                    row["objective_object_layer"].clone()
                )
                .unwrap(),
                "objective object publication {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("score_layer").is_some() {
            assert_eq!(
                build_film_replay_score(&film, &result, FilmReplayScoreOptions::default()),
                serde_json::from_value::<ReplayScoreOutput>(row["score_layer"].clone()).unwrap(),
                "score publication {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("ground_layer").is_some() {
            let placements = build_film_replay_equipment_placements(&film, &result);
            let pickups = build_film_replay_pickups(
                &film,
                &result,
                &placements,
                FilmReplayPickupOptions::default(),
            );
            let ground = build_film_replay_ground(&film, &result, &pickups.pickups);
            let expected: FilmReplayGround =
                serde_json::from_value(row["ground_layer"].clone()).unwrap();
            assert_eq!(
                ground.coverage, expected.coverage,
                "ground coverage {folder}"
            );
            assert_eq!(
                ground.dating, expected.dating,
                "ground pickup dating {folder}"
            );
            assert_eq!(
                ground.items.coverage, expected.items.coverage,
                "ground item coverage {folder}"
            );
            assert_eq!(ground.pads, expected.pads, "ground pads {folder}");
            assert_eq!(ground.pickups, expected.pickups, "ground pickups {folder}");
            assert_eq!(
                ground.items.weapons.len(),
                expected.items.weapons.len(),
                "ground weapon count {folder}"
            );
            for (i, (actual, expected)) in ground
                .items
                .weapons
                .iter()
                .zip(&expected.items.weapons)
                .enumerate()
            {
                assert_eq!(actual, expected, "ground weapon {i} in {folder}");
            }
        }
        if folder != "raids/01-hour-long-raid" || row.get("placement_layer").is_some() {
            assert_eq!(
                build_film_replay_equipment_placements(&film, &result),
                serde_json::from_value::<ReplayEquipmentPlacements>(row["placement_layer"].clone())
                    .unwrap(),
                "equipment placements {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("episode_layer").is_some() {
            assert_eq!(
                build_film_replay_equipment_episodes(&film, &result),
                serde_json::from_value::<ReplayEquipmentEpisodes>(row["episode_layer"].clone())
                    .unwrap(),
                "equipment episodes {folder}"
            );
        }
        if folder != "raids/01-hour-long-raid" || row.get("grapple_layer").is_some() {
            assert_eq!(
                build_film_replay_grapple(&film, &result).unwrap(),
                serde_json::from_value::<ReplayGrapple>(row["grapple_layer"].clone()).unwrap(),
                "grapple publication {folder}"
            );
        }
        // These layers were added after the original raid snapshot.
        if folder != "raids/01-hour-long-raid" || row.get("equipment_layer").is_some() {
            assert_eq!(
                build_film_replay_equipment_changes(&film, &result),
                serde_json::from_value::<ReplayEquipmentChanges>(row["equipment_layer"].clone())
                    .unwrap(),
                "equipment changes {folder}"
            );
            assert_eq!(
                build_film_replay_translocations(&film, &result),
                serde_json::from_value::<ReplayTranslocations>(row["translocation_layer"].clone())
                    .unwrap(),
                "translocations {folder}"
            );
        }
        // Older raid snapshots lack this layer; regenerated snapshots must match it.
        if folder != "raids/01-hour-long-raid" || row.get("inventory_layer").is_some() {
            let inventory = build_film_replay_inventory(&film, &result).unwrap();
            assert_eq!(
                inventory,
                serde_json::from_value::<FilmReplayInventory>(row["inventory_layer"].clone())
                    .unwrap(),
                "inventory publication {folder}"
            );
            let mut unavailable = film.clone();
            unavailable.keyframe_inventory = None;
            assert_eq!(
                build_film_replay_inventory(&unavailable, &result).unwrap(),
                FilmReplayInventory::default(),
                "unavailable inventory {folder}"
            );
        }
        let shots = build_film_replay_shots(&film, &result).unwrap();
        let expected_shots: ReplayShotPublication =
            serde_json::from_value(row["shots"].clone()).unwrap();
        assert_eq!(shots, expected_shots, "shot publication {folder}");
        assert!(shots.coverage.balanced(), "shot coverage {folder}");
        let combat = build_film_replay_combat(&film, &result).unwrap();
        let expected_combat_shots: ReplayShotPublication =
            serde_json::from_value(row["combat_shots"].clone()).unwrap();
        assert_eq!(
            combat.shots, expected_combat_shots,
            "combined shots {folder}"
        );
        let mut expected_combat_vehicles = vehicles.clone();
        expected_combat_vehicles.coverage =
            serde_json::from_value(row["combat_vehicle_coverage"].clone()).unwrap();
        assert_eq!(
            combat.vehicles.as_ref(),
            Some(&expected_combat_vehicles),
            "combined vehicles {folder}"
        );
        assert_eq!(
            combat.vehicle_shot_verdict,
            serde_json::from_value::<Option<String>>(row["vehicle_shot_verdict"].clone()).unwrap(),
            "vehicle shot verdict {folder}"
        );
        let loads: Vec<ReplayLoadout> = serde_json::from_value(row["loadouts"].clone()).unwrap();
        assert_eq!(combat.loadouts, loads, "loadouts {folder}");
        let projectiles: ReplayProjectilePublication =
            serde_json::from_value(row["projectiles"].clone()).unwrap();
        assert_eq!(
            combat.projectiles.unwrap_or_default(),
            projectiles,
            "projectiles {folder}"
        );
        let grenades: ReplayGrenadePublication =
            serde_json::from_value(row["grenades"].clone()).unwrap();
        assert_eq!(
            combat.grenades.unwrap_or_default(),
            grenades,
            "grenades {folder}"
        );
        let grenade_reads: ReplayGrenadeReads =
            serde_json::from_value(row["grenade_reads"].clone()).unwrap();
        assert_eq!(
            combat.grenade_reads, grenade_reads,
            "carried grenades {folder}"
        );
        println!(
            "native player assembly {folder}: {} tracks, {} points",
            expected.len(),
            expected.iter().map(|t| t.points().len()).sum::<usize>()
        );
    }
}

#[test]
#[ignore = "requires six downloaded films; compares all raw native radial readings and scan counters"]
fn local_radial_scan() {
    use std::io::Read;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/film-players-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    for row in rows {
        if row.get("radial_scan").is_none() {
            continue;
        }
        let folder = row["folder"].as_str().unwrap();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/films")
            .join(folder);
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: 0,
                    file_relative_path: String::new(),
                },
                data: fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let registry = decode_registry(&chunks).unwrap();
        let identity = decode_film_identity(&chunks[0].data, &registry)
            .unwrap()
            .unwrap();
        let mut profile = resolve_v41_profile(&registry, Some(&identity), None).unwrap();
        profile.map = Some(serde_json::from_value(row["map"].clone()).unwrap());
        let encoding = profile
            .frame_encoding(RecordIdLayout {
                low_bits: 13,
                base: 0,
            })
            .unwrap();
        let scan = scan_navpoint_radial(
            &chunks,
            &registry,
            &encoding,
            &chunks
                .iter()
                .map(|c| (c.metadata.index, c.metadata.start_time_offset_ms))
                .collect(),
        )
        .unwrap();
        let expected: NavpointRadialScan =
            serde_json::from_value(row["radial_scan"].clone()).unwrap();
        assert_eq!(scan.reads, expected.reads, "radial readings {folder}");
        let mut published = scan.clone();
        published.attempts.clear();
        assert_eq!(published, expected, "radial coverage {folder}");
        let packets: std::collections::BTreeMap<_, _> = chunks
            .iter()
            .map(|c| {
                (
                    c.metadata.index,
                    super::fire_events::native_chunk_packets(c),
                )
            })
            .collect();
        for a in &scan.attempts {
            let source = a.source.unwrap();
            assert_eq!(
                packets[&source.chunk_index][a.packet_index.unwrap()],
                source
            );
        }
        assert_eq!(
            serde_json::from_value::<NavpointRadialScan>(serde_json::to_value(&scan).unwrap())
                .unwrap(),
            scan
        );
        println!(
            "radial {folder}: {} readings, {} deltas, {} keyframes",
            scan.reads.len(),
            scan.records,
            scan.key_records
        );
    }
}

#[test]
#[ignore = "requires six downloaded films; compares native managed-property reads and counters"]
fn local_managed_property_scan() {
    use std::io::Read;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/managed-property-corpus-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    for row in rows {
        let folder = row["folder"].as_str().unwrap();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/films")
            .join(folder);
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: 0,
                    file_relative_path: String::new(),
                },
                data: fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let registry = decode_registry(&chunks).unwrap();
        let identity = decode_film_identity(&chunks[0].data, &registry)
            .unwrap()
            .unwrap();
        let mut profile = resolve_v41_profile(&registry, Some(&identity), None).unwrap();
        profile.map = Some(serde_json::from_value(row["map"].clone()).unwrap());
        let encoding = profile
            .frame_encoding(RecordIdLayout {
                low_bits: 13,
                base: 0,
            })
            .unwrap();
        let scan = scan_managed_properties(&chunks, &registry, &encoding);
        if !row["error"].as_str().unwrap().is_empty() {
            assert!(scan.is_err(), "expected managed scan failure {folder}");
            continue;
        }
        let scan = scan.unwrap();
        let expected: ManagedPropertyScan = serde_json::from_value(row["scan"].clone()).unwrap();
        assert_eq!(scan.reads, expected.reads, "managed readings {folder}");
        let mut published = scan.clone();
        published.attempts.clear();
        assert_eq!(published, expected, "managed counters {folder}");
        for a in &scan.attempts {
            let source = a.source.unwrap();
            let chunk = chunks
                .iter()
                .find(|c| c.metadata.index == source.chunk_index)
                .unwrap();
            assert_eq!(
                super::fire_events::native_chunk_packets(chunk)[a.packet_index.unwrap()],
                source
            );
            assert_eq!(source.timestamp_us, a.timestamp_us);
            assert_eq!(
                a.in_bounds,
                a.component.end_bit <= (source.payload_size * 8) as i64
            );
        }
        assert_eq!(
            serde_json::from_value::<ManagedPropertyScan>(serde_json::to_value(&scan).unwrap())
                .unwrap(),
            scan
        );
        println!(
            "managed {folder}: {} reads, {} records",
            scan.reads.len(),
            scan.records
        );
    }
}

#[test]
#[ignore = "requires the downloaded Bazaar v41 film"]
fn local_source_roundtrip() {
    use std::io::Write;
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films/maps/01-bazaar-idle");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
    let mut clear = Vec::new();
    let mut mixed = Vec::new();
    let mut metadata = Vec::new();
    for (i, c) in manifest["chunks"].as_array().unwrap().iter().enumerate() {
        let bytes = fs::read(dir.join(c["file"].as_str().unwrap())).unwrap();
        metadata.push(FilmSourceMetadata {
            index: c["index"].as_i64().unwrap(),
            chunk_type: c["chunk_type"].as_i64().unwrap(),
            start_ms: c["start_time_offset_ms"].as_i64().unwrap(),
        });
        if i % 2 == 0 {
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&bytes).unwrap();
            mixed.push(encoder.finish().unwrap());
        } else {
            mixed.push(bytes.clone());
        }
        clear.push(bytes);
    }
    let raw = FilmSource::load(&clear, &metadata).unwrap();
    let loaded = FilmSource::load(&mixed, &metadata).unwrap();
    assert_eq!(raw.all_packets(), loaded.all_packets());
    for i in 0..raw.num_chunks() {
        assert_eq!(raw.chunk(i), loaded.chunk(i));
    }
    let expected =
        LegacyFilm::try_from_chunks(&raw.into_chunks().unwrap(), DecodeOptions::v41()).unwrap();
    let actual =
        LegacyFilm::try_from_chunks(&loaded.into_chunks().unwrap(), DecodeOptions::v41()).unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}

fn assert_inventory_parity(
    inventory: &InventoryDeltaStream,
    row: &serde_json::Value,
    folder: &str,
) {
    assert_eq!(
        serde_json::json!(inventory.stats),
        row["inventory_stats"],
        "inventory stats {folder}"
    );
    let expected_inventory = row["inputs"]["InventoryDeltas"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        inventory.records.len(),
        expected_inventory.len(),
        "inventory count {folder}: {:?}",
        inventory.stats
    );
    assert_eq!(
        serde_json::json!(inventory.stats.ammo_refused),
        row["inputs"]["InventoryDeltaAmmoRefused"]
    );
    for (a, e) in inventory.records.iter().zip(&expected_inventory) {
        let ammo:Vec<_>=a.ammo.iter().map(|v|serde_json::json!({"WeaponSlot":v.weapon_slot,"Mag":v.magazine,"FracQ":v.fraction_quantum,"Res":v.reserve})).collect();
        let ammo = if ammo.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::json!(ammo)
        };
        let sel = a
            .selection
            .as_ref()
            .map_or(0, |s| s.rank.map_or(-1, i32::from));
        assert_eq!(
            serde_json::json!({"Slot":a.slot,"Chunk":a.source.chunk_index,"PacketIndex":a.packet_index.expect("loaded inventory packet ordinal"),"TimestampUS":a.source.timestamp_us,"Grenades":a.grenades,"SelRead":a.selection.is_some(),"Sel":sel,"Mask":a.selection.as_ref().map_or(0,|s|s.mask),"Ammo":ammo}),
            *e,
            "inventory {folder}"
        );
    }
}

#[test]
#[ignore = "requires four downloaded films; focused native inventory fields and exported ordinals"]
fn inventory_scan_matches_production_films() {
    use std::io::Read;
    let mut json = String::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/gameplay-levelup-v41.json.zlib")[..])
        .read_to_string(&mut json)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(rows.len(), 4);
    let mut total = 0;
    for row in rows {
        let folder = row["folder"].as_str().unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/films")
            .join(folder);
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: c["decompressed_size"].as_i64().unwrap(),
                    file_relative_path: c["file_relative_path"].as_str().unwrap().into(),
                },
                data: fs::read(path.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
        let registry = decode_registry(&chunks).unwrap();
        let teleports = scan_translocator_events(&chunks, Some(&map)).unwrap();
        let stream =
            scan_biped_positions(&chunks, &map, BipedScanOptions::default(), &teleports).unwrap();
        let inventory =
            scan_inventory_deltas(&chunks, &registry, &stream, &map.position_encoding()).unwrap();
        assert_inventory_parity(&inventory, &row, folder);
        assert_biped_gameplay_parity(&chunks, &registry, &stream, &map, &row, folder);
        total += inventory.records.len();
        println!(
            "{folder}: {} inventory records matched",
            inventory.records.len()
        );
    }
    assert!(total > 0);
}

fn assert_biped_gameplay_parity(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    stream: &BipedPositionStream,
    map: &FilmMapBounds,
    row: &serde_json::Value,
    folder: &str,
) -> BipedChannels {
    let channels = scan_biped_channels(chunks, registry, stream, &map.position_encoding()).unwrap();
    let charges = scan_ability_charges(chunks, registry, stream, &map.position_encoding()).unwrap();
    let charge_records:Vec<_>=charges.records.iter().map(|a|serde_json::json!({"Slot":a.slot,"Chunk":a.source.chunk_index,"PacketIndex":a.packet_index.expect("loaded charge packet ordinal"),"TimestampUS":a.source.timestamp_us,"Emplacement":a.emplacement,"Charges":a.charges,"Low":a.low})).collect();
    assert_eq!(
        charge_records,
        row["inputs"]["AbilityCharges"]
            .as_array()
            .cloned()
            .unwrap_or_default(),
        "charges {folder}"
    );
    assert_eq!(
        serde_json::json!(charges.stats),
        row["inputs"]["AbilityChargeStats"],
        "charge stats {folder}"
    );
    println!("{folder}: {} charge reads", charge_records.len());
    let ability_states =
        scan_biped_ability_states(chunks, registry, stream, &map.position_encoding()).unwrap();
    let impulses:Vec<_>=ability_states.impulses.iter().map(|a|serde_json::json!({"Slot":a.slot,"Chunk":a.source.chunk_index,"PacketIndex":a.packet_index.expect("loaded ability packet ordinal"),"TimestampUS":a.source.timestamp_us,"Predicted":a.predicted})).collect();
    assert_eq!(
        impulses,
        row["inputs"]["AbilityImpulses"]
            .as_array()
            .cloned()
            .unwrap_or_default(),
        "impulses {folder}"
    );
    let grapple:Vec<_>=ability_states.grapple.iter().map(|a|serde_json::json!({"Slot":a.slot,"Chunk":a.source.chunk_index,"PacketIndex":a.packet_index.expect("loaded ability packet ordinal"),"TimestampUS":a.source.timestamp_us,"Heavy":a.heavy,"PosQ":a.position_quantized})).collect();
    assert_eq!(
        grapple,
        row["inputs"]["GrappleReads"]
            .as_array()
            .cloned()
            .unwrap_or_default(),
        "grapple {folder}"
    );
    assert_eq!(
        serde_json::json!(ability_states.impulse_stats),
        row["inputs"]["AbilityImpulseStats"],
        "impulse stats {folder}"
    );
    assert_eq!(
        serde_json::json!(ability_states.grapple_stats),
        row["grapple_stats"],
        "grapple stats {folder}"
    );
    println!(
        "{folder}: {} impulses, {} grapple reads",
        impulses.len(),
        grapple.len()
    );
    let s = &channels.camo_stats;
    assert_eq!(
        serde_json::json!({"Records":s.records,"WithI28":s.with_component,"Read":s.read,"Unread":s.unread,"NoChannel":s.gated}),
        row["camo_stats"]
    );
    let s = &channels.ability_stats;
    assert_eq!(
        serde_json::json!({"Records":s.records,"WithI48":s.with_component,"Read":s.read,"Unread":s.unread,"Gated":s.gated}),
        row["ability_stats"]
    );
    let camo: Vec<_> = channels.camo_states().map(|r| serde_json::json!({"Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.expect("loaded channel packet ordinal"),"TimestampUS":r.source.timestamp_us,"Q":r.quantum})).collect();
    let ranks: Vec<_> = channels.ability_ranks().map(|r| serde_json::json!({"Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index.expect("loaded channel packet ordinal"),"TimestampUS":r.source.timestamp_us,"Counter":r.counter,"Rank":r.rank.unwrap()})).collect();
    assert_eq!(
        camo,
        row["inputs"]["CamoStates"]
            .as_array()
            .cloned()
            .unwrap_or_default(),
        "camo {folder}"
    );
    assert_eq!(
        ranks,
        row["inputs"]["AbilityRanks"]
            .as_array()
            .cloned()
            .unwrap_or_default(),
        "ability ranks {folder}"
    );
    println!(
        "{folder}: {} camo readings, {} ability ranks; stats {:?} / {:?}",
        camo.len(),
        ranks.len(),
        channels.camo_stats,
        channels.ability_stats
    );
    channels
}
