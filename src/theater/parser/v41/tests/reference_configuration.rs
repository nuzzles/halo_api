//! Shared reference ranges come from packet configuration, never profile defaults.
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::*;
use crate::theater::parser::packet::*;
use crate::theater::parser::test_support::{
    component_registry, empty_summary, vacant_keyframe_slots,
};
use crate::theater::parser::v41::chunks::replication::components::{
    cursor::ComponentCursor, reader::ComponentReader,
};
use crate::theater::parser::v41::chunks::replication::replication_stream::{
    frame::records::{RecordBinding, RecordReadContext, decode_entity_record},
    keyframes::record::decode_keyframe_record_contextual,
};
use crate::theater::parser::v41::chunks::replication::source::*;
use crate::theater::parser::v41::context::V41DecodeConfig;
use serde_json::Value;

fn put(bits: &mut Vec<bool>, width: usize, value: u64) {
    bits.extend((0..width).rev().map(|i| value >> i & 1 != 0));
}
fn pack(bits: &[bool]) -> Vec<u8> {
    let mut bytes = vec![0; bits.len().div_ceil(8)];
    for (i, b) in bits.iter().enumerate() {
        bytes[i / 8] |= u8::from(*b) << (7 - i % 8);
    }
    bytes
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn reader(data: &[u8], start: usize, category_references: bool) -> ComponentReader<'_> {
    ComponentReader {
        category_references,
        snapshot: false,
        cursor: ComponentCursor::guarded(data, start as i64),
        fields: vec![],
        position_encoding: None,
        reference_widths: None,
        width_error: None,
        context_error: None,
        live_grammar: None,
        movement_slot: None,
        references: vec![],
        diagnostics: Default::default(),
    }
}

#[test]
fn reference_configuration_covers_profile_position_handles_but_not_fixed_posture_words() {
    use crate::theater::parser::v41::chunks::replication::components::{movement, position};
    let mut layout = V41DecodeConfig::default().record_layout();
    let encoding = layout.position.as_mut().unwrap();
    encoding.handle_bits = 13;
    for present in [false, true] {
        let mut bits = vec![true, false, true]; // prediction, delta, handle tail
        for _ in 0..3 {
            put(&mut bits, 32, 0);
        }
        put(&mut bits, 1, 1);
        put(&mut bits, 1, u64::from(present));
        let stop = bits.len();
        if present {
            put(&mut bits, 13, 8191);
            put(&mut bits, 2, 3);
        }
        put(&mut bits, 1, 0); // region absent
        let data = pack(&bits);
        for enabled in [false, true] {
            let mut r = reader(&data, 0, enabled);
            r.position_encoding = Some(encoding);
            let result = position::component(&mut r);
            let refused = present && !enabled;
            assert_eq!(result.is_none(), refused);
            assert_eq!(r.reference_configuration_unavailable(), refused);
            assert_eq!(
                r.cursor.position as usize,
                if refused { stop } else { bits.len() }
            );
            assert_eq!(
                r.fields.iter().any(|f| f.name == "handle_value"),
                present && enabled
            );
        }
    }
    // FUN_14080bd28 reads a fixed R15, not a category-table handle.
    let mut bits = vec![];
    for (w, v) in [(2, 3), (15, 32767), (1, 0), (32, 0xffffffff), (2, 3)] {
        put(&mut bits, w, v);
    }
    let data = pack(&bits);
    let mut r = reader(&data, 0, false);
    assert_eq!(movement::posture(&mut r), Some(true));
    assert_eq!(r.cursor.position as usize, bits.len());
    assert!(r.context_error.is_none());
}

#[test]
fn reference_configuration_native_handles_keep_values_and_unknown_range_prefixes() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference-configuration-v41/reference.json"
    ))
    .unwrap();
    for (case, expected) in oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["results"].as_array().unwrap())
    {
        let category = case["category"].as_u64().unwrap() as u8;
        let optional = case["optional"].as_bool().unwrap();
        let present = case["present"].as_bool().unwrap();
        let start = case["start"].as_u64().unwrap() as usize;
        let data = bytes(case["hex"].as_str().unwrap());
        let read = |r: &mut ComponentReader<'_>| {
            if optional {
                r.optional_handle("reference", category)
            } else {
                r.handle("reference", category)
            }
        };
        let mut native = reader(&data, start, true);
        assert!(read(&mut native).is_some());
        assert_eq!(native.cursor.position, expected["End"].as_i64().unwrap());
        if present {
            assert_eq!(
                native.fields[native.fields.len() - 2].raw.low_u64(),
                expected["Value"].as_u64().unwrap()
            );
            assert_eq!(
                native.fields.last().unwrap().raw.low_u64(),
                expected["Tail"].as_u64().unwrap()
            );
        }
        if optional {
            let reference = &native.references[0];
            let source = &expected["Refs"][0];
            assert_eq!(source["Kind"], 0);
            assert_eq!(reference.present, source["Present"].as_bool().unwrap());
            assert_eq!(reference.end_bit, source["EndBit"].as_i64().unwrap());
        }
        let prefix = usize::from(optional) + usize::from(present && category == 1);
        // A category-1 probe is read before the native configuration test.
        // No unknown-width value/generation or present semantic reference is emitted.
        let mut refused = reader(&data, start, false);
        assert_eq!(read(&mut refused).is_none(), present);
        assert_eq!(refused.reference_configuration_unavailable(), present);
        assert_eq!(refused.cursor.position as usize, start + prefix);
        assert_eq!(refused.fields, native.fields[..prefix]);
        assert!(refused.references.iter().all(|r| !r.present));
        // Exact byte-aligned truncations: a missing gate/probe owns the error,
        // while any later cut still stops before the unavailable value width.
        for cut in 0..=case["bits"].as_u64().unwrap() as usize {
            let shift = (8 - cut % 8) % 8;
            let mut bits = vec![false; shift];
            bits.extend((start..start + cut).map(|i| data[i / 8] >> (7 - i % 8) & 1 != 0));
            let short = pack(&bits);
            let mut r = reader(&short, shift, false);
            let result = read(&mut r);
            assert_eq!(
                r.reference_configuration_unavailable(),
                present && cut >= prefix,
                "{case} cut {cut}"
            );
            assert_eq!(result.is_some(), !present && cut >= prefix);
            assert!(r.cursor.position as usize <= shift + cut);
        }
    }
}

