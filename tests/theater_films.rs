use halo_api::clients::hi::models::{RawChunkKind, RawFilmChunk};
use halo_api::theater::Film;
use halo_api::theater::film::{
    EntitySchemaId, EntitySchemaKind, FilmParseOptions, ParseError, RegistryGameMode,
    RegistryMapKind, ReplicationChange, ReplicationEvent, ReplicationEventKind, ReplicationValue,
    SummaryHighlightPayload, TagId, TagKind, TagLink, TagReference, Threading,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Manifest {
    chunks: Vec<ManifestChunk>,
}

#[derive(Deserialize)]
struct ManifestChunk {
    kind: i32,
    index: i32,
    start_ms: i64,
    file: String,
}

fn chunks(name: &str) -> Vec<RawFilmChunk> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/theater")
        .join(name);
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(directory.join("manifest.json")).expect("golden film manifest"),
    )
    .expect("valid golden film manifest");
    manifest
        .chunks
        .into_iter()
        .map(|chunk| RawFilmChunk {
            kind: RawChunkKind(chunk.kind),
            index: chunk.index,
            start_ms: chunk.start_ms,
            data: std::fs::read(directory.join(chunk.file)).expect("golden film chunk"),
        })
        .collect()
}

#[test]
fn threading_modes_produce_identical_films() {
    for name in ["bazaar_idle", "octagon_ar_kill"] {
        let input = chunks(name);
        let default = Film::parse(input.clone()).expect(name);
        let single = Film::parse_with_options(
            input.clone(),
            FilmParseOptions {
                threading: Threading::Single,
            },
        )
        .expect(name);
        let multi = Film::parse_with_options(
            input,
            FilmParseOptions {
                threading: Threading::Multi,
            },
        )
        .expect(name);
        assert_eq!(single, multi, "{name}");
        assert_eq!(default, multi, "{name}");
    }
}

#[test]
fn replication_packet_boundaries_preserve_observations_across_chunks() {
    use std::io::Write;
    let original = chunks("bazaar_idle");
    let expected = Film::parse_with_options(
        original.clone(),
        FilmParseOptions {
            threading: Threading::Single,
        },
    )
    .unwrap();
    let mut split = Vec::new();
    for chunk in original {
        if chunk.kind != RawChunkKind::REPLICATION {
            split.push(chunk);
            continue;
        }
        let inflated = chunk.clone().inflate().unwrap();
        let mut packets = Vec::new();
        let mut offset = 0;
        while offset < inflated.data.len() {
            let header = &inflated.data[offset..offset + 16];
            let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
            let end = offset + 16 + size;
            assert!(end <= inflated.data.len());
            packets.push(offset..end);
            offset = end;
        }
        // More than eight replication chunks exercises speculative workers as
        // well as the sequential path, with bindings carried across boundaries.
        for group in packets.chunks(packets.len().div_ceil(9).max(1)) {
            let bytes = &inflated.data[group[0].start..group.last().unwrap().end];
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
            encoder.write_all(bytes).unwrap();
            let mut part = chunk.clone();
            part.data = encoder.finish().unwrap();
            split.push(part);
        }
    }
    assert!(split.len() >= 10);
    for (index, chunk) in split.iter_mut().enumerate() {
        chunk.index = index as i32;
    }
    for threading in [Threading::Single, Threading::Multi] {
        let actual =
            Film::parse_with_options(split.clone(), FilmParseOptions { threading }).unwrap();
        assert_eq!(actual, expected, "{threading:?}");
    }
}

