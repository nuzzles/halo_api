use super::*;
use crate::theater::film::*;
use crate::theater::parser::{
    EntityComponentAttempt, EntityComponentSpan, FilmSource, RecordHeader,
};

fn recording() -> Film {
    let source = FilmSource::load(
        &[
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            [vec![0, 0, 0, 0], 16u32.to_le_bytes().to_vec(), vec![0; 24]].concat(),
        ],
        &[],
    )
    .unwrap();
    Film::parse(&source, ParseOptions::default()).unwrap()
}

// Independently authored decoded-record fixtures test resolution, not decoding.
fn entity(kind: RecordKind, id: u32, value: u64) -> EntityRecord {
    EntityRecord {
        references: vec![],
        diagnostics: Default::default(),
        header: RecordHeader {
            kind,
            id: Some(id),
            start_bit: 2,
            end_bit: 36,
        },
        archetype: Some(3),
        default_state_bits: None,
        default_state_fallback: false,
        binding_origin: None,
        mask: Some(1),
        fields: vec![ComponentField {
            name: "value".into(),
            bit: 40,
            width: 8,
            raw: value,
        }],
        components: vec![],
        attempts: vec![EntityComponentAttempt {
            span: EntityComponentSpan {
                index: 0,
                name: "test-component".into(),
                start_bit: 40,
                end_bit: 48,
            },
            variant: None,
            status: Some(true),
            observation_range: None,
            field_start: 0,
            field_end: 1,
        }],
        end_bit: 48,
        padded_bits: 0,
        stop: EntityViewStop::Complete,
    }
}
fn packet(film: &Film, timestamp_us: u64, records: Vec<EntityRecord>) -> NativeFilmPacket {
    // Parse a real packet shell, then substitute explicit resolution fixtures.
    let mut p = film.chunks[1].packets[0].clone();
    p.header.timestamp_us = timestamp_us;
    p.header.payload_size = 256;
    p.body = NativeFilmPacketBody::Frame(Box::new(ProductionFrame {
        header_diagnostics: Default::default(),
        admission_diagnostics: None,
        record_prefixes: vec![],
        messages: None,
        records,
        controls: None,
        entity_end: None,
        views_completed: 1,
        end_bit: 48,
        padded_bits: 0,
    }));
    p.event_continuation = None;
    p.event_list = None;
    p
}
fn key() -> EntityKey {
    EntityKey {
        domain: EntityDomain::Runtime,
        slot: 7,
    }
}
fn value(world: &WorldSnapshot) -> u64 {
    world.entities[&key()].components[&0].fields[0].raw
}

#[test]
fn resolved_native_entry_preserves_source_and_decoder_output() {
    use std::io::Write;
    let bootstrap = [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat();
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&bootstrap).unwrap();
    let mut compressed = encoder.finish().unwrap();
    compressed.extend_from_slice(b"trailing transport");
    let source = FilmSource::load(
        &[
            compressed.clone(),
            [vec![0, 0, 0, 0], 16u32.to_le_bytes().to_vec(), vec![0; 24]].concat(),
        ],
        &[],
    )
    .unwrap();
    let native = Film::parse_v41(&source).unwrap();
    let film = Film::parse(&source, Default::default()).unwrap();
    assert_eq!(film, native);
    assert_eq!(film.original_chunks[0], compressed);
    assert_eq!(film.chunks[0].data, bootstrap);
    let json = serde_json::to_vec(&film).unwrap();
    let resolved = film.resolve();
    let second = film.resolve();
    assert!(std::ptr::eq(resolved.film(), &film));
    assert!(std::ptr::eq(second.film(), &film));
    assert_eq!(serde_json::to_vec(&film).unwrap(), json);
    assert_eq!(serde_json::to_vec(resolved.film()).unwrap(), json);
    assert!(resolved.current().entities.is_empty());
}

