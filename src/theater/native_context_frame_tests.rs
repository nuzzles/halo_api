use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_context_frame_records_world_and_live_widths() {
    check_context_frames(include_bytes!("fixtures/context-frame-v41.json.zlib"));
}
#[test]
fn native_context_frame_unused_position_widths_are_lazy() {
    check_context_frames(include_bytes!(
        "fixtures/context-frame-lazy-widths-v41.json.zlib"
    ));
}
fn check_context_frames(fixture: &[u8]) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 128);
    let registry = FilmRegistry {
        archetypes: (0..4)
            .map(|index| FilmArchetype {
                index,
                components: if index == 3 {
                    vec![
                        "biped-emp-timer-component".into(),
                        "weapon-state-rounds-inventory".into(),
                        "unit-crouch-component".into(),
                    ]
                } else {
                    vec![]
                },
                levels: if index == 3 { vec![0; 3] } else { vec![] },
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    for case in cases {
        let c = case["case"].as_u64().unwrap() as usize;
        let hex = case["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            extra_fields: !c.is_multiple_of(2),
            packet_preamble_bits: 2 + (c % 4) as i64,
            ..Default::default()
        };
        if case["lazy"] == true {
            let width = case["position_width"].as_u64().unwrap();
            let movement = &mut cfg.context.profile.movement;
            movement.world_object.index_bits = width;
            movement.world_object.axis_bits = [width; 3];
            movement.traversal.index_bits = width;
            movement.traversal.axis_bits = [width; 3];
            movement.delta_axis_width = width;
        }
        cfg.context.profile.grammar.generation_strict = !c.is_multiple_of(3);
        cfg.context.profile.grammar.corruption_check = c.is_multiple_of(4);
        let widths =
            NativeSharedWidths::from_map([("weapon-state-rounds-inventory".into(), 47)].into());
        cfg.context.profile.grammar.calibrated_widths = Some(widths.clone());
        let observer = NativeFilmObserver::default();
        cfg.context.observer = Some(observer.clone());
        let events = Arc::new(Mutex::new(Vec::<Value>::new()));
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let events = events.clone();
                let widths = widths.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::EmpTimer {
                        quantum,
                    }) = p
                    else {
                        panic!("wrong hook")
                    };
                    events.lock().unwrap().push(json!(["emp", quantum]));
                    widths.insert(
                        "weapon-state-rounds-inventory".into(),
                        i64::from(quantum % 11),
                    );
                })
            }),
        );
        observer.set_hook(
            NativeHookKind::WeaponRounds,
            Some(Arc::new(|_| {
                panic!("calibrated component emitted callback")
            })),
        );
        observer.set_hook(
            NativeHookKind::MovementState,
            Some({
                let events = events.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::MovementState {
                        component,
                        slot,
                        values,
                    }) = p
                    else {
                        panic!("wrong hook")
                    };
                    events
                        .lock()
                        .unwrap()
                        .push(json!(["movement", component, slot, values]));
                })
            }),
        );
        let old = NativeFilmObserver::default();
        old.set_hook(
            NativeHookKind::EmpTimer,
            Some(Arc::new(|_| {
                panic!("old reader observer survived frame installation")
            })),
        );
        let mut reader = NativeFilmReader::with_context(
            &data,
            NativeReaderContext {
                observer: Some(old),
                ..Default::default()
            },
        );
        reader
            .skip(case["start"].as_u64().unwrap() as usize)
            .unwrap();
        let mut world = FilmWorld::default();
        world.bind_full(8, 3);
        world.set_position(8, [1.0, 2.0, 3.0]);
        world.current_view = 2;
        let view = reader
            .read_frame_records(&registry, &mut world, &cfg)
            .unwrap();
        assert_eq!(reader.profile(), cfg.context.profile);
        assert_eq!(json!(reader.bit_position()), case["end"], "case {c}");
        assert_eq!(
            json!(view.stop == EntityViewStop::Complete),
            case["complete"],
            "case {c} {:?}",
            view.stop
        );
        assert_eq!(json!(*events.lock().unwrap()), case["events"], "case {c}");
        assert!(reader.context().observer.unwrap().same_instance(&observer));
        assert_eq!(
            json!(widths.get("weapon-state-rounds-inventory").unwrap()),
            case["width"]
        );
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .collect();
        let expected = case["records"].as_array().unwrap();
        assert_eq!(records.len(), expected.len(), "case {c}");
        for (r, e) in records.iter().zip(expected) {
            assert_eq!(json!(r.header.id), e["ID"]);
            assert_eq!(
                json!(r.header.start_bit),
                json!(e["HeaderBit"].as_u64().unwrap() + if cfg.extra_fields { 32 } else { 0 })
            );
            assert_eq!(r.stop == EntityViewStop::Complete, e["DesyncAt"] == -1);
            if r.header.kind != RecordKind::Delete {
                assert_eq!(json!(r.end_bit), e["Trace"]["EndBit"]);
                assert_eq!(json!(r.mask.unwrap_or(0)), e["Trace"]["Mask"]);
            }
            let comps = e["Trace"]["Comps"].as_array().cloned().unwrap_or_default();
            assert_eq!(r.attempts.len(), comps.len());
            for (a, b) in r.attempts.iter().zip(comps) {
                assert_eq!(json!(a.span.start_bit), b["StartBit"]);
                assert_eq!(json!(a.span.index), b["Index"]);
                assert_eq!(json!(a.span.name), b["Name"]);
                assert_eq!(json!(a.status), b["Ported"]);
            }
        }
        let slots:BTreeMap<_,_>=world.slots.iter().map(|(slot,s)|(slot.to_string(),json!({"TypeIndex":s.archetype,"FullID":s.full_id,"Soft":s.soft,"GenAny":s.generation_any,"Pos":s.position.unwrap_or([0.0;3]),"PosValid":s.position.is_some(),"Vue":s.view.unwrap_or(-1)}))).collect();
        let mut expected_slots = case["slots"].clone();
        for slot in expected_slots.as_object_mut().unwrap().values_mut() {
            let position: [f32; 3] = serde_json::from_value(slot["Pos"].clone()).unwrap();
            slot["Pos"] = json!(position);
        }
        assert_eq!(json!(slots), expected_slots, "world case {c}");
        assert_eq!(
            json!(reader.capture_slot()),
            case["slot"],
            "capture slot case {c}"
        );
        assert_eq!(
            serde_json::from_value::<DecodedEntityView>(serde_json::to_value(&view).unwrap())
                .unwrap(),
            view
        );
    }
}