#[test]
fn recorded_players_and_settings() {
    for name in ["bazaar_idle", "octagon_ar_kill"] {
        let film = Film::parse(chunks(name)).unwrap();
        let settings = &film.registry.settings;
        assert_eq!(settings.build, "HI_1_13_0");
        assert_eq!(settings.name, "Slayer");
        assert_eq!(
            settings.description,
            "Control Power Weapons to eliminate the enemy."
        );
        let mode_options = &settings.mode_options;
        assert_eq!(mode_options.minimum_respawn_seconds, 8);
        assert_eq!(
            mode_options.time_limit_seconds,
            Some(if name == "bazaar_idle" { 720 } else { 60 })
        );
        assert_eq!(
            mode_options.score_to_win,
            if name == "bazaar_idle" {
                Some(50)
            } else {
                None
            }
        );
        let weapons = &mode_options.starting_loadouts[0].weapons;
        assert_eq!(weapons.len(), 3);
        use halo_api::theater::film::{StartingWeapon, WeaponTagId};
        assert_eq!(weapons[0], StartingWeapon::Real(WeaponTagId(1220648237)));
        assert_eq!(weapons[1], StartingWeapon::Real(WeaponTagId(4094171407)));
        assert_eq!(weapons[2], StartingWeapon::Unknown);
        assert_eq!(
            mode_options.starting_loadouts[0].equipment,
            halo_api::theater::film::StartingEquipment::None
        );
        assert_eq!(mode_options.options.len(), 10);
        assert_eq!(
            mode_options.options[0].value,
            halo_api::theater::film::MatchOptionValue::NumberBits(0.6f32.to_bits())
        );
        let initial = &film.registry.players;
        assert_eq!(initial.len(), 32);
        assert!(
            initial
                .iter()
                .enumerate()
                .all(|(i, p)| p.slot as usize == i)
        );
        let occupied: Vec<_> = initial
            .iter()
            .filter(|p| !p.player.gamertag.is_empty())
            .collect();
        assert_eq!(occupied.len(), if name == "bazaar_idle" { 1 } else { 2 });
        assert_eq!(occupied[0].player.gamertag, "Nuzzles");
        assert_eq!(occupied[0].xuid.0, 2535472547643888);
        for p in &occupied {
            let appearance = &p.player.customization.spartan;
            assert!(appearance.regions.is_empty());
            assert_eq!(appearance.variant_name, 0);
            assert_eq!(appearance.coating_name, 0);
            assert_eq!(appearance.action_pose, 0);
        }
        let mut snapshots = 0;
        for observation in &film.replication_stream.observations {
            if let ReplicationChange::Roster { players } = &observation.change {
                snapshots += 1;
                assert_eq!(players.len(), occupied.len());
                for p in players {
                    let initial = occupied.iter().find(|i| i.xuid == p.xuid).unwrap();
                    assert_eq!(initial.player.gamertag, p.player.gamertag);
                    let appearance = &p.player.customization.spartan;
                    assert_eq!(appearance.regions.len(), 22);
                    assert_eq!(appearance.attachments.len(), 24);
                    assert_eq!(appearance.objects.len(), 7);
                    if p.player.gamertag == "Nuzzles" {
                        assert_eq!(appearance.variant_name, 0x101f4ba9);
                        assert_eq!(appearance.style_name, 0x79666886);
                        assert_eq!(appearance.theme_name, 0x1dbd4399);
                        assert_eq!(appearance.coating_name, 0x8de0ee80);
                        assert_eq!(appearance.action_pose, 0x4abd4c24);
                    } else {
                        assert_eq!(p.player.gamertag, "Yet");
                        assert_eq!(appearance.coating_name, 1369882871);
                    }
                }
            }
        }
        assert!(snapshots > 0);
        let json = serde_json::to_vec(&film).unwrap();
        assert_eq!(serde_json::from_slice::<Film>(&json).unwrap(), film);
    }
}

#[test]
fn runtime_replays_roster_snapshots_without_player_entity_inference() {
    use halo_api::theater::{FilmRuntime, film::ReplicationObservation};
    let mut film = Film::parse(chunks("bazaar_idle")).unwrap();
    let players = film
        .replication_stream
        .observations
        .iter()
        .find_map(|o| match &o.change {
            ReplicationChange::Roster { players } if !players.is_empty() => Some(players.clone()),
            _ => None,
        })
        .unwrap();
    film.replication_stream.observations = vec![
        ReplicationObservation {
            timestamp_us: 10,
            change: ReplicationChange::Roster {
                players: players.clone(),
            },
        },
        ReplicationObservation {
            timestamp_us: 15,
            change: ReplicationChange::Bots { players: vec![] },
        },
        ReplicationObservation {
            timestamp_us: 20,
            change: ReplicationChange::Roster { players: vec![] },
        },
    ];
    let mut runtime = FilmRuntime::load(film);
    assert!(runtime.current().roster.is_none());
    assert!(runtime.advance_to(9).roster.is_none());
    assert_eq!(runtime.advance_to(10).roster.as_ref(), Some(&players));
    assert_eq!(runtime.advance_to(15).bots, Some(vec![]));
    assert!(runtime.current().entities.is_empty());
    assert_eq!(runtime.advance_to(20).roster, Some(vec![]));
    assert_eq!(runtime.seek(10).roster.as_ref(), Some(&players));
    assert!(runtime.current().bots.is_none());
    assert!(runtime.advance_to(9).roster.is_none());
    runtime.advance_to(20);
    runtime.rewind();
    assert!(runtime.current().roster.is_none());
    assert!(runtime.current().bots.is_none());
}

#[test]
fn initial_players_are_required_in_registry_json() {
    use halo_api::theater::film::Registry;
    let film = Film::parse(chunks("bazaar_idle")).unwrap();
    let mut json = serde_json::to_value(&film.registry).unwrap();
    assert_eq!(json["players"].as_array().unwrap().len(), 32);
    assert_eq!(
        serde_json::from_value::<Registry>(json.clone()).unwrap(),
        film.registry
    );
    json["players"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<Registry>(json.clone()).is_err());
    json.as_object_mut().unwrap().remove("players");
    assert!(serde_json::from_value::<Registry>(json).is_err());
}

