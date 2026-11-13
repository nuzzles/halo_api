use super::*;
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::*;
use crate::theater::parser::FilmParseContext;
use crate::theater::parser::packet::*;
use crate::theater::parser::v41::chunks::registry::source::*;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::read_reference_event_list;
use crate::theater::parser::v41::chunks::replication::source::components::field::*;
use crate::theater::parser::v41::chunks::replication::source::*;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Case {
    name: String,
    hex: String,
    script_prefix: bool,
    bit_len: usize,
    record_end: usize,
    body_start: usize,
    body_widths: Vec<usize>,
}
fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!("../../../../fixtures/events-v41/script.json")).unwrap()
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
pub(crate) fn copy_bits(dest: &mut [u8], start: usize, source: &[u8], count: usize) {
    for i in 0..count {
        let mask = 1 << (7 - (start + i) % 8);
        let dst = &mut dest[(start + i) / 8];
        *dst = (*dst & !mask) | (((source[i / 8] >> (7 - i % 8)) & 1) * mask);
    }
}
pub(crate) fn registry(simulation: u8, version: u32) -> Vec<u8> {
    use std::io::Read;
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../../../../fixtures/bootstrap-v41/registry-prefix.bin.zlib").as_slice(),
    )
    .read_to_end(&mut data)
    .unwrap();
    let prefix_len = data.len();
    let identity: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/bootstrap-v41/identification.json"
    ))
    .unwrap();
    let options: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/bond-v2/game-options.json"
    ))
    .unwrap();
    let identity_bits = identity[0]["bit_len"].as_u64().unwrap() as usize;
    let options_bits = options[0]["bit_len"].as_u64().unwrap() as usize;
    let body = prefix_len * 8 + identity_bits;
    data.resize((body + options_bits).div_ceil(8), 0);
    copy_bits(
        &mut data,
        prefix_len * 8,
        &bytes(identity[0]["hex"].as_str().unwrap()),
        identity_bits,
    );
    copy_bits(
        &mut data,
        body,
        &bytes(options[0]["hex"].as_str().unwrap()),
        options_bits,
    );
    copy_bits(&mut data, body + 3, &[simulation << 5], 3);
    crate::theater::parser::test_support::set_native_event_versions(&mut data);
    data[prefix_len + 15 * 4..prefix_len + 16 * 4].copy_from_slice(&version.to_le_bytes());
    data
}
fn parse_registry(data: &[u8]) -> FilmRegistryRead {
    crate::theater::parser::v41::chunks::registry::parse_registry_chunk(data).unwrap()
}

#[test]
fn script_prefix_requires_supported_recorded_simulation_and_version() {
    for simulation in 0..8 {
        for version in [0, 1, 2, u32::MAX] {
            let data = registry(simulation, version);
            let parsed = parse_registry(&data);
            let expected = (simulation < 4 && version == 1).then_some(simulation != 2);
            assert_eq!(recorded_script_prefix(&parsed), expected);
        }
    }
    let data = registry(3, 1);
    // Header, identification and options truncation all refuse even if the
    // earlier simulation/version fields were available. Never choose a default.
    for len in [0xcb208 + 123 * 4, 0xce68c, 0xce68c + 2, data.len() - 1] {
        assert_eq!(recorded_script_prefix(&parse_registry(&data[..len])), None);
    }
    let mut old = parse_registry(&data);
    old.bootstrap_header = None;
    assert_eq!(recorded_script_prefix(&old), None);
}

