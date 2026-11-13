use super::*;
use crate::theater::film::{Registry, ReplicationStream, Summary, SummaryHighlightPayload};

fn observation(timestamp_us: u64, change: ReplicationChange) -> ReplicationObservation {
    ReplicationObservation {
        timestamp_us,
        change,
    }
}

fn film(observations: Vec<ReplicationObservation>) -> Film {
    Film {
        registry: Registry {
            version: crate::theater::film::RegistryVersion {
                film: 41,
                format: 27,
            },
            components: vec![],
            options: crate::theater::film::RegistryOptions {
                game_mode: crate::theater::film::RegistryGameMode::Multiplayer,
                recorder_simulation: 0,
                level_tag: crate::theater::film::TagId(0),
            },
            map: crate::theater::film::RegistryMap {
                asset_id: crate::theater::film::MapAssetId([0; 16]),
                version_id: crate::theater::film::MapVersionId([0; 16]),
            },
        },
        replication_stream: ReplicationStream { observations },
        summary: Summary {
            highlights: vec![SummaryHighlight {
                gamertag: "Spartan".into(),
                xuid: crate::theater::film::Xuid(7),
                timestamp_us: 20,
                payload: SummaryHighlightPayload::Kill,
            }],
        },
    }
}

fn component(name: &str) -> ReplicationComponentChange {
    ReplicationComponentChange {
        index: ComponentSlot(3),
        name: name.into(),
        fields: vec![],
        weapon_id: None,
    }
}

#[test]
fn playback_applies_complete_changes_in_time_order_and_rewinds() {
    // Source order can differ from timestamp order; ties retain source order.
    let film = film(vec![
        observation(
            20,
            ReplicationChange::EntityUpdated {
                id: EntityId(9),
                fields: vec![],
                components: vec![component("second")],
            },
        ),
        observation(
            10,
            ReplicationChange::EntityCreated {
                id: EntityId(9),
                archetype: ArchetypeId(2),
                default_fields: vec![],
                fields: vec![],
                components: vec![component("first")],
            },
        ),
        observation(
            20,
            ReplicationChange::EntityUpdated {
                id: EntityId(9),
                fields: vec![],
                components: vec![component("last")],
            },
        ),
    ]);
    let mut runtime = TheaterRuntime::load(film);
    assert_eq!(
        runtime.summary_highlights()[0].xuid,
        crate::theater::film::Xuid(7)
    );
    assert_eq!(runtime.observations().len(), 3);
    assert!(runtime.current().entities.is_empty());
    let key = EntityId(9);
    assert_eq!(
        runtime.advance_to(10).entities[&key].components[&ComponentSlot(3)].name,
        "first"
    );
    assert_eq!(
        runtime.advance_to(20).entities[&key].components[&ComponentSlot(3)].name,
        "last"
    );
    assert!(runtime.seek(9).entities.is_empty());
    assert_eq!(
        runtime.advance_to(10).entities[&key].created_at_us,
        Some(10)
    );
    runtime.rewind();
    assert_eq!(runtime.timestamp_us(), None);
    assert!(runtime.current().entities.is_empty());
}

#[test]
fn generation_checks_prevent_stale_updates_and_deletes() {
    let mut runtime = TheaterRuntime::load(film(vec![
        observation(
            1,
            ReplicationChange::EntityCreated {
                id: EntityId(9),
                archetype: ArchetypeId(2),
                default_fields: vec![],
                fields: vec![],
                components: vec![component("created")],
            },
        ),
        observation(
            2,
            ReplicationChange::EntityCreated {
                id: EntityId((1 << 30) | 9),
                archetype: ArchetypeId(3),
                default_fields: vec![],
                fields: vec![],
                components: vec![component("new generation")],
            },
        ),
        observation(
            3,
            ReplicationChange::EntityUpdated {
                id: EntityId(9),
                fields: vec![],
                components: vec![component("stale")],
            },
        ),
        observation(4, ReplicationChange::EntityDeleted { id: EntityId(9) }),
    ]));
    let world = runtime.seek(4);
    assert!(!world.entities.contains_key(&EntityId(9)));
    let state = &world.entities[&EntityId((1 << 30) | 9)];
    assert_eq!(state.id, EntityId((1 << 30) | 9));
    assert_eq!(state.components[&ComponentSlot(3)].name, "new generation");
}

