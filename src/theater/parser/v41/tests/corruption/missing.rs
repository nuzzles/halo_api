//! Missing source policy must never be treated as a recorded disabled flag.
use super::*;
use crate::theater::parser::v41::context::CorruptionCheckContext;

#[test]
fn missing_corruption_context_preserves_original_event_prefixes() {
    for case in oracle()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == "event")
    {
        let data = bytes(case["hex"].as_str().unwrap());
        let known = read_event_list(
            &data,
            0,
            100,
            EventLayout {
                corruption_check: case["enabled"].as_bool(),
                ..Default::default()
            },
        );
        let unknown = read_event_list(
            &data,
            0,
            100,
            EventLayout {
                corruption_check: None,
                ..Default::default()
            },
        );
        if case["code"] != 21 {
            assert_eq!(unknown, known, "empty descriptors do not have trailers");
            continue;
        }
        assert_eq!(unknown.stop, EventListStop::MissingCorruptionCheckContext);
        assert_eq!(unknown.records.len(), 1);
        assert!(!unknown.records[0].layout_complete);
        assert!(unknown.terminator.is_none());
        let fields: Vec<_> = known.records[0]
            .fields
            .iter()
            .filter(|f| f.stage != EventFieldStage::CorruptionCheck)
            .cloned()
            .collect();
        let end = fields.last().unwrap().bit + fields.last().unwrap().width;
        assert_eq!(unknown.records[0].fields, fields);
        assert_eq!(unknown.end_bit, end);
        assert_eq!(unknown.records[0].end_bit, end);
        assert_eq!(
            serde_json::from_slice::<EventListRead>(&serde_json::to_vec(&unknown).unwrap())
                .unwrap(),
            unknown
        );
    }
}

fn parse_packets(
    registry: RawFilmChunk,
    payloads: &[(u16, Vec<u8>)],
) -> crate::theater::parser::ParsedFilm {
    crate::theater::parser::test_support::parse_fixture_structural([
        registry,
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 1,
            start_ms: 0,
            data: payloads
                .iter()
                .flat_map(|(kind, p)| packet(*kind, p.clone()))
                .collect(),
        },
        empty_summary(),
    ])
    .unwrap()
}

#[test]
fn missing_corruption_context_public_admission_is_local_and_keeps_schema_declarations() {
    let original = crate::theater::parser::test_support::registry_with_corruption_policy(false);
    let parsed =
        crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&original.data)
            .unwrap();
    let index = parsed
        .registry
        .archetype(4)
        .unwrap()
        .components
        .iter()
        .position(|c| c.name() == Some("high-frequency"))
        .unwrap();
    for truncated in [true, false] {
        let mut registry = original.clone();
        if truncated {
            registry.data.truncate(0xcb45c);
        } else {
            registry.data[8] ^= 0x20;
        }
        let parsed =
            crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&registry.data)
                .unwrap();
        assert_eq!(
            parsed
                .bootstrap_header
                .as_ref()
                .and_then(|h| h.corruption_checks()),
            None
        );
        assert_eq!(parsed.bootstrap_header.is_some(), truncated);
        let mut event = vec![true];
        put(&mut event, 1, 1);
        put(&mut event, 7, 21);
        put(&mut event, 3, 0);
        put(&mut event, 2, 3);
        let event_end = event.len();
        put(&mut event, 5, 0); // plausible terminators cannot prove a disabled guard
        let mut new = vec![true, false];
        for (w, v) in [
            (3, 1),
            (13, 0),
            (2, 1),
            (6, 4),
            (1, 1), // NEW components present
            (1, 0),
            (3, 1),
            (6, index as u64),
            (8, 231),
        ] {
            put(&mut new, w, v);
        }
        let component_end = new.len();
        put(&mut new, 4, 0);
        let mut empty_delta = vec![true, false];
        for (w, v) in [(1, 1), (13, 0), (2, 1), (1, 0), (4, 0), (4, 0)] {
            put(&mut empty_delta, w, v);
        }
        let mut empty_event = vec![true];
        for (w, v) in [(1, 1), (7, 3), (3, 0), (5, 0)] {
            put(&mut empty_event, w, v);
        }
        let mut snapshot = vec![true];
        for (w, v) in [
            (32, 0x40000000),
            (32, 4),
            (32, 0),
            (4, 0),
            (8, 0),
            (32, 0),
            (32, 0),
        ] {
            put(&mut snapshot, w, v);
        }
        vacant_keyframe_slots(&mut snapshot, 8190);
        let film = parse_packets(
            registry,
            &[
                (0, pack(&event)),
                (0, pack(&new)),
                (0, pack(&empty_delta)),
                (0, pack(&empty_event)),
                (2, pack(&snapshot)),
            ],
        );
        let chunk = &film.replication_stream.0[0];
        for packet in &chunk.body.packets {
            assert!(packet.source_coverage(&chunk.data).is_some());
        }
        let frame = |i: usize| match &chunk.body.packets[i].body {
            PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(f)) => f.as_ref(),
            _ => panic!(),
        };
        let first = frame(0);
        assert_eq!(
            first.events.stop,
            if truncated {
                EventListStop::MissingCorruptionCheckContext
            } else {
                EventListStop::MissingEventVersion { event_code: 0 }
            }
        );
        assert_eq!(first.events.end_bit, if truncated { event_end } else { 9 });
        assert_eq!(
            first.frame,
            FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
        );
        let new = frame(1).frame.decoded().unwrap();
        assert_eq!(new.records.len(), 1);
        assert!(new.controls.is_none());
        assert_eq!(
            new.records[0].stop,
            EntityViewStop::MissingCorruptionCheckContext
        );
        assert_eq!(new.records[0].end_bit, component_end as i64);
        assert_eq!(
            new.records[0].components.last().unwrap().status,
            ComponentReadStatus::ContextUnavailable
        );
        let delta = frame(2).frame.decoded().unwrap();
        assert_eq!(delta.records[0].stop, EntityViewStop::Complete);
        assert_eq!(delta.end_bit, empty_delta.len() as i64);
        if truncated {
            assert_eq!(frame(3).events.stop, EventListStop::Terminator);
            assert_eq!(
                frame(3).frame.decoded().unwrap().end_bit,
                empty_event.len() as i64
            );
        } else {
            assert_eq!(
                frame(3).events.stop,
                EventListStop::MissingEventVersion { event_code: 0 }
            );
            assert_eq!(frame(3).events.end_bit, 9);
        }
        let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
            &chunk.body.packets[4].body
        else {
            panic!()
        };
        assert_eq!(table.stop, KeyframeChainStop::End);
        assert!(
            table.records[0]
                .record
                .as_ref()
                .unwrap()
                .components
                .is_empty()
        );
    }
}

