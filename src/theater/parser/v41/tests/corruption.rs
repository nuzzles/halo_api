//! Native guard fixtures and public parser/replay propagation.
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::*;
use crate::theater::parser::packet::*;
use crate::theater::parser::test_support::{
    component_registry, empty_summary, vacant_keyframe_slots,
};
use crate::theater::parser::v41::chunks::replication::replication_stream::{
    frame::{
        events::{EventLayout, read_event_list},
        records::{RecordBinding, RecordReadContext, decode_entity_record},
    },
    keyframes::record::decode_keyframe_record_contextual,
};
use crate::theater::parser::v41::chunks::replication::source::*;
use crate::theater::parser::v41::context::V41DecodeConfig;
use serde_json::Value;
use std::io::Read;

fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/corruption-v41/reference.json"
    ))
    .unwrap()
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn put(bits: &mut Vec<bool>, width: usize, value: u64) {
    bits.extend((0..width).rev().map(|i| value >> i & 1 != 0));
}
fn pack(bits: &[bool]) -> Vec<u8> {
    let mut data = vec![0; bits.len().div_ceil(8)];
    for (i, b) in bits.iter().enumerate() {
        data[i / 8] |= u8::from(*b) << (7 - i % 8);
    }
    data
}

#[test]
fn corruption_event_guards_match_native_empty_descriptors_and_boundaries() {
    let oracle = oracle();
    for (case, expected) in oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["results"].as_array().unwrap())
    {
        if case["kind"] != "event" {
            continue;
        }
        let source = bytes(case["hex"].as_str().unwrap());
        let enabled = case["enabled"].as_bool().unwrap();
        let read = read_event_list(
            &source,
            0,
            100,
            EventLayout {
                corruption_check: Some(enabled),
                ..Default::default()
            },
        );
        assert_eq!(read.stop, EventListStop::Terminator, "{case}");
        assert_eq!(read.end_bit, expected["end"].as_u64().unwrap() as usize);
        assert_eq!(
            read.records.iter().map(|r| r.end_bit).collect::<Vec<_>>(),
            serde_json::from_value::<Vec<usize>>(expected["ends"].clone()).unwrap()
        );
        let guards: Vec<_> = read.records[0]
            .fields
            .iter()
            .filter(|f| f.stage == EventFieldStage::CorruptionCheck)
            .collect();
        let guarded = enabled && case["code"] == 21;
        assert_eq!(
            guards.len(),
            if guarded {
                if case["gate"] == 1 { 2 } else { 1 }
            } else {
                0
            }
        );
        if guards.len() == 2 {
            assert_eq!(
                guards[1].value,
                EventFieldValue::Scalar(case["sentinel"].as_u64().unwrap())
            );
        }
        assert!(
            read.records[1]
                .fields
                .iter()
                .all(|f| f.stage != EventFieldStage::CorruptionCheck)
        );
        // Align each truncated prefix to an exact byte boundary; padding cannot
        // masquerade as a missing gate or partial sentinel.
        for cut in 1..read.records[0].end_bit {
            let shift = (8 - cut % 8) % 8;
            let mut bits = vec![false; shift];
            bits.extend((0..cut).map(|i| source[i / 8] >> (7 - i % 8) & 1 != 0));
            let data = pack(&bits);
            let partial = read_event_list(
                &data,
                shift,
                100,
                EventLayout {
                    corruption_check: Some(enabled),
                    ..Default::default()
                },
            );
            assert_eq!(partial.stop, EventListStop::Truncated, "cut {cut}: {case}");
            assert!(!partial.records[0].layout_complete);
            assert!(partial.terminator.is_none());
            assert!(partial.end_bit <= shift + cut);
            assert_eq!(
                partial.records[0].fields.last().unwrap().value,
                EventFieldValue::Unavailable
            );
        }
    }
}

