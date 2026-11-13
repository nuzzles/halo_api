use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    bit_len: usize,
    fields: Vec<Field>,
    hex: String,
}
#[derive(Deserialize)]
struct Field {
    kind: Kind,
    bit: usize,
    width: usize,
    hex: String,
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/bootstrap-v41/cases.json"
    ))
    .unwrap()
}

#[test]
fn bootstrap_fields_preserve_exact_words_buffers_and_every_truncated_prefix() {
    for case in cases() {
        let source = bytes(&case.hex);
        let read = read_fields(&source, 0);
        assert_eq!(read.stop, BootstrapHeaderStop::Complete);
        assert_eq!(read.end_bit, case.bit_len);
        assert_eq!(read.fields.len(), 129);
        for (actual, expected) in read.fields.iter().zip(&case.fields) {
            assert_eq!(
                (actual.kind, actual.bit, actual.width),
                (expected.kind, expected.bit, expected.width)
            );
            assert_eq!(actual.raw.as_ref().unwrap().bytes, bytes(&expected.hex));
        }
        assert_eq!(read.event_version(36), Some(3));
        assert_eq!(read.event_version(2), Some(u32::MAX));
        assert_eq!(read.event_version(123), None);
        assert_eq!(
            serde_json::from_slice::<BootstrapHeaderRead>(&serde_json::to_vec(&read).unwrap())
                .unwrap(),
            read
        );
        for cut in 0..case.bit_len {
            let shift = (8 - cut % 8) % 8;
            let mut data = vec![0; (cut + shift) / 8];
            for i in 0..cut {
                data[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let partial = read_fields(&data, shift);
            assert_eq!(partial.stop, BootstrapHeaderStop::Truncated);
            let count = read
                .fields
                .iter()
                .take_while(|f| f.bit + f.width <= cut)
                .count();
            assert_eq!(partial.fields.len(), count + 1);
            assert!(partial.fields.last().unwrap().raw.is_none());
            assert_eq!(partial.end_bit, read.fields[count].bit + shift);
            for (actual, full) in partial.fields.iter().zip(&read.fields).take(count) {
                assert_eq!(
                    (actual.kind, actual.width, &actual.raw),
                    (full.kind, full.width, &full.raw)
                );
                assert_eq!(actual.bit, full.bit + shift);
            }
        }
    }
}

fn registry() -> Vec<u8> {
    use std::io::Read;
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../../../../../fixtures/bootstrap-v41/registry-prefix.bin.zlib").as_slice(),
    )
    .read_to_end(&mut out)
    .unwrap();
    assert_eq!(out.len(), 0xcb208);
    out
}

#[test]
fn bootstrap_requires_the_supported_registry_layout_not_a_build_string_anchor() {
    let prefix = registry();
    let suffix = bytes(&cases()[0].hex);
    let data = [prefix.clone(), suffix.clone()].concat();
    let read = crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&data).unwrap();
    let header = read.bootstrap_header.unwrap();
    assert_eq!(header.start_bit, prefix.len() * 8);
    assert_eq!(header.end_bit, prefix.len() * 8 + 4769);
    assert_eq!(header.stop, BootstrapHeaderStop::Complete);
    assert_eq!(header.event_version(36), Some(3));
    // Preserve a source-backed prefix even if only part of the table arrived.
    let read = crate::theater::parser::v41::chunks::registry::parse_registry_chunk(
        &data[..prefix.len() + 147],
    )
    .unwrap();
    let header = read.bootstrap_header.unwrap();
    assert_eq!(header.stop, BootstrapHeaderStop::Truncated);
    assert_eq!(header.event_version(36), None);
    for offset in [0, 4, 8 + 256] {
        let mut unknown = data.clone();
        unknown[offset] ^= 1;
        let read =
            crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&unknown).unwrap();
        assert!(read.bootstrap_header.is_none());
    }
    // An empty fake block with header-looking suffix data cannot select it.
    let fake = [[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(), suffix].concat();
    assert!(
        crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&fake)
            .unwrap()
            .bootstrap_header
            .is_none()
    );
}