#[test]
fn parse_rejects_an_unreadable_initial_player_layout() {
    use std::io::{Read, Write};
    let mut input = chunks("bazaar_idle");
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(input[0].data.as_slice())
        .read_to_end(&mut data)
        .unwrap();
    // Change only the fixture's recorded build, leaving its framing and options intact.
    let build = b"HI_1_13_0\0";
    let offsets: Vec<_> = data
        .windows(build.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == build).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1);
    let start = offsets[0];
    data[start..start + build.len()].copy_from_slice(b"HI_9_99_9\0");
    let mut writer = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    writer.write_all(&data).unwrap();
    input[0].data = writer.finish().unwrap();
    assert!(matches!(
        Film::parse(input),
        Err(ParseError::InvalidCustomization)
    ));
}

#[test]
fn serialized_settings_require_decoded_fields_but_allow_recorded_absence() {
    use halo_api::theater::film::{
        MatchOption, MatchSettings, ModeOptions, Registry, StartingLoadout,
    };
    let film = Film::parse(chunks("bazaar_idle")).unwrap();
    use halo_api::theater::film::{KnownMatchOption, MatchOptionName, MatchOptionValue};
    let options = &film.registry.settings.mode_options.options;
    let score = options
        .iter()
        .find(|o| o.name == MatchOptionName::Known(KnownMatchOption::SlayerScorePerKill))
        .unwrap();
    assert_eq!(score.name.hash().raw(), 0x185f4770);
    assert_eq!(score.value, MatchOptionValue::NumberBits(1.0_f32.to_bits()));
    let unknown = options
        .iter()
        .find(|o| {
            o.name
                == MatchOptionName::Unknown {
                    name_hash: 0x8d9d0ac5,
                }
        })
        .unwrap();
    assert_eq!(
        unknown.value,
        MatchOptionValue::NumberBits(0.6_f32.to_bits())
    );
    let json = serde_json::to_value(score).unwrap();
    assert_eq!(
        json["name"],
        serde_json::json!({"Known": "SlayerOptions_scorePerKill"})
    );
    assert!(json.get("name_hash").is_none());
    assert_eq!(serde_json::from_value::<MatchOption>(json).unwrap(), *score);
    let settings = serde_json::to_value(&film.registry.settings).unwrap();
    let mode_options = &settings["mode_options"];
    assert_eq!(
        serde_json::from_value::<ModeOptions>(mode_options.clone()).unwrap(),
        film.registry.settings.mode_options
    );
    assert_eq!(
        serde_json::from_value::<MatchSettings>(settings.clone()).unwrap(),
        film.registry.settings
    );
    for field in mode_options.as_object().unwrap().keys() {
        assert!(
            settings.get(field).is_none(),
            "mode option leaked to metadata: {field}"
        );
    }
    for (field, value) in [
        ("round_limit", serde_json::json!(3)),
        ("rounds_to_win", serde_json::json!(2)),
        ("rounds_to_draw", serde_json::json!(2)),
        ("machinima_controls_enabled", serde_json::json!(false)),
    ] {
        assert!(mode_options.get(field).is_some());
        let mut populated = settings.clone();
        populated["mode_options"][field] = value;
        let decoded: MatchSettings = serde_json::from_value(populated.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), populated);
        for absent in [false, true] {
            let mut missing = settings.clone();
            if absent {
                missing["mode_options"]
                    .as_object_mut()
                    .unwrap()
                    .remove(field);
            } else {
                missing["mode_options"][field] = serde_json::Value::Null;
            }
            let decoded: MatchSettings = serde_json::from_value(missing).unwrap();
            assert!(serde_json::to_value(decoded).unwrap()["mode_options"][field].is_null());
        }
    }
    assert!(mode_options.get("last_chance_enabled").is_none());
    assert!(mode_options.get("last_chance_seconds").is_none());
    assert!(
        mode_options
            .get("sudden_death_enabled_in_overtime")
            .is_some()
    );
    assert!(mode_options.get("sudden_death_seconds").is_some());
    let mut sudden_death = settings.clone();
    sudden_death["mode_options"]["sudden_death_enabled_in_overtime"] = serde_json::json!(true);
    sudden_death["mode_options"]["sudden_death_seconds"] = serde_json::json!(30);
    let decoded: MatchSettings = serde_json::from_value(sudden_death.clone()).unwrap();
    assert_eq!(
        decoded.mode_options.sudden_death_enabled_in_overtime,
        Some(true)
    );
    assert_eq!(decoded.mode_options.sudden_death_seconds, Some(30));
    assert_eq!(serde_json::to_value(decoded).unwrap(), sudden_death);
    for invalid in [
        serde_json::json!(1700000000),
        serde_json::json!("not a timestamp"),
    ] {
        let mut settings = settings.clone();
        settings["recording_start_time"] = invalid;
        assert!(serde_json::from_value::<MatchSettings>(settings).is_err());
    }
    for field in [
        "build",
        "recording_start_time",
        "name",
        "description",
        "engine_type",
        "mode_options",
    ] {
        let mut missing = settings.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<MatchSettings>(missing).is_err(),
            "{field}"
        );
        let mut null = settings.clone();
        null[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<MatchSettings>(null).is_err(),
            "{field}"
        );
    }
    for field in [
        "minimum_respawn_seconds",
        "self_destruction_respawn_penalty_seconds",
        "betrayal_respawn_penalty_seconds",
        "starting_loadouts",
        "options",
    ] {
        for absent in [false, true] {
            let mut invalid = settings.clone();
            if absent {
                invalid["mode_options"]
                    .as_object_mut()
                    .unwrap()
                    .remove(field);
            } else {
                invalid["mode_options"][field] = serde_json::Value::Null;
            }
            assert!(
                serde_json::from_value::<ModeOptions>(invalid["mode_options"].clone()).is_err(),
                "{field}"
            );
            assert!(
                serde_json::from_value::<MatchSettings>(invalid).is_err(),
                "{field}"
            );
        }
    }
    for field in ["name", "value"] {
        let mut missing = mode_options["options"][0].clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<MatchOption>(missing).is_err(),
            "{field}"
        );
        let mut null = mode_options["options"][0].clone();
        null[field] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<MatchOption>(null).is_err(),
            "{field}"
        );
    }
    for field in ["weapons", "equipment"] {
        let mut loadout = mode_options["starting_loadouts"][0].clone();
        loadout.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<StartingLoadout>(loadout).is_err());
        let mut loadout = mode_options["starting_loadouts"][0].clone();
        loadout[field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<StartingLoadout>(loadout).is_err());
    }
    let mut registry = serde_json::to_value(&film.registry).unwrap();
    registry.as_object_mut().unwrap().remove("settings");
    assert!(serde_json::from_value::<Registry>(registry).is_err());
}

