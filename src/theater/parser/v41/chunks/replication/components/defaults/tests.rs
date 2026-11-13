//! Independent creation traces and enclosing default/component boundaries.
use super::*;
use crate::theater::parser::v41::chunks::replication::components::cursor::ComponentCursor;
use crate::theater::parser::v41::chunks::replication::source::components::field::{
    ComponentField, RawBits,
};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Case {
    name: String,
    entity_type: u32,
    snapshot: bool,
    hex: String,
    bit_len: usize,
    complete: bool,
}
impl Case {
    fn bytes(&self) -> Vec<u8> {
        self.hex
            .as_bytes()
            .chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
}
fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../fixtures/creation-v41/cases.json"
    ))
    .unwrap()
}
fn source_property_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../fixtures/property-v41/defaults.json"
    ))
    .unwrap()
}
fn disputed_property(case: &Case) -> bool {
    // Original reference reader masked bodies cycle all tags, including 12..15.
    case.entity_type == 13 && case.name.split('/').nth(2) == Some("True")
}
fn canonical_cases() -> Vec<Case> {
    cases()
        .into_iter()
        .filter(|c| !disputed_property(c))
        .chain(source_property_cases())
        .collect()
}
fn read(
    case: &Case,
    bytes: &[u8],
    start: usize,
    context: bool,
) -> (Option<bool>, i64, Vec<ComponentField>) {
    let mut encoding = crate::theater::parser::v41::context::profile::DecodeProfile::default()
        .component_encoding()
        .unwrap();
    encoding.world_axis_bits = Some([15, 15, 17]);
    encoding.region_axis_bits = [(0, [15, 15, 17])].into();
    let mut r = ComponentReader {
        category_references: true,
        snapshot: case.snapshot,
        cursor: ComponentCursor::guarded(bytes, start as i64),
        fields: Vec::new(),
        position_encoding: context.then_some(&encoding),
        reference_widths: None,
        width_error: None,
        context_error: None,
        live_grammar: None,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
    };
    assert!(has_reference_deserializer(case.entity_type));
    let status = state(&mut r, case.entity_type, [9, 5]);
    (status, r.cursor.position, r.fields)
}

#[test]
fn defaults_match_independent_creation_traces() {
    #[derive(Deserialize)]
    struct Oracle {
        name: String,
        end: i64,
        complete: bool,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Fixture {
        commit: String,
        cases: Vec<Oracle>,
    }
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../../../fixtures/creation-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(fixture.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    let cases = cases();
    assert_eq!(fixture.cases.len(), cases.len());
    // Every nonempty creation reader implemented by the pinned upstream has
    // independent cases. This checks type admission, not exhaustive branches.
    assert_eq!(
        cases.iter().map(|c| c.entity_type).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            3, 5, 6, 9, 10, 11, 12, 13, 14, 17, 21, 29, 35, 36, 37, 38, 39, 40, 41, 42, 43, 47, 49,
        ])
    );
    assert_eq!(cases.iter().filter(|c| disputed_property(c)).count(), 32);
    for (case, expected) in cases.iter().zip(fixture.cases) {
        assert_eq!(case.name, expected.name);
        if disputed_property(case) {
            // These historical bodies encode each enum as 32 bits. They are
            // retained as upstream evidence, replaced in canonical boundary /
            // truncation checks by original-Go source traces for native bodies.
            assert_eq!(case.bit_len, 612);
            assert_eq!(expected.end, 612);
            continue;
        }
        let bytes = case.bytes();
        let (status, end, fields) = read(case, &bytes, 0, true);
        assert_eq!(
            (status, end),
            (Some(expected.complete), expected.end),
            "{}",
            case.name
        );
        assert_eq!(
            (expected.complete, end as usize),
            (case.complete, case.bit_len)
        );
        let mut boundaries = BTreeSet::from([0]);
        let mut cursor = 0;
        for field in &fields {
            assert_eq!(field.bit, cursor, "{}: {}", case.name, field.name);
            assert_eq!(
                field.raw,
                RawBits::from_source(&bytes, field.bit, field.width).unwrap()
            );
            cursor += field.width;
            boundaries.insert(cursor);
        }
        assert_eq!(cursor, case.bit_len);
        for (start, width) in expected.trace {
            assert!(boundaries.contains(&start), "{}: start {start}", case.name);
            assert!(
                boundaries.contains(&(start + width)),
                "{}: end {}",
                case.name,
                start + width
            );
        }
        let json = serde_json::to_string(&fields).unwrap();
        assert_eq!(
            fields,
            serde_json::from_str::<Vec<ComponentField>>(&json).unwrap()
        );
        if case.snapshot {
            for field in &fields {
                if field.name == "projectile.position_bits[1]"
                    || field.name == "media_frame.vector_bits[1]"
                    || field.name == "reference.position.float_bits[1]"
                {
                    assert_eq!((field.raw.low_u64() as u32).swap_bytes(), 0x7FC12345);
                }
                if field.name == "projectile.position_bits[0]"
                    || field.name == "media_frame.vector_bits[0]"
                    || field.name == "reference.position.float_bits[0]"
                {
                    assert_eq!(
                        (field.raw.low_u64() as u32).swap_bytes(),
                        (-0.0f32).to_bits()
                    );
                }
            }
            // Raw snapshot words never require map quantization context.
            assert_eq!(read(case, &bytes, 0, false), (status, end, fields));
        }
    }
}

