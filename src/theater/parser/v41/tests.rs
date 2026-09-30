use super::packets::continue_event_views;
use super::packets::{ContinuationStatePolicy, EventContinuation};
use super::*;
use crate::theater::runtime::interpretation::weapon_damage::read_weapon_damage;
use serde_json::{Value, json};
use std::io::Read;

#[test]
fn reference_data_event_continuation_reference_oracle() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../../fixtures/event-continuation-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 2560);
    let registry = FilmRegistry {
        archetypes: (0..4)
            .map(|index| FilmArchetype {
                index,
                components: if index == 3 {
                    vec![RegistryComponent {
                        name_bytes: [b"high-frequency".as_slice(), &[0; 242]].concat(),
                        precision_level: 1,
                        source: ByteRange { start: 0, end: 260 },
                    }]
                } else {
                    vec![]
                },
                source: ByteRange {
                    start: 0,
                    end: 16640,
                },
                padding: ByteRange {
                    start: if index == 3 { 260 } else { 0 },
                    end: 16640,
                },
            })
            .collect(),
    };
    let mut continued = 0;
    let mut record_count = 0;
    for (index, case) in cases.iter().enumerate() {
        let hex = case["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|b| u8::from_str_radix(&hex[b..b + 2], 16).unwrap())
            .collect();
        let events = read_reference_event_list(&data, 1, case["gate"].as_bool(), data.len());
        assert_eq!(
            json!(events.stop == EventListStop::Terminator),
            case["terminated"],
            "case {index}"
        );
        assert_eq!(
            json!(events.records.iter().filter(|r| r.layout_complete).count()),
            case["count"],
            "case {index}"
        );
        if events.stop == EventListStop::Terminator {
            assert_eq!(json!(events.end_bit), case["event_end"], "case {index}");
        }
        let mut config = FrameConfig {
            id_low_bits: 5,
            extra_fields: case["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        config.context.profile.grammar.corruption_check = case["check"].as_bool().unwrap();
        config.context.profile.grammar.default_state_by_archetype = true;
        config.context.profile.grammar.generation_strict = true;
        let mut world = FilmWorld::default();
        for expected in [case, &case["second"]] {
            let before = world.clone();
            let result = continue_event_views(&data, &events, &config, &registry, &mut world);
            assert_eq!(json!(result.is_some()), case["continued"], "case {index}");
            let Some(result) = result else {
                assert_eq!(world, before);
                continue;
            };
            continued += 1;
            assert_eq!(result.start_bit, events.end_bit);
            let frame = result.frame.decoded().unwrap();
            let expected_end = expected["end"].as_i64().unwrap();
            if expected_end <= (data.len() * 8) as i64 {
                assert_eq!(frame.end_bit, expected_end, "case {index}");
            } else {
                assert!(frame.end_bit <= (data.len() * 8) as i64, "case {index}");
            }
            if expected_end <= (data.len() * 8) as i64 {
                assert_eq!(
                    json!(crate::theater::parser::v41::production_frame::completed_views(frame)),
                    expected["views"],
                    "case {index}"
                );
            } else {
                assert!(
                    crate::theater::parser::v41::production_frame::completed_views(frame)
                        <= expected["views"].as_u64().unwrap() as usize
                );
            }
            assert!(frame.messages.is_none());
            let records = expected["records"].as_array().unwrap();
            assert_eq!(frame.records.len(), records.len(), "case {index}");
            for (record, reference) in frame.records.iter().zip(records) {
                record_count += 1;
                assert_eq!(json!(record.header.id), reference["ID"]);
                let reference_end = reference["Trace"]["EndBit"].as_i64().unwrap_or_default();
                if reference_end <= (data.len() * 8) as i64 {
                    assert_eq!(
                        record.stop == EntityViewStop::Complete,
                        reference["DesyncAt"] == -1
                    );
                } else {
                    assert_ne!(record.stop, EntityViewStop::Complete);
                }
                if record.header.kind != RecordKind::Delete {
                    if reference_end <= (data.len() * 8) as i64 {
                        assert_eq!(record.end_bit, reference_end);
                    } else {
                        assert!(record.end_bit <= (data.len() * 8) as i64);
                    }
                    assert_eq!(json!(record.mask.unwrap_or(0)), reference["Trace"]["Mask"]);
                }
                let components = reference["Trace"]["Comps"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                assert_eq!(record.components.len(), components.len());
                for (actual, expected) in record.components.iter().zip(components) {
                    assert_eq!(json!(actual.start_bit), expected["StartBit"]);
                    assert_eq!(json!(actual.name), expected["Name"]);
                    if actual.status != ComponentReadStatus::Truncated
                        || reference_end <= (data.len() * 8) as i64
                    {
                        assert_eq!(
                            actual.status,
                            serde_json::from_value::<Option<bool>>(expected["Ported"].clone())
                                .unwrap()
                                .into()
                        );
                    }
                }
            }
            assert_eq!(
                serde_json::from_value::<EventContinuation>(json!(result)).unwrap(),
                result
            );
        }
    }
    assert!(continued > 0 && record_count > 0);
}

#[test]
fn frame_views_follow_message_terminator_once() {
    // Config=1, code 3 with three absent references, list terminator at bit 12,
    // entity End at bit 13, empty control view at bit 16.
    let payload = [0xc1, 0x80, 0];
    let mut packet = vec![0; 16];
    packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    packet.extend(payload);
    let source = FixtureFilmSource::load(
        &[[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(), packet],
        &[],
    )
    .unwrap();
    let parsed = Film::parse(test_chunks(&source)).unwrap();
    let packet = &parsed.replication_chunks().next().unwrap().body.packets[0];
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(original)) = &packet.body
    else {
        panic!("missing frame")
    };
    assert_eq!(original.configuration, Some(true));
    assert_eq!(original.events.end_bit, 13);
    let frame = original.frame.decoded().unwrap();
    assert_eq!(
        crate::theater::parser::v41::production_frame::completed_views(frame),
        2
    );
    assert_eq!(frame.end_bit, 17);
    assert_eq!(
        serde_json::from_value::<Film>(json!(parsed)).unwrap(),
        parsed
    );
}

#[test]
fn reference_data_damage_grammar_conflict_isolates_binding_effects() {
    // A zero-filled code-0 body consumes 99 bits under evBody0, but 94 under
    // lot1DecodeDamageAftermath. Neither interpretation may silently win.
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../../fixtures/frames-levelup-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixtures: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    let hex = fixtures[0]["hex"].as_str().unwrap();
    let frame: Vec<_> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let registry = FilmRegistry {
        archetypes: (0..4)
            .map(|index| FilmArchetype {
                index,
                components: if index == 3 {
                    vec![RegistryComponent {
                        name_bytes: [b"high-frequency".as_slice(), &[0; 242]].concat(),
                        precision_level: 1,
                        source: ByteRange { start: 0, end: 260 },
                    }]
                } else {
                    vec![]
                },
                source: ByteRange {
                    start: 0,
                    end: 16640,
                },
                padding: ByteRange {
                    start: if index == 3 { 260 } else { 0 },
                    end: 16640,
                },
            })
            .collect(),
    };
    let config = FrameConfig {
        id_low_bits: 5,
        ..Default::default()
    };
    for damage in [true, false] {
        let prefix_bits = if damage { 112 } else { 13 };
        let total_bits = prefix_bits + frame.len() * 8 - 2;
        let mut payload = vec![0; total_bits.div_ceil(8)];
        if damage {
            payload[0] = 0xc0;
        } else {
            payload[0] = 0xc1;
            payload[1] = 0x80;
        }
        for bit in 2..frame.len() * 8 {
            let dest = prefix_bits + bit - 2;
            payload[dest / 8] |= ((frame[bit / 8] >> (7 - bit % 8)) & 1) << (7 - dest % 8);
        }
        let events = read_reference_event_list(&payload, 1, Some(false), payload.len());
        assert_eq!(events.stop, EventListStop::Terminator);
        assert_eq!(events.end_bit, prefix_bits);
        let mut world = FilmWorld::default();
        let before = world.clone();
        let result =
            continue_event_views(&payload, &events, &config, &registry, &mut world).unwrap();
        assert!(!result.frame.decoded().unwrap().records.is_empty());
        if damage {
            assert_eq!(read_weapon_damage(&payload, 0).end_bit, 106);
            assert_eq!(
                result.state_policy,
                ContinuationStatePolicy::IsolatedConflictingDamageGrammar
            );
            assert_eq!(world, before);
        } else {
            assert_eq!(result.state_policy, ContinuationStatePolicy::Applied);
            assert_ne!(world, before);
        }
        let mut old = json!(result);
        old.as_object_mut().unwrap().remove("state_policy");
        assert_eq!(
            serde_json::from_value::<EventContinuation>(old)
                .unwrap()
                .state_policy,
            ContinuationStatePolicy::Unknown
        );
    }
}

#[test]
fn frame_stops_when_message_boundary_is_unknown_or_conflicting() {
    let registry = FilmRegistry {
        archetypes: Vec::new(),
    };
    let config = FrameConfig::default();
    for (payload, expected) in [
        (vec![0xff], FrameDecodeError::IncompleteMessageList),
        (
            {
                let mut bytes = vec![0; 16];
                bytes[0] = 0xc0;
                bytes
            },
            FrameDecodeError::ConflictingMessageLayout,
        ),
    ] {
        let mut world = FilmWorld::default();
        let before = world.clone();
        let body = super::packets::decode(
            0,
            &payload,
            &mut super::packets::DecodeContext {
                config: &config,
                registry: &registry,
                world: &mut world,
                event_gate15: None,
            },
        )
        .unwrap();
        let ReplicationStreamPacketBody::FramePacketBody(frame) = body else {
            panic!("frame");
        };
        assert_eq!(frame.frame, FrameRead::Refused(expected));
        assert_eq!(world, before);
        assert_eq!(frame.events.records.len(), 1);
        assert_eq!(frame.events.records[0].fields[0].bit, 1);
        assert_eq!(frame.events.records[0].fields[1].bit, 2);
    }
}

#[test]
fn truncated_keyframe_corruption_check_stays_with_its_component() {
    let registry = FilmRegistry {
        archetypes: vec![FilmArchetype {
            index: 0,
            components: vec![RegistryComponent {
                name_bytes: [b"high-frequency".as_slice(), &[0; 242]].concat(),
                precision_level: 1,
                source: ByteRange { start: 0, end: 260 },
            }],
            source: ByteRange {
                start: 0,
                end: 16640,
            },
            padding: ByteRange {
                start: 260,
                end: 16640,
            },
        }],
    };
    let mut payload = vec![0u8; 23];
    let mut write = |start: usize, width: usize, value: u32| {
        for i in 0..width {
            if value & (1 << (width - i - 1)) != 0 {
                payload[(start + i) / 8] |= 1 << (7 - (start + i) % 8);
            }
        }
    };
    // Configuration, 108-bit header, absent default state, components present,
    // an 8-bit counter, then a corruption-check gate without its 32-bit value.
    write(1, 32, 0x4000_0007);
    write(141, 32, 1);
    write(173, 8, 0xab);
    write(181, 1, 1);
    let mut config = FrameConfig::default();
    config.context.profile.grammar.corruption_check = true;
    let table = config.read_keyframe_table(&payload, &registry).unwrap();
    let record = table.records[0].record.as_ref().unwrap();
    assert_eq!(record.stop, KeyframeStop::Truncated);
    assert!(record.default_state.is_none());
    let component = &record.components[0];
    assert_eq!(component.status, ComponentReadStatus::Truncated);
    assert_eq!((component.start_bit, component.end_bit), (173, 182));
    assert_eq!(component.fields.len(), 2);
    assert_eq!(component.fields[0].raw.low_u64(), 0xab);
    assert_eq!(component.fields[1].name, "component_corruption_check.gate");
    assert_eq!(component.fields[1].raw.low_u64(), 1);
    assert!(!record.fields.iter().any(|field| field.bit >= 173));
}

#[test]
fn recorded_new_declaration_survives_a_truncated_body_for_later_delta_grammar() {
    fn pack(fields: &[(usize, u32)]) -> Vec<u8> {
        let mut bits = Vec::new();
        for &(width, value) in fields {
            bits.extend((0..width).rev().map(|shift| value & (1 << shift) != 0));
        }
        let mut bytes = vec![0; bits.len().div_ceil(8)];
        for (bit, set) in bits.into_iter().enumerate() {
            if set {
                bytes[bit / 8] |= 1 << (7 - bit % 8);
            }
        }
        bytes
    }
    let registry = FilmRegistry {
        archetypes: (0..36)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                source: ByteRange { start: 0, end: 0 },
                padding: ByteRange { start: 0, end: 0 },
            })
            .collect(),
    };
    let config = FrameConfig::default();
    let mut world = FilmWorld::default();
    // Empty message view, NEW slot 7, generation 1, archetype 35. The fixed
    // default state is deliberately absent; only the declaration is recorded.
    let new = pack(&[(2, 0), (3, 1), (13, 7), (2, 1), (6, 35)]);
    let first = config
        .decode_production_views(&new, 2, &registry, &mut world)
        .unwrap();
    assert_eq!(first.records.len(), 1);
    assert_eq!(first.records[0].stop, EntityViewStop::Truncated);
    assert_eq!(first.records[0].archetype, Some(35));
    let default_state = first.records[0].default_state.as_ref().unwrap();
    assert_eq!(default_state.status, DefaultStateStatus::Truncated);
    assert_eq!(default_state.source.start, 26);
    assert!(default_state.source.end <= new.len() * 8);
    for field in &default_state.fields {
        assert!(field.bit >= default_state.source.start);
        assert!(field.bit + field.width <= default_state.source.end);
        assert!(!first.records[0].fields.contains(field));
    }
    assert_eq!(world.archetype(7), Some(35));
    // A later DELTA uses that declaration, with no baseline or components,
    // followed by the entity terminator and an empty control view.
    let delta = pack(&[
        (2, 0),
        (1, 1),
        (13, 7),
        (2, 1),
        (1, 0),
        (1, 0),
        (3, 0),
        (3, 0),
        (1, 0),
    ]);
    let second = config
        .decode_production_views(&delta, 2, &registry, &mut world)
        .unwrap();
    assert_eq!(second.records.len(), 1);
    assert_eq!(second.records[0].archetype, None);
    assert_eq!(second.records[0].stop, EntityViewStop::Complete);
    assert_eq!(second.end_bit, 27);

    // A recorded DELETE header clears the parser's schema even if its metadata
    // word is absent. The record still reports truncation to resolution.
    let delete = pack(&[(2, 0), (3, 2), (13, 7), (2, 1)]);
    let third = config
        .decode_production_views(&delete, 2, &registry, &mut world)
        .unwrap();
    assert_eq!(third.records.len(), 1);
    assert_eq!(third.records[0].stop, EntityViewStop::Truncated);
    assert_eq!(world.archetype(7), None);
    let fourth = config
        .decode_production_views(&delta, 2, &registry, &mut world)
        .unwrap();
    assert!(fourth.records.is_empty());
    assert!(matches!(
        fourth.entity_end,
        Some(ProductionEntityEnd::Rejected { .. })
    ));
}

#[test]
fn vehicle_runtime_gate_is_never_assumed_from_plausible_payload_bits() {
    let registry = FilmRegistry {
        archetypes: vec![FilmArchetype {
            index: 0,
            components: vec![RegistryComponent {
                name_bytes: b"vehicle-type-physics-component\0".to_vec(),
                precision_level: 0,
                source: ByteRange { start: 0, end: 260 },
            }],
            source: ByteRange {
                start: 0,
                end: 16640,
            },
            padding: ByteRange {
                start: 260,
                end: 16640,
            },
        }],
    };
    // Empty messages, DELTA slot 1 / generation 1, no baseline, sparse mask
    // containing component zero. Everything after bit 29 is undecodable without
    // vehicle+0x818, even if it resembles either documented body mode.
    let prefix = "00100000000000010100001000000";
    assert_eq!(prefix.len(), 29);
    for byte in [0x00, 0xff] {
        let mut payload = vec![byte; 40];
        for (bit, value) in prefix.bytes().enumerate() {
            let mask = 1 << (7 - bit % 8);
            if value == b'1' {
                payload[bit / 8] |= mask;
            } else {
                payload[bit / 8] &= !mask;
            }
        }
        let mut world = FilmWorld::default();
        world.bind_full(0x4000_0001, 0);
        let frame = FrameConfig::default()
            .decode_production_views(&payload, 2, &registry, &mut world)
            .unwrap();
        assert_eq!(frame.end_bit, 29);
        assert_eq!(frame.records.len(), 1);
        assert_eq!(
            frame.records[0].stop,
            EntityViewStop::RuntimeContextUnavailable {
                index: 0,
                name: "vehicle-type-physics-component".into(),
                field: "vehicle+0x818".into(),
            }
        );
        let component = &frame.records[0].components[0];
        assert_eq!((component.start_bit, component.end_bit), (29, 29));
        assert!(component.fields.is_empty());
        assert!(frame.controls.is_none());
    }
}

#[test]
fn empty_frame_reader_does_not_invent_a_configuration_bit() {
    let registry = FilmRegistry { archetypes: vec![] };
    let config = FrameConfig::default();
    let mut world = FilmWorld::default();
    let body = packets::decode(
        0,
        &[],
        &mut packets::DecodeContext {
            config: &config,
            registry: &registry,
            world: &mut world,
            event_gate15: None,
        },
    )
    .unwrap();
    let ReplicationStreamPacketBody::FramePacketBody(frame) = body else {
        panic!()
    };
    assert_eq!(frame.configuration, None);
    assert_eq!(
        frame.frame,
        FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
    );
    assert!(frame.events.records.is_empty());
}

#[test]
fn partial_nested_event_extent_contains_only_source_backed_fields() {
    // Header: continuation=1, code=85; three absent references; absent
    // victim/killer (gate=1). The 32-bit percentage cannot fit, but the
    // following fields must not be retried at the failed word's position.
    let data = [0xd5, 0x1c];
    let events = read_reference_event_list(&data, 0, None, 8);
    assert_eq!(events.stop, EventListStop::Truncated);
    assert_eq!(events.records.len(), 1);
    let record = &events.records[0];
    assert_eq!(record.body_start_bit, Some(11));
    assert_eq!(record.end_bit, 13);
    assert_eq!(events.end_bit, 13);
    assert!(!record.layout_complete);
    assert!(record.fields.iter().any(|field| field.bit == 13
        && field.width == 32
        && field.value == EventFieldValue::Unavailable));
    assert!(
        !record
            .fields
            .iter()
            .any(|field| field.bit >= 13 && field.value != EventFieldValue::Unavailable)
    );
    for field in &record.fields {
        if field.value != EventFieldValue::Unavailable {
            assert!(field.bit + field.width <= record.end_bit);
            assert!(field.bit + field.width <= data.len() * 8);
        }
    }
}
