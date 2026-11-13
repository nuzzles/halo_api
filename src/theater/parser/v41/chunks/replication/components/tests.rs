//! Independent reference reader primitive traces, with canonical bounded reads.
use super::{cursor::ComponentCursor, reader::ComponentReader, widths::read_component};
use crate::theater::parser::v41::chunks::replication::source::components::field::{
    ComponentField, RawBits,
};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Case {
    name: String,
    component: String,
    version: u32,
    #[serde(default)]
    archetype: u32,
    hex: String,
    bit_len: usize,
    complete: bool,
    #[serde(default)]
    snapshot: bool,
}
impl Case {
    fn bytes(&self) -> Vec<u8> {
        self.hex
            .as_bytes()
            .chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
}
fn reference_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/cases.json"
    ))
    .unwrap()
}
fn cases() -> Vec<Case> {
    reference_cases()
        .into_iter()
        .filter(|c| !property::disagrees(c) && !position_finite::superseded(c))
        .chain(property::cases())
        .collect()
}
fn orientation_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/orientation-source.json"
    ))
    .unwrap()
}
fn tacmap_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/tacmap-source.json"
    ))
    .unwrap()
}

fn predicted_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/predicted-ability-source.json"
    ))
    .unwrap()
}

fn managed_cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/managed-source.json"
    ))
    .unwrap()
}
mod managed;
mod position_finite;
mod property;

fn read(case: &Case, bytes: &[u8], start: usize) -> (Option<bool>, i64, Vec<ComponentField>) {
    read_with_context(case, bytes, start, true)
}
fn read_with_context(
    case: &Case,
    bytes: &[u8],
    start: usize,
    context: bool,
) -> (Option<bool>, i64, Vec<ComponentField>) {
    read_with_precision(case, bytes, start, context, false, false)
}
fn read_with_precision(
    case: &Case,
    bytes: &[u8],
    start: usize,
    context: bool,
    full_precision: bool,
    baseline_scope: bool,
) -> (Option<bool>, i64, Vec<ComponentField>) {
    let mut encoding = crate::theater::parser::v41::context::profile::DecodeProfile::default()
        .component_encoding()
        .unwrap();
    encoding.full_precision = full_precision;
    encoding.baseline_scope = baseline_scope;
    encoding.world_axis_bits = Some([15, 15, 17]);
    encoding.region_axis_bits = [(0, [15, 15, 17])].into();
    let mut r = ComponentReader {
        category_references: true,
        snapshot: case.snapshot,
        cursor: ComponentCursor::guarded(bytes, start as i64),
        fields: Vec::new(),
        position_encoding: context.then_some(&encoding),
        reference_widths: None,
        width_error: None,
        context_error: None,
        live_grammar: None,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
    };
    let status = read_component(
        &mut r,
        &case.component,
        case.version,
        case.archetype,
        None,
        (true, None),
    );
    (status, r.cursor.position, r.fields)
}

