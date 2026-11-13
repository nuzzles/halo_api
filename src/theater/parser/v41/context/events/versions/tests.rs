use super::*;
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::*;
use crate::theater::parser::packet::*;
use crate::theater::parser::test_support::{empty_summary, registry_with_corruption_policy};
use crate::theater::parser::v41::chunks::registry::parse_registry_chunk;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::{
    EventLayout, read_event_list,
};
use crate::theater::parser::v41::chunks::replication::source::*;
use serde_json::Value;

fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/event-admission-v41/reference.json"
    ))
    .unwrap()
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn event_admission_versions_match_original_native_table_and_reject_each_mismatch() {
    let oracle = oracle();
    let native: Vec<u32> = serde_json::from_value(oracle["results"]["Native"].clone()).unwrap();
    assert_eq!(native.len(), 123);
    assert_eq!(
        (0..123)
            .map(|c| EventVersionContext::native_version(c).unwrap())
            .collect::<Vec<_>>(),
        native
    );
    assert_eq!(EventVersionContext::native_version(123), None);
    let original = registry_with_corruption_policy(false).data;
    let mut checked = 0;
    for (case, result) in oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["results"]["Classes"].as_array().unwrap())
    {
        // Other cardinalities/majors describe unsupported bootstrap layouts.
        // Their upstream prefix hypothesis is deliberately not adopted here.
        if case["count"] != 123 || case["major"] != 41 || case["read_major"] != true {
            continue;
        }
        let mut data = original.clone();
        if let Some(code) = case["changed"].as_u64() {
            let at = 0xcb208 + 4 * code as usize;
            data[at..at + 4]
                .copy_from_slice(&(case["value"].as_u64().unwrap() as u32).to_le_bytes());
        }
        let registry = parse_registry_chunk(&data).unwrap();
        let got = EventVersionContext::recorded(&registry);
        assert_eq!(got.is_native(), result["Class"] == 2, "{case}");
        if let Some(code) = case["changed"].as_u64() {
            assert_eq!(
                got,
                EventVersionContext::Unsupported {
                    event_code: code as u8,
                    recorded: case["value"].as_u64().unwrap() as u32,
                    supported: native[code as usize]
                }
            );
            assert_eq!(
                registry
                    .bootstrap_header
                    .as_ref()
                    .unwrap()
                    .event_version(code as u8),
                Some(case["value"].as_u64().unwrap() as u32)
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 371);
    // Every shorter source table keeps its fields but cannot admit a message.
    for count in 0..123 {
        let read = parse_registry_chunk(&original[..0xcb208 + count * 4]).unwrap();
        assert_eq!(
            EventVersionContext::recorded(&read),
            EventVersionContext::Missing {
                event_code: count as u8
            }
        );
    }
    // Once all version words arrived, later missing build/identity fields do
    // not erase them. Corruption/context guards still stop at their own boundary.
    let read = parse_registry_chunk(&original[..0xcb208 + 123 * 4]).unwrap();
    assert!(EventVersionContext::recorded(&read).is_native());
}

#[test]
fn event_admission_heads_and_empty_lists_match_native_without_rewinds_or_prefix_hypotheses() {
    let oracle = oracle();
    let mut differences = 0;
    for (case, expected) in oracle["frames"]
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["results"]["Frames"].as_array().unwrap())
    {
        let count = case["count"].as_u64().unwrap() as u8;
        let versions = if count == 123 {
            EventVersionContext::Native
        } else {
            EventVersionContext::Missing {
                event_code: if count < 123 { count } else { 0 },
            }
        };
        let data = bytes(case["hex"].as_str().unwrap());
        let start = case["start"].as_u64().unwrap() as usize;
        let read = read_event_list(
            &data,
            start,
            100,
            EventLayout {
                versions,
                category_references: case["config"].as_bool().unwrap(),
                corruption_check: case["corruption"].as_bool(),
                ..Default::default()
            },
        );
        let prefix_hypothesis =
            count == 122 && case["config"] == true && matches!(case["code"].as_u64(), Some(3 | 21));
        if prefix_hypothesis {
            assert!(expected["Complete"].as_bool().unwrap());
            assert_eq!(
                read.stop,
                EventListStop::MissingEventVersion { event_code: 122 }
            );
            assert_eq!(read.end_bit, start + 8);
            differences += 1;
        } else {
            assert_eq!(
                read.end_bit,
                expected["End"].as_u64().unwrap() as usize,
                "{case}"
            );
            assert_eq!(
                read.stop == EventListStop::Terminator,
                expected["Complete"].as_bool().unwrap(),
                "{case}"
            );
        }
        if read.stop != EventListStop::Terminator {
            let record = &read.records[0];
            assert!(
                record
                    .fields
                    .iter()
                    .all(|f| f.stage == EventFieldStage::Header)
            );
            assert!(record.body_start_bit.is_none());
            assert!(!record.layout_complete);
            assert!(read.terminator.is_none());
        }
        assert_eq!(
            serde_json::from_slice::<EventListRead>(&serde_json::to_vec(&read).unwrap()).unwrap(),
            read
        );
    }
    assert_eq!(differences, 4);
}

