//! Original source traces for manager layouts where reference reader differs.
use super::*;

fn tlv_case(name: &str, body: &[u8], complete: bool) -> Case {
    // Absent=0, variant=0, then unaligned mode-2 bytes. Variant zero still
    // carries this submessage in the instruction-addressed source reader.
    let bit_len = 6 + body.len() * 8;
    let mut bytes = vec![0; bit_len.div_ceil(8)];
    for (i, &value) in body.iter().enumerate() {
        bytes[i] |= value >> 6;
        bytes[i + 1] |= value << 2;
    }
    Case {
        name: format!("tlv/{name}"),
        component: "object-multiplayer-properties-component".into(),
        version: 2,
        archetype: 0,
        hex: bytes.iter().map(|b| format!("{b:02x}")).collect(),
        bit_len,
        complete,
        snapshot: false,
    }
}

pub(super) fn tlv_refusal_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for kind in [9, 10, 18] {
        cases.push(tlv_case(
            &format!("oversized-body-{kind}"),
            &[1, kind, 0x81, 0x80, 0x40],
            false,
        ));
    }
    for kind in [11, 12] {
        cases.push(tlv_case(
            &format!("oversized-list-{kind}"),
            &[1, kind, 2, 0x81, 0x20],
            false,
        ));
    }
    cases.push(tlv_case("oversized-map", &[1, 13, 2, 4, 0x81, 0x20], false));
    cases.push(tlv_case("header-varint-overflow", &[0x80; 10], false));
    let mut integer = vec![1, 4];
    integer.extend([0xff; 9]);
    integer.push(2);
    cases.push(tlv_case("integer-varint-overflow", &integer, false));
    let mut list = vec![1, 11];
    list.extend([0x4b; 8]); // one nested list at each level
    list.push(0x42); // one byte at depth 8: refuse before that byte
    cases.push(tlv_case("nested-list-limit", &list, false));
    let mut map = vec![1, 13];
    for _ in 0..8 {
        map.extend([19, 13, 1]); // zero-width key, nested map value
    }
    map.extend([2, 2, 1]);
    cases.push(tlv_case("nested-map-limit", &map, false));
    // Exercise both entity-record and keyframe refusal with the same bytes.
    let snapshots = cases
        .iter()
        .map(|case| Case {
            name: format!("{}/snapshot", case.name),
            snapshot: true,
            component: case.component.clone(),
            version: case.version,
            archetype: case.archetype,
            hex: case.hex.clone(),
            bit_len: case.bit_len,
            complete: case.complete,
        })
        .collect::<Vec<_>>();
    cases.extend(snapshots);
    cases
}

#[test]
fn tlv_limits_refuse_at_the_recorded_prefix_without_silent_clamping_or_stubs() {
    use super::super::widths::ComponentWidthOverrides;
    for case in tlv_refusal_cases() {
        let source = case.bytes();
        let (status, end, fields) = read(&case, &source, 0);
        assert_eq!(status, Some(false), "{}", case.name);
        assert_eq!(end as usize, case.bit_len, "{}", case.name);
        assert_eq!(fields.iter().map(|f| f.width).sum::<usize>(), case.bit_len);
        let mut padded = source.clone();
        padded.extend([0; 64]); // available bytes must not turn a rejected length into success
        let mut r = ComponentReader {
            category_references: true,
            snapshot: case.snapshot,
            cursor: ComponentCursor::guarded(&padded, 0),
            fields: vec![],
            position_encoding: None,
            reference_widths: None,
            width_error: None,
            context_error: None,
            live_grammar: None,
            movement_slot: None,
            references: vec![],
            diagnostics: Default::default(),
        };
        let widths = ComponentWidthOverrides {
            stubs: [(case.component.clone(), 8)].into(),
            ..Default::default()
        };
        assert_eq!(
            read_component(
                &mut r,
                &case.component,
                2,
                0,
                Some(&widths),
                (true, Some((&case.component, 8)))
            ),
            Some(false),
            "{}",
            case.name
        );
        assert_eq!(r.cursor.position, end);
        assert_eq!(r.fields, fields);
        assert!(r.context_error.unwrap().starts_with("TLV "));
    }
    // A resource-capped message cannot silently end without a terminator.
    let mut unterminated = vec![1];
    unterminated.extend([19; 4096]); // the source gives this wire type zero body bits
    let case = tlv_case("field-limit", &unterminated, false);
    let (status, end, _) = read(&case, &case.bytes(), 0);
    assert_eq!(status, Some(false));
    assert_eq!(end as usize, case.bit_len);

    let mut legal_integer = vec![1, 4];
    legal_integer.extend([0xff; 9]);
    legal_integer.extend([1, 0]); // u64::MAX and a terminator
    let mut empty_nested = vec![1, 11];
    empty_nested.extend([0x4b; 8]);
    empty_nested.extend([0x22, 0]); // zero-count byte list at the depth limit, then end
    for (name, body) in [
        ("maximum-integer", legal_integer),
        ("empty-nested-list", empty_nested),
        ("maximum-list-count", vec![1, 11, 19, 0x80, 0x20, 0]),
        ("maximum-map-count", vec![1, 13, 19, 19, 0x80, 0x20, 0]),
    ] {
        let case = tlv_case(name, &body, true);
        let (status, end, _) = read(&case, &case.bytes(), 0);
        assert_eq!(status, Some(true), "{name}");
        assert_eq!(end as usize, case.bit_len, "{name}");
    }
}