#[test]
fn native_world_position_widths_match_direct_and_frame_reads() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/world-position-widths-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 144);
    let registry = FilmRegistry {
        archetypes: (0..4)
            .map(|index| FilmArchetype {
                index,
                components: if index == 3 {
                    vec!["object-position-component".into()]
                } else {
                    vec![]
                },
                levels: if index == 3 { vec![0] } else { vec![] },
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    for (i, case) in cases.iter().enumerate() {
        let decode_hex = |key: &str| {
            let hex = case[key].as_str().unwrap();
            (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            ..Default::default()
        };
        cfg.context.profile.movement.world_object.index_bits = case["width"].as_u64().unwrap();
        cfg.context.profile.movement.world_object.axis_bits =
            serde_json::from_value(case["axes"].clone()).unwrap();
        let data = decode_hex("hex");
        let mut reader = cfg.reader(&data);
        reader.set_bit_position(case["start"].as_u64().unwrap() as usize);
        let (status, component) = reader
            .read_component("object-position-component", 0, 3)
            .unwrap();
        assert_eq!(json!(status), case["ok"], "direct status {i}");
        assert_eq!(json!(reader.bit_position()), case["end"], "direct end {i}");
        assert_eq!(json!(reader.read_bits(7)), case["tail"], "direct tail {i}");
        let values: Vec<_> = (0..3)
            .filter_map(|axis| {
                component
                    .fields
                    .iter()
                    .find(|f| f.name == format!("position[{axis}]"))
                    .map(|f| f.raw)
            })
            .collect();
        assert_eq!(json!(values), case["values"], "axis values {i}");
        let frame = decode_hex("frame_hex");
        let mut reader = NativeFilmReader::new(&frame);
        let mut world = FilmWorld::default();
        world.bind_full(1, 3);
        let view = reader
            .read_frame_records(&registry, &mut world, &cfg)
            .unwrap();
        assert_eq!(
            json!(reader.bit_position()),
            case["frame_end"],
            "frame end {i}"
        );
        assert_eq!(
            json!(view.stop == EntityViewStop::Complete),
            case["complete"],
            "frame status {i}"
        );
        assert_eq!(view.records.len(), 2, "delta and End {i}");
        assert_eq!(
            json!(view.records[0].end_bit),
            case["records"][0]["Trace"]["EndBit"]
        );
        let span = &view.records[0].components[0];
        for field in &component.fields {
            let frame_field = view.records[0]
                .fields
                .iter()
                .find(|f| f.name == field.name)
                .unwrap();
            assert_eq!(frame_field.raw, field.raw, "frame value {i} {}", field.name);
            assert_eq!(frame_field.width, field.width);
            assert_eq!(
                frame_field.bit - span.start_bit,
                field.bit - component.start_bit
            );
        }
        assert_eq!(reader.profile(), cfg.context.profile);
    }
}

#[test]
fn native_new_record_tail_positive_gate() {
    check_new_tail_policy(
        include_bytes!("fixtures/new-tail-policy-v41.json.zlib"),
        384,
    );
}
#[test]
fn native_new_record_tail_large_and_lazy() {
    check_new_tail_policy(include_bytes!("fixtures/new-tail-large-v41.json.zlib"), 224);
}
fn check_new_tail_policy(fixture: &[u8], count: usize) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), count);
    for (i, c) in cases.iter().enumerate() {
        let mode = c["mode"].as_u64().unwrap();
        let name = if mode == 2 {
            "not-ported"
        } else {
            "biped-emp-timer-component"
        };
        let registry = FilmRegistry {
            archetypes: (0..4)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 3 {
                        vec![name.into()]
                    } else {
                        vec![]
                    },
                    levels: if index == 3 { vec![0] } else { vec![] },
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile.grammar.default_state_by_archetype = false;
        let tail = c["tail"].as_i64().unwrap();
        cfg.context.profile.grammar.new_record_tail_bits = tail;
        let events = Arc::new(Mutex::new(Vec::<u32>::new()));
        let observer = NativeFilmObserver::default();
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let events = events.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::EmpTimer {
                        quantum,
                    }) = p
                    else {
                        panic!("wrong hook")
                    };
                    events.lock().unwrap().push(*quantum);
                })
            }),
        );
        cfg.context.observer = Some(observer);
        let hex = c["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut reader = NativeFilmReader::new(&data);
        reader.set_bit_position(c["start"].as_u64().unwrap() as usize);
        let mut world = FilmWorld::default();
        world.bind_full(1, 3);
        let result = reader.read_frame_records(&registry, &mut world, &cfg);
        let view = result.unwrap();
        assert_eq!(json!(reader.native_bit_position()), c["end"], "end {i}");
        assert_eq!(
            json!(view.stop == EntityViewStop::Complete),
            c["complete"],
            "stop {i}"
        );
        assert_eq!(json!(*events.lock().unwrap()), c["events"], "callbacks {i}");
        assert_eq!(reader.profile(), cfg.context.profile, "raw metadata {i}");
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .collect();
        assert_eq!(records.len(), c["records"].as_array().unwrap().len());
        for (record, expected) in records.iter().zip(c["records"].as_array().unwrap()) {
            assert_eq!(json!(record.header.id), expected["ID"]);
            assert_eq!(
                record.stop == EntityViewStop::Complete,
                expected["DesyncAt"] == -1
            );
            if record.header.kind != RecordKind::Delete {
                assert_eq!(json!(record.end_bit), expected["Trace"]["EndBit"]);
            }
        }
        let consumed: usize = view
            .records
            .iter()
            .flat_map(|r| &r.fields)
            .filter(|f| f.name.starts_with("new.terminal"))
            .map(|f| usize::try_from(f.width).unwrap())
            .sum();
        let expected = if mode < 2 { tail.max(0) as u64 } else { 0 };
        if expected > 4096 {
            let adjustment = &view.records[0].diagnostics.width_adjustments[0];
            assert_eq!(adjustment.purpose, Some(NativeWidthPurpose::NewRecordTail));
            assert_eq!(adjustment.width, tail);
            assert_eq!(adjustment.retained_bits, Some(consumed));
            assert_eq!(
                serde_json::json!(adjustment.end_bit),
                serde_json::json!(usize::try_from(view.records[0].end_bit).ok())
            );
            assert_eq!(
                serde_json::json!(consumed),
                serde_json::json!((data.len() * 8) as i64 - adjustment.bit)
            );
            for f in view.records[0]
                .fields
                .iter()
                .filter(|f| f.name.starts_with("new.terminal"))
            {
                let raw = (f.bit..f.bit.wrapping_add(f.width as i64)).fold(0u64, |v, bit| {
                    (v << 1)
                        | u64::from(
                            (data[crate::theater::bits::native_address(bit / 8)] >> (7 - bit % 8))
                                & 1,
                        )
                });
                assert_eq!(f.raw, raw, "source tail field {i}");
            }
        } else {
            assert_eq!(consumed as u64, expected, "tail admission {i}");
        }
        assert_eq!(world.slots.len(), c["slots"].as_object().unwrap().len());
    }
}