#[test]
fn corruption_component_and_snapshot_default_guards_match_native() {
    let oracle = oracle();
    let registry = component_registry(&[(4, vec![("high-frequency".into(), 0)])]);
    for (case, expected) in oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["results"].as_array().unwrap())
    {
        let kind = case["kind"].as_str().unwrap();
        if kind == "event" {
            continue;
        }
        let enabled = case["enabled"].as_bool().unwrap();
        let mut config = V41DecodeConfig::default();
        config.profile.grammar.corruption_check = enabled;
        let source = bytes(case["hex"].as_str().unwrap());
        let guard_bits = expected["end"].as_u64().unwrap() as usize;
        if kind == "default" {
            let mut bits = Vec::new();
            for (w, v) in [(32, 0x40000000), (32, 4), (32, 0), (4, 0), (8, 0)] {
                put(&mut bits, w, v);
            }
            bits.extend((0..guard_bits).map(|i| source[i / 8] >> (7 - i % 8) & 1 != 0));
            // Fixture n2=1: one high-frequency body followed by its guard.
            put(&mut bits, 8, 231);
            if enabled {
                put(&mut bits, 1, 0);
            }
            let data = pack(&bits);
            let record = decode_keyframe_record_contextual(
                &data,
                0,
                &registry,
                &config.record_layout(),
                Some(&config.profile),
            )
            .unwrap();
            assert_eq!(record.stop, KeyframeStop::Complete);
            assert_eq!(record.end_bit, bits.len() as i64);
            assert_eq!(record.components[0].start_bit, 108 + guard_bits as i64);
            let positive = case["n1"] == 1;
            assert_eq!(record.default_state.is_some(), positive);
            if positive {
                let fields = &record.default_state.as_ref().unwrap().fields;
                assert_eq!(fields.len(), usize::from(enabled));
                if enabled {
                    assert_eq!(fields[0].raw.low_u64(), 0x12345678);
                }
            }
        } else {
            for new in [true, false] {
                let mut bits = Vec::new();
                put(&mut bits, if new { 3 } else { 1 }, 1);
                put(&mut bits, 13, 0);
                put(&mut bits, 2, 1);
                if new {
                    put(&mut bits, 6, 4);
                }
                put(&mut bits, 1, u64::from(new)); // NEW components present or DELTA baseline absent
                put(&mut bits, 1, 0);
                put(&mut bits, 3, 1);
                put(&mut bits, 6, 0); // sparse mask
                put(&mut bits, 8, 231);
                let start = bits.len();
                bits.extend((0..guard_bits).map(|i| source[i / 8] >> (7 - i % 8) & 1 != 0));
                let data = pack(&bits);
                let record = decode_entity_record(
                    &data,
                    0,
                    &registry,
                    &config.record_layout(),
                    Some(RecordBinding {
                        id: 0x40000000,
                        archetype: 4,
                    }),
                    true,
                    RecordReadContext {
                        context: Some(&config.profile),
                        movement: None,
                    },
                )
                .unwrap();
                assert_eq!(record.stop, EntityViewStop::Complete, "{case}");
                assert_eq!(record.end_bit, (start + guard_bits) as i64);
                let c = &record.components[0];
                assert_eq!(c.fields[0].raw.low_u64(), 231);
                assert_eq!(c.end_bit, record.end_bit);
                assert_eq!(
                    c.fields.len(),
                    1 + if enabled {
                        if guard_bits == 33 { 2 } else { 1 }
                    } else {
                        0
                    }
                );
                for cut in start..start + guard_bits {
                    let shift = (8 - cut % 8) % 8;
                    let partial = pack(&[vec![false; shift], bits[..cut].to_vec()].concat());
                    let read = decode_entity_record(
                        &partial,
                        shift,
                        &registry,
                        &config.record_layout(),
                        Some(RecordBinding {
                            id: 0x40000000,
                            archetype: 4,
                        }),
                        true,
                        RecordReadContext {
                            context: Some(&config.profile),
                            movement: None,
                        },
                    )
                    .unwrap();
                    assert_eq!(read.stop, EntityViewStop::Truncated);
                    assert_eq!(read.components[0].status, ComponentReadStatus::Truncated);
                    assert!(read.end_bit <= (shift + cut) as i64);
                }
            }
            let mut bits = Vec::new();
            for (w, v) in [
                (32, 0x40000000),
                (32, 4),
                (32, 0),
                (4, 0),
                (8, 0),
                (32, 0),
                (32, 1),
                (8, 231),
            ] {
                put(&mut bits, w, v);
            }
            let start = bits.len();
            bits.extend((0..guard_bits).map(|i| source[i / 8] >> (7 - i % 8) & 1 != 0));
            let data = pack(&bits);
            let read = decode_keyframe_record_contextual(
                &data,
                0,
                &registry,
                &config.record_layout(),
                Some(&config.profile),
            )
            .unwrap();
            assert_eq!(read.stop, KeyframeStop::Complete);
            assert_eq!(read.end_bit, bits.len() as i64);
            for cut in start..start + guard_bits {
                let shift = (8 - cut % 8) % 8;
                let partial = pack(&[vec![false; shift], bits[..cut].to_vec()].concat());
                let read = decode_keyframe_record_contextual(
                    &partial,
                    shift,
                    &registry,
                    &config.record_layout(),
                    Some(&config.profile),
                )
                .unwrap();
                assert_eq!(read.stop, KeyframeStop::Truncated);
                assert_eq!(read.components[0].status, ComponentReadStatus::Truncated);
                assert!(read.end_bit <= (shift + cut) as i64);
            }
        }
    }
}