#[test]
fn keyframes_keep_recorded_ids_separate_from_live_entities() {
    let mut runtime = TheaterRuntime::load(film(vec![
        observation(
            1,
            ReplicationChange::Keyframe {
                id: KeyframeId((2 << 30) | 5),
                archetype: ArchetypeId(4),
                default_fields: vec![],
                fields: vec![],
                components: vec![component("baseline")],
            },
        ),
        observation(
            2,
            ReplicationChange::Keyframe {
                id: KeyframeId((3 << 30) | 5),
                archetype: ArchetypeId(7),
                default_fields: vec![],
                fields: vec![],
                components: vec![component("other")],
            },
        ),
        observation(
            3,
            ReplicationChange::EntityUpdated {
                id: EntityId(5),
                fields: vec![],
                components: vec![component("live update")],
            },
        ),
    ]));
    let world = runtime.seek(3);
    assert!(world.entities.is_empty());
    assert_eq!(
        world.keyframes[&KeyframeId((2 << 30) | 5)].components[&ComponentSlot(3)].name,
        "baseline"
    );
    assert_eq!(
        world.keyframes[&KeyframeId((3 << 30) | 5)].components[&ComponentSlot(3)].name,
        "other"
    );
}

#[test]
fn unusual_timestamps_do_not_reorder_observations() {
    let mut runtime = TheaterRuntime::load(film(vec![
        observation(
            20,
            ReplicationChange::EntityCreated {
                id: EntityId(9),
                archetype: ArchetypeId(2),
                default_fields: vec![],
                fields: vec![],
                components: vec![component("created")],
            },
        ),
        observation(
            10,
            ReplicationChange::EntityUpdated {
                id: EntityId(9),
                fields: vec![],
                components: vec![component("later in stream")],
            },
        ),
    ]));
    assert!(runtime.advance_to(10).entities.is_empty());
    assert_eq!(
        runtime.advance_to(20).entities[&EntityId(9)].components[&ComponentSlot(3)].name,
        "later in stream"
    );
    assert_eq!(
        runtime.seek(20).entities[&EntityId(9)].components[&ComponentSlot(3)].name,
        "later in stream"
    );
}

#[test]
fn playback_retains_empty_scalar_and_wide_values() {
    let fields = vec![
        ReplicationField {
            name: "selected_empty".into(),
            value: ReplicationValue::Empty,
        },
        ReplicationField {
            name: "scalar".into(),
            value: ReplicationValue::Scalar {
                value: 42,
                bit_len: 7,
            },
        },
        ReplicationField {
            name: "wide".into(),
            value: ReplicationValue::Wide {
                bit_len: 65,
                bytes: vec![0x80, 0, 0, 0, 0, 0, 0, 0, 0x80],
            },
        },
    ];
    let mut runtime = TheaterRuntime::load(film(vec![observation(
        1,
        ReplicationChange::EntityCreated {
            id: EntityId(9),
            archetype: ArchetypeId(2),
            default_fields: fields.clone(),
            fields: fields.clone(),
            components: vec![ReplicationComponentChange {
                index: ComponentSlot(3),
                name: "mixed".into(),
                fields: fields.clone(),
                weapon_id: None,
            }],
        },
    )]));
    let state = &runtime.seek(1).entities[&EntityId(9)];
    assert_eq!(state.default_fields, fields);
    assert_eq!(state.fields, fields);
    assert_eq!(state.components[&ComponentSlot(3)].fields, fields);
}
