use super::decode_weapon_damage;
use super::*;
use crate::theater::Film;
use crate::theater::parser::test_support::FixtureChunkMetadata;
use crate::theater::parser::test_support::FixtureFilmSource;
use crate::theater::parser::test_support::test_chunks;
use serde_json::{Value, json};
use std::io::Read;

pub(crate) fn check_fields(read: &WeaponDamageTrace, payload: &[u8]) {
    assert_eq!(read.source_bits, payload.len() * 8);
    let mut end = 0;
    let mut padded = 0;
    for field in &read.fields {
        let raw = &field.field;
        assert_eq!(raw.bit, end);
        let start = end;
        end += raw.width;
        let mut expected = 0u64;
        for bit in start..end {
            expected = (expected << 1)
                | u64::from(payload.get(bit / 8).map_or(0, |b| (b >> (7 - bit % 8)) & 1));
        }
        assert_eq!(raw.value, expected, "{}", raw.name);
        assert_eq!(
            field.padded_bits,
            end.saturating_sub(start.max(payload.len() * 8))
        );
        padded += field.padded_bits;
    }
    assert_eq!(end, read.end_bit);
    assert_eq!(padded, read.padding_bits);
    if let Some(projected) = &read.read {
        assert_eq!(projected.end_bit, read.end_bit);
        assert_eq!(projected.padding_bits, read.padding_bits);
    }
}

#[test]
fn historical_damage_inputs_match_corrected_structural_boundaries() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("../../../../fixtures/weapon-hit-scan-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 2048);
    let mut padded = 0;
    let mut opaque_nonzero = 0;
    let mut generations = 0;
    for (i, case) in cases.iter().enumerate() {
        let hex = case["damage_bytes"].as_str().unwrap();
        let payload: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let read = read_weapon_damage(&payload, i as u64);
        let projection = read.read.as_ref().unwrap();
        let expected: WeaponDamage = serde_json::from_value(case["damage"].clone()).unwrap();
        // The older fixture inverted field_0x10's gate. Its header/source
        // expectations remain valid; later values and its endpoint do not.
        assert_eq!(projection.damage.timestamp_us, expected.timestamp_us);
        assert_eq!(projection.damage.victim_idx, expected.victim_idx);
        assert_eq!(projection.damage.responsible_idx, expected.responsible_idx);
        assert_eq!(projection.damage.source, expected.source);
        assert_eq!(projection.damage.has_source, expected.has_source);
        // Compare the complete corrected body against the independent structural
        // implementation, with zero padding explicit for this legacy projection.
        let mut padded_payload = payload.clone();
        padded_payload.resize(payload.len().max(read.end_bit.div_ceil(8)) + 1, 0);
        let structural = crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::read_reference_event_list(
            &padded_payload, 1, Some(false), 1,
        );
        assert!(structural.records[0].layout_complete, "case {i}");
        assert_eq!(structural.records[0].end_bit, read.end_bit, "case {i}");
        assert_eq!(read.read, decode_weapon_damage(&payload, i as u64));
        check_fields(&read, &payload);
        padded += usize::from(read.padding_bits > 0);
        opaque_nonzero += read
            .fields
            .iter()
            .filter(|f| f.opaque && f.field.value != 0)
            .count();
        generations += read
            .fields
            .iter()
            .filter(|f| f.field.name.ends_with(".generation"))
            .count();
        assert_eq!(
            serde_json::from_value::<WeaponDamageTrace>(json!(read)).unwrap(),
            read
        );
    }
    assert!(padded > 100 && opaque_nonzero > 0 && generations > 0);
    let refused = read_weapon_damage(&[0xc0, 0x80], 0);
    assert!(refused.read.is_none());
    assert_eq!(refused.end_bit, 9);
    check_fields(&refused, &[0xc0, 0x80]);
}

