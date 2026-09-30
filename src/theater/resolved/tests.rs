use super::*;
use crate::theater::film::*;
use crate::theater::parser::v41::test_chunks;
use crate::theater::parser::v41::{FixtureFilmSource, RecordHeader};

fn replication_chunk_mut(film: &mut Film) -> &mut ReplicationStreamChunk {
    film.chunks
        .iter_mut()
        .find_map(|chunk| match chunk {
            FilmDataChunk::Replication(chunk) => Some(chunk),
            _ => None,
        })
        .unwrap()
}

fn recording() -> Film {
    let source = FixtureFilmSource::load(
        &[
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            [vec![0, 0, 0, 0], 16u32.to_le_bytes().to_vec(), vec![0; 24]].concat(),
        ],
        &[],
    )
    .unwrap();
    Film::parse(test_chunks(&source)).unwrap()
}

// Independently authored decoded-record fixtures test resolution, not decoding.
fn entity(kind: RecordKind, id: u32, value: u64) -> EntityRecord {
    EntityRecord {
        default_state: None,
        header: RecordHeader {
            prefix: None,
            kind,
            id: Some(id),
            start_bit: 2,
            end_bit: 36,
        },
        archetype: (kind == RecordKind::New).then_some(3),
        mask: Some(1),
        fields: vec![],
        components: vec![EntityComponentRead {
            index: 0,
            name: "test-component".into(),
            start_bit: 40,
            end_bit: 48,
            status: ComponentReadStatus::Complete,
            fields: vec![ComponentField {
                name: "value".into(),
                bit: 40,
                width: 8,
                raw: RawBits::from_low(value, 8),
            }],
        }],
        end_bit: 48,
        stop: EntityViewStop::Complete,
    }
}
fn packet(film: &Film, timestamp_us: u64, records: Vec<EntityRecord>) -> ReplicationStreamPacket {
    // Parse a real packet shell, then substitute explicit resolution fixtures.
    let mut p = film.replication_chunks().next().unwrap().body.packets[0].clone();
    p.header.timestamp_us = timestamp_us;
    p.header.payload_size = 256;
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &mut p.body
    else {
        panic!("missing frame packet")
    };
    frame.frame = FrameRead::Decoded(Box::new(ProductionFrame {
        messages: None,
        records,
        controls: None,
        entity_end: None,
        end_bit: 48,
    }));
    p
}
fn key() -> EntityKey {
    EntityKey {
        domain: EntityDomain::Runtime,
        slot: 7,
    }
}
fn value(world: &WorldSnapshot) -> u64 {
    world.entities[&key()].components[&0].fields[0]
        .raw
        .low_u64()
}

#[test]
fn resolved_reference_entry_preserves_source_and_decoder_output() {
    use std::io::Write;
    let bootstrap = [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat();
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&bootstrap).unwrap();
    let mut compressed = encoder.finish().unwrap();
    compressed.extend_from_slice(b"trailing transport");
    let source = FixtureFilmSource::load(
        &[
            compressed.clone(),
            [vec![0, 0, 0, 0], 16u32.to_le_bytes().to_vec(), vec![0; 24]].concat(),
        ],
        &[],
    )
    .unwrap();
    let reference = Film::parse(test_chunks(&source)).unwrap();
    let film = Film::parse(test_chunks(&source)).unwrap();
    assert_eq!(film, reference);
    assert_eq!(film.registry.source.data, compressed);
    assert_eq!(film.registry.data, bootstrap);
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
    replication_chunk_mut(&mut film).body.packets = vec![
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
        matches!(resolved.record(delta.source), Some(Record::Entity(r)) if r.components[0].fields[0].raw.low_u64() == 2)
    );
    let change = delta.change.as_ref().unwrap();
    assert_eq!(
        change.previous.as_ref().unwrap().components[&0].fields[0]
            .raw
            .low_u64(),
        1
    );
    assert_eq!(
        change.new.as_ref().unwrap().components[&0].fields[0]
            .raw
            .low_u64(),
        2
    );
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
    replication_chunk_mut(&mut film).body.packets = (0..1200)
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
    partial.components[0].status = ComponentReadStatus::Truncated;
    partial.stop = EntityViewStop::Truncated;
    let mut padded = entity(RecordKind::Delta, id, 99);
    padded.end_bit = 4096;
    let mut opaque = packet(&film, 15, vec![]);
    opaque.body = PacketRead::Opaque {
        reason: PacketDecodeError::UnsupportedLayout { packet_type: 99 },
    };
    replication_chunk_mut(&mut film).body.packets = vec![
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
        Some(Record::ReplicationPacket(p)) if matches!(p.body, PacketRead::Opaque { .. })))
    );
    assert!(resolved.events().last().unwrap().change.is_none());
}