#[test]
fn resolved_chronology_generations_filters_and_source_references() {
    let mut film = recording();
    let id = 0x4000_0007;
    film.chunks[1].packets = vec![
        packet(&film, 20, vec![entity(RecordKind::Delta, id, 2)]),
        packet(&film, 10, vec![entity(RecordKind::New, id, 1)]),
        packet(
            &film,
            20,
            vec![
                entity(RecordKind::Delete, id, 0),
                entity(RecordKind::New, 0x8000_0007, 3),
            ],
        ),
        packet(&film, 30, vec![entity(RecordKind::Delete, id, 0)]), // stale generation
    ];
    let mut resolved = film.resolve();
    assert_eq!(resolved.events()[0].source.packet, 1);
    assert_eq!(value(resolved.advance_to(10)), 1);
    assert_eq!(value(resolved.advance_to(20)), 3);
    assert_eq!(value(resolved.advance_to(30)), 3);
    let tied: Vec<_> = resolved
        .query(EventFilter {
            start_us: Some(20),
            end_us: Some(20),
            ..Default::default()
        })
        .collect();
    assert_eq!(
        tied.iter().map(|e| e.order).collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4]
    );
    let delta = resolved
        .query(EventFilter {
            kind: Some(EventKind::EntityDelta),
            entity_id: Some(id),
            ..Default::default()
        })
        .next()
        .unwrap();
    assert!(
        matches!(resolved.record(delta.source), Some(Record::Entity(r)) if r.fields[0].raw == 2)
    );
    let change = delta.change.as_ref().unwrap();
    assert_eq!(
        change.previous.as_ref().unwrap().components[&0].fields[0].raw,
        1
    );
    assert_eq!(change.new.as_ref().unwrap().components[&0].fields[0].raw, 2);
    assert_eq!(
        resolved
            .query(EventFilter {
                start_us: Some(30),
                end_us: Some(10),
                ..Default::default()
            })
            .count(),
        0
    );
}

#[test]
fn resolved_seek_matches_sequential_across_checkpoints_and_ties() {
    let mut film = recording();
    let id = 0x4000_0007;
    film.chunks[1].packets = (0..1200)
        .map(|i| {
            packet(
                &film,
                i / 3,
                vec![entity(
                    if i == 0 {
                        RecordKind::New
                    } else {
                        RecordKind::Delta
                    },
                    id,
                    i,
                )],
            )
        })
        .collect();
    let mut sequential = film.resolve();
    let mut random = sequential.clone();
    let mut expected = BTreeMap::new();
    for t in [0, 1, 170, 171, 340, 341, 399, 1000] {
        expected.insert(t, sequential.advance_to(t).clone());
    }
    for t in [399, 0, 340, 171, 1000, 1, 341, 170] {
        assert_eq!(random.seek(t), &expected[&t], "seek {t}");
    }
    assert_eq!(random.advance_to(1), &expected[&1]);
    random.rewind();
    assert_eq!(random.timestamp_us(), None);
    assert!(random.current().entities.is_empty());
    assert_eq!(random.advance_to(0), &expected[&0]);
}

#[test]
fn resolved_unknowns_partial_updates_and_padding_remain_explicit() {
    let mut film = recording();
    let id = 0x4000_0007;
    let mut partial = entity(RecordKind::Delta, id, 2);
    partial.attempts[0].status = None;
    partial.stop = EntityViewStop::Truncated;
    let mut padded = entity(RecordKind::Delta, id, 99);
    padded.padded_bits = 1;
    let mut opaque = packet(&film, 15, vec![]);
    opaque.body = NativeFilmPacketBody::Opaque;
    film.chunks[1].packets = vec![
        packet(&film, 10, vec![entity(RecordKind::New, id, 1)]),
        opaque,
        packet(&film, 20, vec![partial]),
        packet(&film, 30, vec![padded]),
    ];
    let mut resolved = film.resolve();
    assert_eq!(value(resolved.advance_to(30)), 2);
    assert!(!resolved.current().entities[&key()].components[&0].complete);
    assert!(
        resolved
            .events()
            .iter()
            .any(|e| matches!(resolved.record(e.source),
        Some(Record::Packet(p)) if matches!(p.body, NativeFilmPacketBody::Opaque)))
    );
    assert!(resolved.events().last().unwrap().change.is_none());
}