#[test]
fn native_new_default_lazy_widths_and_rewinds() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/new-default-policy-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 624);
    for (i, c) in cases.iter().enumerate() {
        let width = c["width"].as_i64().unwrap();
        let mode = c["mode"].as_u64().unwrap();
        let registry = FilmRegistry {
            archetypes: (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            new_default_state_bits: width,
            ..Default::default()
        };
        cfg.context.profile.grammar.default_state_by_archetype = mode != 1;
        let text = c["hex"].as_str().unwrap();
        let data: Vec<_> = (0..text.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&text[j..j + 2], 16).unwrap())
            .collect();
        let mut reader = NativeFilmReader::new(&data);
        reader.set_bit_position(c["start"].as_u64().unwrap() as usize);
        let mut world = FilmWorld::default();
        world.bind_full(1, 3);
        let result = reader.read_frame_records(&registry, &mut world, &cfg);
        let view = result.unwrap_or_else(|error| panic!("case {i}: {error:?}"));
        assert_eq!(json!(reader.native_bit_position()), c["end"], "end {i}");
        assert_eq!(
            json!(view.stop == EntityViewStop::Complete),
            c["complete"],
            "stop {i}"
        );
        assert_eq!(reader.profile(), cfg.context.profile);
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .collect();
        assert_eq!(
            records.len(),
            c["records"].as_array().unwrap().len(),
            "records {i}"
        );
        for (record, expected) in records.iter().zip(c["records"].as_array().unwrap()) {
            assert_eq!(json!(record.header.id), expected["ID"]);
            assert_eq!(json!(record.header.start_bit), expected["HeaderBit"]);
            if record.header.kind == RecordKind::New {
                assert_eq!(
                    json!(record.default_state_bits),
                    expected["Trace"]["DefaultBits"],
                    "raw default metadata {i}"
                );
            } else {
                assert!(record.default_state_bits.is_none());
            }
            if record.header.kind != RecordKind::Delete {
                assert_eq!(
                    json!(record.end_bit),
                    expected["Trace"]["EndBit"],
                    "record end {i}"
                );
                if let Some(mask) = record.mask {
                    assert_eq!(json!(mask), expected["Trace"]["Mask"]);
                }
            }
        }
        let expected_slots = c["slots"].as_object().unwrap();
        assert_eq!(world.slots.len(), expected_slots.len(), "world {i}");
        for (slot, s) in &world.slots {
            let expected = &expected_slots[&slot.to_string()];
            assert_eq!(json!(s.archetype), expected["TypeIndex"]);
            assert_eq!(json!(s.full_id), expected["FullID"]);
            assert_eq!(json!(s.soft), expected["Soft"]);
            assert_eq!(json!(s.generation_any), expected["GenAny"]);
            assert_eq!(json!(s.position.is_some()), expected["PosValid"]);
            let pos: [f32; 3] = serde_json::from_value(expected["Pos"].clone()).unwrap();
            assert_eq!(
                s.position.unwrap_or([0.; 3]).map(f32::to_bits),
                pos.map(f32::to_bits)
            );
            assert_eq!(json!(s.view.unwrap_or(0)), expected["Vue"]);
        }
        let adjustments: Vec<_> = records
            .iter()
            .flat_map(|r| &r.diagnostics.width_adjustments)
            .filter(|a| a.purpose == Some(NativeWidthPurpose::NewRecordDefault))
            .collect();
        if mode < 2 && !(0..=4096).contains(&width) {
            assert_eq!(adjustments.len(), 1, "adjustment {i}");
            let a = adjustments[0];
            assert_eq!(a.width, width);
            assert_eq!(a.native_end_bit(), Some(a.bit.wrapping_add(width)));
            if width < 0 {
                assert_eq!(a.end_bit, Some((a.bit + width) as usize));
                assert_eq!(a.retained_bits, None);
            } else {
                assert_eq!(
                    serde_json::json!(a.retained_bits),
                    serde_json::json!(Some((data.len() * 8) as i64 - a.bit))
                );
            }
        } else {
            assert!(adjustments.is_empty(), "unexpected adjustment {i}");
        }
        for record in records {
            for f in record
                .fields
                .iter()
                .filter(|f| f.name.starts_with("new.default_skipped"))
            {
                let raw = (f.bit..f.bit.wrapping_add(f.width as i64)).fold(0u64, |v, bit| {
                    (v << 1)
                        | u64::from(
                            data.get(crate::theater::bits::native_address(bit / 8))
                                .map_or(0, |b| (b >> (7 - bit % 8)) & 1),
                        )
                });
                assert_eq!(f.raw, raw, "retained source {i}");
            }
        }
        let restored: DecodedEntityView =
            serde_json::from_slice(&serde_json::to_vec(&view).unwrap()).unwrap();
        assert_eq!(restored, view, "roundtrip {i}");
        let mut old = serde_json::to_value(&view).unwrap();
        for record in old["records"].as_array_mut().unwrap() {
            record.as_object_mut().unwrap().remove("default_state_bits");
        }
        let old: DecodedEntityView = serde_json::from_value(old).unwrap();
        assert!(
            old.records.iter().all(|r| r.default_state_bits.is_none()),
            "old metadata absence {i}"
        );
    }
}