#[test]
fn invalid_roster_customization_fails_the_whole_film_in_both_threading_modes() {
    use std::io::{Read, Write};
    let mut input = chunks("bazaar_idle");
    let chunk = &mut input[1];
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(chunk.data.as_slice())
        .read_to_end(&mut data)
        .unwrap();
    // Locate this fixture's recorded variant/style pair even when bit-unaligned.
    // Its count lives 0x13c bytes earlier in the same appearance block.
    let needle = [0x101f4ba9u32.to_le_bytes(), 0x79666886u32.to_le_bytes()].concat();
    let bit = |bytes: &[u8], i: usize| (bytes[i / 8] >> (7 - i % 8)) & 1;
    let anchor = (0x13c * 8..=data.len() * 8 - needle.len() * 8)
        .find(|start| (0..needle.len() * 8).all(|i| bit(&data, start + i) == bit(&needle, i)))
        .expect("fixture appearance variant/style");
    let count_start = anchor - 0x13c * 8;
    let original = 22u32.to_le_bytes();
    assert!((0..32).all(|i| bit(&data, count_start + i) == bit(&original, i)));
    let invalid = 34u32.to_le_bytes();
    for i in 0..32 {
        let offset = count_start + i;
        let mask = 1 << (7 - offset % 8);
        data[offset / 8] = (data[offset / 8] & !mask) | (bit(&invalid, i) << (7 - offset % 8));
    }
    let mut writer = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    writer.write_all(&data).unwrap();
    chunk.data = writer.finish().unwrap();
    // Exercise sequential, parallel projection, and speculative chunk admission.
    for replication_count in [1, 3, 8] {
        let original = chunks("bazaar_idle");
        let mut baseline = vec![original[0].clone()];
        for index in 1..=replication_count {
            let mut chunk = original[1].clone();
            chunk.index = index;
            baseline.push(chunk);
        }
        let mut summary = original.last().unwrap().clone();
        summary.index = replication_count + 1;
        baseline.push(summary);
        for threading in [Threading::Single, Threading::Multi] {
            Film::parse_with_options(baseline.clone(), FilmParseOptions { threading }).unwrap();
            let mut corrupted = baseline.clone();
            corrupted[1] = input[1].clone();
            assert!(matches!(
                Film::parse_with_options(corrupted, FilmParseOptions { threading }),
                Err(ParseError::InvalidCustomization)
            ));
        }
    }
}

fn local_corpus_manifests() -> Vec<PathBuf> {
    fn visit(path: &std::path::Path, manifests: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(&entry.path(), manifests);
            } else if entry.file_name() == "film.json" {
                manifests.push(entry.path());
            }
        }
    }
    let root = std::env::var_os("HALO_FILM_CORPUS").expect("HALO_FILM_CORPUS");
    let mut manifests = vec![];
    visit(std::path::Path::new(&root), &mut manifests);
    manifests.sort();
    assert!(!manifests.is_empty());
    manifests
}

fn local_corpus_chunks(manifest: &std::path::Path) -> Vec<RawFilmChunk> {
    use std::io::Write;
    let json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    json["chunks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let mut data =
                std::fs::read(manifest.parent().unwrap().join(c["file"].as_str().unwrap()))
                    .unwrap();
            // Some corpus entries store inflated bytes despite their .zlib suffix.
            let compressed = data
                .get(..2)
                .is_some_and(|h| h[0] & 15 == 8 && u16::from_be_bytes([h[0], h[1]]) % 31 == 0);
            if !compressed {
                let mut writer =
                    flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
                writer.write_all(&data).unwrap();
                data = writer.finish().unwrap();
            }
            RawFilmChunk {
                kind: RawChunkKind(c["chunk_type"].as_i64().unwrap() as i32),
                index: c["index"].as_i64().unwrap() as i32,
                start_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                data,
            }
        })
        .collect()
}