fn film(registry: RawFilmChunk, config: bool, message: bool) -> crate::theater::parser::ParsedFilm {
    let mut bits = vec![config, message];
    if message {
        bits.extend((0..7).rev().map(|i| 21 >> i & 1 != 0));
        bits.extend([false; 3]);
        bits.extend([true; 2]);
        bits.push(false);
    }
    bits.extend([false; 4]);
    let mut payload = vec![0; bits.len().div_ceil(8)];
    for (i, b) in bits.iter().enumerate() {
        payload[i / 8] |= u8::from(*b) << (7 - i % 8);
    }
    let mut data = vec![0; 16];
    data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    data.extend(payload);
    crate::theater::parser::test_support::parse_fixture_structural([
        registry,
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            data,
            index: 1,
            start_ms: 0,
        },
        empty_summary(),
    ])
    .unwrap()
}

#[test]
fn event_admission_public_film_preserves_raw_tables_and_blocks_following_views() {
    let original = registry_with_corruption_policy(false);
    for failure in [None, Some((0, 0)), Some((21, 2)), Some((122, u32::MAX))] {
        for config in [false, true] {
            for message in [false, true] {
                let mut registry = original.clone();
                if let Some((code, value)) = failure {
                    let at = 0xcb208 + 4 * code;
                    registry.data[at..at + 4].copy_from_slice(&value.to_le_bytes());
                }
                let expected = registry.data.clone();
                let film = film(registry, config, message);
                assert_eq!(film.registry.data, expected);
                let chunk = &film.replication_stream.0[0];
                let packet = &chunk.body.packets[0];
                let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(read)) =
                    &packet.body
                else {
                    panic!()
                };
                let refused = message && (failure.is_some() || !config);
                if refused {
                    let stop = if let Some((code, value)) = failure {
                        EventListStop::UnsupportedEventVersion {
                            event_code: code as u8,
                            recorded: value,
                            supported: EventVersionContext::native_version(code).unwrap(),
                        }
                    } else {
                        EventListStop::UnsupportedReferenceConfiguration
                    };
                    assert_eq!(read.events.stop, stop);
                    assert_eq!(read.events.end_bit, 9);
                    assert_eq!(read.events.records[0].fields.len(), 2);
                    assert_eq!(
                        read.frame,
                        FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
                    );
                    let coverage = packet.source_coverage(&chunk.data).unwrap();
                    assert_eq!(coverage.last().unwrap().kind, SourceRegionKind::Unparsed);
                    assert_eq!(coverage.last().unwrap().source.start, 9);
                } else {
                    assert_eq!(read.events.stop, EventListStop::Terminator);
                    assert!(read.frame.decoded().is_some());
                }
                assert!(packet.source_coverage(&chunk.data).is_some());
            }
        }
    }
    for count in [0, 21, 122] {
        let mut registry = original.clone();
        registry.data.truncate(0xcb208 + 4 * count);
        let film = film(registry, true, true);
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(read)) =
            &film.replication_stream.0[0].body.packets[0].body
        else {
            panic!()
        };
        assert_eq!(
            read.events.stop,
            EventListStop::MissingEventVersion {
                event_code: count as u8
            }
        );
        assert_eq!(read.events.end_bit, 9);
    }
}