#[test]
fn native_mpp_widths_are_lazy_and_preserve_wide_reads() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/mpp-width-policy-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1632);
    let registry = FilmRegistry {
        archetypes: (0..37)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                levels: vec![],
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    for (i, c) in rows.iter().enumerate() {
        let width = c["width"].as_i64().unwrap();
        let mode = c["mode"].as_u64().unwrap();
        let axis = c["axis"].as_u64().unwrap();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile.grammar.default_state_by_archetype = mode != 2;
        if axis == 0 {
            cfg.context.profile.mpp.lead = width;
        } else {
            cfg.context.profile.mpp.index = width;
        }
        let events = Arc::new(Mutex::new(Vec::<Value>::new()));
        let observer = NativeFilmObserver::default();
        observer.set_hook(
            NativeHookKind::Mpp,
            Some({
                let events = events.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::Mpp {
                        field,
                        value,
                        present,
                    }) = p
                    else {
                        panic!("wrong hook")
                    };
                    events
                        .lock()
                        .unwrap()
                        .push(json!({"field":field,"value":value,"present":present}));
                })
            }),
        );
        cfg.context.observer = Some(observer);
        let hex = c["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut reader = NativeFilmReader::new(&data);
        reader.set_bit_position(c["start"].as_u64().unwrap() as usize);
        let mut world = FilmWorld::default();
        world.bind_full(1, 3);
        let view = reader
            .read_frame_records(&registry, &mut world, &cfg)
            .unwrap_or_else(|e| panic!("case {i}: {e:?}"));
        assert_eq!(json!(reader.bit_position()), c["end"], "end {i}");
        assert_eq!(
            json!(view.stop == EntityViewStop::Complete),
            c["complete"],
            "completion {i}"
        );
        assert_eq!(reader.profile(), cfg.context.profile, "profile {i}");
        assert_eq!(json!(*events.lock().unwrap()), c["events"], "callbacks {i}");
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .collect();
        assert_eq!(records.len(), c["records"].as_array().unwrap().len());
        for (r, e) in records.iter().zip(c["records"].as_array().unwrap()) {
            assert_eq!(json!(r.header.id), e["ID"]);
            assert_eq!(json!(r.header.start_bit), e["HeaderBit"]);
            if r.header.kind != RecordKind::Delete {
                assert_eq!(json!(r.end_bit), e["Trace"]["EndBit"], "record end {i}");
            }
            if let Some(mask) = r.mask {
                assert_eq!(json!(mask), e["Trace"]["Mask"]);
            }
            for f in r.fields.iter().filter(|f| {
                f.name == "mpp.lead"
                    || f.name == "mpp.index"
                    || f.name.starts_with("mpp.lead.discarded")
                    || f.name.starts_with("mpp.index.discarded")
            }) {
                let value = (f.bit..f.bit.wrapping_add(f.width as i64)).fold(0u64, |v, b| {
                    (v << 1)
                        | u64::from(
                            data.get(crate::theater::bits::native_address(b / 8))
                                .map_or(0, |v| (v >> (7 - b % 8)) & 1),
                        )
                });
                assert_eq!(f.raw, value, "MPP retained field {i}");
            }
        }
        if mode < 2 {
            let field = if axis == 0 { "mpp.lead" } else { "mpp.index" };
            let f = records[0].fields.iter().find(|f| f.name == field).unwrap();
            assert_eq!(f.width as i64, width, "consumed raw width {i}");
        }
        let expected = c["slots"].as_object().unwrap();
        assert_eq!(world.slots.len(), expected.len());
        for (slot, s) in &world.slots {
            let e = &expected[&slot.to_string()];
            assert_eq!(json!(s.archetype), e["TypeIndex"]);
            assert_eq!(json!(s.full_id), e["FullID"]);
            assert_eq!(json!(s.soft), e["Soft"]);
            assert_eq!(json!(s.generation_any), e["GenAny"]);
            assert_eq!(json!(s.position.is_some()), e["PosValid"]);
            let pos: [f32; 3] = serde_json::from_value(e["Pos"].clone()).unwrap();
            assert_eq!(
                s.position.unwrap_or([0.; 3]).map(f32::to_bits),
                pos.map(f32::to_bits)
            );
            assert_eq!(json!(s.view.unwrap_or(0)), e["Vue"]);
        }
        let restored: DecodedEntityView =
            serde_json::from_slice(&serde_json::to_vec(&view).unwrap()).unwrap();
        assert_eq!(restored, view, "roundtrip {i}");
    }

    // Analytical overflow controls, separate from the executable native oracle:
    // a -1 profile width becomes u64::MAX and reaches i64::MIN before completion.
    // The native per-bit loop from an ordinary position is impractical to run.
    if !cfg!(target_arch = "wasm32") {
        let c = rows
            .iter()
            .find(|r| r["width"] == 5 && r["axis"] == 0 && r["mode"] == 0 && r["start"] == 0)
            .unwrap();
        let hex = c["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        for axis in 0..2 {
            let mut cfg = NativeFrameConfig {
                id_low_bits: 5,
                packet_preamble_bits: 0,
                ..Default::default()
            };
            cfg.context.profile.mpp.lead = 5;
            if axis == 0 {
                cfg.context.profile.mpp.lead = -1;
            } else {
                cfg.context.profile.mpp.index = -1;
            }
            let mut reader = NativeFilmReader::new(&data);
            let mut world = FilmWorld::default();
            world.bind_full(1, 3);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                reader.read_frame_records(&registry, &mut world, &cfg)
            }));
            assert!(result.is_err());
            assert_eq!(reader.native_bit_position(), i64::MIN);
            assert_eq!(reader.profile(), cfg.context.profile);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || cfg.decode_inference_view(&data, 0, &registry, &mut world)
                ))
                .is_err()
            );
        }
    }
}