#[test]
fn pinned_component_traces_preserve_boundaries_fields_and_versions() {
    #[derive(Deserialize)]
    struct Oracle {
        name: String,
        end: i64,
        complete: bool,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Fixture {
        commit: String,
        cases: Vec<Oracle>,
    }
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(fixture.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    let cases = reference_cases();
    assert_eq!(fixture.cases.len(), cases.len());
    assert_eq!(cases.iter().filter(|c| property::disagrees(c)).count(), 4);
    for (case, expected) in cases.iter().zip(fixture.cases) {
        assert_eq!(case.name, expected.name);
        let bytes = case.bytes();
        let (status, end, fields) = read(case, &bytes, 0);
        if property::disagrees(case) {
            // Keep the original upstream bytes/oracle: its 32-bit enum body
            // consumes 28 bits belonging to the next field in native data.
            assert_eq!(expected.end, 36);
            assert_eq!((status, end), (Some(true), 8));
            assert_eq!(fields[1].width, 4);
            continue;
        }
        if position_finite::superseded(case) {
            // The old reader consumed a two-bit footer after an authored NaN.
            // Native FUN_140492128 refuses after the raw words/parent tail.
            assert!(expected.complete);
            assert_eq!(expected.end as usize, case.bit_len);
            assert_eq!(status, Some(false));
            assert!(end < expected.end);
            assert_eq!(
                fields
                    .iter()
                    .filter(|f| f.name.starts_with("vector_bits["))
                    .count(),
                3
            );
            continue;
        }
        assert_eq!(status, Some(expected.complete), "{}", case.name);
        assert_eq!(end, expected.end, "{}", case.name);
        assert_eq!(expected.complete, case.complete);
        assert_eq!(end as usize, case.bit_len);
        let mut cursor = 0;
        let mut boundaries = BTreeSet::from([0]);
        for field in &fields {
            assert_eq!(field.bit, cursor, "{}: {}", case.name, field.name);
            assert_eq!(
                field.raw,
                RawBits::from_source(&bytes, field.bit, field.width).unwrap()
            );
            cursor += field.width;
            boundaries.insert(cursor);
        }
        assert_eq!(cursor, case.bit_len);
        // Our fields may subdivide aggregate upstream skips. Every primitive
        // read/skip boundary must still occur, including metadata interleaving.
        for (start, width) in expected.trace {
            assert!(boundaries.contains(&start), "{}: start {start}", case.name);
            assert!(
                boundaries.contains(&(start + width)),
                "{}: end {}",
                case.name,
                start + width
            );
        }
        let json = serde_json::to_string(&fields).unwrap();
        assert_eq!(
            fields,
            serde_json::from_str::<Vec<ComponentField>>(&json).unwrap()
        );
    }
}

#[test]
fn predicted_ability_matches_initialized_source_state_across_row_versions() {
    #[derive(Deserialize)]
    struct Trace {
        name: String,
        end: usize,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Export {
        commit: String,
        blobs: std::collections::BTreeMap<String, String>,
        cases: Vec<Trace>,
    }
    let oracle: Export = serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/predicted-ability-source-reference.json"
    ))
    .unwrap();
    assert_eq!(oracle.commit, "413971bcc485be53a2fa8cf229e170539a5146f8");
    assert_eq!(oracle.blobs.len(), 2);
    assert_eq!(
        oracle.blobs["components_biped_spartan.go"],
        "973e443021d6048ce4569e11c33924cb23a68162"
    );
    assert_eq!(
        oracle.blobs["lecteur_position.go"],
        "6033b0366c28cf899883ab35681ba4eb1f5f8b16"
    );
    let cases = predicted_cases();
    assert_eq!(cases.len(), 140);
    assert_eq!(oracle.cases.len(), cases.len());
    for (case, trace) in cases.into_iter().zip(oracle.cases) {
        assert_eq!(case.name, trace.name);
        let bytes = case.bytes();
        let (status, end, fields) = read(&case, &bytes, 0);
        assert_eq!(status, Some(true), "{}", case.name);
        assert_eq!(end as usize, trace.end, "{}", case.name);
        assert_eq!(trace.end, case.bit_len);
        let mut cursor = 0;
        let mut boundaries = BTreeSet::from([0]);
        for field in &fields {
            assert_eq!(field.bit, cursor, "{}: {}", case.name, field.name);
            assert_eq!(
                field.raw,
                RawBits::from_source(&bytes, field.bit, field.width).unwrap()
            );
            cursor += field.width;
            boundaries.insert(cursor);
        }
        assert_eq!(cursor, case.bit_len);
        for (start, width) in trace.trace {
            assert!(boundaries.contains(&start), "{}: {start}", case.name);
            assert!(boundaries.contains(&(start + width)), "{}", case.name);
        }
        assert_eq!(
            fields,
            serde_json::from_str::<Vec<ComponentField>>(&serde_json::to_string(&fields).unwrap())
                .unwrap()
        );
    }
}

#[test]
fn biped_points_retain_explicit_full_precision_and_baseline_overrides() {
    // Independently authored raw snapshot bodies also exercise the explicit
    // process/baseline switches on non-snapshot reads, at every position site.
    for mut case in cases()
        .into_iter()
        .chain(predicted_cases())
        .chain(managed_cases())
        .chain(managed::tlv_refusal_cases())
        .filter(|c| {
            c.snapshot
                && matches!(
                    c.component.as_str(),
                    "biped-spartan-ability-component"
                        | "biped-mobility-action-component"
                        | "biped-posture-physics-component"
                )
        })
    {
        let bytes = case.bytes();
        let expected = read(&case, &bytes, 0);
        assert_eq!(expected.0, Some(true), "{}", case.name);
        case.snapshot = false;
        for (full, baseline) in [(true, false), (false, true), (true, true)] {
            assert_eq!(
                read_with_precision(&case, &bytes, 0, true, full, baseline),
                expected,
                "{}: full {full}, baseline {baseline}",
                case.name
            );
        }
    }
    // Control context has a different gate: snapshot/baseline alone do not
    // expand its two-bit word. Preserve the following one-bit field boundary.
    for snapshot in [false, true] {
        for baseline in [false, true] {
            for full in [false, true] {
                let case = Case {
                    name: "control-context precision".into(),
                    component: "biped-control-context-component".into(),
                    version: 2,
                    archetype: 0,
                    hex: String::new(),
                    bit_len: if full { 5 } else { 3 },
                    complete: true,
                    snapshot,
                };
                let (status, end, fields) =
                    read_with_precision(&case, &[0xf8], 0, true, full, baseline);
                assert_eq!(status, Some(true));
                assert_eq!(end as usize, case.bit_len);
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].width, case.bit_len - 1);
                assert_eq!(fields[0].raw.low_u64(), if full { 15 } else { 3 });
                assert_eq!(fields[1].raw.low_u64(), 1);
            }
        }
    }
}