#[test]
fn resolved_keyframe_baselines_do_not_invent_runtime_generation_or_spawn_time() {
    use crate::theater::parser::v41::{KeyframeChainAttempt, KeyframeChainStop, KeyframeTable};
    let mut film = recording();
    let mut keyframe = packet(&film, 10, vec![]);
    keyframe.body = PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(
        Box::new(KeyframeTable {
            records: vec![KeyframeChainAttempt {
                start_bit: 1,
                end_bit: 65,
                id: 0x8000_0007,
                archetype: 3,
                record: Some(KeyframeRecord {
                    default_state: None,
                    start_bit: 1,
                    end_bit: 65,
                    id: 0x8000_0007,
                    archetype: 3,
                    fields: vec![],
                    components: vec![],
                    stop: KeyframeStop::Complete,
                }),
            }],
            stop: KeyframeChainStop::End,
        }),
    ));
    let mut padded = keyframe.clone();
    padded.header.timestamp_us = 30;
    if let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
        &mut padded.body
    {
        table.records[0].record.as_mut().unwrap().end_bit = 4096;
    }
    replication_chunk_mut(&mut film).body.packets = vec![
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
    let rejected = packet(&film, 20, vec![entity(RecordKind::New, id, 99)]);
    replication_chunk_mut(&mut film).body.packets = vec![
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
    if let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &mut p.body {
        frame.frame.decoded_mut().unwrap().controls =
            Some(crate::theater::parser::v41::DecodedFrameView {
                control_entries: vec![ControlEntry {
                    start_bit: 2040,
                    end_bit: 2050,
                    index: 2,
                    baseline: None,
                    short: None,
                    analog: None,
                    third_analog: None,
                    extra: None,
                    flags: None,
                }],
                start_bit: 2040,
                end_bit: 2050,
                kinds: vec![0],
                fields: vec![],
                stop: crate::theater::parser::v41::FrameViewStop::Truncated,
            });
    }
    replication_chunk_mut(&mut film).body.packets = vec![p];
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
fn resolved_query_indices_intersect_filters_and_preserve_order() {
    let mut film = recording();
    replication_chunk_mut(&mut film).body.packets = vec![
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
        matches!(resolved.record(rows[0].source), Some(Record::Entity(r)) if r.components[0].fields[0].raw.low_u64() == 4)
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

#[test]
fn summaries_use_recorded_times_and_only_unambiguous_player_links() {
    use crate::theater::resolved::PlayerTable;
    use crate::theater::resolved::PlayerTableSlot;
    use serde_json::json;
    let film = Film::parse([
        FilmChunk {
            kind: ChunkKind::Registry,
            index: None,
            start_ms: None,
            data: [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
        },
        FilmChunk {
            kind: ChunkKind::Summary,
            index: None,
            start_ms: None,
            data: [
                9u16.to_le_bytes().as_slice(),
                &[0, 0],
                &4u32.to_le_bytes(),
                &999_999u64.to_le_bytes(),
                &0u32.to_be_bytes(),
            ]
            .concat(),
        },
    ])
    .unwrap();
    let summary = |time: u64, xuid| SummaryEventRead {
        xuid,
        gamertag_utf16: {
            let mut units = [0; 16];
            for (slot, unit) in units.iter_mut().zip("Recorded player".encode_utf16()) {
                *slot = unit;
            }
            units
        },
        timestamp_ms: (time / 1000) as u32,
        metadata: 0,
        medal_flag: 0,
        type_code: 50,
        source: BitRange { start: 0, end: 0 },
        identity_source: BitRange { start: 0, end: 0 },
    };
    // Independent interpretation fixture; these candidates are deliberately not
    // injected into the canonical Film or generated by the search under test.
    let mut interpretations = Interpretations::from_film(&film);
    interpretations.summary_packets = vec![interpretation::SummaryPacketInterpretation {
        source: SourceRef {
            chunk: 1,
            packet: 0,
            record: RecordRef::Packet,
        },
        events: vec![summary(2000, 42), summary(1000, 42), summary(3000, 99)],
    }];
    let resolved = ResolvedFilm::from_interpretations(&film, interpretations.clone());
    assert_eq!(resolved.summaries().len(), 3);
    assert_eq!(resolved.summaries()[0].timestamp_us, 2000);
    assert_eq!(resolved.summaries()[0].gamertag, "Recorded player");
    assert_eq!(resolved.summaries()[0].kind, SummaryKind::Kill);
    assert_eq!(
        resolved.summaries()[0].derivation,
        SummaryDerivation::GuardedV41Layout
    );
    assert!(
        resolved
            .events()
            .iter()
            .filter(|event| event.kind == EventKind::Summary)
            .all(|event| event.provenance == Provenance::DerivedSummary)
    );
    assert_eq!(resolved.summaries()[0].source.record, RecordRef::Summary(0));
    let times: Vec<_> = resolved
        .query(EventFilter {
            kind: Some(EventKind::Summary),
            ..Default::default()
        })
        .map(|e| e.timestamp_us)
        .collect();
    assert_eq!(times, vec![1000, 2000, 3000]);
    assert_eq!(
        resolved
            .query(EventFilter {
                kind: Some(EventKind::Summary),
                start_us: Some(1000),
                end_us: Some(1000),
                ..Default::default()
            })
            .count(),
        1
    );
    let player:PlayerTableSlot=serde_json::from_value(json!({
        "film_index":7,"xuid":42,"gamertag":"Recorded player","session_token":0,"bit":0,"total_bits":0,
        "shorts":{"tete":0,"deux":0,"repr":0,"q64":0,"f10":0,"f14":0,"f6":0,"f8":0,"f7":0,"f1":0}
    })).unwrap();
    let mut table = PlayerTable {
        slots: vec![player.clone()],
        report: Default::default(),
        error: None,
    };
    let actors: Vec<_> = index(&film, Some(&table), &interpretations.summary_packets)
        .into_iter()
        .filter(|e| e.kind == EventKind::Summary)
        .map(|e| e.player_index)
        .collect();
    assert_eq!(actors, vec![Some(7), Some(7), None]);
    table.slots.push(PlayerTableSlot {
        film_index: 8,
        ..player
    });
    assert!(
        index(&film, Some(&table), &interpretations.summary_packets)
            .iter()
            .filter(|e| e.kind == EventKind::Summary)
            .all(|e| e.player_index.is_none())
    );
}