#[test]
fn native_record_id_widths_and_wrapping_match_reference() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/record-id-policy-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 8640);
    let registry = FilmRegistry {
        archetypes: (0..4)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                levels: vec![],
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    for (i, c) in rows.iter().enumerate() {
        let width = c["width"].as_i64().unwrap();
        let base = c["base"].as_u64().unwrap() as u32;
        let start = c["start"].as_u64().unwrap() as usize;
        let hex = c["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut cfg = NativeFrameConfig {
            id_low_bits: width,
            id_base: base,
            packet_preamble_bits: 0,
            ..Default::default()
        };
        cfg.context.profile.grammar.default_state_by_archetype = false;
        let mut world = FilmWorld::default();
        if let Some(id) = c["id"].as_u64() {
            world.bind_full(id as u32, 3);
        }
        let mut reader = NativeFilmReader::new(&data);
        reader.set_bit_position(start);
        let view = reader
            .read_frame_records(&registry, &mut world, &cfg)
            .unwrap_or_else(|e| panic!("case {i}: {e:?}"));
        let header = &view.records[0].header;
        let kind = match header.kind {
            RecordKind::End => 0,
            RecordKind::New => 1,
            RecordKind::Delete => 2,
            RecordKind::Delta => 3,
        };
        assert_eq!(json!(kind), c["kind"], "kind {i}");
        assert_eq!(json!(header.id), c["id"], "id {i}");
        assert_eq!(json!(header.end_bit), c["end"], "header end {i}");
        assert_eq!(
            json!(reader.native_bit_position()),
            c["frame_end"],
            "frame end {i}"
        );
        assert_eq!(
            json!(view.stop == EntityViewStop::Complete),
            c["complete"],
            "complete {i}"
        );
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .collect();
        let expected = c["records"].as_array().unwrap();
        assert_eq!(records.len(), expected.len(), "records {i}");
        for (r, e) in records.iter().zip(expected) {
            assert_eq!(json!(r.header.id), e["ID"], "record id {i}");
            assert_eq!(
                json!(r.header.start_bit),
                e["HeaderBit"],
                "record start {i}"
            );
            if r.header.kind != RecordKind::Delete {
                assert_eq!(json!(r.end_bit), e["Trace"]["EndBit"], "record end {i}");
            }
            if let Some(mask) = r.mask {
                assert_eq!(json!(mask), e["Trace"]["Mask"], "mask {i}");
            }
        }
        let expected = c["slots"].as_object().unwrap();
        assert_eq!(world.slots.len(), expected.len(), "world size {i}");
        for (slot, s) in &world.slots {
            let e = &expected[&slot.to_string()];
            assert_eq!(json!(s.archetype), e["TypeIndex"]);
            assert_eq!(json!(s.full_id), e["FullID"]);
            assert_eq!(json!(s.soft), e["Soft"]);
            assert_eq!(json!(s.generation_any), e["GenAny"]);
            assert_eq!(json!(s.position.is_some()), e["PosValid"]);
            let pos: [f32; 3] = serde_json::from_value(e["Pos"].clone()).unwrap();
            assert_eq!(
                s.position.unwrap_or([0.; 3]).map(f32::to_bits),
                pos.map(f32::to_bits)
            );
            assert_eq!(json!(s.view.unwrap_or(0)), e["Vue"]);
        }
        if (0..=30).contains(&width) && base <= 0x3fff_ffff {
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                component_widths: Default::default(),
                new_record: NewRecordEncoding {
                    deserialize_defaults: false,
                    ..Default::default()
                },
                position_capture: None,
                native_id_low_bits: None,
                ids: RecordIdLayout {
                    low_bits: width as usize,
                    base,
                },
                mpp_widths: [5, 2],
                position: None,
                extra_fields: false,
                corruption_check: false,
            };
            let mut bindings = EntityBindings::default();
            if let Some(id) = c["id"].as_u64() {
                bindings.bind(id as u32, 3);
            }
            let legacy =
                decode_native_entity_view(&data, start, &registry, &encoding, &mut bindings, true);
            assert_eq!(
                legacy.records[0].header, *header,
                "legacy native header {i}"
            );
            assert_eq!(legacy.end_bit, view.end_bit, "legacy native end {i}");
            assert_eq!(legacy.stop, view.stop, "legacy native stop {i}");
            let mut world = FilmWorld::default();
            if let Some(id) = c["id"].as_u64() {
                world.bind_full(id as u32, 3);
            }
            let inferred = decode_inference_frame(
                &data,
                start,
                &registry,
                &KillWalkProfile {
                    encoding: encoding.clone(),
                    simulation_complete: false,
                },
                &mut world,
                InferenceFrameOptions {
                    chain_inference: false,
                    view_tables: false,
                },
            )
            .unwrap();
            let first = match &inferred.stop {
                InferenceFrameStop::End(h) if inferred.records.is_empty() => h,
                _ => &inferred.records[0].header,
            };
            assert_eq!(first, header, "legacy inference header {i}");
            // Native wrapping must not relax the separately bounded helper.
            if header.kind != RecordKind::End
                && width == 30
                && base == 0x3fff_ffff
                && c["value"].as_u64().unwrap() != 0
            {
                assert!(
                    decode_record_header(&data, start, encoding.ids).is_none(),
                    "bounded sum gate {i}"
                );
            }
        }
        let restored: DecodedEntityView =
            serde_json::from_slice(&serde_json::to_vec(&view).unwrap()).unwrap();
        assert_eq!(restored, view, "roundtrip {i}");
    }
}