#[test]
fn reference_configuration_stops_component_envelopes_before_guards_and_successors() {
    let registry =
        component_registry(&[(4, vec![("equipment-control-signal-component".into(), 1)])]);
    for snapshot in [false, true] {
        for new in [false, true] {
            if snapshot && new {
                continue;
            }
            for category_references in [false, true] {
                for present in [false, true] {
                    let mut layout = V41DecodeConfig::default().record_layout();
                    layout.category_references = category_references;
                    layout.corruption_check = Some(true);
                    let mut bits = vec![];
                    if snapshot {
                        for (w, v) in [(32, 5), (32, 4), (44, 0), (32, 0), (32, 1)] {
                            put(&mut bits, w, v);
                        }
                    } else {
                        put(&mut bits, if new { 3 } else { 1 }, 1);
                        put(&mut bits, 13, 5);
                        put(&mut bits, 2, 0);
                        if new {
                            put(&mut bits, 6, 4);
                        }
                        put(&mut bits, 1, u64::from(new)); // NEW components present or DELTA baseline absent
                        put(&mut bits, 1, 0);
                        put(&mut bits, 3, 1);
                        put(&mut bits, 6, 0);
                    }
                    put(&mut bits, 4, 7);
                    put(&mut bits, 1, u64::from(present));
                    let stop = bits.len();
                    if present {
                        put(&mut bits, 9, 511);
                        put(&mut bits, 2, 3);
                    }
                    put(&mut bits, 1, 1);
                    put(&mut bits, 32, 0x12345678);
                    let complete = bits.len();
                    put(&mut bits, 8, 0xff);
                    let data = pack(&bits);
                    let refused = present && !category_references;
                    let (end, fields, status) = if snapshot {
                        let r =
                            decode_keyframe_record_contextual(&data, 0, &registry, &layout, None)
                                .unwrap();
                        assert_eq!(
                            r.stop,
                            if refused {
                                KeyframeStop::UnsupportedReferenceConfiguration
                            } else {
                                KeyframeStop::Complete
                            }
                        );
                        (
                            r.end_bit,
                            r.components[0].fields.clone(),
                            r.components[0].status,
                        )
                    } else {
                        let r = decode_entity_record(
                            &data,
                            0,
                            &registry,
                            &layout,
                            Some(RecordBinding {
                                id: 5,
                                archetype: 4,
                            }),
                            true,
                            RecordReadContext {
                                context: None,
                                movement: None,
                            },
                        )
                        .unwrap();
                        assert_eq!(
                            r.stop,
                            if refused {
                                EntityViewStop::UnsupportedReferenceConfiguration
                            } else {
                                EntityViewStop::Complete
                            }
                        );
                        (
                            r.end_bit,
                            r.components[0].fields.clone(),
                            r.components[0].status,
                        )
                    };
                    assert_eq!(end as usize, if refused { stop } else { complete });
                    assert_eq!(
                        status,
                        if refused {
                            ComponentReadStatus::ContextUnavailable
                        } else {
                            ComponentReadStatus::Complete
                        }
                    );
                    assert_eq!(
                        fields.iter().any(|f| f.name.contains("corruption")),
                        !refused
                    );
                }
            }
        }
    }
}

