use super::*;

const TEST_LIMIT: usize = 64;

fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/theater/fixtures/control-limit-v41/cases.json"
    )))
    .unwrap()
}
fn shifted(data: &[u8], bits: usize, shift: usize) -> Vec<u8> {
    let mut out = vec![0; (bits + shift).div_ceil(8)];
    for i in 0..bits {
        out[(i + shift) / 8] |= ((data[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
    }
    out
}
fn prefix_bits(case: &Case) -> usize {
    case.kinds
        .iter()
        .take(TEST_LIMIT)
        .map(|kind| match kind {
            0 => 11,
            1 => 10,
            2 => 5,
            _ => unreachable!(),
        })
        .sum()
}

#[test]
fn control_budget_boundary_matches_upstream_terminators_and_preserves_limit_prefix() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/theater/fixtures/control-limit-v41/reference.json"
    )))
    .unwrap();
    let cases = cases();
    assert_eq!(cases.len(), 24);
    assert_eq!(oracle["cases"].as_array().unwrap().len(), cases.len());
    for (case, expected) in cases.iter().zip(oracle["cases"].as_array().unwrap()) {
        assert_eq!(expected["name"], case.name);
        assert_eq!(expected["end"], case.bit_len);
        for shift in 0..8 {
            let data = shifted(&case.data(), case.bit_len, shift);
            let view =
                decode_control_view_with_limit(&data, shift as i64, None, None, true, TEST_LIMIT);
            let complete = case.kinds.len() <= TEST_LIMIT;
            let end = if complete {
                case.bit_len
            } else {
                prefix_bits(case) + 1
            };
            assert_eq!(
                view.stop,
                if complete {
                    FrameViewStop::Complete
                } else {
                    FrameViewStop::RecordLimit
                }
            );
            assert_eq!(view.end_bit, (shift + end) as i64);
            assert_eq!(view.kinds, case.kinds[..case.kinds.len().min(TEST_LIMIT)]);
            assert_eq!(view.control_entries.len(), case.kinds.len().min(TEST_LIMIT));
            let mut boundaries = BTreeSet::from([shift]);
            let mut at = shift;
            for field in &view.fields {
                assert_eq!(field.bit, at);
                assert_eq!(
                    field.raw,
                    RawBits::from_source(&data, at, field.width).unwrap()
                );
                at += field.width;
                boundaries.insert(at);
            }
            assert_eq!(at, shift + end);
            for read in expected["trace"].as_array().unwrap() {
                let start = read[0].as_u64().unwrap() as usize;
                let width = read[1].as_u64().unwrap() as usize;
                if start + width <= end {
                    assert!(boundaries.contains(&(shift + start)));
                    assert!(boundaries.contains(&(shift + start + width)));
                }
            }
            let last = view.fields.last().unwrap();
            assert_eq!(
                last.name,
                format!("record[{}].present", case.kinds.len().min(TEST_LIMIT))
            );
            assert_eq!(last.raw.low_u64(), u64::from(!complete));
        }
    }
}

#[test]
fn exactly_full_control_list_requires_a_real_terminator_bit() {
    for case in cases().iter().filter(|c| c.kinds.len() == TEST_LIMIT) {
        let cut = case.bit_len - 1;
        let shift = (8 - cut % 8) % 8;
        let data = shifted(&case.data(), cut, shift);
        assert_eq!((shift + cut) % 8, 0);
        let view =
            decode_control_view_with_limit(&data, shift as i64, None, None, true, TEST_LIMIT);
        assert_eq!(view.stop, FrameViewStop::Truncated);
        assert_eq!(view.end_bit, (shift + cut) as i64);
        assert_eq!(view.control_entries.len(), TEST_LIMIT);
        assert!(!view.fields.iter().any(|f| f.name == "record[64].present"));
    }
}

#[test]
fn control_limit_closure_and_refusal_propagate_to_public_film_coverage() {
    use crate::theater::film::Film;
    use crate::theater::film::replication::ReplicationChange;
    use crate::theater::parser::packet::{PacketRead, SourceRegionKind};
    use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;
    for case in cases()
        .iter()
        .filter(|c| c.name == "mixed/64" || c.name == "mixed/65")
    {
        let payload = shifted(&case.data(), case.bit_len, 5);
        let film = film_from_payload_with_limits(
            &payload,
            crate::theater::parser::ParseLimits {
                max_control_records: TEST_LIMIT,
                ..Default::default()
            },
        );
        let chunk = &film.replication_stream.0[0];
        let packet = &chunk.body.packets[0];
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
        else {
            panic!()
        };
        let control = frame.frame.decoded().unwrap().controls.as_ref().unwrap();
        let end = 5 + prefix_bits(case) + 1;
        assert_eq!(control.end_bit, end as i64);
        assert_eq!(
            control.stop,
            if case.kinds.len() == 64 {
                FrameViewStop::Complete
            } else {
                FrameViewStop::RecordLimit
            }
        );
        assert_eq!(chunk.payload(packet).unwrap(), payload);
        let coverage = packet.source_coverage(&chunk.data).unwrap();
        assert_eq!(coverage[0].kind, SourceRegionKind::Fields);
        assert_eq!(coverage[0].source, BitRange { start: 0, end });
        if case.kinds.len() == 65 {
            assert!(
                coverage
                    .iter()
                    .any(|r| r.kind == SourceRegionKind::Unparsed && r.source.start == end)
            );
        }
        let typed = crate::theater::parser::test_support::parse_fixture_with_limits(
            typed_film_input(&payload),
            crate::theater::parser::ParseLimits {
                max_control_records: TEST_LIMIT,
                ..Default::default()
            },
        )
        .unwrap();
        let original = serde_json::to_vec(&typed).unwrap();
        assert_eq!(serde_json::from_slice::<Film>(&original).unwrap(), typed);
        let runtime = crate::theater::TheaterRuntime::load(typed);
        assert_eq!(
            runtime
                .observations()
                .iter()
                .filter(|observation| matches!(
                    &observation.change,
                    ReplicationChange::Control { .. }
                ))
                .count(),
            64
        );
    }
}

#[test]
fn default_film_budget_admits_upstream_65_and_128_record_control_lists() {
    use crate::theater::parser::packet::PacketRead;
    use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;
    for case in cases()
        .iter()
        .filter(|c| c.name == "mixed/65" || c.name == "mixed/128")
    {
        let payload = shifted(&case.data(), case.bit_len, 5);
        let film = film_from_payload(&payload);
        let chunk = &film.replication_stream.0[0];
        let packet = &chunk.body.packets[0];
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
        else {
            panic!("frame packet")
        };
        let control = frame.frame.decoded().unwrap().controls.as_ref().unwrap();
        assert_eq!(control.stop, FrameViewStop::Complete, "{}", case.name);
        assert_eq!(control.end_bit, (5 + case.bit_len) as i64);
        assert_eq!(control.control_entries.len(), case.kinds.len());
        assert_eq!(control.kinds, case.kinds);
        assert_eq!(chunk.payload(packet).unwrap(), payload);
        assert!(packet.source_coverage(&chunk.data).is_some());
    }
}
