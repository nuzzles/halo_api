use halo_api::clients::hi::models::{RawChunkKind, RawFilmChunk};
use halo_api::theater::Film;
use halo_api::theater::film::{
    EntitySchemaId, EntitySchemaKind, ParseError, RegistryGameMode, RegistryMapKind,
    ReplicationChange, ReplicationEvent, ReplicationEventKind, ReplicationValue,
    SummaryHighlightPayload, TagId, TagKind, TagLink, TagReference,
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
fn captured_films_parse_into_typed_sections() {
    for (name, observations, highlights, changes) in [
        ("bazaar_idle", 10_051, 0, [310, 6210, 29, 2401, 878, 223]),
        (
            "octagon_ar_kill",
            13_668,
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
            .fold([0usize; 3], |mut counts, event| {
                match event {
                    ReplicationEvent::DamageAftermath { .. } => counts[0] += 1,
                    ReplicationEvent::WeaponFire { .. } => counts[1] += 1,
                    ReplicationEvent::Kill { .. } => counts[2] += 1,
                    ReplicationEvent::Raw { .. } => {}
                }
                counts
            });
        assert_eq!(
            combat,
            if name == "bazaar_idle" {
                [0, 7, 0]
            } else {
                [19, 30, 0]
            },
            "{name}"
        );
        assert!(
            film.replication_stream
                .observations
                .iter()
                .all(|observation| {
                    !matches!(
                        &observation.change,
                        ReplicationChange::Event {
                            event: ReplicationEvent::Raw {
                                kind: ReplicationEventKind::Damage
                                    | ReplicationEventKind::WeaponFire
                                    | ReplicationEventKind::Kill,
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
    assert_eq!(highlights[0].medal_code, None);
    assert_eq!(highlights[1].medal_code, None);
    assert!(highlights[2].medal_code.is_some());
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