pub(super) fn registry(enabled: bool) -> Vec<u8> {
    let mut prefix = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../../../fixtures/bootstrap-v41/registry-prefix.bin.zlib").as_slice(),
    )
    .read_to_end(&mut prefix)
    .unwrap();
    let identities: Value = serde_json::from_str(include_str!(
        "../../../fixtures/bootstrap-v41/identification.json"
    ))
    .unwrap();
    let options: Value =
        serde_json::from_str(include_str!("../../../fixtures/bond-v2/game-options.json")).unwrap();
    let mut bits = Vec::new();
    for case in [&identities[usize::from(enabled)], &options[0]] {
        let source = bytes(case["hex"].as_str().unwrap());
        bits.extend(
            (0..case["bit_len"].as_u64().unwrap() as usize)
                .map(|i| source[i / 8] >> (7 - i % 8) & 1 != 0),
        );
    }
    // Complete, explicitly vacant bootstrap roster for the replay prerequisite.
    bits.resize(bits.len() + 32 * (1683 + 1852 * 8), false);
    prefix.extend(pack(&bits));
    crate::theater::parser::test_support::set_native_event_versions(&mut prefix);
    prefix
}
fn packet(kind: u16, payload: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0; 16];
    out[..2].copy_from_slice(&kind.to_le_bytes());
    out[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    out[8..16].copy_from_slice(&1_000_000u64.to_le_bytes());
    out.extend(payload);
    out
}

#[test]
fn corruption_recorded_flag_reaches_public_film_and_replay() {
    for enabled in [false, true] {
        let registry = registry(enabled);
        let parsed =
            crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&registry).unwrap();
        assert_eq!(
            parsed
                .bootstrap_header
                .as_ref()
                .unwrap()
                .corruption_checks(),
            Some(enabled)
        );
        let index = parsed
            .registry
            .archetype(4)
            .unwrap()
            .components
            .iter()
            .position(|c| c.name() == Some("high-frequency"))
            .unwrap();
        let mut snapshot = vec![true];
        for (w, v) in [(32, 0x40000000), (32, 4), (32, 0), (4, 0), (8, 0), (32, 1)] {
            put(&mut snapshot, w, v);
        }
        if enabled {
            put(&mut snapshot, 32, 0x12345678);
        }
        put(&mut snapshot, 32, 0);
        vacant_keyframe_slots(&mut snapshot, 8190);
        let mut frame = vec![true];
        put(&mut frame, 1, 1);
        put(&mut frame, 7, 21);
        put(&mut frame, 3, 0);
        put(&mut frame, 2, 3);
        if enabled {
            put(&mut frame, 1, 1);
            put(&mut frame, 32, 0xffffffff);
        }
        put(&mut frame, 1, 0);
        for new in [true, false] {
            put(&mut frame, if new { 3 } else { 1 }, 1);
            put(&mut frame, 13, 0);
            put(&mut frame, 2, 1);
            if new {
                put(&mut frame, 6, 4);
            }
            put(&mut frame, 1, u64::from(new)); // NEW components present or DELTA baseline absent
            put(&mut frame, 1, 0);
            put(&mut frame, 3, 1);
            put(&mut frame, 6, index as u64);
            put(&mut frame, 8, 231);
            if enabled {
                put(&mut frame, 1, 1);
                put(&mut frame, 32, 0x12345678);
            }
        }
        put(&mut frame, 4, 0);
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
            chunk(ChunkKind::Registry, registry),
            chunk(
                ChunkKind::Replication,
                [packet(2, pack(&snapshot)), packet(0, pack(&frame))].concat(),
            ),
            empty_summary(),
        ])
        .unwrap();
        let packets = &film.replication_stream.0[0].body.packets;
        let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
            &packets[0].body
        else {
            panic!()
        };
        assert_eq!(table.stop, KeyframeChainStop::End);
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(read)) =
            &packets[1].body
        else {
            panic!()
        };
        assert_eq!(read.events.stop, EventListStop::Terminator);
        assert_eq!(read.frame.decoded().unwrap().end_bit, frame.len() as i64);
        for p in packets {
            assert!(
                p.source_coverage(&film.replication_stream.0[0].data)
                    .is_some()
            );
        }
    }
}

mod missing;