#[test]
fn missing_corruption_context_snapshot_stops_do_not_consume_size_words_or_components() {
    let registry = component_registry(&[(4, vec![("high-frequency".into(), 0)])]);
    let config = V41DecodeConfig {
        corruption_checks: CorruptionCheckContext::Unavailable,
        ..Default::default()
    };
    for n1 in [0, 1, u32::MAX] {
        for n2 in [0, 1] {
            let mut bits = Vec::new();
            for (w, v) in [
                (32, 0x40000000),
                (32, 4),
                (32, 0),
                (4, 0),
                (8, 0),
                (32, n1 as u64),
            ] {
                put(&mut bits, w, v);
            }
            let default_end = bits.len();
            put(&mut bits, 32, n2);
            if n2 != 0 {
                put(&mut bits, 8, 231);
            }
            let component_end = bits.len();
            put(&mut bits, 33, 0x1ffffffff);
            let data = pack(&bits);
            let read = decode_keyframe_record_contextual(
                &data,
                0,
                &registry,
                &config.record_layout(),
                Some(&config.profile),
            )
            .unwrap();
            if n1 == 1 {
                assert_eq!(read.stop, KeyframeStop::MissingCorruptionCheckContext);
                assert_eq!(read.end_bit, default_end as i64);
                let default = read.default_state.as_ref().unwrap();
                assert_eq!(default.status, DefaultStateStatus::ContextUnavailable);
                assert!(default.fields.is_empty());
                assert!(read.components.is_empty());
                assert!(!read.fields.iter().any(|f| f.name == "components_guard"));
            } else if n2 == 1 {
                assert_eq!(read.stop, KeyframeStop::MissingCorruptionCheckContext);
                assert_eq!(read.end_bit, component_end as i64);
                assert_eq!(
                    read.components[0].status,
                    ComponentReadStatus::ContextUnavailable
                );
                assert_eq!(read.components[0].fields.len(), 1);
                assert_eq!(read.components[0].fields[0].raw.low_u64(), 231);
            } else {
                assert_eq!(read.stop, KeyframeStop::Complete);
                assert_eq!(read.end_bit, component_end as i64);
            }
            assert_eq!(
                serde_json::from_slice::<KeyframeRecord>(&serde_json::to_vec(&read).unwrap())
                    .unwrap(),
                read
            );
        }
    }
}