fn packet(kind: u16, bits: &[bool]) -> Vec<u8> {
    let payload = pack(bits);
    let mut data = vec![0; 16];
    data[..2].copy_from_slice(&kind.to_le_bytes());
    data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    data[8..16].copy_from_slice(&1_000_000u64.to_le_bytes());
    data.extend(payload);
    data
}
fn default_body(bits: &mut Vec<bool>, present: bool) -> usize {
    put(bits, 1, 0);
    put(bits, 1, u64::from(present));
    let stop = bits.len();
    if present {
        put(bits, 13, 8191);
        put(bits, 2, 3);
    }
    put(bits, 1, 0);
    put(bits, 1, 0);
    put(bits, 8, 0);
    stop
}
fn control(bits: &mut Vec<bool>) -> usize {
    for (w, v) in [
        (1, 1),
        (2, 0),
        (1, 0),
        (5, 0),
        (1, 1),
        (1, 0),
        (12, 0),
        (1, 0),
        (1, 0),
        (1, 0),
        (1, 1),
        (1, 0),
        (1, 0),
        (1, 0),
        (3, 0),
        (1, 1),
        (2, 2),
    ] {
        put(bits, w, v);
    }
    let stop = bits.len();
    for (w, v) in [(8, 255), (2, 3), (1, 0), (8, 0), (1, 0), (1, 0), (1, 0)] {
        put(bits, w, v);
    }
    stop
}

#[test]
fn reference_configuration_parser_keeps_independent_prefixes() {
    for configuration in [false, true] {
        for present in [false, true] {
            let mut snapshot = vec![configuration];
            vacant_keyframe_slots(&mut snapshot, 1);
            for (w, v) in [(32, 5), (32, 3), (44, 0), (32, 1)] {
                put(&mut snapshot, w, v);
            }
            let snapshot_stop = default_body(&mut snapshot, present);
            put(&mut snapshot, 32, 0);
            vacant_keyframe_slots(&mut snapshot, 8189);
            let mut new = vec![configuration, false];
            for (w, v) in [(3, 1), (13, 5), (2, 0), (6, 3)] {
                put(&mut new, w, v);
            }
            let new_stop = default_body(&mut new, present);
            put(&mut new, 1, 0);
            put(&mut new, 4, 0);
            put(&mut new, 4, 0);
            let mut delta = vec![configuration, false];
            for (w, v) in [(1, 1), (13, 5), (2, 0), (1, 0), (4, 0), (4, 0)] {
                put(&mut delta, w, v);
            }
            let mut controls = vec![configuration, false, false, false, false];
            let control_stop = control(&mut controls);
            let data = [
                packet(2, &snapshot),
                packet(0, &new),
                packet(0, &delta),
                packet(0, &controls),
            ]
            .concat();
            let film = crate::theater::parser::test_support::parse_fixture_structural([
                RawFilmChunk {
                    kind: ChunkKind::Registry.into(),
                    data: super::corruption::registry(false),
                    index: 0,
                    start_ms: 0,
                },
                RawFilmChunk {
                    kind: ChunkKind::Replication.into(),
                    data,
                    index: 1,
                    start_ms: 0,
                },
                empty_summary(),
            ])
            .unwrap();
            let chunk = &film.replication_stream.0[0];
            let refused = present && !configuration;
            let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
                &chunk.body.packets[0].body
            else {
                panic!()
            };
            assert_eq!(
                table.stop,
                if refused {
                    KeyframeChainStop::Desync
                } else {
                    KeyframeChainStop::End
                }
            );
            let record = table.records[0].record.as_ref().unwrap();
            assert_eq!(
                record.default_state.as_ref().unwrap().status,
                if refused {
                    DefaultStateStatus::ContextUnavailable
                } else {
                    DefaultStateStatus::Complete
                }
            );
            if refused {
                assert_eq!(record.end_bit as usize, snapshot_stop);
            }
            for (i, p) in chunk.body.packets.iter().enumerate() {
                assert!(p.source_coverage(&chunk.data).is_some());
                if i == 0 {
                    continue;
                }
                let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
                    &p.body
                else {
                    panic!()
                };
                assert_eq!(frame.events.stop, EventListStop::Terminator);
                let read = frame.frame.decoded().unwrap();
                if i == 1 {
                    let record = &read.records[0];
                    assert_eq!(
                        record.stop,
                        if refused {
                            EntityViewStop::UnsupportedReferenceConfiguration
                        } else {
                            EntityViewStop::Complete
                        }
                    );
                    if refused {
                        assert_eq!(record.end_bit as usize, new_stop);
                        assert!(read.controls.is_none());
                    }
                } else if i == 2 {
                    // NEW's known schema survives its incomplete default body.
                    assert_eq!(read.records[0].stop, EntityViewStop::Complete);
                } else if !configuration {
                    let controls = read.controls.as_ref().unwrap();
                    assert_eq!(
                        controls.stop,
                        FrameViewStop::Unsupported {
                            reason: "global reference range".into()
                        }
                    );
                    assert_eq!(controls.end_bit as usize, control_stop);
                    assert!(
                        !controls
                            .fields
                            .iter()
                            .any(|f| f.name == "actions.tail.reference.value")
                    );
                } else {
                    assert_eq!(
                        read.controls.as_ref().unwrap().stop,
                        FrameViewStop::Complete
                    );
                }
            }
        }
    }
}
