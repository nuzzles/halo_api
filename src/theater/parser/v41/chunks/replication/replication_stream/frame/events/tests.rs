//! Independent upstream traces, bounded prefixes, and actual frame integration.
use super::*;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Case {
    name: String,
    code: u8,
    hex: String,
    bit_len: usize,
    record_end: usize,
    complete: bool,
    indexed_points: Vec<usize>,
    #[serde(default)]
    start_bit: usize,
    #[serde(default)]
    fire_v3: bool,
}
impl Case {
    fn bytes(&self) -> Vec<u8> {
        self.hex
            .as_bytes()
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }
}
fn upstream_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-v41/cases.json"
    ))
    .unwrap()
}
fn cases() -> Vec<Case> {
    let mut cases = upstream_cases();
    // These old cases contain references only, because their pin refused the
    // body. Keep checking their exact reference traces below; full-body and
    // truncation coverage now comes from the new pin's independent fixtures.
    cases.retain(|c| !matches!(c.code, 14 | 104 | 106));
    cases.extend(new_cases());
    cases.extend(
        serde_json::from_str::<Vec<Case>>(include_str!(
            "../../../../../../../fixtures/events-v41/projectile-source.json"
        ))
        .unwrap(),
    );
    cases
}
fn new_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-f050371-v41/cases.json"
    ))
    .unwrap()
}
fn fields(read: &EventListRead) -> impl Iterator<Item = &EventField> {
    read.records
        .iter()
        .flat_map(|r| &r.fields)
        .chain(read.terminator.iter())
}
fn vector_encoding() -> PrecisionDescriptor {
    PrecisionDescriptor {
        regions: Default::default(),
        index_bits: 1,
        axis_bits: [15, 15, 17],
        region: 0,
    }
}

fn parse(data: &[u8], start: usize, fire_v3: bool) -> EventListRead {
    let vector = vector_encoding();
    read_contextual_event_list(
        data,
        start,
        None,
        256,
        Some(&vector),
        fire_v3.then_some((&vector).into()),
        Some(&projectile_encoding()),
    )
}

fn projectile_encoding() -> ProjectilePositionEncoding {
    ProjectilePositionEncoding {
        detonation: PrecisionDescriptor {
            regions: Default::default(),
            index_bits: 1,
            axis_bits: [14, 14, 16],
            region: 0,
        },
        impact: PrecisionDescriptor {
            regions: Default::default(),
            index_bits: 1,
            axis_bits: [11, 11, 13],
            region: 0,
        },
    }
}

