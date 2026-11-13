//! Raw scope closure from reference analysis components_position_i0.go (413971b).
//! Stream words are byte reversed before the native exponent predicate.
use super::*;

#[test]
fn captured_prediction_prefixes_follow_native_reader_without_upstream_raw_relabeling() {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../../../../fixtures/position-prefix-f050371-v41/cases.json"
    ))
    .unwrap();
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../fixtures/position-prefix-f050371-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(
        reference["commit"],
        "f050371a6cda83b9887eeb83a1884336fce28e53"
    );
    assert_eq!(cases.len(), 2);
    for (case, upstream) in cases.iter().zip(reference["cases"].as_array().unwrap()) {
        assert_eq!(case["name"], upstream["name"]);
        assert_eq!(case["upstream_end"], upstream["end"]);
        assert_eq!(case["upstream_complete"], upstream["complete"]);
        let bytes: Vec<u8> = case["hex"]
            .as_str()
            .unwrap()
            .as_bytes()
            .chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        let encoding = crate::theater::parser::v41::context::profile::DecodeProfile::default()
            .component_encoding()
            .unwrap();
        let mut r = ComponentReader {
            category_references: true,
            snapshot: false,
            cursor: ComponentCursor::guarded(&bytes, 0),
            fields: Vec::new(),
            position_encoding: Some(&encoding),
            reference_widths: None,
            width_error: None,
            context_error: None,
            live_grammar: None,
            movement_slot: None,
            references: Vec::new(),
            diagnostics: Default::default(),
        };
        assert_eq!(
            read_component(
                &mut r,
                "object-position-dynamic-precision-component",
                1,
                35,
                None,
                (true, None),
            ),
            Some(true),
            "{}",
            case["name"]
        );
        assert_eq!(
            r.cursor.position as u64,
            case["local_end"].as_u64().unwrap()
        );
        assert_eq!(r.context_error, None);
        assert_eq!(r.fields[0].raw.low_u64(), 1);
        assert_eq!(
            r.fields[1].raw.low_u64(),
            case["name"].as_str().unwrap().contains("chunk-140") as u64
        );
        assert_eq!(r.fields[2].raw.low_u64(), 0);
        for field in &r.fields {
            assert_eq!(
                field.raw,
                RawBits::from_source(&bytes, field.bit, field.width).unwrap()
            );
        }
        assert_eq!(r.fields.last().unwrap().name, "baseline_vector_bits[2]");
        assert_ne!(case["local_end"], upstream["end"]);
    }
}

pub(super) fn superseded(case: &Case) -> bool {
    case.name.starts_with("dynamic-position/snapshot/0/")
        || case.name.starts_with("object-position/snapshot/")
}

fn bits(width: usize, value: u64) -> String {
    format!("{value:0width$b}")
}

fn source(wire: &str) -> Vec<u8> {
    let padded = format!("{wire}{}", "0".repeat((8 - wire.len() % 8) % 8));
    padded
        .as_bytes()
        .chunks(8)
        .map(|octet| u8::from_str_radix(std::str::from_utf8(octet).unwrap(), 2).unwrap())
        .collect()
}