/// Audit retained fields and ensure retired payloads are not published.
#[test]
#[ignore = "set HALO_FILM_CORPUS to audit event field presence in local and checked-in films"]
fn local_corpus_event_field_presence() {
    const FIELDS: &[&str] = &[
        "WeaponReload/player",
        "Throw/player",
        "Melee/weapon",
        "Melee/player",
        "Pickup/item",
        "DamageAftermath/victim",
        "DamageAftermath/attacker",
        "DamageAftermath/effect",
        "DamageAftermath/attacker_player",
        "WeaponFire/shooter",
        "WeaponFire/player",
        "WeaponFire/weapon",
        "ProjectileDetonation/position",
        "ProjectileImpact/position",
    ];
    #[derive(Default)]
    struct Counts {
        present: usize,
        absent: usize,
        first_absent: Option<String>,
    }
    let mut counts: Vec<Counts> = FIELDS.iter().map(|_| Counts::default()).collect();
    let mut kinds = std::collections::BTreeMap::<String, usize>::new();
    let manifests = local_corpus_manifests();
    let mut audit = |name: &str, input| {
        let film = Film::parse(input).unwrap_or_else(|e| panic!("{name}: {e}"));
        for observation in &film.replication_stream.observations {
            let ReplicationChange::Event { event } = &observation.change else {
                continue;
            };
            let json = serde_json::to_value(event).unwrap();
            if let ReplicationEvent::ProjectileDetonation { position }
            | ReplicationEvent::ProjectileImpact { position } = event
            {
                assert!(
                    position.iter().all(|axis| axis.is_finite()),
                    "{name}: invalid XYZ"
                );
                let body = json.as_object().unwrap().values().next().unwrap();
                assert_eq!(body["position"].as_array().unwrap().len(), 3);
            }
            for retired in [
                "/Kill",
                "/Teleport",
                "/Throw/item",
                "/ProjectileDetonation/data",
                "/ProjectileImpact/data",
                "/ProjectileDetonation/variant",
                "/ProjectileDetonation/tag",
                "/ProjectileImpact/variant",
            ] {
                assert!(
                    json.pointer(retired).is_none(),
                    "{name}: retained {retired}"
                );
            }
            let kind = json.as_object().unwrap().keys().next().unwrap();
            *kinds.entry(kind.clone()).or_default() += 1;
            for (path, count) in FIELDS.iter().zip(&mut counts) {
                let (parent, field) = path.rsplit_once('/').unwrap();
                let Some(parent) = json.pointer(&format!("/{parent}")).filter(|p| !p.is_null())
                else {
                    continue;
                };
                let value = if let Some(array) = parent.as_array() {
                    &array[field.parse::<usize>().unwrap()]
                } else {
                    parent
                        .get(field)
                        .unwrap_or_else(|| panic!("missing audit field {path}"))
                };
                if value.is_null() {
                    count.absent += 1;
                    count.first_absent.get_or_insert_with(|| {
                        format!("{name} @ {}us: {json}", observation.timestamp_us)
                    });
                } else {
                    count.present += 1;
                }
            }
        }
        eprintln!("audited {name}");
    };
    for name in ["bazaar_idle", "octagon_ar_kill"] {
        audit(name, chunks(name));
    }
    for manifest in &manifests {
        audit(
            &manifest.display().to_string(),
            local_corpus_chunks(manifest),
        );
    }
    eprintln!("{} films; event counts: {kinds:?}", manifests.len() + 2);
    for (path, count) in FIELDS.iter().zip(counts) {
        eprintln!(
            "{path}: present={} absent={} first_absent={}",
            count.present,
            count.absent,
            count.first_absent.as_deref().unwrap_or("none observed")
        );
    }
}