#[test]
fn film_uses_explicit_script_prefix_only_when_recorder_simulation_is_unavailable() {
    let mut registry_data = registry(3, 1);
    registry_data.truncate(0xce68c);
    let chunks = |payload: &[u8], registry_data: &[u8]| {
        let mut packet = vec![0; 16];
        packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(payload);
        [
            RawFilmChunk {
                kind: ChunkKind::Registry.into(),
                index: 0,
                start_ms: 0,
                data: registry_data.to_vec(),
            },
            RawFilmChunk {
                kind: ChunkKind::Replication.into(),
                index: 1,
                start_ms: 0,
                data: packet,
            },
            RawFilmChunk {
                kind: ChunkKind::Summary.into(),
                index: 2,
                start_ms: 0,
                data: vec![],
            },
        ]
    };
    fn events(film: &crate::theater::parser::ParsedFilm) -> &EventListRead {
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
            &film.replication_stream.0[0].body.packets[0].body
        else {
            panic!("frame packet")
        };
        &frame.events
    }
    for case in cases().into_iter().filter(|c| c.name.ends_with("/0/False")) {
        let payload = bytes(&case.hex);
        let ordinary = crate::theater::parser::test_support::parse_fixture_structural(chunks(
            &payload,
            &registry_data,
        ))
        .unwrap();
        assert_eq!(events(&ordinary).stop, EventListStop::MissingRuntimeGate15);
        let supplied = crate::theater::parser::test_support::parse_fixture_structural_with_context(
            chunks(&payload, &registry_data),
            FilmParseContext {
                script_event_prefix: Some(case.script_prefix),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(events(&supplied).stop, EventListStop::Terminator);
        assert_eq!(events(&supplied).end_bit, case.bit_len);
        assert_eq!(
            events(&supplied).script_prefix_context_used,
            Some(case.script_prefix)
        );
        let short_payload = &payload[..(case.body_start / 8) + 1];
        let short = crate::theater::parser::test_support::parse_fixture_structural_with_context(
            chunks(short_payload, &registry_data),
            FilmParseContext {
                script_event_prefix: Some(case.script_prefix),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(events(&short).stop, EventListStop::Truncated);
        assert_eq!(
            events(&short).script_prefix_context_used,
            Some(case.script_prefix)
        );
        let complete = registry(if case.script_prefix { 3 } else { 2 }, 1);
        let recorded = crate::theater::parser::test_support::parse_fixture_structural(chunks(
            &payload, &complete,
        ))
        .unwrap();
        assert_eq!(events(&recorded).script_prefix_context_used, None);
        assert_eq!(events(&recorded).end_bit, case.bit_len);
        assert!(matches!(
            crate::theater::parser::test_support::parse_fixture_structural_with_context(
                chunks(&payload, &complete),
                FilmParseContext {
                    script_event_prefix: Some(!case.script_prefix),
                    ..Default::default()
                }
            ),
            Err(ParseError::ContradictoryScriptPrefixContext)
        ));
    }
}

#[test]
fn script_writer_branches_preserve_fields_truncation_and_upstream_boundaries() {
    #[derive(Deserialize)]
    struct Reference {
        cases: Vec<Trace>,
    }
    #[derive(Deserialize)]
    struct Trace {
        name: String,
        end: usize,
        trace: Vec<(usize, usize)>,
    }
    let reference: Reference = serde_json::from_str(include_str!(
        "../../../../fixtures/events-v41/script-reference.json"
    ))
    .unwrap();
    assert_eq!(reference.cases.len(), 16);
    for case in cases() {
        let data = bytes(&case.hex);
        let parsed = read_reference_event_list(&data, 1, Some(case.script_prefix), data.len());
        assert_eq!(parsed.stop, EventListStop::Terminator, "{}", case.name);
        assert_eq!(parsed.end_bit, case.bit_len);
        assert_eq!(parsed.records[0].end_bit, case.record_end);
        assert_eq!(parsed.records[0].body_start_bit, Some(case.body_start));
        assert_eq!(parsed.records[1].code, Some(21));
        let body: Vec<_> = parsed.records[0]
            .fields
            .iter()
            .filter(|f| f.stage == EventFieldStage::Body)
            .collect();
        assert_eq!(
            body.iter().map(|f| f.width).collect::<Vec<_>>(),
            case.body_widths
        );
        let mut boundaries = BTreeSet::from([1]);
        for field in parsed
            .records
            .iter()
            .flat_map(|r| &r.fields)
            .chain(parsed.terminator.iter())
        {
            if let EventFieldValue::Scalar(value) = field.value {
                assert_eq!(
                    value,
                    RawBits::from_source(&data, field.bit, field.width)
                        .unwrap()
                        .low_u64()
                );
            }
            boundaries.insert(field.bit);
            boundaries.insert(field.bit + field.width);
        }
        assert_eq!(
            serde_json::from_slice::<EventListRead>(&serde_json::to_vec(&parsed).unwrap()).unwrap(),
            parsed
        );
        if case.script_prefix {
            let trace = reference
                .cases
                .iter()
                .find(|r| r.name == case.name)
                .unwrap();
            assert_eq!(trace.end, case.bit_len);
            for (start, width) in &trace.trace {
                assert!(boundaries.contains(start) && boundaries.contains(&(start + width)));
            }
        }
        let missing = read_reference_event_list(&data, 1, None, data.len());
        assert_eq!(missing.stop, EventListStop::MissingRuntimeGate15);
        assert_eq!(missing.end_bit, case.body_start);
        for cut in 2..case.record_end {
            let shift = (8 - cut % 8) % 8;
            let mut partial = vec![0; (cut + shift) / 8];
            copy_bits(&mut partial, shift, &data, cut);
            let partial = read_reference_event_list(
                &partial,
                1 + shift,
                Some(case.script_prefix),
                partial.len(),
            );
            assert!(
                !partial.records[0].layout_complete,
                "{} at {cut}",
                case.name
            );
            assert!(partial.end_bit <= cut + shift);
            for field in &partial.records[0].fields {
                if field.value != EventFieldValue::Unavailable {
                    let original = parsed.records[0]
                        .fields
                        .iter()
                        .find(|f| f.bit + shift == field.bit)
                        .unwrap();
                    assert_eq!(field.width, original.width);
                    assert_eq!(field.value, original.value);
                }
            }
        }
    }
}

#[test]
fn canonical_script_admission_reaches_the_following_event_entities_and_controls() {
    for case in cases() {
        let data = registry(if case.script_prefix { 3 } else { 2 }, 1);
        let payload = bytes(&case.hex);
        let mut packet = vec![0; 16];
        packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(&payload);
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
        let film = crate::theater::parser::test_support::parse_fixture_structural([
            chunk(ChunkKind::Registry, data),
            chunk(ChunkKind::Replication, packet),
            crate::theater::parser::test_support::empty_summary(),
        ])
        .unwrap();
        let stream = &film.replication_stream.0[0];
        let packet = &stream.body.packets[0];
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
        else {
            panic!("frame")
        };
        assert_eq!(
            frame.events.stop,
            EventListStop::Terminator,
            "{}",
            case.name
        );
        assert_eq!(frame.events.end_bit, case.bit_len);
        assert_eq!(
            frame.frame.decoded().unwrap().end_bit,
            (case.bit_len + 4) as i64
        );
        assert_eq!(packet.payload(&stream.data).unwrap(), payload);
        assert!(packet.source_coverage(&stream.data).is_some());
    }
}