#[test]
fn events_match_independent_reader_traces_and_reference_kinds() {
    #[derive(Deserialize)]
    struct Oracle {
        name: String,
        end: usize,
        reader_end: usize,
        complete: bool,
        count: usize,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Fixture {
        commit: String,
        handle_kinds: Vec<[i8; 3]>,
        cases: Vec<Oracle>,
    }
    let oracle: Fixture = serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(oracle.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    assert_eq!(oracle.handle_kinds, CONFIG);
    let cases = upstream_cases();
    assert_eq!(cases.len(), oracle.cases.len());
    for (case, expected) in cases.iter().zip(oracle.cases) {
        let data = case.bytes();
        let parsed = parse(&data, case.start_bit, case.fire_v3);
        assert_eq!(case.name, expected.name);
        if matches!(case.code, 14 | 104 | 106) {
            assert!(!expected.complete);
            let retained: Vec<_> = fields(&parsed)
                .filter(|f| f.bit + f.width <= expected.end)
                .collect();
            assert_eq!(
                retained.last().unwrap().bit + retained.last().unwrap().width,
                expected.end
            );
            let boundaries: BTreeSet<_> = retained
                .iter()
                .flat_map(|f| [f.bit, f.bit + f.width])
                .collect();
            for (start, width) in expected.trace {
                assert!(boundaries.contains(&start));
                assert!(boundaries.contains(&(start + width)));
            }
            for field in retained {
                if let EventFieldValue::Scalar(v) = field.value {
                    assert_eq!(
                        crate::theater::parser::v41::chunks::replication::source::components::field::RawBits::from_source(&data, field.bit, field.width)
                            .unwrap()
                            .low_u64(),
                        v
                    );
                }
            }
            continue;
        }
        if expected.reader_end != expected.end {
            assert_eq!(case.name, "action/18/refused");
            assert!(!expected.complete);
            assert_eq!(expected.end, expected.reader_end + 1);
        }
        assert_eq!(
            parsed.stop,
            if expected.complete {
                EventListStop::Terminator
            } else {
                EventListStop::UnsupportedBody
            },
            "{}",
            case.name
        );
        assert_eq!(parsed.end_bit, expected.end, "{}", case.name);
        assert_eq!(parsed.end_bit, case.bit_len);
        assert_eq!(
            parsed.records.iter().filter(|r| r.layout_complete).count(),
            expected.count
        );
        assert_eq!(parsed.records[0].code, Some(case.code));
        assert_eq!(parsed.records[0].end_bit, case.record_end);
        if case.code == 64 {
            let body = &parsed.records[0].fields;
            let floats = &body[body.len() - 4..body.len() - 1];
            assert_eq!(
                floats
                    .iter()
                    .map(|f| (&f.value, f.width))
                    .collect::<Vec<_>>(),
                [
                    (&EventFieldValue::Scalar(0x00000080), 32),
                    (&EventFieldValue::Scalar(0x4523C17F), 32),
                    (&EventFieldValue::Scalar(0x0000803F), 32),
                ]
            );
        }
        if case.complete {
            assert_eq!(parsed.records[1].code, Some(21));
            assert_eq!(
                parsed.records[1].fields.last().unwrap().value,
                EventFieldValue::Scalar(2)
            );
        }
        let mut boundaries = BTreeSet::from([case.start_bit]);
        let mut end = case.start_bit;
        for field in fields(&parsed) {
            assert_eq!(field.bit, end);
            let raw =
                crate::theater::parser::v41::chunks::replication::source::components::field::RawBits::from_source(&data, field.bit, field.width).unwrap();
            if let EventFieldValue::Scalar(value) = field.value {
                assert_eq!(value, raw.low_u64());
            } else {
                assert_eq!(field.value, EventFieldValue::Opaque);
            }
            end += field.width;
            boundaries.insert(end);
        }
        assert_eq!(end, case.bit_len);
        for (start, width) in expected.trace {
            assert!(boundaries.contains(&start), "{}: start {start}", case.name);
            assert!(
                boundaries.contains(&(start + width)),
                "{}: end {}",
                case.name,
                start + width
            );
        }
        let roundtrip: EventListRead =
            serde_json::from_slice(&serde_json::to_vec(&parsed).unwrap()).unwrap();
        assert_eq!(parsed, roundtrip);
    }
}

#[test]
fn projectile_branches_match_original_source_traces() {
    #[derive(Deserialize)]
    struct SourceCase {
        #[serde(flatten)]
        case: Case,
        body_start: usize,
        wide: bool,
    }
    #[derive(Deserialize)]
    struct Oracle {
        name: String,
        end: usize,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Fixture {
        commit: String,
        cases: Vec<Oracle>,
    }
    let source: Vec<SourceCase> = serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-v41/projectile-source.json"
    ))
    .unwrap();
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-v41/projectile-source-reference.json"
    ))
    .unwrap();
    assert_eq!(fixture.commit, "413971bcc485be53a2fa8cf229e170539a5146f8");
    assert_eq!(source.len(), fixture.cases.len());
    for (source, oracle) in source.iter().zip(fixture.cases) {
        let case = &source.case;
        let data = case.bytes();
        let parsed = parse(&data, 0, false);
        assert_eq!(case.name, oracle.name);
        assert_eq!(parsed.stop, EventListStop::Terminator, "{}", case.name);
        assert_eq!(parsed.end_bit, case.bit_len);
        assert_eq!(parsed.records.len(), 2);
        let event = &parsed.records[0];
        assert!(event.layout_complete);
        assert_eq!(event.body_start_bit, Some(source.body_start));
        assert_eq!(event.end_bit, case.record_end);
        assert_eq!(event.end_bit, oracle.end);
        assert_eq!(parsed.records[1].code, Some(21));
        assert_eq!(
            parsed.records[1].fields.last().unwrap().value,
            EventFieldValue::Scalar(2)
        );
        if case.code == 5 {
            let direction = &event.fields[event.fields.len() - 2];
            assert_eq!(direction.width, if source.wide { 19 } else { 8 });
            assert_eq!(
                direction.value,
                EventFieldValue::Scalar(if source.wide { 0x5ABCD } else { 213 })
            );
        }
        let mut boundaries = BTreeSet::from([0]);
        let mut end = 0;
        for field in fields(&parsed) {
            assert_eq!(field.bit, end);
            let raw =
                crate::theater::parser::v41::chunks::replication::source::components::field::RawBits::from_source(&data, field.bit, field.width).unwrap();
            assert_eq!(field.value, EventFieldValue::Scalar(raw.low_u64()));
            end += field.width;
            boundaries.insert(end);
        }
        assert_eq!(end, case.bit_len);
        for (start, width) in oracle.trace {
            assert!(boundaries.contains(&start), "{}: start {start}", case.name);
            assert!(
                boundaries.contains(&(start + width)),
                "{}: end {}",
                case.name,
                start + width
            );
        }
        let restored: EventListRead =
            serde_json::from_slice(&serde_json::to_vec(&parsed).unwrap()).unwrap();
        assert_eq!(parsed, restored);
    }
}

