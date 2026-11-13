use super::*;
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
mod limits;
mod secondary_f050;

#[derive(Deserialize)]
struct Case {
    name: String,
    hex: String,
    bit_len: usize,
    kinds: Vec<u8>,
    command_counts: Vec<Option<usize>>,
    secondary: Vec<bool>,
    index_bits: usize,
    region_widths: BTreeMap<u32, [usize; 3]>,
}

impl Case {
    fn data(&self) -> Vec<u8> {
        self.hex
            .as_bytes()
            .chunks(2)
            .map(|hex| u8::from_str_radix(std::str::from_utf8(hex).unwrap(), 16).unwrap())
            .collect()
    }
    fn encoding(&self) -> PositionEncoding {
        let mut encoding = DecodeProfile::default().component_encoding().unwrap();
        encoding.index_bits = self.index_bits;
        encoding.world_axis_bits = Some([15, 15, 17]);
        encoding.region_axis_bits = [(0, [15, 15, 17])].into();
        encoding.region_axis_bits.extend(&self.region_widths);
        encoding
    }
}

fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/theater/fixtures/controls-v41/cases.json"
    )))
    .unwrap()
}

fn active_cases() -> Vec<Case> {
    cases()
        .into_iter()
        .filter(|c| !c.name.starts_with("secondary-full-"))
        .chain(secondary_f050::cases())
        .collect()
}

fn read(case: &Case) -> DecodedFrameView {
    decode_control_view_contextual(&case.data(), 0, Some(&case.encoding()), None, true)
}

#[test]
fn pinned_control_oracle_preserves_wire_fields_and_boundaries() {
    #[derive(Deserialize)]
    struct Oracle {
        name: String,
        end: i64,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Fixture {
        commit: String,
        cases: Vec<Oracle>,
    }
    let oracle: Fixture = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/theater/fixtures/controls-v41/reference.json"
    )))
    .unwrap();
    assert_eq!(oracle.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    let cases = cases();
    assert_eq!(cases.len(), 17);
    assert_eq!(oracle.cases.len(), cases.len());
    let mut superseded = 0;
    for (case, expected) in cases.iter().zip(oracle.cases) {
        assert_eq!(case.name, expected.name);
        if case.name.starts_with("secondary-full-") {
            // Preserve the old oracle's actual trace. These three bodies encode
            // the old wrong handle/local/target branches and are not current
            // format goldens. The f050371 fixture proves their replacements.
            let mut end = 0;
            for (start, width) in expected.trace {
                assert_eq!(start, end);
                end += width;
                assert!(end <= case.bit_len);
            }
            assert_eq!(end, case.bit_len);
            assert_eq!(expected.end as usize, case.bit_len);
            superseded += 1;
            continue;
        }
        let view = read(case);
        assert_eq!(view.stop, FrameViewStop::Complete, "{}", case.name);
        assert_eq!(view.end_bit, expected.end, "{}", case.name);
        assert_eq!(view.kinds, case.kinds, "{}", case.name);
        assert_eq!(view.control_entries.len(), case.kinds.len());
        let bytes = case.data();
        let mut end = 0;
        let mut boundaries = BTreeSet::from([0]);
        for field in &view.fields {
            assert_eq!(field.bit, end, "{}: {}", case.name, field.name);
            assert_eq!(
                field.raw,
                RawBits::from_source(&bytes, field.bit, field.width).unwrap()
            );
            end += field.width;
            boundaries.insert(end);
        }
        assert_eq!(end, case.bit_len);
        // Each upstream read/skip boundary must survive. Our model can split a
        // reference skip into individual source fields, but cannot merge past
        // a reference field boundary or classify any consumed bit as padding.
        for (start, width) in expected.trace {
            assert!(boundaries.contains(&start), "{}: start {start}", case.name);
            assert!(
                boundaries.contains(&(start + width)),
                "{}: end {}",
                case.name,
                start + width
            );
        }
        let counts: Vec<_> = view
            .control_entries
            .iter()
            .filter_map(|e| match &e.payload {
                ControlPayload::Commands { records, .. } => Some(records.as_ref().map(Vec::len)),
                _ => None,
            })
            .collect();
        assert_eq!(counts, case.command_counts);
        let secondary: Vec<_> = view
            .control_entries
            .iter()
            .filter_map(|e| match &e.payload {
                ControlPayload::Input(input) => Some(input.secondary.is_some()),
                _ => None,
            })
            .collect();
        assert_eq!(secondary, case.secondary);
        for entry in &view.control_entries {
            let ranges: Vec<&BitRange> = match &entry.payload {
                ControlPayload::Input(input) => input.secondary.iter().collect(),
                ControlPayload::Commands { records, .. } => records.iter().flatten().collect(),
                ControlPayload::Kind2 { .. } => vec![],
            };
            for range in ranges {
                assert!(entry.start_bit <= range.start as i64 && range.end as i64 <= entry.end_bit);
                assert!(range.start < range.end);
            }
        }
        let json = serde_json::to_string(&view).unwrap();
        assert_eq!(view, serde_json::from_str(&json).unwrap());
    }
    assert_eq!(superseded, 3);
}