/// Opt-in regression against a local corpus with film.json manifests. No downloads.
#[test]
#[ignore = "set HALO_FILM_CORPUS to a directory of local film.json fixtures"]
fn local_corpus_preserves_recorded_metadata_across_threading_modes() {
    let manifests = local_corpus_manifests();
    let mut populations = 0;
    let mut event_counts = [0usize; 7];
    for manifest in &manifests {
        let input = local_corpus_chunks(manifest);
        let single = Film::parse_with_options(
            input.clone(),
            FilmParseOptions {
                threading: Threading::Single,
            },
        )
        .unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
        let multi = Film::parse_with_options(
            input,
            FilmParseOptions {
                threading: Threading::Multi,
            },
        )
        .unwrap();
        assert_eq!(single, multi, "{}", manifest.display());
        assert_eq!(single.registry.players.len(), 32, "{}", manifest.display());
        assert!(
            !single.registry.settings.build.is_empty(),
            "{}",
            manifest.display()
        );
        for o in &single.replication_stream.observations {
            match &o.change {
                ReplicationChange::Roster { players } => {
                    populations += 1;
                    for p in players {
                        assert_eq!(p.player.customization.spartan.attachments.len(), 24);
                    }
                }
                ReplicationChange::Event { event } => {
                    let index = match event {
                        ReplicationEvent::WeaponReload { .. } => 0,
                        ReplicationEvent::Throw { .. } => 1,
                        ReplicationEvent::Melee { .. } => 2,
                        ReplicationEvent::Pickup { .. } => 3,
                        ReplicationEvent::Zoom { .. } => 4,
                        ReplicationEvent::ProjectileDetonation { .. } => 5,
                        ReplicationEvent::ProjectileImpact { .. } => 6,
                        ReplicationEvent::Other { kind }
                            if matches!(kind.code(), 5 | 6 | 9 | 21 | 38 | 39 | 40) =>
                        {
                            panic!("{}: event {kind:?} was not projected", manifest.display())
                        }
                        _ => continue,
                    };
                    event_counts[index] += 1;
                }
                _ => {}
            }
        }
        eprintln!("validated {}", manifest.display());
    }
    eprintln!(
        "{} films, {populations} rosters, new typed events: {event_counts:?}",
        manifests.len()
    );
    assert!(populations > 0);
    // A user's corpus need not contain every retained event kind.
    assert!(event_counts.iter().any(|n| *n > 0));
}

#[test]
fn captured_films_parse_into_typed_sections() {
    for (name, observations, highlights, changes) in [
        ("bazaar_idle", 10_059, 0, [310, 6210, 29, 2401, 878, 223]),
        (
            "octagon_ar_kill",
            13_680,
            3,
            [223, 6058, 3, 6134, 1098, 152],
        ),
    ] {
        let film = Film::parse(chunks(name)).expect(name);
        let combat = film
            .replication_stream
            .observations
            .iter()
            .filter_map(|observation| match &observation.change {
                ReplicationChange::Event { event } => Some(event),
                _ => None,
            })
            .fold([0usize; 2], |mut counts, event| {
                match event {
                    ReplicationEvent::DamageAftermath { .. } => counts[0] += 1,
                    ReplicationEvent::WeaponFire { .. } => counts[1] += 1,
                    _ => {}
                }
                counts
            });
        assert_eq!(
            combat,
            if name == "bazaar_idle" {
                [0, 7]
            } else {
                [19, 30]
            },
            "{name}"
        );
        let mut references = [0usize; 3];
        for observation in &film.replication_stream.observations {
            match &observation.change {
                ReplicationChange::Event {
                    event:
                        ReplicationEvent::DamageAftermath {
                            victim, attacker, ..
                        },
                } => {
                    assert!(matches!(victim.kind, 1 | 4));
                    references[0] += 1;
                    references[1] += usize::from(attacker.is_some());
                }
                ReplicationChange::Event {
                    event: ReplicationEvent::WeaponFire { shooter, .. },
                } => {
                    assert!(matches!(shooter.kind, 1 | 4));
                    references[2] += 1;
                }
                _ => {}
            }
        }
        assert!(
            references[2] > 0,
            "{name}: lost recorded shooter references"
        );
        if name == "octagon_ar_kill" {
            assert!(references[0] > 0, "lost recorded victim references");
            assert!(references[1] > 0, "lost recorded attacker references");
        }
        assert!(
            film.replication_stream
                .observations
                .iter()
                .all(|observation| {
                    !matches!(
                        &observation.change,
                        ReplicationChange::Event {
                            event: ReplicationEvent::Other {
                                kind: ReplicationEventKind::Damage
                                    | ReplicationEventKind::WeaponFire,
                                ..
                            }
                        }
                    )
                }),
            "{name}: a complete combat event was not typed"
        );
        assert_eq!(film.registry.version.film, 41, "{name}");
        assert_eq!(film.registry.version.format, 27, "{name}");
        assert_eq!(
            film.registry.options.game_mode,
            RegistryGameMode::Multiplayer
        );
        assert_eq!(film.registry.schemas.len(), 50, "{name}");
        let expected_map = if name == "bazaar_idle" {
            RegistryMapKind::Bazaar
        } else {
            // This custom Octagon asset has no entry in the bundled catalogue.
            RegistryMapKind::Unknown
        };
        assert_eq!(film.registry.map.kind, expected_map, "{name}");
        for (index, kind) in [
            (5, EntitySchemaKind::PlayerState),
            (6, EntitySchemaKind::StatTracker),
            (9, EntitySchemaKind::Player),
            (11, EntitySchemaKind::ManagedObjective),
            (17, EntitySchemaKind::MusicState),
            (18, EntitySchemaKind::EffectState),
            (19, EntitySchemaKind::SoundPlacement),
            (2, EntitySchemaKind::ManagedEngineObject),
            (10, EntitySchemaKind::ManagedEngineObject),
            (35, EntitySchemaKind::Biped),
            (40, EntitySchemaKind::Vehicle),
            (8, EntitySchemaKind::Unknown),
        ] {
            let schema = film
                .registry
                .schemas
                .iter()
                .find(|schema| schema.index == EntitySchemaId(index))
                .unwrap();
            assert_eq!(schema.kind, kind, "{name}, schema {index}");
        }
        assert_eq!(
            film.registry
                .schemas
                .iter()
                .filter(|schema| schema.kind != EntitySchemaKind::Unknown)
                .count(),
            49,
            "{name}"
        );
        assert_eq!(
            film.replication_stream.observations.len(),
            observations,
            "{name}"
        );
        assert_eq!(film.summary.highlights.len(), highlights, "{name}");
        assert!(
            film.replication_stream
                .observations
                .windows(2)
                .all(|pair| pair[0].timestamp_us <= pair[1].timestamp_us),
            "{name}"
        );
        let mut actual = [0; 6];
        for observation in &film.replication_stream.observations {
            if let ReplicationChange::EntityCreated {
                schema,
                schema_kind,
                ..
            }
            | ReplicationChange::Keyframe {
                schema,
                schema_kind,
                ..
            } = &observation.change
            {
                let recorded = film
                    .registry
                    .schemas
                    .iter()
                    .find(|entry| entry.index == *schema)
                    .map_or(EntitySchemaKind::Unknown, |entry| entry.kind);
                assert_eq!(*schema_kind, recorded, "{name}, schema {schema}");
            }
            let slot = match observation.change {
                ReplicationChange::EntityCreated { .. } => 0,
                ReplicationChange::EntityUpdated { .. } => 1,
                ReplicationChange::EntityDeleted { .. } => 2,
                ReplicationChange::Control { .. } => 3,
                ReplicationChange::Keyframe { .. } => 4,
                ReplicationChange::Event { .. } => 5,
                ReplicationChange::Roster { .. } | ReplicationChange::Bots { .. } => continue,
            };
            actual[slot] += 1;
        }
        assert_eq!(actual, changes, "{name}");
    }
}