#[test]
fn new_event_bodies_match_f050371_traces_at_every_alignment() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-f050371-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(oracle["commit"], "f050371a6cda83b9887eeb83a1884336fce28e53");
    let cases = new_cases();
    let expected = oracle["cases"].as_array().unwrap();
    assert_eq!(cases.len(), expected.len());
    assert_eq!(cases.len(), 162);
    assert_eq!(
        cases.iter().map(|c| c.code).collect::<BTreeSet<_>>(),
        [2, 8, 12, 13, 14, 80, 104, 105, 106].into()
    );
    for (case, expected) in cases.iter().zip(expected) {
        assert_eq!(case.name, expected["name"]);
        for shift in 0..8 {
            let source = case.bytes();
            let mut data = vec![0; (shift + case.bit_len).div_ceil(8)];
            for i in 0..case.bit_len {
                data[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let read = parse(&data, shift, false);
            assert_eq!(
                read.stop,
                EventListStop::Terminator,
                "{} shift {shift}",
                case.name
            );
            assert_eq!(read.end_bit, case.bit_len + shift);
            assert_eq!(read.records.len(), 2);
            assert_eq!(read.records[0].end_bit, case.record_end + shift);
            assert_eq!(read.records[1].code, Some(21));
            let mut boundaries = BTreeSet::from([0]);
            for field in fields(&read) {
                boundaries.extend([field.bit - shift, field.bit + field.width - shift]);
                assert_eq!(
                    field.value,
                    EventFieldValue::Scalar(
                        crate::theater::parser::v41::chunks::replication::source::components::field::RawBits::from_source(&data, field.bit, field.width)
                            .unwrap()
                            .low_u64()
                    )
                );
            }
            for entry in expected["trace"].as_array().unwrap() {
                let start = entry[0].as_u64().unwrap() as usize;
                let width = entry[1].as_u64().unwrap() as usize;
                assert!(boundaries.contains(&start), "{} start {start}", case.name);
                assert!(
                    boundaries.contains(&(start + width)),
                    "{} end {}",
                    case.name,
                    start + width
                );
            }
            let restored: EventListRead =
                serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap();
            assert_eq!(read, restored);
        }
    }
}

#[test]
fn events_retain_every_complete_prefix_without_padding_or_zero_fill() {
    for case in cases() {
        let source = case.bytes();
        let full = parse(&source, case.start_bit, case.fire_v3);
        for cut in case.start_bit..case.bit_len {
            // A fire fixture always keeps a separate configuration=1 at bit
            // zero, even when its event list is shifted for an exact bit cut.
            let shift = if case.fire_v3 {
                8 - cut % 8
            } else {
                (8 - cut % 8) % 8
            };
            let mut data = vec![0; (cut + shift) / 8];
            if case.fire_v3 {
                data[0] = 0x80;
            }
            for i in 0..cut {
                data[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let partial = parse(&data, shift + case.start_bit, case.fire_v3);
            assert!(
                matches!(
                    partial.stop,
                    EventListStop::Truncated | EventListStop::SourceBoundary
                ),
                "{}: cut {cut}: {:?}",
                case.name,
                partial.stop
            );
            assert!(partial.end_bit <= cut + shift);
            let actual: Vec<_> = fields(&partial)
                .filter(|f| f.value != EventFieldValue::Unavailable)
                .collect();
            let expected: Vec<_> = fields(&full)
                .take_while(|f| f.bit + f.width <= cut)
                .collect();
            assert_eq!(actual.len(), expected.len(), "{}: cut {cut}", case.name);
            for (a, b) in actual.iter().zip(expected) {
                assert_eq!(
                    (a.bit, a.width, a.stage, &a.value),
                    (b.bit + shift, b.width, b.stage, &b.value)
                );
            }
        }
    }
}

#[test]
fn default_box_projectiles_do_not_become_map_region_positions() {
    let encoding = projectile_encoding();
    for case in cases().into_iter().filter(|c| c.code == 5) {
        let data = case.bytes();
        let events = parse(&data, case.start_bit, case.fire_v3);
        let event = &events.records[0];
        assert!(event.layout_complete);
        let map_position = detonation_position(&data, event, &encoding);
        if case.indexed_points.is_empty() {
            assert_eq!(map_position, None, "{}", case.name);
            let no_map = read_reference_event_list(&data, 0, None, 256);
            assert_eq!(no_map, events);
        } else {
            assert_eq!(map_position, Some([16383, 16382, 65533]), "{}", case.name);
        }
    }
}

#[test]
fn event_points_require_context_only_after_the_recorded_indexed_gate() {
    for case in cases() {
        let data = case.bytes();
        let full = parse(&data, case.start_bit, case.fire_v3);
        let no_map = read_contextual_event_list(
            &data,
            case.start_bit,
            None,
            256,
            None,
            case.fire_v3.then_some(FireV3 { position: None }),
            None,
        );
        if let Some(point) = case.indexed_points.first() {
            assert_eq!(no_map.stop, EventListStop::UnsupportedBody);
            assert_eq!(no_map.end_bit, point + 1, "{}", case.name);
            assert_eq!(
                fields(&no_map).collect::<Vec<_>>(),
                fields(&full)
                    .take_while(|f| f.bit <= *point)
                    .collect::<Vec<_>>()
            );
        } else {
            assert_eq!(no_map, full, "{}", case.name);
        }
        for (index_bits, axes, region) in [
            (31, [15, 15, 17], 0),
            (1, [33, 15, 17], 0),
            (1, [15, 33, 17], 0),
            (1, [15, 15, 17], 1),
        ] {
            let encoding = PrecisionDescriptor {
                index_bits,
                axis_bits: axes,
                region,
                regions: Default::default(),
            };
            let projectile = ProjectilePositionEncoding {
                detonation: encoding.clone(),
                impact: encoding.clone(),
            };
            let read = read_contextual_event_list(
                &data,
                case.start_bit,
                None,
                256,
                Some(&encoding),
                case.fire_v3.then_some((&encoding).into()),
                Some(&projectile),
            );
            if let Some(point) = case.indexed_points.first() {
                assert_eq!(read.stop, EventListStop::UnsupportedBody);
                assert_eq!(read.end_bit, point + if region == 1 { 2 } else { 1 });
            } else {
                assert_eq!(read, full, "{}", case.name);
            }
        }
    }
}

#[test]
fn event_boundaries_control_admission_to_entity_and_control_views() {
    use crate::theater::parser::packet::PacketRead;
    use crate::theater::parser::v41::chunks::replication::source::{
        FrameDecodeError, FrameRead, ReplicationStreamPacketBody,
    };
    for case in cases() {
        let source = case.bytes();
        // Explicit configuration, event list, entity terminator, empty controls.
        let prefix = 1 - case.start_bit;
        let mut payload = vec![0; (prefix + case.bit_len + 4).div_ceil(8)];
        payload[0] = 0x80; // recorded category-relative reference configuration
        for i in case.start_bit..case.bit_len {
            payload[(i + prefix) / 8] |=
                ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + prefix) % 8);
        }
        let film = film(&payload, case.fire_v3);
        let chunk = &film.replication_stream.0[0];
        let packet = &chunk.body.packets[0];
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
        else {
            panic!("frame");
        };
        if case.fire_v3 {
            assert_eq!(
                frame.events.stop,
                EventListStop::UnsupportedEventVersion {
                    event_code: 36,
                    recorded: 0,
                    supported: 3,
                }
            );
            assert_eq!(frame.events.end_bit, 9);
            assert!(frame.events.records[0].body_start_bit.is_none());
            assert_eq!(
                frame.frame,
                FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
            );
        } else if case.code == 116
            && case.name.split('/').nth(2).unwrap().parse::<u8>().unwrap() & 1 != 0
        {
            let start = frame.events.records[0].body_start_bit.unwrap();
            assert_eq!(frame.events.stop, EventListStop::MissingVariantContext);
            assert_eq!(frame.events.end_bit, start + 1);
            assert_eq!(
                frame.frame,
                FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
            );
        } else if let Some(point) = case.indexed_points.first() {
            // Film's default config has no map descriptor: retain the recorded
            // region gate, then refuse before the index width is needed.
            assert_eq!(frame.events.end_bit, 1 + point + 1);
            assert_eq!(frame.events.stop, EventListStop::UnsupportedBody);
            assert_eq!(
                frame.frame,
                FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
            );
        } else if case.complete {
            assert_eq!(frame.events.records[0].end_bit, 1 + case.record_end);
            assert_eq!(frame.events.end_bit, 1 + case.bit_len);
            let decoded = frame.frame.decoded().unwrap();
            assert!(decoded.records.is_empty());
            assert_eq!(decoded.end_bit, (1 + case.bit_len + 4) as i64);
            assert_eq!(
                decoded.controls.as_ref().unwrap().stop,
                crate::theater::parser::v41::chunks::replication::source::FrameViewStop::Complete
            );
        } else {
            assert_eq!(frame.events.records[0].end_bit, 1 + case.record_end);
            assert_eq!(
                frame.frame,
                FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
            );
        }
        assert!(packet.source_coverage(&chunk.data).is_some());
        assert_eq!(packet.payload(&chunk.data).unwrap(), payload);
        if case.complete && (case.fire_v3 || !case.indexed_points.is_empty()) {
            use crate::theater::parser::v41::{
                chunks::replication::{
                    replication_stream::{DecodeContext, decode},
                    state::ReplicationDecodeState,
                },
                context::V41DecodeConfig,
            };
            let mut config = V41DecodeConfig {
                event_variant: VariantContext {
                    engine_type: Some(2),
                    ..Default::default()
                },
                event_quantized_vectors: true,
                event_fire_v3: case.fire_v3,
                projectile_event_positions: Some(projectile_encoding()),
                ..Default::default()
            };
            config.profile.movement.world_object = vector_encoding();
            let mut state = ReplicationDecodeState::default();
            let mut context = DecodeContext {
                config: &config,
                registry: &film.registry.body.registry,
                state: &mut state,
                event_gate15: None,
            };
            let ReplicationStreamPacketBody::FramePacketBody(frame) =
                decode(0, &payload, &mut context).unwrap()
            else {
                panic!("frame");
            };
            assert_eq!(frame.events.end_bit, prefix + case.bit_len, "{}", case.name);
            assert_eq!(
                frame.frame.decoded().unwrap().end_bit,
                (prefix + case.bit_len + 4) as i64
            );
        }
    }
}

#[test]
fn fire_requires_explicit_v3_configuration_and_matching_point_context() {
    for case in cases().into_iter().filter(|c| c.fire_v3) {
        assert_eq!(case.start_bit, 1);
        let data = case.bytes();
        let full = parse(&data, case.start_bit, true);
        assert_eq!(full.stop, EventListStop::Terminator);
        let body_start = full.records[0].body_start_bit.unwrap();
        let no_version = parse(&data, case.start_bit, false);
        assert_eq!(no_version.stop, EventListStop::UnsupportedBody);
        assert_eq!(no_version.end_bit, body_start);
        let mut configuration_zero = data.clone();
        configuration_zero[0] &= 0x7f;
        let no_configuration = parse(&configuration_zero, case.start_bit, true);
        assert_eq!(no_configuration, no_version, "{}", case.name);
        for (index_bits, axis_bits, region) in [
            (31, [15, 15, 17], 0),
            (1, [33, 15, 17], 0),
            (1, [15, 33, 17], 0),
            (1, [15, 15, 17], 1),
        ] {
            let encoding = PrecisionDescriptor {
                index_bits,
                axis_bits,
                region,
                regions: Default::default(),
            };
            let parsed = read_contextual_event_list(
                &data,
                case.start_bit,
                None,
                256,
                None,
                Some((&encoding).into()),
                None,
            );
            if let Some(point) = case.indexed_points.first() {
                assert_eq!(parsed.stop, EventListStop::UnsupportedBody);
                assert_eq!(parsed.end_bit, point + if region == 1 { 2 } else { 1 });
                assert_eq!(
                    fields(&parsed).collect::<Vec<_>>(),
                    fields(&full)
                        .take_while(|f| f.bit < parsed.end_bit)
                        .collect::<Vec<_>>(),
                    "{}",
                    case.name,
                );
            } else {
                assert_eq!(parsed, full, "{}", case.name);
            }
        }
    }
}

fn film(payload: &[u8], unsupported_fire: bool) -> crate::theater::parser::ParsedFilm {
    use crate::clients::hi::models::RawFilmChunk;
    use crate::theater::film::ChunkKind;
    let mut data = vec![0; 16];
    data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    data.extend_from_slice(payload);
    let chunk = |kind: ChunkKind, data| RawFilmChunk {
        kind: kind.into(),
        data,
        index: match kind {
            ChunkKind::Registry => 0,
            ChunkKind::Replication => 1,
            ChunkKind::Summary => 2,
        },
        start_ms: 0,
    };
    let mut registry = crate::theater::parser::test_support::registry_with_corruption_policy(false);
    if unsupported_fire {
        registry.data[0xcb208 + 36 * 4..0xcb208 + 37 * 4].fill(0);
    }
    crate::theater::parser::test_support::parse_fixture_structural(vec![
        registry,
        chunk(ChunkKind::Replication, data),
        crate::theater::parser::test_support::empty_summary(),
    ])
    .unwrap()
}