#[test]
fn native_record_id_overflow_panics_preserve_cursor() {
    // These analytical overflow controls complement the executable native oracle.
    // Running the native per-bit loop for i64::MAX bits is impractical.
    if cfg!(target_arch = "wasm32") {
        return;
    }
    let registry = FilmRegistry {
        archetypes: vec![],
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    for extra in [false, true] {
        let mut data = if extra { vec![0; 4] } else { vec![] };
        data.extend([0x20, 0, 0, 0, 0, 0, 0, 0]);
        let bit = if extra { 35 } else { 3 };
        for (width, end) in [(i64::MAX, i64::MIN), (i64::MAX - bit, i64::MIN + 1)] {
            let cfg = NativeFrameConfig {
                id_low_bits: width,
                packet_preamble_bits: 0,
                extra_fields: extra,
                ..Default::default()
            };
            let mut reader = NativeFilmReader::new(&data);
            let mut world = FilmWorld::default();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                reader.read_frame_records(&registry, &mut world, &cfg)
            }));
            assert!(result.is_err());
            assert_eq!(reader.native_bit_position(), end);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || cfg.decode_inference_view(&data, 0, &registry, &mut world)
                ))
                .is_err()
            );
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || cfg.decode_production_views(&data, 0, &registry, &mut world)
                ))
                .is_err()
            );
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cfg.decode_resync_frame(
                    &data,
                    &registry,
                    &mut world,
                    &Default::default(),
                    |_, _, _| true
                )))
                .is_err()
            );
        }
    }
}