#[test]
fn resolved_keyframe_baselines_do_not_invent_runtime_generation_or_spawn_time() {
    use crate::theater::parser::{KeyframeChainAttempt, KeyframeChainStop, NativeKeyframeTable};
    let mut film = recording();
    let mut keyframe = packet(&film, 10, vec![]);
    keyframe.body = NativeFilmPacketBody::Keyframes(NativeKeyframeTable {
        records: vec![KeyframeChainAttempt {
            start_bit: 1,
            end_bit: 65,
            id: 0x8000_0007,
            archetype: 3,
            record: Some(KeyframeRecord {
                references: vec![],
                diagnostics: Default::default(),
                start_bit: 1,
                end_bit: 65,
                id: 0x8000_0007,
                archetype: 3,
                fields: vec![],
                components: vec![],
                attempts: vec![],
                stop: KeyframeStop::Complete,
            }),
        }],
        stop: KeyframeChainStop::End,
        diagnostics: Default::default(),
    });
    let mut padded = keyframe.clone();
    padded.header.timestamp_us = 30;
    if let NativeFilmPacketBody::Keyframes(table) = &mut padded.body {
        table.records[0].record.as_mut().unwrap().end_bit = 4096;
    }
    film.chunks[1].packets = vec![
        keyframe,
        packet(&film, 20, vec![entity(RecordKind::Delta, 0x4000_0007, 8)]),
        padded,
    ];
    let mut resolved = film.resolve();
    let baseline = &resolved.advance_to(10).entities[&key()];
    assert_eq!(baseline.id, None);
    assert_eq!(baseline.created_at_us, None);
    assert_eq!(value(resolved.advance_to(20)), 8);
    assert_eq!(resolved.current().entities[&key()].id, Some(0x4000_0007));
    assert_eq!(resolved.current().entities[&key()].created_at_us, None);
    assert_eq!(value(resolved.advance_to(30)), 8);
    assert_eq!(
        resolved.events().last().unwrap().provenance,
        Provenance::PartialRead
    );
}

#[test]
fn resolved_rejected_new_binding_does_not_replace_existing_entity() {
    let mut film = recording();
    let id = 0x4000_0007;
    let mut rejected = packet(&film, 20, vec![entity(RecordKind::New, id, 99)]);
    if let NativeFilmPacketBody::Frame(frame) = &mut rejected.body {
        frame.header_diagnostics.new_binding_refusals.push(
            crate::theater::parser::NativeNewBindingRefusal {
                record_bit: 2,
                id,
                slot: 7,
                existing_archetype: 3,
                proposed_archetype: 4,
            },
        );
    }
    film.chunks[1].packets = vec![
        packet(&film, 10, vec![entity(RecordKind::New, id, 1)]),
        rejected,
    ];
    let mut resolved = film.resolve();
    assert_eq!(value(resolved.advance_to(20)), 1);
    assert!(resolved.events().last().unwrap().change.is_none());
}