#[test]
fn canonical_fire_uses_recorded_version_without_inventing_map_context() {
    use crate::clients::hi::models::RawFilmChunk;
    use crate::theater::film::ChunkKind;
    use crate::theater::parser::packet::PacketRead;
    use crate::theater::parser::v41::chunks::replication::source::{
        EventListStop, ReplicationStreamPacketBody,
    };
    #[derive(Deserialize)]
    struct FireCase {
        name: String,
        hex: String,
        bit_len: usize,
        indexed_points: Vec<usize>,
    }
    let fire_cases: Vec<FireCase> = serde_json::from_str(include_str!(
        "../../../../../fixtures/events-v41/cases.json"
    ))
    .unwrap();
    let selected = [
        "fire/prefix/True/3/7",
        "fire/block/0/True/False/3",
        "fire/block/0/False/False/3",
    ];
    assert_eq!(
        fire_cases
            .iter()
            .filter(|c| selected.contains(&c.name.as_str()))
            .count(),
        selected.len()
    );
    let prefix = registry();
    for case in fire_cases
        .iter()
        .filter(|c| selected.contains(&c.name.as_str()))
    {
        let mut payload = bytes(&case.hex);
        payload.resize((case.bit_len + 4).div_ceil(8), 0); // entity end and empty controls
        let mut packet = vec![0; 16];
        packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(&payload);
        for (version, complete_header) in [0u32, 1, 2, 3, 4, u32::MAX]
            .into_iter()
            .flat_map(|version| [true, false].map(|complete| (version, complete)))
        {
            let mut header = [prefix.clone(), bytes(&cases()[0].hex)].concat();
            crate::theater::parser::test_support::set_native_event_versions(&mut header);
            let offset = prefix.len() + 36 * 4;
            header[offset..offset + 4].copy_from_slice(&version.to_le_bytes());
            if !complete_header {
                header.truncate(prefix.len() + 123 * 4);
            }
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
                chunk(ChunkKind::Registry, header),
                chunk(ChunkKind::Replication, packet.clone()),
                crate::theater::parser::test_support::empty_summary(),
            ])
            .unwrap();
            let stream = &film.replication_stream.0[0];
            let packet = &stream.body.packets[0];
            let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
                &packet.body
            else {
                panic!("frame");
            };
            if version != 3 {
                assert_eq!(
                    frame.events.stop,
                    EventListStop::UnsupportedEventVersion {
                        event_code: 36,
                        recorded: version,
                        supported: 3,
                    }
                );
                assert_eq!(frame.events.end_bit, 9);
                assert!(frame.events.records[0].body_start_bit.is_none());
            } else if let Some(point) = case.indexed_points.first() {
                assert_eq!(frame.events.stop, EventListStop::UnsupportedBody);
                assert_eq!(frame.events.end_bit, point + 1);
            } else if !complete_header {
                assert_eq!(
                    frame.events.stop,
                    EventListStop::MissingCorruptionCheckContext
                );
                assert!(!frame.events.records[0].layout_complete);
                assert!(frame.events.terminator.is_none());
            } else {
                assert_eq!(frame.events.stop, EventListStop::Terminator);
                assert_eq!(frame.events.end_bit, case.bit_len);
                assert_eq!(
                    frame.frame.decoded().unwrap().end_bit,
                    (case.bit_len + 4) as i64
                );
            }
            assert_eq!(packet.payload(&stream.data).unwrap(), payload);
            assert!(packet.source_coverage(&stream.data).is_some());
        }
    }
}

#[test]
fn corruption_flag_distinguishes_missing_and_overrides_profile_defaults() {
    use crate::theater::parser::v41::context::V41DecodeConfig;
    for case in cases() {
        let source = bytes(&case.hex);
        let header = read_fields(&source, 0);
        let flag = source[596] & 0x80 != 0;
        assert_eq!(header.corruption_checks(), Some(flag));
        for old in [false, true] {
            let mut config = V41DecodeConfig::default();
            config.profile.grammar.corruption_check = old;
            config.apply_recorded_corruption_checks(Some(&header));
            assert_eq!(config.profile.grammar.corruption_check, flag);
            assert_eq!(config.record_layout().corruption_check, Some(flag));
            config.profile.grammar.corruption_check = !flag;
            assert_eq!(config.corruption_check(), Some(flag));
            config.profile.grammar.corruption_check = flag;
            let partial = read_fields(&source[..596], 0);
            assert_eq!(partial.corruption_checks(), None);
            config.apply_recorded_corruption_checks(Some(&partial));
            assert_eq!(config.record_layout().corruption_check, None);
            assert_eq!(config.profile.grammar.corruption_check, flag);
            config.apply_recorded_corruption_checks(None);
            assert_eq!(config.record_layout().corruption_check, None);
            assert_eq!(config.profile.grammar.corruption_check, flag);
        }
        let mut invalid = header;
        invalid
            .fields
            .last_mut()
            .unwrap()
            .raw
            .as_mut()
            .unwrap()
            .bit_len = 0;
        assert_eq!(invalid.corruption_checks(), None);
    }
}
