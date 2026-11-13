use super::*;
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::ChunkKind;
use crate::theater::parser::bits::Bits;
use crate::theater::parser::test_support::{component_registry, vacant_keyframe_slots};
use serde::Deserialize;

mod captured;

#[test]
fn captured_damage_section_refusal_matches_upstream_without_skipping_the_count() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../../fixtures/snapshot-refusal-v41/reference.json"
    ))
    .unwrap();
    let data = include_bytes!("../../../../../../../fixtures/snapshot-refusal-v41/packet.bin");
    let rows: Vec<(String, u32)> = serde_json::from_value(reference["registry"].clone()).unwrap();
    let registry = component_registry(&[(35, rows)]);
    let config = V41DecodeConfig::default();
    let start = reference["start"].as_u64().unwrap() as usize;
    let read = super::super::record::decode_keyframe_record_contextual(
        data,
        start,
        &registry,
        &config.record_layout(),
        Some(&config.profile),
    )
    .unwrap();
    assert_eq!(read.id, 0x400002f3);
    assert_eq!(read.end_bit, 227003);
    assert_eq!(
        read.stop,
        KeyframeStop::UnsupportedComponent {
            index: 7,
            name: "object-damage-sections-component".into()
        }
    );
    let expected = reference["components"].as_array().unwrap();
    assert_eq!(read.components.len(), expected.len());
    for (c, e) in read.components.iter().zip(expected) {
        assert_eq!(c.index, e["index"].as_u64().unwrap() as usize);
        assert_eq!(c.start_bit, e["start"].as_i64().unwrap());
        assert_eq!(c.end_bit, e["end"].as_i64().unwrap());
        assert_eq!(
            c.status == ComponentReadStatus::Complete,
            e["complete"].as_bool().unwrap()
        );
        let mut source_bits = std::collections::BTreeSet::new();
        for pair in e["trace"].as_array().unwrap() {
            let at = pair[0].as_u64().unwrap() as usize;
            let width = pair[1].as_u64().unwrap() as usize;
            source_bits.extend(at..at + width);
        }
        let mut retained = std::collections::BTreeSet::new();
        for field in &c.fields {
            assert_eq!(
                Some(&field.raw),
                RawBits::from_source(data, field.bit, field.width).as_ref()
            );
            retained.extend(field.bit..field.bit + field.width);
        }
        assert_eq!(retained, source_bits);
    }
    let c = read.components.last().unwrap();
    assert_eq!(c.fields.len(), 1);
    assert_eq!(c.fields[0].name, "count");
    assert_eq!(c.fields[0].raw.low_u64(), 39);
}

#[derive(Deserialize, Default)]
struct Entry {
    slot: usize,
    id: u32,
    archetype: u32,
    #[serde(default)]
    word: u32,
    #[serde(default)]
    flags: u8,
    #[serde(default)]
    byte: u8,
    #[serde(default)]
    n1: u32,
    #[serde(default)]
    n2: u32,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    count: usize,
    #[serde(default)]
    entries: Vec<Entry>,
    truncate_bytes: Option<usize>,
    #[serde(default)]
    nonzero_tail: bool,
}
#[derive(Deserialize)]
struct Result {
    name: String,
    bits: usize,
    headers: usize,
    end: usize,
    ok: bool,
    live: Vec<i64>,
}
#[derive(Deserialize)]
struct Fixture {
    commit: String,
    cases: Vec<Case>,
    results: Vec<Result>,
}
fn word(bits: &mut Vec<bool>, width: usize, v: u32) {
    bits.extend((0..width).rev().map(|i| v & (1 << i) != 0));
}
fn header(bits: &mut Vec<bool>, e: &Entry) {
    for v in [e.id, e.archetype, e.word] {
        word(bits, 32, v);
    }
    word(bits, 4, u32::from(e.flags));
    word(bits, 8, u32::from(e.byte));
}
fn pack(bits: &[bool]) -> Vec<u8> {
    let mut bytes = vec![0; bits.len().div_ceil(8)];
    for (i, b) in bits.iter().enumerate() {
        bytes[i / 8] |= u8::from(*b) << (7 - i % 8);
    }
    bytes
}
fn payload(c: &Case) -> Vec<u8> {
    let mut bits = vec![true];
    for i in 0..c.count {
        if let Some(e) = c.entries.iter().find(|e| e.slot == i) {
            header(&mut bits, e);
            if e.archetype < 50 {
                word(&mut bits, 32, e.n1);
                word(&mut bits, 32, e.n2);
            }
        } else {
            vacant_keyframe_slots(&mut bits, 1);
        }
    }
    bits.resize(bits.len().next_multiple_of(8), c.nonzero_tail);
    let mut bytes = pack(&bits);
    if let Some(n) = c.truncate_bytes {
        bytes.truncate(n);
    }
    bytes
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!(
        "../../../../../../../fixtures/keyframe-slots-v41/reference.json"
    ))
    .unwrap()
}