#[test]
fn managers_match_original_source_primitives_and_timer_field_order() {
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
    #[derive(Deserialize)]
    struct Values {
        name: String,
        #[serde(default)]
        timers: Vec<Vec<u64>>,
    }
    let oracle: Export = serde_json::from_str(include_str!(
        "../../../../../../fixtures/components-v41/managed-source-reference.json"
    ))
    .unwrap();
    assert_eq!(oracle.commit, "413971bcc485be53a2fa8cf229e170539a5146f8");
    let expected_blobs = [
        (
            "components_moteur_de_partie.go",
            "4b6f127a8941c9f16097c7d512c2ec08b49e16c5",
        ),
        (
            "lecteur_minuteur.go",
            "dc382adaa1bbfe38e4f8004f094aaf04da636bd6",
        ),
        (
            "dispatch_player.go",
            "3e58e6eb74179f6f69c96e20592b1dfb2c98d2dc",
        ),
        (
            "components_batch7.go",
            "73d02925cc5051252605236dfb3290af79c37a84",
        ),
        ("tlv_mode2.go", "3709d03b18693a463257d899d9b34804ab418b64"),
    ];
    assert_eq!(oracle.blobs.len(), expected_blobs.len());
    for (name, blob) in expected_blobs {
        assert_eq!(oracle.blobs[name], blob);
    }
    let values: Vec<Values> = serde_json::from_str(include_str!(
        "../../../../../../fixtures/components-v41/managed-source.json"
    ))
    .unwrap();
    let cases = managed_cases();
    assert_eq!(oracle.cases.len(), cases.len());
    assert_eq!(values.len(), cases.len());
    for ((case, trace), values) in cases.into_iter().zip(oracle.cases).zip(values) {
        assert_eq!(case.name, trace.name);
        assert_eq!(case.name, values.name);
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
            assert!(
                boundaries.contains(&start),
                "{}: primitive start {start}",
                case.name
            );
            assert!(
                boundaries.contains(&(start + width)),
                "{}: primitive end {}",
                case.name,
                start + width
            );
        }
        for timer in values.timers {
            let slot = timer[0];
            let field = |suffix: &str| {
                fields
                    .iter()
                    .find(|f| f.name == format!("timer[{slot}].{suffix}"))
                    .unwrap()
                    .raw
                    .low_u64()
            };
            assert_eq!(field("tag"), timer[1], "{} slot {slot}", case.name);
            if timer[1] != 0 {
                assert_eq!(field("value[0]"), timer[2]);
                assert_eq!(field("value[1]"), timer[3]);
                assert_eq!(field("tail"), timer[4]);
                if timer[1] == 1 {
                    assert_eq!(field("value[2]"), timer[5]);
                }
            }
        }
        assert_eq!(
            fields,
            serde_json::from_str::<Vec<ComponentField>>(&serde_json::to_string(&fields).unwrap())
                .unwrap()
        );
    }
}