#[test]
fn control_truncation_at_every_bit_preserves_only_the_recorded_prefix() {
    for case in active_cases() {
        let complete = read(&case);
        let source = case.data();
        for cut in 0..case.bit_len {
            // Align the retained prefix's END to a byte boundary. No zero tail
            // is available for the decoder to mistake for a gate or terminator.
            let shift = (8 - cut % 8) % 8;
            let mut data = vec![0; (cut + shift) / 8];
            for i in 0..cut {
                data[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let partial = decode_control_view_contextual(
                &data,
                shift as i64,
                Some(&case.encoding()),
                None,
                true,
            );
            assert_eq!(
                partial.stop,
                FrameViewStop::Truncated,
                "{} cut {cut}",
                case.name
            );
            assert!(partial.end_bit <= data.len() as i64 * 8);
            for (got, original) in partial.fields.iter().zip(&complete.fields) {
                assert_eq!(
                    (&got.name, got.bit, got.width, &got.raw),
                    (
                        &original.name,
                        original.bit + shift,
                        original.width,
                        &original.raw
                    ),
                    "{} cut {cut}",
                    case.name
                );
            }
            assert!(partial.fields.len() <= complete.fields.len());
            assert_eq!(
                partial.control_entries.len(),
                complete
                    .control_entries
                    .iter()
                    .filter(|entry| entry.end_bit <= cut as i64)
                    .count(),
                "{} cut {cut}: only complete payloads may be published",
                case.name
            );
            for entry in &partial.control_entries {
                assert!(entry.end_bit <= (shift + cut) as i64);
            }
        }
    }
}

#[test]
fn commands_keep_raw_float_bits_and_relative_prefixes() {
    let cases = cases();
    let absolute = read(
        cases
            .iter()
            .find(|c| c.name == "command-absolute-full")
            .unwrap(),
    );
    let nan = absolute
        .fields
        .iter()
        .find(|f| f.name == "control.commands[0].vectors[0].bits[2]")
        .unwrap();
    assert_eq!((nan.width, nan.raw.low_u64()), (32, 0x7fc12345));
    let relative = read(
        cases
            .iter()
            .find(|c| c.name == "command-relative-full")
            .unwrap(),
    );
    let one_bit = relative
        .fields
        .iter()
        .find(|f| f.name == "control.commands[0].vectors[0][1].delta_bits")
        .unwrap();
    assert_eq!((one_bit.width, one_bit.raw.low_u64()), (1, 1));
    let minimal = read(
        cases
            .iter()
            .find(|c| c.name == "command-relative-minimal")
            .unwrap(),
    );
    assert_eq!(minimal.control_entries[0].baseline, Some(0));
    assert!(!minimal.fields.iter().any(|f| f.name.ends_with("tick.bits")));
}

#[test]
fn kind3_stops_before_the_disputed_baseline_and_retains_prior_records() {
    // A complete absent kind-2 body followed by kind 3 and a plausible tail.
    let data = [0b11000111, 0xff, 0];
    let view = decode_control_view_contextual(&data, 0, None, None, true);
    assert_eq!(view.end_bit, 8);
    assert_eq!(view.kinds, [2, 3]);
    assert_eq!(view.control_entries.len(), 1);
    assert_eq!(view.control_entries[0].index(), None);
    assert_eq!(view.stop, unsupported("control handler kind 3"));
}

#[test]
fn secondary_position_requires_context_and_keeps_the_prefix() {
    let case = secondary_f050::cases()
        .into_iter()
        .find(|c| c.name == "full/none/none/world/0/0")
        .unwrap();
    let view = decode_control_view_contextual(&case.data(), 0, None, None, true);
    assert_eq!(
        view.stop,
        unsupported("secondary control position context unavailable")
    );
    assert!(view.control_entries.is_empty());
    assert_eq!(
        view.fields.last().unwrap().name,
        "control.secondary.default_region"
    );
    assert!(view.end_bit < case.bit_len as i64);
}

#[test]
fn control_records_integrate_with_film_coverage_and_runtime_player_queries() {
    use crate::theater::film::replication::ReplicationChange;
    use crate::theater::parser::packet::{PacketRead, SourceRegionKind};
    use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;
    let case = cases()
        .into_iter()
        .find(|c| c.name == "mixed-wire-order")
        .unwrap();
    // Configuration 0, empty event view, entity terminator 000; then controls.
    let mut payload = vec![0; (5 + case.bit_len).div_ceil(8)];
    let control = case.data();
    for i in 0..case.bit_len {
        payload[(5 + i) / 8] |= ((control[i / 8] >> (7 - i % 8)) & 1) << (7 - (5 + i) % 8);
    }
    let film = film_from_payload(&payload);
    let packet = &film.replication_stream.0[0].body.packets[0];
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
    else {
        panic!("frame")
    };
    let controls = frame.frame.decoded().unwrap().controls.as_ref().unwrap();
    assert_eq!(controls.stop, FrameViewStop::Complete);
    assert_eq!(controls.end_bit, (5 + case.bit_len) as i64);
    assert_eq!(
        controls
            .control_entries
            .iter()
            .map(ControlEntry::index)
            .collect::<Vec<_>>(),
        [None, Some(17), Some(4)]
    );
    let coverage = packet
        .source_coverage(&film.replication_stream.0[0].data)
        .unwrap();
    assert_eq!(coverage[0].kind, SourceRegionKind::Fields);
    assert_eq!(
        coverage[0].source,
        BitRange {
            start: 0,
            end: 5 + case.bit_len
        }
    );
    let runtime = crate::theater::TheaterRuntime::load(typed_film_from_payload(&payload));
    let events: Vec<_> = runtime
        .observations()
        .iter()
        .filter_map(|observation| match &observation.change {
            ReplicationChange::Control { control } => Some(control),
            _ => None,
        })
        .collect();
    assert_eq!(events.len(), 3);
    assert_eq!(
        events
            .iter()
            .map(|control| typed_control_index(control))
            .collect::<Vec<_>>(),
        [None, Some(17), Some(4)]
    );
    assert_eq!(
        events
            .iter()
            .filter(|control| typed_control_index(control) == Some(17))
            .count(),
        1
    );
}

#[test]
fn secondary_positions_use_the_callers_profile_widths() {
    use crate::theater::parser::v41::context::V41DecodeConfig;
    for case in secondary_f050::cases().iter() {
        let mut config = V41DecodeConfig::default();
        config.profile.movement.world_object.index_bits = case.index_bits as u64;
        config.profile.movement.world_object.axis_bits = case
            .region_widths
            .values()
            .next()
            .copied()
            .unwrap_or([15, 15, 17])
            .map(|w| w as u64);
        // record_layout deliberately contains placeholder widths: the reader
        // must obtain the caller's actual widths through ComponentWidths.
        let layout = config.record_layout();
        let actual = decode_control_view_contextual(
            &case.data(),
            0,
            layout.position.as_ref(),
            Some(&config.profile),
            true,
        );
        assert_eq!(actual, read(case), "{}", case.name);
    }
}

fn film_from_payload(payload: &[u8]) -> crate::theater::parser::ParsedFilm {
    film_from_payload_with_limits(payload, crate::theater::parser::ParseLimits::default())
}

fn film_from_payload_with_limits(
    payload: &[u8],
    limits: crate::theater::parser::ParseLimits,
) -> crate::theater::parser::ParsedFilm {
    crate::theater::parser::test_support::parse_fixture_structural_with_limits(
        film_input(payload),
        limits,
    )
    .unwrap()
}

fn film_input(payload: &[u8]) -> Vec<crate::clients::hi::models::RawFilmChunk> {
    use crate::clients::hi::models::RawFilmChunk;
    use crate::theater::film::ChunkKind;
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
    let mut packet = vec![0; 16];
    packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    packet[8..16].copy_from_slice(&123u64.to_le_bytes());
    packet.extend_from_slice(payload);
    vec![
        chunk(
            ChunkKind::Registry,
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
        ),
        chunk(ChunkKind::Replication, packet),
        crate::theater::parser::test_support::empty_summary(),
    ]
}

fn typed_film_from_payload(payload: &[u8]) -> crate::theater::film::Film {
    crate::theater::parser::test_support::parse_fixture(typed_film_input(payload)).unwrap()
}

fn typed_film_input(payload: &[u8]) -> Vec<crate::clients::hi::models::RawFilmChunk> {
    let mut input = film_input(payload);
    input[0] = crate::theater::parser::test_support::registry_with_complete_setup();
    input
}

fn typed_control_index(
    control: &crate::theater::film::replication::ReplicationControl,
) -> Option<u8> {
    match control {
        crate::theater::film::replication::ReplicationControl::Input { index, .. }
        | crate::theater::film::replication::ReplicationControl::Commands { index, .. } => {
            Some(index.raw())
        }
        crate::theater::film::replication::ReplicationControl::Kind2 { .. } => None,
    }
}

#[test]
fn rejected_entity_body_cannot_be_published_as_controls() {
    use crate::theater::film::replication::ReplicationChange;
    use crate::theater::parser::packet::{PacketRead, SourceRegionKind};
    use crate::theater::parser::v41::chunks::replication::source::{
        ProductionEntityEnd, ReplicationStreamPacketBody,
    };
    let case = cases()
        .into_iter()
        .find(|case| case.name == "mixed-wire-order")
        .unwrap();
    // Configuration, event terminator, then an unbound DELTA (slot 77,
    // generation 1). No entity terminator establishes a control-view start.
    let mut prefix = vec![false, false, true];
    prefix.extend((0..13).rev().map(|shift| 77 & (1 << shift) != 0));
    prefix.extend([false, true]);
    let start = prefix.len();
    assert_eq!(start, 18);
    let mut payload = vec![0u8; (start + case.bit_len).div_ceil(8)];
    for (i, bit) in prefix.into_iter().enumerate() {
        payload[i / 8] |= u8::from(bit) << (7 - i % 8);
    }
    let tail = case.data();
    for i in 0..case.bit_len {
        payload[(start + i) / 8] |= ((tail[i / 8] >> (7 - i % 8)) & 1) << (7 - (start + i) % 8);
    }
    let film = film_from_payload(&payload);
    let chunk = &film.replication_stream.0[0];
    let packet = &chunk.body.packets[0];
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
    else {
        panic!("frame")
    };
    let frame = frame.frame.decoded().unwrap();
    assert!(matches!(
        frame.entity_end,
        Some(ProductionEntityEnd::Rejected { .. })
    ));
    assert_eq!(frame.end_bit, start as i64);
    assert!(frame.controls.is_none());
    let coverage = packet.source_coverage(&chunk.data).unwrap();
    let unread = coverage.last().unwrap();
    assert_eq!(unread.kind, SourceRegionKind::Unparsed);
    assert_eq!(
        unread.source,
        BitRange {
            start,
            end: payload.len() * 8
        }
    );
    assert_eq!(chunk.payload(packet).unwrap(), payload);
    let runtime = crate::theater::TheaterRuntime::load(typed_film_from_payload(&payload));
    assert_eq!(
        runtime
            .observations()
            .iter()
            .filter(|observation| matches!(&observation.change, ReplicationChange::Control { .. }))
            .count(),
        0
    );
}