#[test]
fn native_raw_finiteness_closes_only_finite_positions_after_the_parent_tail() {
    // Signed zero, a subnormal and ordinary finite numbers have no all-ones
    // exponent. Infinity and both kinds of NaN are rejected before R(2).
    let samples: [[u32; 3]; 6] = [
        [0, 0x8000_0000, 0x0000_0001],
        [0x3f80_0000, 0xbf80_0000, 0x7f7f_ffff],
        [0x7f80_0000, 0, 0],
        [0, 0xff80_0000, 0],
        [0, 0, 0x7fc1_2345],
        [0x7f80_0001, 0, 0],
    ];
    for (sample, floats) in samples.into_iter().enumerate() {
        let finite = sample < 2;
        let words = floats.map(|word| word.swap_bytes());
        assert_eq!(super::super::position::raw_words_finite(words), finite);
        for kind in ["dynamic-snapshot", "dynamic-writer", "object-snapshot"] {
            for attached in [false] {
                let mut wire = String::new();
                if kind == "object-snapshot" {
                    wire.push(if attached { '1' } else { '0' });
                } else {
                    wire.push_str("000"); // no prediction/delta; parent gate clear
                }
                for word in words {
                    wire.push_str(&bits(32, word as u64));
                }
                let payload_end = wire.len();
                if finite {
                    wire.push_str("10");
                }
                let end = wire.len();
                wire.push_str("11010110"); // distinguish a successor from R(2)
                let bytes = source(&wire);
                let mut encoding =
                    crate::theater::parser::v41::context::profile::DecodeProfile::default()
                        .component_encoding()
                        .unwrap();
                encoding.writer_absolute = kind == "dynamic-writer";
                encoding.baseline_scope = kind == "dynamic-writer";
                let mut reader = ComponentReader {
                    category_references: true,
                    snapshot: kind != "dynamic-writer",
                    cursor: ComponentCursor::guarded(&bytes, 0),
                    fields: Vec::new(),
                    position_encoding: Some(&encoding),
                    reference_widths: None,
                    width_error: None,
                    context_error: None,
                    live_grammar: None,
                    movement_slot: None,
                    references: Vec::new(),
                    diagnostics: Default::default(),
                };
                let name = if kind == "object-snapshot" {
                    "object-position-component"
                } else {
                    "object-position-dynamic-precision-component"
                };
                let status = read_component(&mut reader, name, 1, 3, None, (true, None));
                assert_eq!(status, Some(finite), "{kind} sample {sample}");
                assert_eq!(
                    reader.cursor.position as usize, end,
                    "{kind} sample {sample}"
                );
                assert_eq!(
                    reader.context_error,
                    (!finite).then_some("nonfinite raw position")
                );
                assert_eq!(
                    reader.fields.last().unwrap().name,
                    if finite { "finite" } else { "vector_bits[2]" }
                );
                for (i, word) in words.into_iter().enumerate() {
                    let field = reader
                        .fields
                        .iter()
                        .find(|f| f.name == format!("vector_bits[{i}]"))
                        .unwrap();
                    assert_eq!((field.width, field.raw.low_u64()), (32, word as u64));
                    assert_eq!(
                        field.raw,
                        RawBits::from_source(&bytes, field.bit, field.width).unwrap()
                    );
                }
                assert_eq!(
                    reader.cursor.position as usize,
                    payload_end + if finite { 2 } else { 0 }
                );
                assert_eq!(
                    reader.cursor.raw_bits(end as i64, 8).unwrap().low_u64(),
                    0xd6
                );
            }
        }
    }
}

#[test]
fn snapshot_parent_gate_keeps_existing_finite_boundary_pending_differential_oracle() {
    for kind in ["dynamic", "object"] {
        let mut wire = if kind == "dynamic" {
            "001".to_owned()
        } else {
            "1".to_owned()
        };
        for word in [0_u32, 0, 0] {
            wire.push_str(&bits(32, word as u64));
        }
        if kind == "object" {
            wire.push_str("00"); // absent handle and region gates
        }
        wire.push_str("00");
        let end = wire.len();
        let bytes = source(&wire);
        let mut encoding = crate::theater::parser::v41::context::profile::DecodeProfile::default()
            .component_encoding()
            .unwrap();
        encoding.writer_absolute = true;
        encoding.baseline_scope = true;
        let mut r = ComponentReader {
            category_references: true,
            snapshot: true,
            cursor: ComponentCursor::guarded(&bytes, 0),
            fields: Vec::new(),
            position_encoding: Some(&encoding),
            reference_widths: None,
            width_error: None,
            context_error: None,
            live_grammar: None,
            movement_slot: None,
            references: Vec::new(),
            diagnostics: Default::default(),
        };
        let name = if kind == "dynamic" {
            "object-position-dynamic-precision-component"
        } else {
            "object-position-component"
        };
        // The aliases have distinct pinned parent-gate layouts: object-position
        // reads two absent tail gates; the dynamic snapshot reads no tail.
        assert_eq!(
            read_component(&mut r, name, 1, 3, None, (true, None)),
            Some(true)
        );
        assert_eq!(r.cursor.position as usize, end);
        assert_eq!(r.context_error, None);
    }
}