#[test]
fn native_march_unused_widths_preserve_calibration() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/march-lazy-policy-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 90);
    let registry = FilmRegistry {
        archetypes: (0..4)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                levels: vec![],
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    for (i, c) in rows.iter().enumerate() {
        let width = c["width"].as_i64().unwrap();
        let axis = c["axis"].as_u64().unwrap();
        let mut cfg = NativeFrameConfig::default();
        match axis {
            0 => {
                cfg.id_low_bits = width;
                cfg.id_base = u32::MAX;
            }
            1 => cfg.context.profile.mpp.lead = width,
            2 => cfg.context.profile.mpp.index = width,
            3 => cfg.new_default_state_bits = width,
            4 => cfg.context.profile.movement.world_object.index_bits = width as u64,
            _ => unreachable!(),
        }
        let hex = c["hex"].as_str().unwrap();
        let payload: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut packet = vec![0; 16];
        packet[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        packet[8..16].copy_from_slice(&1234u64.to_le_bytes());
        packet.extend(payload);
        let metadata = |index, chunk_type| crate::clients::hi::models::FilmChunk {
            index,
            chunk_type,
            start_time_offset_ms: 0,
            duration_ms: 0,
            size: 0,
            file_relative_path: String::new(),
        };
        let chunks = vec![
            crate::clients::hi::models::FilmChunkData {
                metadata: metadata(0, 1),
                data: vec![41, 0, 0, 0, 27, 0, 0, 0],
            },
            crate::clients::hi::models::FilmChunkData {
                metadata: metadata(1, 0),
                data: packet,
            },
        ];
        let out = scan_film_march_facts_with_native_config(&chunks, &registry, &cfg)
            .unwrap_or_else(|e| panic!("march case {i}: {e:?}"));
        let e = &c["facts"];
        let st = &e["Stats"];
        assert_eq!(json!(out.facts.deaths), e["Deaths"]);
        assert_eq!(json!(out.facts.occupancy), e["Occupancy"]);
        for (actual, key) in [
            (out.keyframes, "Keyframes"),
            (out.deltas, "Deltas"),
            (out.packets, "Packets"),
            (out.event_packets, "EventPackets"),
            (out.located_packets, "LocatedPackets"),
        ] {
            assert_eq!(json!(actual), st[key], "{key} {i}");
        }
        for (actual, key) in [
            (&out.facts.coverage.records, "Records"),
            (&out.facts.coverage.clean_records, "CleanRecords"),
            (&out.facts.coverage.mask_declared, "MaskDeclared"),
            (
                &out.facts.coverage.mask_declared_desync,
                "MaskDeclaredDesync",
            ),
        ] {
            assert_eq!(json!(actual), st[key], "{key} {i}");
        }
        let config = out.native_config.as_ref().unwrap();
        assert_eq!(*config, cfg.snapshot(), "raw config {i}");
        assert_eq!(json!(config.id_low_bits), st["Config"]["IDLowBits"]);
        assert_eq!(json!(config.id_base), st["Config"]["IDBase"]);
        assert_eq!(
            json!(config.new_default_state_bits),
            st["Config"]["NewDefaultStateBits"]
        );
        let cal = out.calibration.as_ref().unwrap();
        assert_eq!(json!(cal.retained_default), st["CadreParDefaut"]);
        assert_eq!(json!(cal.best.located), st["CadreLocalises"]);
        assert_eq!(json!(cal.runner_up.located), st["CadreDauphin"]);
        assert_eq!(json!(cal.best.events), st["CadreEvenements"]);
        let projected = match axis {
            0 => false,
            1 | 2 => (1..=32).contains(&width),
            3 => width >= 0 && u64::try_from(width).unwrap() <= usize::MAX as u64,
            4 => (0..=30).contains(&width),
            _ => unreachable!(),
        };
        assert_eq!(cal.encoding.is_some(), projected, "projection {i}");
        assert_eq!(
            cal.encoding_error.is_some(),
            !projected,
            "projection diagnostic {i}"
        );
        let restored: FilmMarchFacts = serde_json::from_value(json!(out)).unwrap();
        assert_eq!(restored, out);
        if projected {
            let mut old = json!(cal);
            old.as_object_mut().unwrap().remove("encoding_error");
            let old: MarchCalibration = serde_json::from_value(old).unwrap();
            assert_eq!(&old, cal);
        }
    }
}

#[test]
fn native_biped_default_boundary_d61443e() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/biped-default-boundary-d61443e-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 16);
    let registry = FilmRegistry {
        archetypes: (0..36)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                levels: vec![],
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    let cfg = NativeFrameConfig {
        id_low_bits: 5,
        packet_preamble_bits: 0,
        ..Default::default()
    };
    for (i, c) in cases.iter().enumerate() {
        let hex = c["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
            .collect();
        let mut reader = NativeFilmReader::new(&data);
        reader.set_bit_position(c["start"].as_u64().unwrap() as usize);
        let view = reader
            .read_frame_records(&registry, &mut FilmWorld::default(), &cfg)
            .unwrap();
        assert_eq!(view.stop, EntityViewStop::Complete, "{i}");
        assert_eq!(json!(reader.native_bit_position()), c["end"], "{i}");
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .collect();
        assert_eq!(records.len(), 1);
        let record = records[0];
        assert_eq!(record.header.kind, RecordKind::New);
        assert_eq!(record.archetype, Some(35));
        assert_eq!(record.mask, Some(0));
        assert_eq!(json!(record.end_bit), c["record_end"], "{i}");
        assert!(!record.default_state_fallback);
        let reference = record.references.last().unwrap();
        assert_eq!(reference.present, c["gate"].as_bool().unwrap());
        assert_eq!(
            reference.value,
            if reference.present { 0x7F5C4A1C } else { 0 }
        );
        assert_eq!(reference.kind, NativeUnitReferenceKind::GatedWord32);
    }
}