#[test]
fn reference_data_damage_admission_and_provenance() {
    fn packet(kind: u16, payload: &[u8]) -> Vec<u8> {
        let mut data = vec![0; 16];
        data[..2].copy_from_slice(&kind.to_le_bytes());
        data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        data[8..16].copy_from_slice(&123u64.to_le_bytes());
        data.extend(payload);
        data
    }
    let mut stream = packet(0, &[0xc0, 0]); // accepted projection with synthetic tail
    stream.extend(packet(0, &[0xc0, 0x80])); // wrong full event code, selected refusal
    stream.extend(packet(0, &[0xc0])); // too short for reference selection
    stream.extend(packet(6, &[0xc0, 0])); // wrong packet type
    stream.extend(packet(0, &[0x80, 0])); // wrong first byte
    let source = FixtureFilmSource::load(
        &[
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            stream.clone(),
            stream,
        ],
        &[
            FixtureChunkMetadata {
                index: 0,
                chunk_type: 1,
                start_ms: 0,
            },
            FixtureChunkMetadata {
                index: 9,
                chunk_type: 2,
                start_ms: 0,
            },
            FixtureChunkMetadata {
                index: 10,
                chunk_type: 3,
                start_ms: 0,
            },
        ],
    )
    .unwrap();
    let parsed = Film::parse(test_chunks(&source)).unwrap();
    let packets = &parsed.replication_stream.first().unwrap().body.packets;
    let resolved = crate::theater::runtime::resolved::ResolvedFilm::from_film(std::sync::Arc::new(
        parsed.clone(),
    ));
    let damage = |packet| {
        resolved
            .interpretations()
            .packets
            .iter()
            .find(|entry| entry.source.chunk == 1 && entry.source.packet == packet)
            .and_then(|entry| entry.damage.as_ref())
    };
    let read = damage(0).unwrap();
    check_fields(read, &[0xc0, 0]);
    assert!(read.padding_bits > 0);
    let projection = read.read.as_ref().unwrap();
    assert_eq!(projection.source, Some(packets[0].header));
    assert_eq!(projection.packet_index, Some(0));
    assert_eq!(projection.damage.timestamp_us, 123);
    assert!(damage(1).unwrap().read.is_none());
    assert!((2..packets.len()).all(|packet| damage(packet).is_none()));
    assert!(
        resolved
            .interpretations()
            .packets
            .iter()
            .all(|entry| entry.source.chunk != 2)
    );
    assert_eq!(
        serde_json::from_value::<Film>(json!(parsed)).unwrap(),
        parsed
    );
}

#[test]
fn damage_absent_flag_preserves_following_fields_and_boundary() {
    use crate::theater::film::EventListStop;
    use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::read_reference_event_list;
    for absent in [false, true] {
        let mut fields = vec![
            (1, 1),
            (1, 1),
            (0, 7),
            (0, 3), // config, event, three absent refs
            (1, 1),
            (0x12345678, 32), // damage source
            (u64::from(absent), 1),
        ];
        if !absent {
            fields.push((21, 5));
        }
        fields.extend([
            (123, 19),
            (1, 1),
            (456, 19),
            (321, 12),
            (2, 5),
            (3, 5),
            (4, 6),
            (1, 1),
            (5, 5),
            (0x1555, 14),
            (1, 1),
            (0x87654321, 32),
            (1, 1),
            (3, 3),
            (17, 5),
            (9, 5),
            (1, 1),
            (2, 4),
            (0, 1),
            (123, 10),
            (1, 4),
            (1, 1),
            (0x11223344, 32),
            (0xab, 8),
            (3, 4),
            (1, 1),
            (321, 13),
            (2, 2),
        ]);
        let end: usize = fields.iter().map(|(_, width)| width).sum();
        fields.push((0, 1)); // event-list terminator
        let mut bits = Vec::new();
        for (value, width) in fields {
            bits.extend((0..width).rev().map(|i| ((value >> i) & 1) as u8));
        }
        let mut payload = vec![0; bits.len().div_ceil(8)];
        for (bit, value) in bits.iter().enumerate() {
            payload[bit / 8] |= value << (7 - bit % 8);
        }
        let read = read_weapon_damage(&payload, 123);
        let projected = read.read.as_ref().unwrap();
        assert_eq!(read.end_bit, end);
        assert_eq!(read.padding_bits, 0);
        assert_eq!(projected.damage.source, 0x12345678);
        assert_eq!(projected.damage.mag_raw, 17);
        assert_eq!(projected.secondary_mag_raw, 9);
        assert!(projected.damage.negative);
        assert_eq!(projected.body_victim_idx, Some(321));
        assert_eq!(
            read.fields
                .iter()
                .any(|f| f.field.name == "body.field_0x10"),
            !absent
        );
        check_fields(&read, &payload);
        let structural = read_reference_event_list(&payload, 1, None, payload.len());
        assert_eq!(structural.stop, EventListStop::Terminator);
        assert_eq!(structural.records[0].end_bit, end);
        assert_eq!(structural.end_bit, end + 1);
        for size in 0..(end + 1).div_ceil(8) {
            let truncated = read_reference_event_list(&payload[..size], 1, None, payload.len());
            assert_ne!(truncated.stop, EventListStop::Terminator);
            assert!(
                truncated
                    .records
                    .iter()
                    .all(|record| record.end_bit <= size * 8)
            );
        }
    }
}