#[test]
fn parsed_tags_embed_catalogue_reference_data() {
    use halo_api::theater::film::catalog::items::Weapon;

    let film = Film::parse(chunks("bazaar_idle")).unwrap();
    let mut tags = Vec::new();
    let mut record = |fields: &[halo_api::theater::film::ReplicationField]| {
        tags.extend(fields.iter().filter_map(|field| match &field.value {
            ReplicationValue::Tag { .. } => Some(field.value.clone()),
            _ => None,
        }));
    };
    for observation in &film.replication_stream.observations {
        match &observation.change {
            ReplicationChange::EntityCreated {
                default_fields,
                fields,
                components,
                ..
            }
            | ReplicationChange::Keyframe {
                default_fields,
                fields,
                components,
                ..
            } => {
                record(default_fields);
                record(fields);
                for component in components {
                    record(&component.fields);
                }
            }
            ReplicationChange::EntityUpdated {
                fields, components, ..
            } => {
                record(fields);
                for component in components {
                    record(&component.fields);
                }
            }
            _ => {}
        }
    }
    let cindershot = tags
        .iter()
        .find(
            |value| matches!(value, ReplicationValue::Tag { id, .. } if *id == TagId(0x2304_47b1)),
        )
        .expect("captured Cindershot tag");
    let ReplicationValue::Tag {
        id,
        kind,
        reference: Some(reference),
    } = cindershot
    else {
        panic!("catalogued Cindershot details must be in the film");
    };
    assert_eq!(*id, TagId(0x2304_47b1));
    assert_eq!(*kind, TagKind::Weapon(Weapon::Cindershot));
    let TagReference::Firing(firing) = reference.as_ref() else {
        panic!("Cindershot tag must carry firing reference data");
    };
    assert_eq!(firing.rounds_per_second, 1.1);
    assert_eq!(firing.magazine, Some([6, 12, 24]));
    let Some(TagLink::Known(projectile)) = &firing.projectile else {
        panic!("linked projectile data");
    };
    assert_eq!(projectile.tag, TagId(0x8331_4c19));
    assert_eq!(projectile.velocity, 20.0);
    let Some(TagLink::Known(impact)) = &projectile.impact else {
        panic!("linked damage effect");
    };
    assert_eq!(impact.tag, TagId(0xa073_779a));
    assert_eq!(
        impact.cause,
        halo_api::theater::film::DamageCause::Weapon { weapon: None }
    );
    let json = serde_json::to_string(cindershot).unwrap();
    assert_eq!(
        serde_json::from_str::<ReplicationValue>(&json).unwrap(),
        *cindershot
    );
}

