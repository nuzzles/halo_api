use super::super::tests::{copy_bits, registry};
use super::*;
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::*;
use crate::theater::parser::FilmParseContext;
use crate::theater::parser::packet::*;
use crate::theater::parser::v41::chunks::replication::source::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    hex: String,
    bit_len: usize,
    expected: (Option<i32>, Option<bool>, Option<bool>),
}
fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/bond-v2/variants.json"
    ))
    .unwrap()
}
fn source(case: &Case) -> Vec<u8> {
    let mut data = registry(3, 1);
    let body = 0xce68c * 8 + 1;
    data.resize((body + case.bit_len).div_ceil(8), 0);
    let bytes: Vec<_> = case
        .hex
        .as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    copy_bits(&mut data, body, &bytes, case.bit_len);
    data
}

#[test]
fn variant_schema_preserves_unknowns_and_scopes_omitted_defaults() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../fixtures/bond-v2/variants-reference.json"
    ))
    .unwrap();
    let reference = reference["cases"].as_array().unwrap();
    assert_eq!(reference.len(), 66);
    for case in cases() {
        let data = source(&case);
        let parsed =
            crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&data).unwrap();
        let original = parsed.clone();
        let got = recorded_variant(&parsed);
        assert_eq!(
            (got.engine_type, got.killcam, got.play_of_game),
            case.expected,
            "{}",
            case.name
        );
        if let Some(expected) = reference.iter().find(|r| r["name"] == case.name) {
            assert_eq!(
                serde_json::json!([got.engine_type, got.killcam, got.play_of_game]),
                expected["value"]
            );
        }
        assert_eq!(parsed, original);
        assert_eq!(
            got.excludes_kill_tail(),
            (case.expected.1 == Some(false) || case.expected.0 == Some(1))
                && case.expected.2 == Some(false)
        );
        let options = parsed
            .bootstrap_header
            .as_ref()
            .unwrap()
            .identification
            .as_ref()
            .unwrap()
            .game_options
            .as_ref()
            .unwrap();
        assert_eq!(options.stop, GameOptionsStop::Complete, "{}", case.name);
        let partial = crate::theater::parser::v41::chunks::registry::parse_registry_chunk(
            &data[..data.len() - 1],
        )
        .unwrap();
        assert_eq!(recorded_variant(&partial), VariantContext::default());
    }
}