#[test]
fn native_table_matches_original_go_traversal_and_retains_headers() {
    let f = fixture();
    assert_eq!(f.commit, "413971bcc485be53a2fa8cf229e170539a5146f8");
    assert_eq!(f.cases.len(), 24);
    let registry = component_registry(&[(0, vec![])]);
    for (case, expected) in f.cases.iter().zip(&f.results) {
        assert_eq!(case.name, expected.name);
        let data = payload(case);
        let table = decode(&data, &registry, &V41DecodeConfig::default(), |_| None);
        let slots = table.slots.as_ref().unwrap();
        assert_eq!(data.len() * 8, expected.bits);
        assert_eq!(slots.headers.len(), expected.headers, "{}", case.name);
        assert_eq!(
            table.stop == KeyframeChainStop::End,
            expected.ok,
            "{}",
            case.name
        );
        assert_eq!(
            table
                .records
                .iter()
                .map(|r| r.start_bit)
                .collect::<Vec<_>>(),
            expected.live
        );
        // Native Go returns the failing slot's start. Rust additionally retains
        // fields consumed before that failure; successful boundaries are identical.
        if expected.ok || table.stop == KeyframeChainStop::TrailingData {
            assert_eq!(slots.end_bit, expected.end);
        } else if !data.is_empty() {
            assert!(slots.end_bit >= expected.end);
        }
        for h in &slots.headers {
            for (offset, width, v) in [
                (0, 32, h.id),
                (32, 32, h.archetype),
                (64, 32, h.word),
                (96, 4, u32::from(h.flags)),
                (100, 8, u32::from(h.byte)),
            ] {
                assert_eq!(
                    Bits(&data).read(h.start_bit + offset, width),
                    Some(u64::from(v))
                );
            }
        }
        assert!(slots.end_bit <= data.len() * 8);
        assert_eq!(
            slots.tail,
            BitRange {
                start: slots.end_bit,
                end: data.len() * 8
            }
        );
        let json = serde_json::to_vec(&table).unwrap();
        assert_eq!(
            serde_json::from_slice::<KeyframeTable>(&json).unwrap(),
            table
        );
    }
}

#[test]
fn partial_table_and_unsupported_component_never_report_end_or_scan_a_successor() {
    let registry = component_registry(&[(0, vec![("unknown-slot-component".into(), 1)])]);
    let mut bits = vec![false];
    vacant_keyframe_slots(&mut bits, 2);
    header(
        &mut bits,
        &Entry {
            id: 4,
            ..Default::default()
        },
    );
    word(&mut bits, 32, 0);
    word(&mut bits, 32, 1);
    let stop_bit = bits.len();
    vacant_keyframe_slots(&mut bits, 8188);
    let data = pack(&bits);
    let table = decode(&data, &registry, &V41DecodeConfig::default(), |_| None);
    assert_eq!(table.stop, KeyframeChainStop::Desync);
    assert_eq!(table.slots.as_ref().unwrap().end_bit, stop_bit);
    assert_eq!(table.slots.as_ref().unwrap().headers.len(), 3);
    assert!(matches!(
        table.records[0].record.as_ref().unwrap().stop,
        KeyframeStop::UnsupportedComponent { index: 0, .. }
    ));
    for n in 0..=stop_bit / 8 {
        let read = decode(&data[..n], &registry, &V41DecodeConfig::default(), |_| None);
        assert_ne!(read.stop, KeyframeChainStop::End);
        assert!(read.slots.as_ref().unwrap().end_bit <= n * 8);
    }
}

#[test]
fn complete_vacant_table_accounts_for_every_bit_and_extra_data_stays_unparsed() {
    use crate::theater::parser::packet::coverage::partition;
    let f = fixture();
    let registry = component_registry(&[(0, vec![])]);
    for name in ["all-vacant", "interleaved-live", "extra-slot", "one-vacant"] {
        let case = f.cases.iter().find(|c| c.name == name).unwrap();
        let data = payload(case);
        let table = decode(&data, &registry, &V41DecodeConfig::default(), |_| None);
        let end = table.slots.as_ref().unwrap().end_bit;
        let complete = table.stop == KeyframeChainStop::End;
        let body = ReplicationStreamPacketBody::KeyframesPacketBody(Box::new(table));
        let regions = partition(data.len() * 8, body.source_regions()).unwrap();
        assert_eq!(regions[0].source, BitRange { start: 0, end });
        assert_eq!(regions[0].kind, SourceRegionKind::Fields);
        assert_eq!(
            regions.last().unwrap().kind,
            if complete {
                SourceRegionKind::Padding
            } else {
                SourceRegionKind::Unparsed
            }
        );
    }
}