#[test]
fn full_scope_handle_tail_requires_engine_width_and_preserves_its_fields() {
    // reference analysis FUN_14076e3e4: h, raw vec3, a category-0 optional handle,
    // optional region, then the finite footer. The next byte is a successor.
    for (engine_type, possible) in [(None, true), (Some(1), true), (Some(2), false)] {
        let mut config = crate::theater::parser::v41::context::V41DecodeConfig::default();
        config.event_variant.engine_type = engine_type;
        assert_eq!(
            config.record_layout().position.unwrap().engine_one_possible,
            possible
        );
    }
    for handle_present in [false, true] {
        for known_other_engine in [false, true] {
            let mut wire = "001".to_owned(); // prediction=0, delta=0, h=1
            for _ in 0..3 {
                wire.push_str(&bits(32, 0));
            }
            wire.push(if handle_present { '1' } else { '0' });
            if handle_present {
                wire.push_str(&bits(13, 0x1a5));
                wire.push_str("10");
            }
            wire.push('1'); // region present
            wire.push_str(&bits(11, 0x456));
            wire.push_str("10"); // finite footer
            let end = wire.len();
            wire.push_str("11010110");
            let bytes = source(&wire);
            let mut encoding =
                crate::theater::parser::v41::context::profile::DecodeProfile::default()
                    .component_encoding()
                    .unwrap();
            encoding.writer_absolute = true;
            encoding.baseline_scope = true;
            encoding.engine_one_possible = !known_other_engine;
            let mut r = ComponentReader {
                category_references: true,
                snapshot: false,
                cursor: ComponentCursor::guarded(&bytes, 0),
                fields: Vec::new(),
                position_encoding: Some(&encoding),
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
                "object-position-dynamic-precision-component",
                1,
                3,
                None,
                (true, None),
            );
            if known_other_engine {
                assert_eq!(status, Some(true));
                assert_eq!(r.cursor.position as usize, end);
                assert_eq!(r.context_error, None);
                assert_eq!(r.runtime_context_unavailable(), None);
                assert_eq!(r.cursor.raw_bits(end as i64, 8).unwrap().low_u64(), 0xd6);
                let names: Vec<_> = r.fields.iter().map(|f| f.name.as_str()).collect();
                assert!(names.contains(&"scope.handle.present"));
                assert_eq!(names.contains(&"scope.handle.value"), handle_present);
                assert!(names.contains(&"scope.region"));
                assert_eq!(r.fields.last().unwrap().name, "finite");
            } else {
                assert_eq!(status, Some(false));
                assert_eq!(r.cursor.position, 99);
                assert_eq!(r.context_error, Some("engine-one handle width"));
                assert_eq!(
                    r.runtime_context_unavailable(),
                    Some("engine-one handle width")
                );
                assert!(r.fields.iter().all(|f| !f.name.starts_with("scope.")));
            }
        }
    }

    let mut wire = "001".to_owned();
    for _ in 0..3 {
        wire.push_str(&bits(32, 0));
    }
    wire.push('1'); // handle present, but its global reference range is absent
    let bytes = source(&wire);
    let mut encoding = crate::theater::parser::v41::context::profile::DecodeProfile::default()
        .component_encoding()
        .unwrap();
    encoding.writer_absolute = true;
    encoding.baseline_scope = true;
    encoding.engine_one_possible = false;
    let mut r = ComponentReader {
        category_references: false,
        snapshot: false,
        cursor: ComponentCursor::guarded(&bytes, 0),
        fields: Vec::new(),
        position_encoding: Some(&encoding),
        reference_widths: None,
        width_error: None,
        context_error: None,
        live_grammar: None,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
    };
    assert_eq!(
        read_component(
            &mut r,
            "object-position-dynamic-precision-component",
            1,
            3,
            None,
            (true, None),
        ),
        Some(false)
    );
    assert_eq!(r.cursor.position, 100);
    assert_eq!(r.context_error, Some("global reference range"));
}

#[test]
fn nonfinite_raw_position_cannot_be_repaired_by_a_component_stub() {
    use crate::theater::parser::v41::chunks::replication::components::widths::ComponentWidthOverrides;
    let mut wire = "000".to_owned();
    for word in [0_u32, 0x4523_c17f, 0] {
        wire.push_str(&bits(32, word as u64));
    }
    wire.push_str("10101010");
    let bytes = source(&wire);
    let mut r = ComponentReader {
        category_references: true,
        snapshot: true,
        cursor: ComponentCursor::guarded(&bytes, 0),
        fields: Vec::new(),
        position_encoding: None,
        reference_widths: None,
        width_error: None,
        context_error: None,
        live_grammar: None,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
    };
    let mut overrides = ComponentWidthOverrides::default();
    overrides
        .stubs
        .insert("object-position-dynamic-precision-component".into(), 8);
    assert_eq!(
        read_component(
            &mut r,
            "object-position-dynamic-precision-component",
            1,
            3,
            Some(&overrides),
            (true, None),
        ),
        Some(false)
    );
    assert_eq!(r.cursor.position, 99);
    assert_eq!(r.context_error, Some("nonfinite raw position"));
    assert!(r.fields.iter().all(|f| f.name != "component.stub"));
    assert_eq!(r.cursor.raw_bits(99, 8).unwrap().low_u64(), 0xaa);
}