#[test]
#[ignore = "requires the separately retained 32-film corpus"]
fn captured_variant_context() {
    use std::path::{Path, PathBuf};
    fn manifests(path: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                manifests(&p, out)
            } else if p.file_name().is_some_and(|n| n == "film.json") {
                out.push(p)
            }
        }
    }
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("experiments/films"));
    let mut paths = Vec::new();
    manifests(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    for path in paths {
        let m: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let c = m["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["chunk_type"] == 1)
            .unwrap();
        let data = std::fs::read(path.parent().unwrap().join(c["file"].as_str().unwrap())).unwrap();
        let film = crate::theater::parser::test_support::parse_fixture_structural([
            RawFilmChunk {
                kind: ChunkKind::Registry.into(),
                data,
                index: 0,
                start_ms: 0,
            },
            crate::theater::parser::test_support::empty_replication(),
            crate::theater::parser::test_support::empty_summary(),
        ])
        .unwrap();
        let got = recorded_variant(&film.registry.body);
        eprintln!("{}: {got:?}", path.strip_prefix(&root).unwrap().display());
        assert_eq!(
            got,
            VariantContext {
                engine_type: Some(2),
                killcam: Some(false),
                play_of_game: Some(true)
            }
        );
    }
}

fn event_packet(code: u8, body: &str) -> (Vec<u8>, usize, usize) {
    let mut bits = format!("11{code:07b}000{body}");
    let end = bits.len();
    bits.push_str("10010101000100"); // zoom(level=2), then list terminator
    let event_end = bits.len();
    bits.push_str("0000");
    let mut bytes = vec![0; bits.len().div_ceil(8)];
    for (i, b) in bits.bytes().enumerate() {
        bytes[i / 8] |= (b - b'0') << (7 - i % 8)
    }
    (bytes, end, event_end)
}

#[test]
fn variant_gates_stop_at_the_first_unproven_field_and_report_runtime_assumptions() {
    use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::{
        EventLayout, read_event_list,
    };
    let read = |data: &[u8], variant: VariantContext, runtime_defaults: bool| {
        read_event_list(
            data,
            1,
            data.len(),
            EventLayout {
                variant,
                runtime_defaults,
                ..Default::default()
            },
        )
    };
    let known = VariantContext {
        engine_type: Some(2),
        killcam: Some(false),
        play_of_game: Some(false),
    };
    for orientation in [false, true] {
        // No map positions. The standard orientation is inverse-gated 19 bits + 8 bits.
        let body = if orientation {
            format!("10{:019b}{:08b}00", 0x12345, 0x97)
        } else {
            "000".into()
        };
        let (data, end, list_end) = event_packet(116, &body);
        let parsed = read(&data, known, false);
        assert_eq!(parsed.stop, EventListStop::Terminator);
        assert_eq!(parsed.records[0].end_bit, end);
        assert_eq!(parsed.end_bit, list_end);
        assert!(parsed.assumptions.is_empty());
        for engine in [None, Some(1)] {
            let (engine_data, _, engine_list_end) = if orientation && engine == Some(1) {
                event_packet(116, &format!("10{:030b}{:030b}00", 0x1234567, 0x2345678))
            } else {
                (data.clone(), end, list_end)
            };
            for runtime_defaults in [false, true] {
                let parsed = read(
                    &engine_data,
                    VariantContext {
                        engine_type: engine,
                        ..known
                    },
                    runtime_defaults,
                );
                if orientation && engine.is_none() && !runtime_defaults {
                    assert_eq!(parsed.end_bit, 13);
                    assert_eq!(
                        parsed.stop,
                        if engine.is_none() {
                            EventListStop::MissingVariantContext
                        } else {
                            EventListStop::UnsupportedBody
                        }
                    );
                    assert!(!parsed.records[0].layout_complete);
                    assert!(parsed.assumptions.is_empty());
                } else {
                    assert_eq!(parsed.end_bit, engine_list_end);
                    assert_eq!(parsed.stop, EventListStop::Terminator);
                    assert_eq!(
                        parsed.assumptions,
                        if orientation && engine.is_none() {
                            vec![EventAssumption::TeleportEngineNotOne]
                        } else {
                            vec![]
                        }
                    );
                }
            }
        }
    }
    let kill = format!("11{:032b}11{:032b}", 0x12345678u32, 0xabcdef01u32);
    let (data, end, list_end) = event_packet(85, &kill);
    for engine_type in [None, Some(1), Some(2)] {
        for killcam in [None, Some(false), Some(true)] {
            for play_of_game in [None, Some(false), Some(true)] {
                let variant = VariantContext {
                    engine_type,
                    killcam,
                    play_of_game,
                };
                let proven = (engine_type == Some(1) || killcam == Some(false))
                    && play_of_game == Some(false);
                for defaults in [false, true] {
                    let parsed = read(&data, variant, defaults);
                    assert_eq!(
                        parsed.stop,
                        if proven || defaults {
                            EventListStop::Terminator
                        } else {
                            EventListStop::MissingKillPlaybackContext
                        }
                    );
                    assert_eq!(
                        parsed.end_bit,
                        if proven || defaults { list_end } else { end }
                    );
                    assert_eq!(parsed.records[0].layout_complete, proven || defaults);
                    assert_eq!(
                        parsed.assumptions,
                        if !proven && defaults {
                            vec![EventAssumption::KillPlaybackDisabled]
                        } else {
                            vec![]
                        }
                    );
                }
            }
        }
    }
    for cut in 2..end {
        let shift = (8 - cut % 8) % 8;
        let mut partial = vec![0; (cut + shift) / 8];
        copy_bits(&mut partial, shift, &data, cut);
        let parsed = read_event_list(
            &partial,
            1 + shift,
            partial.len(),
            EventLayout {
                variant: VariantContext::default(),
                runtime_defaults: true,
                ..Default::default()
            },
        );
        assert!(!parsed.records[0].layout_complete);
        assert!(parsed.end_bit <= cut + shift);
        assert!(parsed.assumptions.is_empty());
    }
    // The possible tail resembles a complete event. It must stay unparsed.
    let (tail, _, _) = event_packet(85, &format!("{kill}10010101000100{}", "1".repeat(54)));
    let refused = read(&tail, VariantContext::default(), false);
    assert_eq!(refused.end_bit, end);
    assert_eq!(refused.records.len(), 1);
    assert!(refused.terminator.is_none());
}

#[test]
fn external_kill_tail_context_reads_exact_fields_and_refuses_conflicts() {
    use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::{
        EventLayout, read_event_list,
    };
    use crate::theater::parser::v41::chunks::replication::source::EventFieldValue;
    let base = format!("11{:032b}11{:032b}", 0x12345678u32, 0xabcdef01u32);
    let tail = format!("{:032b}{:032b}{:04b}", 0x13579bdfu32, 0x2468ace0u32, 9);
    let (data, tail_end, list_end) = event_packet(85, &format!("{base}{tail}"));
    let base_end = tail_end - 68;
    let read = |bytes: &[u8], tail: Option<bool>, variant: VariantContext, start| {
        read_event_list(
            bytes,
            start,
            bytes.len(),
            EventLayout {
                variant,
                kill_tail: tail,
                ..Default::default()
            },
        )
    };
    let full = read(&data, Some(true), VariantContext::default(), 1);
    assert_eq!(full.stop, EventListStop::Terminator);
    assert_eq!(full.end_bit, list_end);
    assert_eq!(full.kill_tail_context_used, Some(true));
    assert!(full.assumptions.is_empty());
    let fields = &full.records[0].fields;
    assert!(
        fields.ends_with(&[
            crate::theater::parser::v41::chunks::replication::source::EventField {
                bit: base_end,
                width: 32,
                stage:
                    crate::theater::parser::v41::chunks::replication::source::EventFieldStage::Body,
                value: EventFieldValue::Scalar(0x13579bdf),
            },
            crate::theater::parser::v41::chunks::replication::source::EventField {
                bit: base_end + 32,
                width: 32,
                stage:
                    crate::theater::parser::v41::chunks::replication::source::EventFieldStage::Body,
                value: EventFieldValue::Scalar(0x2468ace0),
            },
            crate::theater::parser::v41::chunks::replication::source::EventField {
                bit: base_end + 64,
                width: 4,
                stage:
                    crate::theater::parser::v41::chunks::replication::source::EventFieldStage::Body,
                value: EventFieldValue::Scalar(9),
            },
        ]),
        "{:?}",
        fields.iter().rev().take(5).collect::<Vec<_>>()
    );
    let absent = read(&data, Some(false), VariantContext::default(), 1);
    assert_eq!(absent.kill_tail_context_used, Some(false));
    assert_eq!(absent.records[0].end_bit, base_end);
    let conflict = read(
        &data,
        Some(true),
        VariantContext {
            engine_type: Some(1),
            killcam: None,
            play_of_game: Some(false),
        },
        1,
    );
    assert_eq!(
        conflict.stop,
        EventListStop::ContradictoryKillPlaybackContext
    );
    assert_eq!(conflict.end_bit, base_end);
    assert_eq!(conflict.kill_tail_context_used, None);
    for cut in base_end..base_end + 68 {
        let shift = (8 - cut % 8) % 8;
        let mut partial = vec![0; (cut + shift) / 8];
        copy_bits(&mut partial, shift, &data, cut);
        let parsed = read(&partial, Some(true), VariantContext::default(), 1 + shift);
        assert_eq!(parsed.stop, EventListStop::Truncated, "cut {cut}");
        assert_eq!(parsed.kill_tail_context_used, Some(true));
        assert!(parsed.end_bit <= cut + shift);
    }
}

#[test]
fn canonical_variant_context_controls_following_views_without_altering_source() {
    for name in ["valid/1/2/0/0", "valid/1/2/0/1", "valid/1/1/1/0"] {
        let case = cases().into_iter().find(|c| c.name == name).unwrap();
        for code in [85, 116] {
            let body = if code == 85 {
                format!("11{:032b}11{:032b}", 1, 2)
            } else if case.expected.0 == Some(1) {
                format!("10{:030b}{:030b}00", 0x1234567, 0x2345678)
            } else {
                format!("10{:019b}{:08b}00", 0x12345, 0x97)
            };
            let (payload, end, list_end) = event_packet(code, &body);
            let mut packet = vec![0; 16];
            packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
            packet.extend_from_slice(&payload);
            let film = crate::theater::parser::test_support::parse_fixture_structural([
                RawFilmChunk {
                    kind: ChunkKind::Registry.into(),
                    data: source(&case),
                    index: 0,
                    start_ms: 0,
                },
                RawFilmChunk {
                    kind: ChunkKind::Replication.into(),
                    data: packet,
                    index: 1,
                    start_ms: 0,
                },
                crate::theater::parser::test_support::empty_summary(),
            ])
            .unwrap();
            let stream = &film.replication_stream.0[0];
            let packet = &stream.body.packets[0];
            let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
                &packet.body
            else {
                panic!("frame")
            };
            let expected = if code == 85 && case.expected.2 == Some(true) {
                Some((end, EventListStop::MissingKillPlaybackContext))
            } else {
                None
            };
            if let Some((end, stop)) = expected {
                assert_eq!(frame.events.stop, stop);
                assert_eq!(frame.events.end_bit, end);
                assert!(!frame.events.records[0].layout_complete);
                assert!(frame.frame.decoded().is_none());
            } else {
                assert_eq!(frame.events.stop, EventListStop::Terminator);
                assert_eq!(frame.events.end_bit, list_end);
                assert_eq!(
                    frame.frame.decoded().unwrap().end_bit,
                    (list_end + 4) as i64
                );
            }
            assert!(frame.events.assumptions.is_empty());
            assert_eq!(packet.payload(&stream.data).unwrap(), payload);
            assert!(packet.source_coverage(&stream.data).is_some());
        }
    }
}

#[test]
fn film_parse_with_explicit_kill_tail_context_reaches_the_following_views() {
    let case = cases()
        .into_iter()
        .find(|c| c.name == "valid/1/2/0/1")
        .unwrap();
    let body = format!(
        "11{:032b}11{:032b}{:032b}{:032b}{:04b}",
        1, 2, 0x13579bdfu32, 0x2468ace0u32, 9
    );
    let (payload, tail_end, list_end) = event_packet(85, &body);
    let base_end = tail_end - 68;
    let mut packet = vec![0; 16];
    packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    packet.extend_from_slice(&payload);
    let chunks = || {
        [
            RawFilmChunk {
                kind: ChunkKind::Registry.into(),
                data: source(&case),
                index: 0,
                start_ms: 0,
            },
            RawFilmChunk {
                kind: ChunkKind::Replication.into(),
                data: packet.clone(),
                index: 1,
                start_ms: 0,
            },
            crate::theater::parser::test_support::empty_summary(),
        ]
    };
    let ordinary =
        crate::theater::parser::test_support::parse_fixture_structural(chunks()).unwrap();
    let with_context = crate::theater::parser::test_support::parse_fixture_structural_with_context(
        chunks(),
        FilmParseContext {
            kill_event_tail: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    fn frame(
        film: &crate::theater::parser::ParsedFilm,
    ) -> &crate::theater::parser::v41::chunks::replication::source::FramePacket {
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
            &film.replication_stream.0[0].body.packets[0].body
        else {
            panic!("frame packet")
        };
        frame.as_ref()
    }
    assert_eq!(
        frame(&ordinary).events.stop,
        EventListStop::MissingKillPlaybackContext
    );
    assert_eq!(frame(&ordinary).events.end_bit, base_end);
    assert_eq!(frame(&with_context).events.stop, EventListStop::Terminator);
    assert_eq!(frame(&with_context).events.end_bit, list_end);
    assert_eq!(
        frame(&with_context).events.kill_tail_context_used,
        Some(true)
    );
    assert!(frame(&with_context).events.assumptions.is_empty());
    assert_eq!(
        frame(&with_context).frame.decoded().unwrap().end_bit,
        (list_end + 4) as i64
    );
    let chunk = &with_context.replication_stream.0[0];
    assert_eq!(chunk.body.packets[0].payload(&chunk.data).unwrap(), payload);
    assert!(chunk.body.packets[0].source_coverage(&chunk.data).is_some());
}
