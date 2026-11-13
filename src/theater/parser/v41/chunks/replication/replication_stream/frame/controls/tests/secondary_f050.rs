//! Corrected control-unit body and its following record, from upstream f050371.
use super::*;
mod captured;

struct Fixture {
    case: Case,
    meta: Metadata,
}

#[derive(Deserialize)]
struct Metadata {
    generation: u64,
    position_mode: String,
    unit: String,
    reference: String,
    target: bool,
    secondary_end: usize,
}

fn fixtures() -> Vec<Fixture> {
    let rows: Vec<serde_json::Value> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/theater/fixtures/controls-f050371-v41/cases.json"
    )))
    .unwrap();
    rows.into_iter()
        .map(|row| Fixture {
            case: serde_json::from_value(row.clone()).unwrap(),
            meta: serde_json::from_value(row).unwrap(),
        })
        .collect()
}

pub(super) fn cases() -> Vec<Case> {
    fixtures().into_iter().map(|f| f.case).collect()
}

fn shifted(data: &[u8], bits: usize, shift: usize) -> Vec<u8> {
    let mut out = vec![0; (bits + shift).div_ceil(8)];
    for i in 0..bits {
        out[(i + shift) / 8] |= ((data[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
    }
    out
}

#[test]
fn corrected_control_oracle_matches_every_alignment_source_field_and_successor() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/theater/fixtures/controls-f050371-v41/reference.json"
    )))
    .unwrap();
    assert_eq!(oracle["commit"], "f050371a6cda83b9887eeb83a1884336fce28e53");
    let fixtures = fixtures();
    assert_eq!(fixtures.len(), 336);
    assert_eq!(oracle["cases"].as_array().unwrap().len(), fixtures.len());
    for (fixture, expected) in fixtures.iter().zip(oracle["cases"].as_array().unwrap()) {
        let case = &fixture.case;
        assert_eq!(expected["name"], case.name);
        assert_eq!(expected["end"], case.bit_len);
        for shift in 0..8 {
            let data = shifted(&case.data(), case.bit_len, shift);
            let view = decode_control_view_contextual(
                &data,
                shift as i64,
                Some(&case.encoding()),
                None,
                true,
            );
            assert_eq!(
                view.stop,
                FrameViewStop::Complete,
                "{} shift {shift}",
                case.name
            );
            assert_eq!(view.end_bit, (case.bit_len + shift) as i64, "{}", case.name);
            assert_eq!(view.kinds, [0, 2]);
            assert_eq!(view.control_entries.len(), 2);
            assert_eq!(
                view.control_entries[0].end_bit,
                (fixture.meta.secondary_end + shift) as i64
            );
            assert_eq!(view.control_entries[0].index(), Some(21));
            assert_eq!(
                view.control_entries[0].baseline,
                (fixture.meta.generation % 2 == 1).then_some(0)
            );
            assert!(matches!(
                view.control_entries[1].payload,
                ControlPayload::Kind2 {
                    value: Some(0x15ab)
                }
            ));
            let mut end = shift;
            let mut boundaries = BTreeSet::from([shift]);
            for field in &view.fields {
                assert_eq!(field.bit, end);
                assert_eq!(
                    field.raw,
                    RawBits::from_source(&data, field.bit, field.width).unwrap()
                );
                end += field.width;
                boundaries.insert(end);
            }
            assert_eq!(end, shift + case.bit_len);
            for primitive in expected["trace"].as_array().unwrap() {
                let at = primitive[0].as_u64().unwrap() as usize + shift;
                let width = primitive[1].as_u64().unwrap() as usize;
                assert!(boundaries.contains(&at), "{}: {at}", case.name);
                assert!(
                    boundaries.contains(&(at + width)),
                    "{}: {}",
                    case.name,
                    at + width
                );
            }
            assert_fields(fixture, &view);
            assert_eq!(
                serde_json::from_slice::<DecodedFrameView>(&serde_json::to_vec(&view).unwrap())
                    .unwrap(),
                view
            );
        }
    }
}

fn assert_fields(f: &Fixture, view: &DecodedFrameView) {
    let field = |name: &str| view.fields.iter().find(|v| v.name == name);
    let scalar = |name: &str, width, value| {
        let got = field(name).unwrap_or_else(|| panic!("{}: {name}", f.case.name));
        assert_eq!(
            (got.width, got.raw.low_u64()),
            (width, value),
            "{}: {name}",
            f.case.name
        );
    };
    for (prefix, form) in [("unit", &f.meta.unit), ("reference", &f.meta.reference)] {
        if prefix == "reference" && f.meta.position_mode == "none" {
            assert!(field("control.secondary.reference.present").is_none());
            continue;
        }
        let name = format!("control.secondary.{prefix}");
        scalar(&format!("{name}.present"), 1, u64::from(form != "none"));
        if form != "none" {
            scalar(&format!("{name}.category4"), 1, u64::from(form == "unit"));
            let width = if form == "unit" { 9 } else { 13 };
            scalar(&format!("{name}.value"), width, (1 << width) - 1);
            scalar(&format!("{name}.generation"), 2, f.meta.generation);
        }
    }
    if f.meta.position_mode == "none" {
        return;
    }
    scalar(
        "control.secondary.local_position",
        1,
        u64::from(f.meta.position_mode == "local"),
    );
    if f.meta.position_mode == "local" {
        for (i, value) in [0, 0x1555, 0x3fff].into_iter().enumerate() {
            scalar(&format!("control.secondary.local_axes[{i}]"), 14, value);
        }
        assert!(field("control.secondary.default_region").is_none());
    }
    scalar(
        "control.secondary.target.present",
        1,
        u64::from(f.meta.target),
    );
    if f.meta.target {
        scalar("control.secondary.target.value", 13, 8191);
        scalar("control.secondary.target.generation", 2, f.meta.generation);
        scalar("control.secondary.target.signed_word", 11, 2047);
    } else {
        assert!(field("control.secondary.target.value").is_none());
        assert!(field("control.secondary.target.signed_word").is_none());
    }
    scalar("control.secondary.tail", 7, 101);
}

