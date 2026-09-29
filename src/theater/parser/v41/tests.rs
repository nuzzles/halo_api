use super::replication::continue_event_views;
use super::*;
use serde_json::{Value, json};
use std::io::Read;

#[test]
fn native_data_event_continuation_reference_oracle() {
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
                    vec!["high-frequency".into()]
                } else {
                    vec![]
                },
                levels: if index == 3 { vec![1] } else { vec![] },
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    let mut continued = 0;
    let mut record_count = 0;
    for (index, case) in cases.iter().enumerate() {
        let hex = case["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|b| u8::from_str_radix(&hex[b..b + 2], 16).unwrap())
            .collect();
        let events = read_native_event_list(&data, 1, case["gate"].as_bool(), data.len());
        assert_eq!(
            json!(events.stop == NativeEventListStop::Terminator),
            case["terminated"],
            "case {index}"
        );
        assert_eq!(
            json!(events.records.iter().filter(|r| r.layout_complete).count()),
            case["count"],
            "case {index}"
        );
        if events.stop == NativeEventListStop::Terminator {
            assert_eq!(json!(events.end_bit), case["event_end"], "case {index}");
        }
        let mut config = NativeFrameConfig {
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
            let frame = result.frame.as_ref().unwrap();
            assert_eq!(json!(frame.end_bit), expected["end"], "case {index}");
            assert_eq!(
                json!(frame.views_completed),
                expected["views"],
                "case {index}"
            );
            assert!(frame.messages.is_none());
            let records = expected["records"].as_array().unwrap();
            assert_eq!(frame.records.len(), records.len(), "case {index}");
            for (record, reference) in frame.records.iter().zip(records) {
                record_count += 1;
                assert_eq!(json!(record.header.id), reference["ID"]);
                assert_eq!(
                    record.stop == EntityViewStop::Complete,
                    reference["DesyncAt"] == -1
                );
                if record.header.kind != RecordKind::Delete {
                    assert_eq!(json!(record.end_bit), reference["Trace"]["EndBit"]);
                    assert_eq!(json!(record.mask.unwrap_or(0)), reference["Trace"]["Mask"]);
                }
                let components = reference["Trace"]["Comps"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                assert_eq!(record.attempts.len(), components.len());
                for (actual, expected) in record.attempts.iter().zip(components) {
                    assert_eq!(json!(actual.span.start_bit), expected["StartBit"]);
                    assert_eq!(json!(actual.span.name), expected["Name"]);
                    assert_eq!(json!(actual.status), expected["Ported"]);
                }
            }
            assert_eq!(
                serde_json::from_value::<NativeEventContinuation>(json!(result)).unwrap(),
                result
            );
        }
    }
    assert!(continued > 0 && record_count > 0);
}

#[test]
fn native_data_event_continuation_preserves_original_stop() {
    // Config=1, code 3 with three absent references, list terminator at bit 12,
    // entity End at bit 13, empty control view at bit 16.
    let payload = [0xc1, 0x80, 0];
    let mut packet = vec![0; 16];
    packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    packet.extend(payload);
    let source = FilmSource::load(
        &[[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(), packet],
        &[],
    )
    .unwrap();
    let parsed = Film::parse(test_chunks(&source)).unwrap();
    let packet = &parsed.replication.chunks[0].packets[0];
    let FilmPacketBody::Frame(original) = &packet.body else {
        panic!("missing frame")
    };
    assert_eq!(original.views_completed, 0);
    assert_eq!(original.end_bit, 9);
    let continuation = packet.event_continuation.as_ref().unwrap();
    assert_eq!(continuation.start_bit, 13);
    let frame = continuation.frame.as_ref().unwrap();
    assert_eq!(frame.views_completed, 2);
    assert_eq!(frame.end_bit, 17);
    assert_eq!(
        serde_json::from_value::<Film>(json!(parsed)).unwrap(),
        parsed
    );
    let mut old_packet = json!(packet);
    old_packet
        .as_object_mut()
        .unwrap()
        .remove("event_continuation");
    assert!(
        serde_json::from_value::<FilmPacket>(old_packet)
            .unwrap()
            .event_continuation
            .is_none()
    );
}

#[test]
fn native_data_damage_grammar_conflict_isolates_binding_effects() {
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
                    vec!["high-frequency".into()]
                } else {
                    vec![]
                },
                levels: if index == 3 { vec![1] } else { vec![] },
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    let config = NativeFrameConfig {
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
        let events = read_native_event_list(&payload, 1, Some(false), payload.len());
        assert_eq!(events.stop, NativeEventListStop::Terminator);
        assert_eq!(events.end_bit, prefix_bits);
        let mut world = FilmWorld::default();
        let before = world.clone();
        let result =
            continue_event_views(&payload, &events, &config, &registry, &mut world).unwrap();
        assert!(!result.frame.as_ref().unwrap().records.is_empty());
        if damage {
            assert_eq!(read_native_weapon_damage(&payload, 0).end_bit, 106);
            assert_eq!(
                result.state_policy,
                NativeContinuationStatePolicy::IsolatedConflictingDamageGrammar
            );
            assert_eq!(world, before);
        } else {
            assert_eq!(result.state_policy, NativeContinuationStatePolicy::Applied);
            assert_ne!(world, before);
        }
        let mut old = json!(result);
        old.as_object_mut().unwrap().remove("state_policy");
        assert_eq!(
            serde_json::from_value::<NativeEventContinuation>(old)
                .unwrap()
                .state_policy,
            NativeContinuationStatePolicy::Unknown
        );
    }
}