#[test]
fn component_truncation_at_every_bit_retains_only_complete_fields() {
    for case in cases()
        .into_iter()
        .chain(tacmap_cases())
        .chain(predicted_cases())
        .chain(managed_cases())
        .chain(managed::tlv_refusal_cases())
    {
        let source = case.bytes();
        let (_, _, complete) = read(&case, &source, 0);
        for cut in 0..case.bit_len {
            let shift = (8 - cut % 8) % 8;
            let mut bytes = vec![0u8; (cut + shift) / 8];
            for i in 0..cut {
                bytes[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let (status, end, fields) = read(&case, &bytes, shift);
            assert_eq!(status, None, "{}: cut {cut}", case.name);
            assert!(end <= bytes.len() as i64 * 8);
            assert_eq!(
                fields.len(),
                complete
                    .iter()
                    .take_while(|f| f.bit + f.width <= cut)
                    .count(),
                "{}: cut {cut}",
                case.name
            );
            for (actual, expected) in fields.iter().zip(&complete) {
                assert_eq!(
                    (&actual.name, actual.bit, actual.width, &actual.raw),
                    (
                        &expected.name,
                        expected.bit + shift,
                        expected.width,
                        &expected.raw
                    ),
                    "{} cut {cut}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn legacy_filter_metadata_is_interleaved_with_each_value() {
    let cases = cases();
    let case = cases
        .iter()
        .find(|c| c.name == "managed-navpoint-visible-offscreen-filters-component/v1")
        .unwrap();
    let (status, _, fields) = read(case, &case.bytes(), 0);
    assert_eq!(status, Some(true));
    for i in 0..3 {
        let flag = fields
            .iter()
            .position(|f| f.name == format!("values[{i}]"))
            .unwrap();
        assert_eq!(fields[flag].raw.low_u64(), i as u64 % 2);
        assert_eq!(fields[flag + 1].name, format!("legacy[{i}]"));
        assert_eq!(fields[flag + 1].raw.low_u64(), 10 + i as u64);
    }
}

#[test]
fn component_registry_versions_preserve_the_following_record_boundary() {
    use crate::theater::parser::test_support::component_registry;
    use crate::theater::parser::v41::chunks::replication::source::{
        ComponentReadStatus, EntityViewStop,
    };
    use crate::theater::parser::v41::{
        chunks::replication::replication_stream::frame::records::{
            RecordBinding, RecordReadContext, decode_entity_record,
        },
        context::V41DecodeConfig,
    };
    fn push(wire: &mut Vec<bool>, width: usize, value: u64) {
        wire.extend((0..width).rev().map(|bit| value & (1 << bit) != 0));
    }
    for case in cases()
        .into_iter()
        .chain(orientation_cases())
        .chain(tacmap_cases())
        .chain(predicted_cases())
        .chain(managed_cases())
        .chain(managed::tlv_refusal_cases())
        .filter(|case| !case.snapshot)
    {
        let archetype = if case.archetype == 0 {
            3
        } else {
            case.archetype
        };
        let registry = component_registry(&[(
            archetype as usize,
            vec![
                (case.component.clone(), case.version),
                ("equipment-being-hacked-component".into(), 1),
            ],
        )]);
        let mut config = V41DecodeConfig::default();
        config.profile.movement.world_object.axis_bits = [15, 15, 17];
        let mut wire = vec![true]; // DELTA
        push(&mut wire, 13, 42);
        push(&mut wire, 2, 1); // generation
        wire.push(false); // no baseline
        wire.push(true); // dense component mask
        push(&mut wire, 64, 3);
        let start = wire.len();
        let source = case.bytes();
        wire.extend((0..case.bit_len).map(|i| source[i / 8] & (1 << (7 - i % 8)) != 0));
        push(&mut wire, 8, 0xD3); // following independent byte component
        let mut bytes = vec![0; wire.len().div_ceil(8)];
        for (i, bit) in wire.into_iter().enumerate() {
            bytes[i / 8] |= u8::from(bit) << (7 - i % 8);
        }
        let record = decode_entity_record(
            &bytes,
            0,
            &registry,
            &config.record_layout(),
            Some(RecordBinding {
                id: (1 << 30) | 42,
                archetype,
            }),
            true,
            RecordReadContext {
                context: Some(&config.profile),
                movement: None,
            },
        )
        .unwrap();
        let component = &record.components[0];
        assert_eq!(component.start_bit, start as i64, "{}", case.name);
        assert_eq!(
            component.end_bit,
            (start + case.bit_len) as i64,
            "{}",
            case.name
        );
        if case.complete {
            assert_eq!(record.stop, EntityViewStop::Complete, "{}", case.name);
            assert_eq!(record.components.len(), 2);
            assert_eq!(component.status, ComponentReadStatus::Complete);
            let next = &record.components[1];
            assert_eq!(next.start_bit, component.end_bit);
            assert_eq!(next.fields[0].name, "value");
            assert_eq!(next.fields[0].raw.low_u64(), 0xD3);
            assert_eq!(record.end_bit, (start + case.bit_len + 8) as i64);
        } else {
            assert!(matches!(
                record.stop,
                EntityViewStop::UnsupportedComponent { index: 0, .. }
            ));
            assert_eq!(component.status, ComponentReadStatus::Unsupported);
            assert_eq!(record.components.len(), 1);
        }
    }
}

#[test]
fn snapshot_components_preserve_boundaries_and_refuse_unknown_branches() {
    use crate::theater::parser::test_support::component_registry;
    use crate::theater::parser::v41::chunks::replication::source::{
        ComponentReadStatus, KeyframeStop,
    };
    use crate::theater::parser::v41::{
        chunks::replication::replication_stream::keyframes::record::decode_reference_keyframe_record_contextual,
        context::V41DecodeConfig,
    };
    let mut selected: Vec<_> = cases()
        .into_iter()
        .chain(tacmap_cases())
        .chain(predicted_cases())
        .chain(managed_cases())
        .chain(managed::tlv_refusal_cases())
        .filter(|case| case.snapshot)
        .collect();
    for component in [
        "vehicle-type-state-component",
        "vehicle-type-physics-component",
    ] {
        for hex in ["00", "ff"] {
            selected.push(Case {
                name: "snapshot/unknown-vehicle-flag".into(),
                component: component.into(),
                version: 1,
                archetype: 0,
                hex: hex.into(),
                bit_len: 8,
                complete: false,
                snapshot: true,
            });
        }
    }
    for case in selected {
        let archetype = if case.archetype == 0 {
            3
        } else {
            case.archetype
        };
        let registry = component_registry(&[(
            archetype as usize,
            vec![
                (case.component.clone(), case.version),
                ("equipment-being-hacked-component".into(), 1),
            ],
        )]);
        let config = V41DecodeConfig::default();
        assert!(config.profile.grammar.simulation_complete);
        let mut wire = Vec::new();
        let mut push = |width: usize, value: u64| {
            wire.extend((0..width).rev().map(|bit| value & (1 << bit) != 0))
        };
        push(32, (1 << 30) | 42);
        push(32, u64::from(archetype)); // archetype is the low six bits of the second header word
        push(32, 0);
        push(4, 0);
        push(8, 0);
        push(32, 0); // no creation/default body
        push(32, 1); // component list is present; this is NOT its wire length
        let start = wire.len();
        assert_eq!(start, 172);
        let source = case.bytes();
        wire.extend((0..case.bit_len).map(|i| source[i / 8] & (1 << (7 - i % 8)) != 0));
        wire.extend((0..8).rev().map(|bit| 0xD3 & (1 << bit) != 0));
        let mut bytes = vec![0; wire.len().div_ceil(8)];
        for (i, bit) in wire.into_iter().enumerate() {
            bytes[i / 8] |= u8::from(bit) << (7 - i % 8);
        }
        let record = decode_reference_keyframe_record_contextual(
            &bytes,
            0,
            &registry,
            &config.record_layout(),
            Some(&config.profile),
        )
        .unwrap();
        let component = &record.components[0];
        assert_eq!(component.start_bit, start as i64);
        if case.complete {
            assert_eq!(record.stop, KeyframeStop::Complete, "{}", case.name);
            assert_eq!(component.status, ComponentReadStatus::Complete);
            assert_eq!(
                component.end_bit,
                (start + case.bit_len) as i64,
                "{}",
                case.name
            );
            assert_eq!(record.components[1].fields[0].raw.low_u64(), 0xD3);
            assert_eq!(record.end_bit, (start + case.bit_len + 8) as i64);
            // Snapshot words can contain zeros, negative zero or arbitrary NaN
            // payloads. Compare every field with its own authored source rather
            // than assuming all fixtures put the same NaN in the second axis.
            for field in &component.fields {
                assert_eq!(
                    field.raw,
                    RawBits::from_source(&source, field.bit - start, field.width).unwrap(),
                    "{}: {}",
                    case.name,
                    field.name
                );
            }
        } else if case.component == "object-position-dynamic-precision-component" {
            assert!(matches!(
                record.stop,
                KeyframeStop::UnsupportedComponent { index: 0, .. }
            ));
            assert_eq!(component.status, ComponentReadStatus::Unsupported);
            assert_eq!(component.end_bit, (start + 2) as i64);
            assert_eq!(component.fields.len(), 2);
            assert_eq!(record.components.len(), 1);
        } else if matches!(
            case.component.as_str(),
            "biped-action-component"
                | "device-dispenser-monitors-changed-component"
                | "object-damage-sections-component"
                | "managed-navpoint-docking-filter-component"
                | "managed-navpoint-visibility-filter-component"
                | "object-multiplayer-properties-component"
        ) {
            assert!(matches!(
                record.stop,
                KeyframeStop::UnsupportedComponent { index: 0, .. }
            ));
            assert_eq!(component.status, ComponentReadStatus::Unsupported);
            assert_eq!(component.end_bit, (start + case.bit_len) as i64);
            assert_eq!(record.components.len(), 1);
        } else {
            assert!(
                matches!(record.stop, KeyframeStop::RuntimeContextUnavailable { index: 0, ref field, .. } if field == "vehicle+0x818")
            );
            assert_eq!(component.status, ComponentReadStatus::Unsupported);
            assert_eq!(component.end_bit, start as i64);
            assert!(component.fields.is_empty());
            assert_eq!(record.components.len(), 1);
        }
    }
}

#[test]
fn shared_points_only_require_context_at_the_indexed_box_branch() {
    for case in cases()
        .into_iter()
        .chain(tacmap_cases())
        .chain(predicted_cases())
        .chain(managed_cases())
        .chain(managed::tlv_refusal_cases())
        .filter(|c| {
            matches!(
                c.component.as_str(),
                "low-frequency"
                    | "managed-navpoint-position-offset"
                    | "nav-cutscene-flag-component"
                    | "player-desired-respawn-location-component"
                    | "crew-order-component"
                    | "flock-position-component"
                    | "flock-destination-component"
                    | "simulation-state-component"
                    | "unit-actor-state-component"
                    | "unit-actor-control-component"
                    | "tacmap-waypointstate"
                    | "tacmap-cooptetherarea"
                    | "tacmap-displayasset"
                    | "tacmap-areaofinterest"
                    | "tacmap-poiicon"
                    | "generic-rigid-body-transforms-component"
                    | "biped-spartan-ability-non-predicted-state"
                    | "biped-spartan-ability-component"
                    | "biped-mobility-action-component"
                    | "biped-posture-physics-component"
            )
        })
    {
        let bytes = case.bytes();
        let full = read(&case, &bytes, 0);
        let bounded = read_with_context(&case, &bytes, 0, false);
        if case.snapshot {
            assert_eq!(bounded, full, "{}", case.name);
            continue;
        }
        let missing = full
            .2
            .iter()
            .find(|f| f.name.ends_with(".default_region") && f.raw.low_u64() == 0);
        if let Some(gate) = missing {
            assert_eq!(bounded.0, Some(false), "{}", case.name);
            assert_eq!(bounded.1 as usize, gate.bit + gate.width);
            assert_eq!(
                bounded.2,
                full.2
                    .iter()
                    .take_while(|f| f.bit <= gate.bit)
                    .cloned()
                    .collect::<Vec<_>>()
            );
        } else {
            assert_eq!(bounded, full, "{}", case.name);
        }
    }
}

#[test]
fn attached_object_positions_preserve_local_codes_and_independent_optional_tails() {
    let selected: Vec<_> = cases()
        .into_iter()
        .filter(|c| c.name.starts_with("object-position/attached/"))
        .collect();
    assert_eq!(selected.len(), 160);
    for case in selected {
        let bytes = case.bytes();
        let (status, end, fields) = read_with_context(&case, &bytes, 0, false);
        assert_eq!(status, Some(true), "{}", case.name);
        assert_eq!(end as usize, case.bit_len);
        let field = |name: &str| fields.iter().find(|f| f.name == name);
        for (i, raw) in [0, 16383, 8193].into_iter().enumerate() {
            let f = field(&format!("local_position[{i}]")).unwrap();
            assert_eq!((f.width, f.raw.low_u64()), (14, raw));
        }
        assert!(field("position[0]").is_none());
        assert!(field("high_precision_body").is_none());
        let handle = field("position_handle.present").unwrap().raw.low_u64() != 0;
        assert_eq!(field("position_handle.value").is_some(), handle);
        if handle {
            assert_eq!(field("position_handle.value").unwrap().raw.low_u64(), 8191);
        }
        let region = field("position_region.gate").unwrap().raw.low_u64() != 0;
        assert_eq!(field("position_region").is_some(), region);
        assert_eq!(
            case.bit_len,
            47 + usize::from(handle) * 15 + usize::from(region) * 11
        );
        assert_eq!(field("finite").unwrap().width, 2);
    }
}

#[test]
fn dynamic_orientation_matches_instruction_addressed_source_when_upstream_disagrees() {
    #[derive(Deserialize)]
    struct Oracle {
        name: String,
        end: usize,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Export {
        commit: String,
        blob: String,
        cases: Vec<Oracle>,
    }
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/orientation-source.json"
    ))
    .unwrap();
    let oracle: Export = serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/orientation-source-reference.json"
    ))
    .unwrap();
    assert_eq!(oracle.commit, "413971bcc485be53a2fa8cf229e170539a5146f8");
    assert_eq!(oracle.blob, "a32dc647e20c3d32146358145ad93b979adbe83e");
    assert_eq!(cases.len(), 32);
    assert_eq!(oracle.cases.len(), 32);
    for (case, oracle) in cases.into_iter().zip(oracle.cases) {
        assert_eq!(case.name, oracle.name);
        let source = case.bytes();
        let (status, end, fields) = read(&case, &source, 0);
        assert_eq!(status, Some(true), "{}", case.name);
        assert_eq!(end as usize, oracle.end);
        assert_eq!(oracle.end, case.bit_len);
        let boundaries: BTreeSet<_> = std::iter::once(0)
            .chain(fields.iter().map(|f| f.bit + f.width))
            .collect();
        for (at, width) in oracle.trace {
            assert!(boundaries.contains(&at));
            assert!(boundaries.contains(&(at + width)));
        }
        if case.name.starts_with("raw-overridden/") {
            assert!(fields.iter().any(|f| f.name == "roll" && f.width == 30));
            assert!(
                !fields
                    .iter()
                    .any(|f| f.name.starts_with("orientation_vector_bits"))
            );
        }
        for cut in 0..case.bit_len {
            let shift = (8 - cut % 8) % 8;
            let mut bytes = vec![0; (cut + shift) / 8];
            for i in 0..cut {
                bytes[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let (status, end, prefix) = read(&case, &bytes, shift);
            assert_eq!(status, None, "{} cut {cut}", case.name);
            assert!(end <= bytes.len() as i64 * 8);
            assert_eq!(
                prefix.len(),
                fields.iter().take_while(|f| f.bit + f.width <= cut).count()
            );
        }
    }
}

#[test]
fn high_frequency_refuses_archetypes_without_a_known_descriptor_table() {
    for archetype in [0, 1, 2, 5, 35, 40, u32::MAX] {
        let case = Case {
            name: "unknown frequency table".into(),
            component: "high-frequency".into(),
            version: 4,
            archetype,
            hex: "ffffffff".into(),
            bit_len: 32,
            complete: false,
            snapshot: false,
        };
        let (status, end, fields) = read(&case, &case.bytes(), 0);
        assert_eq!(status, Some(false));
        assert_eq!(end, 0);
        assert!(fields.is_empty());
    }
}

#[test]
fn source_only_tacmap_aliases_match_snapshot_readers_and_traced_point_subgrammar() {
    #[derive(Deserialize)]
    struct Layout {
        #[serde(flatten)]
        case: Case,
        point_start: usize,
        point_len: usize,
        point_case: String,
    }
    #[derive(Deserialize)]
    struct Trace {
        name: String,
        end: usize,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Export {
        commit: String,
        #[serde(default)]
        blob: String,
        cases: Vec<Trace>,
    }
    let layouts: Vec<Layout> = serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/tacmap-source.json"
    ))
    .unwrap();
    let snapshots: Export = serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/tacmap-source-reference.json"
    ))
    .unwrap();
    let upstream: Export = serde_json::from_str(include_str!(
        "../../../../../fixtures/components-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(snapshots.commit, "413971bcc485be53a2fa8cf229e170539a5146f8");
    assert_eq!(snapshots.blob, "699712aa7591344f6b60831506ce1e74e8c14f78");
    assert_eq!(upstream.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    assert_eq!(layouts.len(), 81);
    assert_eq!(snapshots.cases.len(), 27);
    let point_cases = cases();
    for layout in layouts {
        let case = layout.case;
        let source = case.bytes();
        let (status, end, fields) = read(&case, &source, 0);
        assert_eq!(status, Some(true), "{}", case.name);
        assert_eq!(end as usize, case.bit_len, "{}", case.name);
        let mut at = 0;
        let mut boundaries = BTreeSet::from([0]);
        for field in &fields {
            assert_eq!(field.bit, at, "{}: {}", case.name, field.name);
            assert_eq!(
                field.raw,
                RawBits::from_source(&source, at, field.width).unwrap()
            );
            at += field.width;
            boundaries.insert(at);
        }
        assert_eq!(at, case.bit_len);
        let serialized = serde_json::to_string(&fields).unwrap();
        assert_eq!(
            fields,
            serde_json::from_str::<Vec<ComponentField>>(&serialized).unwrap()
        );

        // Compare the actual point bytes and primitive boundaries against the
        // original reference reader tether reader. The enclosing delta grammar is
        // authored from the game call sites, not claimed as an upstream trace.
        let point = point_cases
            .iter()
            .find(|c| c.name == layout.point_case)
            .unwrap();
        assert_eq!(point.bit_len, layout.point_len + 24);
        assert_eq!(
            RawBits::from_source(&source, layout.point_start, layout.point_len),
            RawBits::from_source(&point.bytes(), 0, layout.point_len)
        );
        let trace = upstream
            .cases
            .iter()
            .find(|t| t.name == point.name)
            .unwrap();
        assert_eq!(trace.end, point.bit_len);
        for &(start, width) in trace
            .trace
            .iter()
            .filter(|&&(s, w)| s + w <= layout.point_len)
        {
            assert!(
                boundaries.contains(&(layout.point_start + start)),
                "{}",
                case.name
            );
            assert!(
                boundaries.contains(&(layout.point_start + start + width)),
                "{}",
                case.name
            );
        }
        if case.snapshot {
            let trace = snapshots
                .cases
                .iter()
                .find(|t| t.name == case.name)
                .unwrap();
            assert_eq!(trace.end, case.bit_len);
            for &(start, width) in &trace.trace {
                assert!(boundaries.contains(&start), "{}", case.name);
                assert!(boundaries.contains(&(start + width)), "{}", case.name);
            }
        }
    }
}