#[test]
fn corrected_control_points_only_require_context_for_indexed_boxes() {
    for f in fixtures() {
        let actual = decode_control_view_contextual(&f.case.data(), 0, None, None, true);
        if matches!(f.meta.position_mode.as_str(), "world" | "region3") {
            assert_eq!(
                actual.stop,
                unsupported("secondary control position context unavailable")
            );
            assert!(actual.control_entries.is_empty());
            assert_eq!(
                actual.fields.last().unwrap().name,
                "control.secondary.default_region"
            );
        } else {
            assert_eq!(actual, read(&f.case), "{}", f.case.name);
        }
    }
    let f = fixtures()
        .into_iter()
        .find(|f| f.case.name == "full/none/none/region3/0/0")
        .unwrap();
    let mut profile = DecodeProfile::default();
    profile.movement.world_object.index_bits = 2;
    profile.movement.world_object.regions = [(0, [15, 15, 17])].into();
    let actual = decode_control_view_contextual(
        &f.case.data(),
        0,
        Some(&f.case.encoding()),
        Some(&profile),
        true,
    );
    assert_eq!(actual.stop, unsupported("unknown world region"));
    assert_eq!(
        actual.fields.last().unwrap().name,
        "control.secondary.region_index"
    );
    assert!(actual.control_entries.is_empty());
}

#[test]
fn corrected_control_reference_selector_never_supplies_a_missing_global_domain() {
    for form in ["object", "unit"] {
        let f = fixtures()
            .into_iter()
            .find(|f| f.case.name == format!("full/none/{form}/local/0/0"))
            .unwrap();
        let view = decode_control_view_contextual(&f.case.data(), 0, None, None, false);
        assert_eq!(view.stop, unsupported("global reference range"));
        assert!(view.control_entries.is_empty());
        let last = view.fields.last().unwrap();
        assert_eq!(last.name, "control.secondary.reference.category4");
        assert_eq!(last.raw.low_u64(), u64::from(form == "unit"));
        assert!(
            !view
                .fields
                .iter()
                .any(|f| f.name == "control.secondary.reference.value")
        );
    }
}

#[test]
fn corrected_secondary_controls_reach_public_film_coverage_and_runtime_in_source_order() {
    use crate::clients::hi::models::RawFilmChunk;
    use crate::theater::film::ChunkKind;
    use crate::theater::film::replication::ReplicationChange;
    use crate::theater::parser::packet::{PacketRead, SourceRegionKind};
    use crate::theater::parser::test_support::{empty_summary, registry_with_corruption_policy};
    use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;
    for f in fixtures().into_iter().filter(|f| {
        f.meta.generation == 0
            && f.meta.unit == "none"
            && matches!(f.meta.position_mode.as_str(), "local" | "default")
    }) {
        // Configuration one establishes the global reference domains. Then an
        // empty event list and the recorded entity terminator precede controls.
        let mut payload = shifted(&f.case.data(), f.case.bit_len, 5);
        payload[0] |= 0x80;
        let mut data = vec![0; 16];
        data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        data[8..16].copy_from_slice(&123u64.to_le_bytes());
        data.extend_from_slice(&payload);
        let source = RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 1,
            start_ms: 0,
            data,
        };
        let film = crate::theater::parser::test_support::parse_fixture_structural([
            registry_with_corruption_policy(false),
            source.clone(),
            empty_summary(),
        ])
        .unwrap();
        let chunk = &film.replication_stream.0[0];
        assert_eq!(chunk.index, usize::try_from(source.index).unwrap());
        assert_eq!(chunk.start_ms, source.start_ms);
        assert_eq!(chunk.data, source.data);
        let packet = &chunk.body.packets[0];
        assert_eq!(packet.payload(&chunk.data), Some(payload.as_slice()));
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
        else {
            panic!("frame")
        };
        let controls = frame.frame.decoded().unwrap().controls.as_ref().unwrap();
        assert_eq!(controls.stop, FrameViewStop::Complete, "{}", f.case.name);
        assert_eq!(controls.end_bit, (5 + f.case.bit_len) as i64);
        assert_eq!(controls.control_entries.len(), 2);
        assert_eq!(
            controls.control_entries[0].end_bit,
            (5 + f.meta.secondary_end) as i64
        );
        assert_fields(&f, controls);
        let coverage = packet.source_coverage(&chunk.data).unwrap();
        assert_eq!(coverage[0].kind, SourceRegionKind::Fields);
        assert_eq!(
            coverage[0].source,
            BitRange {
                start: 0,
                end: 5 + f.case.bit_len
            }
        );
        let typed = crate::theater::parser::test_support::parse_fixture([
            crate::theater::parser::test_support::registry_with_complete_setup(),
            source,
            empty_summary(),
        ])
        .unwrap();
        let runtime = crate::theater::TheaterRuntime::load(typed);
        let indices: Vec<_> = runtime
            .observations()
            .iter()
            .filter_map(|observation| match &observation.change {
                ReplicationChange::Control { control } => Some(typed_control_index(control)),
                _ => None,
            })
            .collect();
        assert_eq!(indices, [Some(21), None]);
    }
}