#[test]
fn tag_reference_catalogues_do_not_overlap() {
    use halo_api::theater::film::catalog::Catalog;
    use std::collections::HashSet;

    let catalog = Catalog;
    let mut tags = HashSet::new();
    for tag in catalog.firing_catalogue().iter().map(|row| row.tag) {
        assert!(tags.insert(tag), "duplicate firing tag {tag:?}");
    }
    for tag in catalog.projectile_catalogue().iter().map(|row| row.tag) {
        assert!(tags.insert(tag), "overlapping projectile tag {tag:?}");
    }
    for tag in catalog.damage_effects().iter().map(|row| row.tag) {
        assert!(tags.insert(tag), "overlapping damage tag {tag:?}");
    }
    for tag in catalog.vehicle_models().iter().map(|row| row.0) {
        assert!(tags.insert(tag), "overlapping vehicle tag {tag:?}");
    }
}

#[test]
fn captured_film_exposes_classified_object_tags() {
    let film = Film::parse(chunks("bazaar_idle")).unwrap();
    let contains_tag = |fields: &[halo_api::theater::film::ReplicationField]| {
        fields
            .iter()
            .any(|field| matches!(field.value, ReplicationValue::Tag { .. }))
    };
    assert!(
        film.replication_stream
            .observations
            .iter()
            .any(|observation| {
                match &observation.change {
                    ReplicationChange::EntityCreated {
                        default_fields,
                        fields,
                        components,
                        ..
                    }
                    | ReplicationChange::Keyframe {
                        default_fields,
                        fields,
                        components,
                        ..
                    } => {
                        contains_tag(default_fields)
                            || contains_tag(fields)
                            || components
                                .iter()
                                .any(|component| contains_tag(&component.fields))
                    }
                    ReplicationChange::EntityUpdated {
                        fields, components, ..
                    } => {
                        contains_tag(fields)
                            || components
                                .iter()
                                .any(|component| contains_tag(&component.fields))
                    }
                    _ => false,
                }
            })
    );
}

#[test]
fn captured_kill_film_has_typed_highlights() {
    let film = Film::parse(chunks("octagon_ar_kill")).unwrap();
    let highlights = &film.summary.highlights;
    assert_eq!(highlights.len(), 3);
    assert_eq!(highlights[0].payload, SummaryHighlightPayload::Kill);
    assert_eq!(highlights[1].payload, SummaryHighlightPayload::Death);
    assert!(matches!(
        highlights[2].payload,
        SummaryHighlightPayload::Medal(_)
    ));
}

#[test]
fn parse_orders_manifest_chunks_and_rejects_missing_or_duplicate_indices() {
    let original = chunks("bazaar_idle");
    let expected = Film::parse(original.clone()).unwrap();
    let mut reversed = original.clone();
    reversed.reverse();
    assert_eq!(Film::parse(reversed).unwrap(), expected);

    let mut duplicate = original.clone();
    duplicate.push(original[1].clone());
    assert!(matches!(
        Film::parse(duplicate),
        Err(ParseError::DuplicateIndex { index: 1 })
    ));

    let mut missing = original;
    missing.remove(1);
    assert!(matches!(
        Film::parse(missing),
        Err(ParseError::MissingIndex { index: 1 })
    ));
}

#[test]
fn parse_rejects_missing_summary_and_damaged_zlib() {
    let mut no_summary = chunks("bazaar_idle");
    no_summary.pop();
    assert!(matches!(
        Film::parse(no_summary),
        Err(ParseError::SummaryNotLast)
    ));

    let mut damaged = chunks("bazaar_idle");
    damaged[1].data.truncate(2);
    assert!(matches!(
        Film::parse(damaged),
        Err(ParseError::Inflate { index: 1, .. })
    ));

    let mut damaged_both = chunks("bazaar_idle");
    let summary_index = damaged_both.len() - 1;
    damaged_both[1].data.truncate(2);
    damaged_both[summary_index].data.truncate(2);
    assert!(matches!(
        Film::parse(damaged_both),
        Err(ParseError::Inflate { index: 1, .. })
    ));

    let mut damaged_summary = chunks("bazaar_idle");
    damaged_summary[summary_index].data.truncate(2);
    assert!(matches!(
        Film::parse(damaged_summary),
        Err(ParseError::Inflate { index, .. }) if index == summary_index
    ));
}

#[test]
fn parse_rejects_unsupported_kind_and_version() {
    let mut unknown_kind = chunks("bazaar_idle");
    unknown_kind[1].kind = RawChunkKind(99);
    assert!(matches!(
        Film::parse(unknown_kind),
        Err(ParseError::UnsupportedChunkKind(99))
    ));

    let mut unsupported_version = chunks("bazaar_idle");
    let mut registry = Vec::new();
    std::io::copy(
        &mut flate2::read::ZlibDecoder::new(&unsupported_version[0].data[..]),
        &mut registry,
    )
    .unwrap();
    registry[..4].copy_from_slice(&75u32.to_le_bytes());
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    std::io::Write::write_all(&mut encoder, &registry).unwrap();
    unsupported_version[0].data = encoder.finish().unwrap();
    assert!(matches!(
        Film::parse(unsupported_version),
        Err(ParseError::UnsupportedVersion(75))
    ));
}