#[test]
fn creation_truncation_preserves_each_complete_source_field() {
    for case in canonical_cases() {
        let source = case.bytes();
        let (_, _, complete) = read(&case, &source, 0, true);
        for cut in 0..case.bit_len {
            let shift = (8 - cut % 8) % 8;
            let mut bytes = vec![0; (cut + shift) / 8];
            for i in 0..cut {
                bytes[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let (status, end, fields) = read(&case, &bytes, shift, true);
            assert_eq!(status, None, "{}: cut {cut}", case.name);
            assert!(end <= bytes.len() as i64 * 8);
            assert_eq!(
                fields.len(),
                complete
                    .iter()
                    .take_while(|f| f.bit + f.width <= cut)
                    .count(),
                "{} cut {cut}",
                case.name
            );
            for (actual, expected) in fields.iter().zip(&complete) {
                assert_eq!(
                    (&actual.name, actual.bit, actual.width, &actual.raw),
                    (
                        &expected.name,
                        expected.bit + shift,
                        expected.width,
                        &expected.raw
                    ),
                    "{} cut {cut}",
                    case.name
                );
            }
        }
    }
}

fn push(wire: &mut Vec<bool>, width: usize, value: u64) {
    wire.extend((0..width).rev().map(|bit| value & (1 << bit) != 0));
}
fn bytes(wire: Vec<bool>) -> Vec<u8> {
    let mut bytes = vec![0; wire.len().div_ceil(8)];
    for (i, bit) in wire.into_iter().enumerate() {
        bytes[i / 8] |= u8::from(bit) << (7 - i % 8);
    }
    bytes
}

#[test]
fn managed_player_participant_matches_f050371_at_every_value_and_alignment() {
    #[derive(Deserialize)]
    struct ManagedCase {
        name: String,
        hex: String,
        at: usize,
        bit_len: usize,
        participant: u8,
    }
    #[derive(Deserialize)]
    struct OracleCase {
        name: String,
        value: Option<u32>,
    }
    #[derive(Deserialize)]
    struct Oracle {
        commit: String,
        cases: Vec<OracleCase>,
    }
    let cases: Vec<ManagedCase> = serde_json::from_str(include_str!(
        "../../../../../../fixtures/managed-participant-v41/cases.json"
    ))
    .unwrap();
    let oracle: Oracle = serde_json::from_str(include_str!(
        "../../../../../../fixtures/managed-participant-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(oracle.commit, "f050371a6cda83b9887eeb83a1884336fce28e53");
    assert_eq!(cases.len(), 1024);
    assert_eq!(oracle.cases.len(), cases.len());
    for (case, expected) in cases.iter().zip(oracle.cases) {
        assert_eq!(case.name, expected.name);
        assert_eq!(
            expected.value,
            (case.participant < 32).then_some(u32::from(case.participant))
        );
        let c = Case {
            name: case.name.clone(),
            entity_type: 9,
            snapshot: false,
            hex: case.hex.clone(),
            bit_len: case.bit_len,
            complete: true,
        };
        let data = c.bytes();
        let (status, end, fields) = read(&c, &data, case.at, true);
        assert_eq!(status, Some(true), "{}", case.name);
        assert_eq!(end as usize, case.at + case.bit_len, "{}", case.name);
        let field = fields.iter().find(|f| f.name == "player_index").unwrap();
        assert_eq!(
            (field.width, field.raw.low_u64()),
            (6, u64::from(case.participant))
        );
        assert_eq!(
            field.raw,
            RawBits::from_source(&data, field.bit, field.width).unwrap()
        );
        assert_eq!(
            RawBits::from_source(&data, end as usize, 8)
                .unwrap()
                .low_u64(),
            0xd6
        );
        let mut default = crate::theater::parser::v41::chunks::replication::source::DefaultState {
            source: crate::theater::parser::packet::BitRange {
                start: case.at,
                end: end as usize,
            },
            fields,
            status: crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::Complete,
        };
        assert_eq!(
            default.managed_player_participant().map(u32::from),
            expected.value
        );
        default.status =
            crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::Truncated;
        assert_eq!(default.managed_player_participant(), None);
    }
}

#[test]
fn managed_player_participant_is_available_on_complete_new_and_keyframe_records_only() {
    use crate::theater::parser::test_support::component_registry;
    use crate::theater::parser::v41::chunks::replication::source::{EntityViewStop, KeyframeStop};
    use crate::theater::parser::v41::{
        chunks::replication::replication_stream::{
            frame::records::{RecordReadContext, decode_entity_record},
            keyframes::record::decode_reference_keyframe_record_contextual,
        },
        context::V41DecodeConfig,
    };
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../../../../fixtures/managed-participant-v41/cases.json"
    ))
    .unwrap();
    let registry = component_registry(&[(9, vec![("equipment-being-hacked-component".into(), 1)])]);
    let config = V41DecodeConfig::default();
    for versioned in [false, true] {
        for participant in [0_u8, 31, 32, 63] {
            let name = format!(
                "{}/{participant}/0",
                if versioned { "versioned" } else { "default" }
            );
            let case = cases.iter().find(|c| c["name"] == name).unwrap();
            let source: Vec<u8> = case["hex"]
                .as_str()
                .unwrap()
                .as_bytes()
                .chunks(2)
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect();
            let body_len = case["bit_len"].as_u64().unwrap() as usize;
            let expected = (participant < 32).then_some(participant);
            for snapshot in [false, true] {
                let mut wire = Vec::new();
                if snapshot {
                    push(&mut wire, 32, (1 << 30) | 42);
                    push(&mut wire, 32, 9);
                    push(&mut wire, 32, 0);
                    push(&mut wire, 4, 0);
                    push(&mut wire, 8, 0);
                    push(&mut wire, 32, 1);
                } else {
                    push(&mut wire, 3, 1);
                    push(&mut wire, 13, 42);
                    push(&mut wire, 2, 1);
                    push(&mut wire, 6, 9);
                }
                wire.extend((0..body_len).map(|i| source[i / 8] & (1 << (7 - i % 8)) != 0));
                if snapshot {
                    push(&mut wire, 32, 1);
                } else {
                    push(&mut wire, 1, 1);
                    push(&mut wire, 1, 1);
                    push(&mut wire, 64, 1);
                }
                push(&mut wire, 8, 0xd3);
                let data = bytes(wire);
                if snapshot {
                    let mut record = decode_reference_keyframe_record_contextual(
                        &data,
                        0,
                        &registry,
                        &config.record_layout(),
                        Some(&config.profile),
                    )
                    .unwrap();
                    assert_eq!(record.stop, KeyframeStop::Complete, "{name}");
                    assert_eq!(record.managed_player_participant(), expected, "{name}");
                    record.stop = KeyframeStop::Truncated;
                    assert_eq!(record.managed_player_participant(), None);
                } else {
                    let mut record = decode_entity_record(
                        &data,
                        0,
                        &registry,
                        &config.record_layout(),
                        None,
                        false,
                        RecordReadContext {
                            context: Some(&config.profile),
                            movement: None,
                        },
                    )
                    .unwrap();
                    assert_eq!(record.stop, EntityViewStop::Complete, "{name}");
                    assert_eq!(record.managed_player_participant(), expected, "{name}");
                    record.stop = EntityViewStop::Truncated;
                    assert_eq!(record.managed_player_participant(), None);
                }
            }
        }
    }
}

#[test]
fn creation_defaults_preserve_new_record_and_keyframe_boundaries() {
    use crate::theater::parser::test_support::component_registry;
    use crate::theater::parser::v41::chunks::replication::source::{
        DefaultStateStatus, EntityViewStop, KeyframeStop,
    };
    use crate::theater::parser::v41::{
        chunks::replication::replication_stream::{
            frame::records::{RecordReadContext, decode_entity_record},
            keyframes::record::decode_reference_keyframe_record_contextual,
        },
        context::V41DecodeConfig,
    };
    for case in canonical_cases() {
        let has_initial_components = case.snapshot
            || !matches!(
                case.entity_type,
                7 | 13 | 14 | 20 | 23 | 28 | 29 | 46 | 47 | 49
            );
        let registry = component_registry(&[(
            case.entity_type as usize,
            vec![("equipment-being-hacked-component".into(), 1)],
        )]);
        let mut config = V41DecodeConfig::default();
        config.profile.movement.world_object.axis_bits = [15, 15, 17];
        let mut wire = Vec::new();
        if case.snapshot {
            push(&mut wire, 32, (1 << 30) | 42);
            push(&mut wire, 32, u64::from(case.entity_type));
            push(&mut wire, 32, 0);
            push(&mut wire, 4, 0);
            push(&mut wire, 8, 0);
            push(&mut wire, 32, 1); // presence guard, not a body length
        } else {
            // NEW: prefix 001, entity index, generation, creation archetype.
            push(&mut wire, 3, 1);
            push(&mut wire, 13, 42);
            push(&mut wire, 2, 1);
            push(&mut wire, 6, u64::from(case.entity_type));
        }
        let start = wire.len();
        let source = case.bytes();
        wire.extend((0..case.bit_len).map(|i| source[i / 8] & (1 << (7 - i % 8)) != 0));
        if case.snapshot {
            push(&mut wire, 32, 1); // following component list guard
        } else if has_initial_components {
            push(&mut wire, 1, 1); // component list present
            push(&mut wire, 1, 1); // dense mask
            push(&mut wire, 64, 1);
        }
        push(&mut wire, 8, 0xD3);
        let data = bytes(wire);
        let (default, following, end) = if case.snapshot {
            let record = decode_reference_keyframe_record_contextual(
                &data,
                0,
                &registry,
                &config.record_layout(),
                Some(&config.profile),
            )
            .unwrap();
            assert_eq!(
                record.stop,
                if case.complete {
                    KeyframeStop::Complete
                } else {
                    KeyframeStop::UnsupportedDefault
                },
                "{}",
                case.name
            );
            (
                record.default_state.unwrap(),
                record
                    .components
                    .into_iter()
                    .map(|c| c.fields)
                    .collect::<Vec<_>>(),
                record.end_bit,
            )
        } else {
            let record = decode_entity_record(
                &data,
                0,
                &registry,
                &config.record_layout(),
                None,
                false,
                RecordReadContext {
                    context: Some(&config.profile),
                    movement: None,
                },
            )
            .unwrap();
            assert_eq!(
                record.stop,
                if case.complete {
                    EntityViewStop::Complete
                } else {
                    EntityViewStop::UnsupportedDefault {
                        archetype: case.entity_type,
                    }
                },
                "{}",
                case.name
            );
            (
                record.default_state.unwrap(),
                record
                    .components
                    .into_iter()
                    .map(|c| c.fields)
                    .collect::<Vec<_>>(),
                record.end_bit,
            )
        };
        assert_eq!(default.source.start, start, "{}", case.name);
        assert_eq!(default.source.end, start + case.bit_len, "{}", case.name);
        if case.complete {
            assert_eq!(default.status, DefaultStateStatus::Complete);
            assert_eq!(following.len(), usize::from(has_initial_components));
            if has_initial_components {
                assert_eq!(following[0][0].raw.low_u64(), 0xD3);
            }
            assert_eq!(
                end as usize,
                start
                    + case.bit_len
                    + if case.snapshot {
                        40
                    } else if has_initial_components {
                        74
                    } else {
                        0
                    }
            );
        } else {
            assert_eq!(default.status, DefaultStateStatus::Unsupported);
            assert!(following.is_empty());
            assert_eq!(end as usize, start + case.bit_len);
        }
    }
}

#[test]
fn property_defaults_match_original_source_reader_at_every_alignment() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../fixtures/property-v41/source-reference.json"
    ))
    .unwrap();
    assert_eq!(
        oracle["blobs"]["default_state_arch.go"],
        "53f0146423b00f404c41791e5ca00f4eb00a6ea9"
    );
    let traces = oracle["defaults"].as_array().unwrap();
    let cases = source_property_cases();
    assert_eq!(cases.len(), 128);
    assert_eq!(cases.len(), traces.len());
    for (case, trace) in cases.iter().zip(traces) {
        assert_eq!(case.name, trace["name"]);
        for shift in 0..8 {
            let mut wire = vec![false; shift];
            let source = case.bytes();
            wire.extend((0..case.bit_len).map(|i| source[i / 8] & (1 << (7 - i % 8)) != 0));
            push(&mut wire, 32, 0xdeadbeef);
            let data = bytes(wire);
            let (status, end, fields) = read(case, &data, shift, true);
            assert_eq!(status, Some(true));
            assert_eq!(end as usize, shift + case.bit_len);
            assert_eq!(case.bit_len, trace["end"].as_u64().unwrap() as usize);
            let boundaries: BTreeSet<_> = std::iter::once(shift)
                .chain(fields.iter().map(|f| f.bit + f.width))
                .collect();
            for t in trace["trace"].as_array().unwrap() {
                let at = t[0].as_u64().unwrap() as usize;
                let n = t[1].as_u64().unwrap() as usize;
                assert!(boundaries.contains(&(shift + at)));
                assert!(boundaries.contains(&(shift + at + n)));
            }
            for f in &fields {
                assert_eq!(
                    RawBits::from_source(&data, f.bit, f.width).as_ref(),
                    Some(&f.raw)
                );
            }
            assert_eq!(
                RawBits::from_source(&data, end as usize, 32)
                    .unwrap()
                    .low_u64(),
                0xdeadbeef
            );
            assert_eq!(
                fields,
                serde_json::from_str::<Vec<ComponentField>>(
                    &serde_json::to_string(&fields).unwrap()
                )
                .unwrap()
            );
        }
    }
}