#[test]
fn native_framing_refuses_reference_width_overrides() {
    let registry = component_registry(&[(0, vec![])]);
    let mut config = V41DecodeConfig::default();
    config.profile.keyframe.header_bits = 109;
    assert_eq!(
        decode(&[0; 24], &registry, &config, |_| None).stop,
        KeyframeChainStop::InvalidEncoding
    );
}

#[test]
fn later_live_ids_bind_without_using_physical_ordinals_or_out_of_domain_ids() {
    use crate::theater::parser::v41::chunks::replication::{
        replication_stream::DecodeContext, state::ReplicationDecodeState,
    };
    let f = fixture();
    let case = f
        .cases
        .iter()
        .find(|c| c.name == "interleaved-live")
        .unwrap();
    let registry = component_registry(&[(0, vec![])]);
    let config = V41DecodeConfig::default();
    let mut state = ReplicationDecodeState::default();
    let mut context = DecodeContext {
        config: &config,
        registry: &registry,
        state: &mut state,
        event_gate15: None,
    };
    let body = super::super::decode(&payload(case), &mut context, |_| None).unwrap();
    let ReplicationStreamPacketBody::KeyframesPacketBody(table) = body else {
        panic!()
    };
    assert_eq!(table.records.len(), 3);
    assert_eq!(table.records[0].id, u32::MAX);
    assert_eq!(state.slots.keys().copied().collect::<Vec<_>>(), vec![2, 5]);
    assert_eq!(state.keyframe_namespace, Some(0));
}

#[test]
fn exact_snapshot_site_supplies_vehicle_gate_without_changing_other_sites() {
    use crate::theater::parser::v41::chunks::replication::{
        V41ReplicationStreamChunkReader, replication_stream::DecodeContext,
        state::ReplicationDecodeState,
    };
    use crate::theater::parser::{FilmParseContext, VehicleSnapshotSite};
    let id = 0x4000_0300;
    let registry = component_registry(&[(
        40,
        vec![
            ("vehicle-type-state-component".into(), 0),
            ("vehicle-type-physics-component".into(), 0),
        ],
    )]);
    let packet = |gate| {
        let mut bits = vec![true]; // packet configuration
        header(
            &mut bits,
            &Entry {
                id,
                archetype: 40,
                flags: 3,
                ..Default::default()
            },
        );
        word(&mut bits, 32, 0); // no default body
        word(&mut bits, 32, 1); // component state present
        if gate {
            word(&mut bits, 2, 1); // vehicle type state
            word(&mut bits, 6, 0x15);
            word(&mut bits, 1, 1); // raw physics
            for value in 1..=9 {
                word(&mut bits, 32, value);
            }
        }
        let payload = pack(&bits);
        let mut packet = vec![0_u8; 16];
        packet[0..2].copy_from_slice(&2_u16.to_le_bytes());
        packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(&payload);
        packet
    };
    let site = VehicleSnapshotSite {
        chunk_index: 5,
        packet_index: 0,
        id,
    };
    for (selected, supplied_gate, expected_end, expected_gate) in [
        (None, true, 173, None),
        (
            Some(VehicleSnapshotSite {
                packet_index: 1,
                ..site
            }),
            true,
            173,
            None,
        ),
        (Some(site), true, 470, Some(true)),
        (Some(site), false, 173, Some(false)),
    ] {
        let context = FilmParseContext {
            vehicle_snapshot_gates: selected
                .into_iter()
                .map(|site| (site, supplied_gate))
                .collect(),
            ..Default::default()
        };
        let inflated = RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 5,
            start_ms: 0,
            data: crate::theater::parser::test_support::zlib_fixture_data(&packet(supplied_gate)),
        }
        .inflate()
        .unwrap();
        let config = V41DecodeConfig::default();
        let mut state = ReplicationDecodeState::default();
        let mut decode = DecodeContext {
            config: &config,
            registry: &registry,
            state: &mut state,
            event_gate15: None,
        };
        let chunk =
            V41ReplicationStreamChunkReader::read(inflated, &mut decode, &context, 65536).unwrap();
        let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
            &chunk.body.packets[0].body
        else {
            panic!("keyframe packet")
        };
        let record = table.records[0].record.as_ref().unwrap();
        assert_eq!(record.end_bit, expected_end);
        assert_eq!(record.external_vehicle_type_gate_used, expected_gate);
        assert_eq!(
            record.stop == KeyframeStop::Complete,
            selected == Some(site)
        );
    }
}