#[test]
fn resolved_incomplete_new_and_padded_control_are_not_recorded_state() {
    let mut film = recording();
    let mut incomplete = entity(RecordKind::New, 0x4000_0007, 99);
    incomplete.stop = EntityViewStop::Truncated;
    let mut p = packet(&film, 10, vec![incomplete]);
    if let NativeFilmPacketBody::Frame(frame) = &mut p.body {
        frame.controls = Some(crate::theater::parser::DecodedFrameView {
            control_entries: vec![NativeControlEntry {
                start_bit: 2040,
                end_bit: 2050,
                index: 2,
                baseline: None,
                short: None,
                analog: None,
                third_analog: None,
                extra: None,
                flags: None,
                action: None,
            }],
            diagnostics: None,
            start_bit: 2040,
            end_bit: 2050,
            padded_bits: 2,
            kinds: vec![0],
            fields: vec![],
            stop: crate::theater::parser::FrameViewStop::Truncated,
        });
    }
    film.chunks[1].packets = vec![p];
    let mut resolved = film.resolve();
    assert!(resolved.advance_to(10).entities.is_empty());
    for event in resolved
        .events()
        .iter()
        .filter(|e| e.kind != EventKind::Packet)
    {
        assert_eq!(event.provenance, Provenance::PartialRead);
        assert!(event.change.is_none());
    }
    assert_eq!(
        resolved
            .query(EventFilter {
                player_index: Some(2),
                category: Some(EventCategory::Input),
                ..Default::default()
            })
            .count(),
        1
    );
}

#[test]
fn resolved_recovery_candidates_never_create_world_entities() {
    use crate::theater::parser::{AnchorRecovery, RecoveredKeyframeAnchor};
    let mut film = recording();
    let mut p = packet(&film, 10, vec![]);
    p.body = NativeFilmPacketBody::Opaque;
    p.keyframe_candidates = Some(vec![NativeKeyframeCandidate {
        anchor: RecoveredKeyframeAnchor {
            id: 0x4000_0007,
            archetype: 3,
            bit: 1,
            recovery: AnchorRecovery::Scan { from_bit: 0 },
        },
        read: Ok(KeyframeRecord {
            references: vec![],
            diagnostics: Default::default(),
            start_bit: 1,
            end_bit: 65,
            id: 0x4000_0007,
            archetype: 3,
            fields: vec![],
            components: vec![],
            attempts: vec![],
            stop: KeyframeStop::Complete,
        }),
        crosses_next_anchor: false,
    }]);
    film.chunks[1].packets = vec![p];
    let mut resolved = film.resolve();
    assert!(resolved.advance_to(10).entities.is_empty());
    assert_eq!(resolved.events().len(), 1);
    assert!(
        resolved.film().chunks[1].packets[0]
            .keyframe_candidates
            .is_some()
    );
}

#[test]
fn resolved_query_indices_intersect_filters_and_preserve_order() {
    let mut film = recording();
    film.chunks[1].packets = vec![
        packet(
            &film,
            10,
            vec![entity(RecordKind::New, 7, 1), entity(RecordKind::New, 8, 2)],
        ),
        packet(
            &film,
            20,
            vec![
                entity(RecordKind::Delta, 8, 3),
                entity(RecordKind::Delta, 7, 4),
            ],
        ),
        packet(&film, 30, vec![entity(RecordKind::Delete, 7, 0)]),
    ];
    let resolved = film.resolve();
    assert_eq!(resolved.query_indices.entities[&7].len(), 3);
    let rows: Vec<_> = resolved
        .query(EventFilter {
            start_us: Some(15),
            end_us: Some(25),
            entity_id: Some(7),
            kind: Some(EventKind::EntityDelta),
            category: Some(EventCategory::State),
            ..Default::default()
        })
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].timestamp_us, 20);
    assert!(
        matches!(resolved.record(rows[0].source), Some(Record::Entity(r)) if r.fields[0].raw == 4)
    );
    assert_eq!(
        resolved
            .query(EventFilter {
                entity_id: Some(999),
                ..Default::default()
            })
            .count(),
        0
    );
    assert_eq!(
        resolved
            .query(EventFilter {
                player_index: Some(999),
                ..Default::default()
            })
            .count(),
        0
    );
    assert_eq!(
        resolved
            .query(EventFilter {
                entity_id: Some(7),
                category: Some(EventCategory::Input),
                ..Default::default()
            })
            .count(),
        0
    );
    assert_eq!(
        resolved
            .query(EventFilter {
                entity_id: Some(7),
                start_us: Some(30),
                end_us: Some(10),
                ..Default::default()
            })
            .count(),
        0
    );
}
