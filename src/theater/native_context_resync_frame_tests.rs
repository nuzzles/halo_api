use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_context_resync_frame_live_recovery() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-resync-frame-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 128);
    for case in cases {
        let c = case["case"].as_u64().unwrap() as usize;
        let names: Vec<Vec<String>> = serde_json::from_value(case["names"].clone()).unwrap();
        let registry = FilmRegistry {
            archetypes: names
                .into_iter()
                .enumerate()
                .map(|(index, components)| FilmArchetype {
                    index,
                    levels: vec![0; components.len()],
                    components,
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let hex = case["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            extra_fields: c.is_multiple_of(4),
            ..Default::default()
        };
        cfg.context.profile.grammar.chain_inference = true;
        cfg.context.profile.grammar.default_state_by_archetype = false;
        cfg.context.profile.grammar.view_tables = c.is_multiple_of(5);
        cfg.context.profile.grammar.mobility_action_body = false;
        let observer = NativeFilmObserver::default();
        cfg.context.observer = Some(observer.clone());
        if c.is_multiple_of(3) {
            observer.take_absolute_indices();
        } else if c % 3 == 1 {
            observer.record_absolute(7);
        }
        let events = Arc::new(Mutex::new(Vec::<Value>::new()));
        let movement = |tag: &'static str| -> NativeHook {
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
                    .push(json!([tag, component, slot, values]));
            })
        };
        let reference = |tag: &'static str| -> NativeHook {
            let events = events.clone();
            Arc::new(move |p| {
                let NativeHookPublication::Component(FilmComponentObservation::UnitReference {
                    reference: r,
                }) = p
                else {
                    panic!("wrong hook")
                };
                events.lock().unwrap().push(json!([
                    tag,
                    r.kind.native_name(),
                    r.start_bit,
                    r.end_bit,
                    r.present,
                    r.value,
                    r.tail,
                    r.probe
                ]));
            })
        };
        observer.set_hook(NativeHookKind::MovementState, Some(movement("old-move")));
        observer.set_hook(NativeHookKind::UnitReference, Some(reference("old-ref")));
        observer.set_hook(
            NativeHookKind::Position,
            Some({
                let events = events.clone();
                Arc::new(move |_| events.lock().unwrap().push(json!(["old-position"])))
            }),
        );
        observer.set_hook(
            NativeHookKind::EmpTimer,
            Some({
                let events = events.clone();
                let observer = observer.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::EmpTimer {
                        quantum,
                    }) = p
                    else {
                        panic!("wrong hook")
                    };
                    events
                        .lock()
                        .unwrap()
                        .push(json!(["emp", quantum, raw_counts(&observer, c)]));
                })
            }),
        );
        observer.set_hook(
            NativeHookKind::MobilityAction,
            Some({
                let events = events.clone();
                let observer = observer.clone();
                let movement = movement("new-move");
                let reference = reference("new-ref");
                Arc::new(move |p| {
                    let NativeHookPublication::MobilityAction([a, b]) = p else {
                        panic!("wrong hook")
                    };
                    events.lock().unwrap().push(json!(["mobility", a, b]));
                    if c.is_multiple_of(2) {
                        observer.set_hook(NativeHookKind::MovementState, Some(movement.clone()));
                        observer.set_hook(NativeHookKind::UnitReference, Some(reference.clone()));
                    }
                })
            }),
        );
        let mut world = FilmWorld::default();
        if c.is_multiple_of(11) {
            world.bind_soft(50, 0)
        } else {
            world.bind_full(50, 0)
        }
        let mut calls = Vec::new();
        let frame = cfg
            .decode_resync_frame(
                &data,
                &registry,
                &mut world,
                &[50].into(),
                |slot, pos, has| {
                    calls.push(json!([slot, pos.map(f32::to_bits), has]));
                    c % 3 != 2 || calls.len().is_multiple_of(2)
                },
            )
            .unwrap();
        let expected = case["records"].as_array().unwrap();
        assert_eq!(frame.records.len(), expected.len(), "record count {c}");
        for (actual, expected) in frame.records.iter().zip(expected) {
            assert_eq!(json!(actual.header.id), expected["ID"], "record id {c}");
            assert_eq!(
                json!(actual.archetype.unwrap_or(0)),
                expected["TypeIndex"],
                "type {c}"
            );
            assert_eq!(
                json!(actual.mask.unwrap_or(0)),
                expected["Trace"]["Mask"],
                "mask {c}"
            );
            if actual.header.kind != RecordKind::Delete {
                assert_eq!(
                    json!(actual.end_bit),
                    expected["Trace"]["EndBit"],
                    "end {c}"
                );
                let components = expected["Trace"]["Comps"]
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                assert_eq!(actual.attempts.len(), components.len(), "components {c}");
                for (index, (a, e)) in actual.attempts.iter().zip(components).enumerate() {
                    assert_eq!(json!(a.span.name), e["Name"]);
                    assert_eq!(json!(a.span.start_bit), e["StartBit"]);
                    assert_eq!(
                        &json!(a.span.end_bit),
                        components
                            .get(index + 1)
                            .map(|e| &e["StartBit"])
                            .unwrap_or(&expected["Trace"]["EndBit"])
                    );
                }
            }
        }
        let restored: NativeResyncFrame =
            serde_json::from_value(serde_json::to_value(&frame).unwrap()).unwrap();
        assert_eq!(restored, frame);
        assert_eq!(json!(calls), case["calls"], "accept case {c}");
        assert_eq!(
            json!(*events.lock().unwrap()),
            case["frame_events"],
            "case {c}"
        );
        assert_eq!(
            json!(raw_counts(&observer, c)),
            case["counts"],
            "counts case {c}"
        );
        observer.publish(NativeHookPublication::Component(
            &FilmComponentObservation::Position {
                position_kind: NativePositionKind::Baseline,
                vector_bits: [0; 3],
                bit: 0,
                slot: 0,
            },
        ));
        observer.publish(NativeHookPublication::Component(
            &FilmComponentObservation::UnitReference {
                reference: NativeUnitReference {
                    kind: NativeUnitReferenceKind::Word32,
                    start_bit: 0,
                    end_bit: 0,
                    present: true,
                    value: 123,
                    tail: 0,
                    probe: false,
                },
            },
        ));
        let mut probe = cfg.reader(&[0, 0]);
        probe.set_capture_slot(99);
        probe.read_component("unit-crouch-component", 0, 0).unwrap();
        assert_eq!(
            json!(*events.lock().unwrap()),
            case["events"],
            "restoration case {c}"
        );
        let slots:BTreeMap<_,_>=world.slots.iter().map(|(slot,s)|(slot.to_string(),json!({"TypeIndex":s.archetype,"FullID":s.full_id,"Soft":s.soft,"GenAny":s.generation_any,"Pos":s.position.unwrap_or([0.0;3]),"PosValid":s.position.is_some(),"Vue":s.view.unwrap_or(-1)}))).collect();
        let mut expected = case["slots"].clone();
        for s in expected.as_object_mut().unwrap().values_mut() {
            let p: [f32; 3] = serde_json::from_value(s["Pos"].clone()).unwrap();
            s["Pos"] = json!(p);
        }
        assert_eq!(json!(slots), expected);
        observer.set_hook(NativeHookKind::MobilityAction, None);
        observer.set_hook(NativeHookKind::EmpTimer, None);
    }
}

fn raw_counts(observer: &NativeFilmObserver, c: usize) -> Value {
    let d = observer.counters();
    let absolute = if c % 3 == 2 && d.absolute_indices.is_empty() {
        Value::Null
    } else {
        json!(d.absolute_indices)
    };
    json!({"absolute":absolute,"resync":d.validated_resyncs})
}

#[test]
fn native_resync_acceptance_removes_live_width() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/resync-mutation-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 16);
    let mut failed_rereads = 0;
    for row in rows {
        let c = row["case"].as_u64().unwrap();
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["not-ported".into()],
                levels: vec![0],
            }],
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let widths = NativeSharedWidths::from_map([("not-ported".into(), 8)].into());
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            extra_fields: c % 2 == 1,
            ..Default::default()
        };
        if row["stub"] == true {
            cfg.context.profile.grammar.stub_widths = Some(widths.clone());
        } else {
            cfg.context.profile.grammar.calibrated_widths = Some(widths.clone());
        }
        cfg.context.observer = Some(NativeFilmObserver::default());
        let mut world = FilmWorld::default();
        world.bind_full(50, 0);
        let mut calls = Vec::new();
        let frame = cfg
            .decode_resync_frame(
                &data,
                &registry,
                &mut world,
                &[50].into(),
                |slot, _, has| {
                    calls.push(json!([slot, has]));
                    if row["remove"] == true {
                        widths.remove("not-ported");
                    }
                    true
                },
            )
            .unwrap();
        assert_eq!(json!(calls), row["calls"], "calls {c}");
        assert_eq!(json!(widths.snapshot()), row["widths"], "shared map {c}");
        assert_eq!(
            json!(frame.resync_bits),
            json!([row["target"]]),
            "landing {c}"
        );
        let expected = row["records"].as_array().unwrap();
        assert_eq!(frame.records.len(), expected.len(), "records {c}");
        for (record, native) in frame.records.iter().zip(expected) {
            assert_eq!(json!(record.header.id), native["ID"]);
            assert_eq!(json!(record.archetype), native["TypeIndex"]);
            assert_eq!(
                record.stop == EntityViewStop::Complete,
                native["DesyncAt"] == -1
            );
            assert_eq!(json!(record.end_bit), native["Trace"]["EndBit"]);
            assert_eq!(json!(record.mask), native["Trace"]["Mask"]);
            let comps = native["Trace"]["Comps"].as_array().unwrap();
            assert_eq!(record.attempts.len(), comps.len());
            for (attempt, component) in record.attempts.iter().zip(comps) {
                assert_eq!(json!(attempt.span.name), component["Name"]);
                assert_eq!(json!(attempt.span.index), component["Index"]);
                assert_eq!(json!(attempt.span.start_bit), component["StartBit"]);
                assert_eq!(json!(attempt.status), component["Ported"]);
                let fields = &record.fields[attempt.field_start..attempt.field_end];
                if row["remove"] == true {
                    assert!(fields.is_empty());
                    assert_eq!(attempt.span.start_bit, record.end_bit);
                    failed_rereads += 1;
                } else {
                    assert_eq!(fields.len(), 1);
                    assert_eq!(fields[0].width, 8);
                    assert_eq!(fields[0].raw, 192 + c);
                }
            }
        }
        assert_eq!(world.slots.len(), row["slots"].as_object().unwrap().len());
        assert_eq!(
            json!(world.slots[&50].full_id),
            row["slots"]["50"]["FullID"]
        );
        assert_eq!(
            json!(world.slots[&50].archetype),
            row["slots"]["50"]["TypeIndex"]
        );
        let restored: NativeResyncFrame = serde_json::from_value(json!(frame)).unwrap();
        assert_eq!(restored, frame);
    }
    assert_eq!(failed_rereads, 8);
}

#[test]
fn native_resync_record_guard() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/resync-guard-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 6);
    let mut total = 0;
    for row in rows {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let cfg = NativeFrameConfig {
            id_low_bits: 11,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        let registry = FilmRegistry {
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![],
                levels: vec![],
            }],
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let mut world = FilmWorld::default();
        world.bind_full(50, 0);
        world.bind_full(51, 0);
        let mut calls = 0;
        let frame = cfg
            .decode_resync_frame(&data, &registry, &mut world, &[50, 51].into(), |_, _, _| {
                calls += 1;
                true
            })
            .unwrap();
        assert_eq!(json!(calls), row["calls"]);
        let records: Vec<_> = frame
            .records
            .iter()
            .map(|r| {
                assert_eq!(r.stop, EntityViewStop::Complete);
                json!({"id":r.header.id,"end":r.end_bit,"desync":-1})
            })
            .collect();
        assert_eq!(json!(records), row["records"]);
        assert!(frame.resync_bits.is_empty());
        assert_eq!(world.slots.len(), 2);
        if row["count"].as_u64().unwrap() >= 4096 {
            assert_eq!(frame.stop, InferenceFrameStop::RecordLimit);
        } else {
            assert!(matches!(frame.stop, InferenceFrameStop::End(_)));
        }
        total += records.len();
    }
    assert_eq!(total, 24574);
}

#[test]
fn native_resync_final_cursor() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/resync-cursor-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 70);
    let mut total_records = 0;
    let mut total_calls = 0;
    for row in rows {
        let c = row["case"].as_u64().unwrap();
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec!["not-ported".into()],
                levels: vec![0],
            }],
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let widths = NativeSharedWidths::from_map([("not-ported".into(), 8)].into());
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            extra_fields: row["extra"].as_bool().unwrap(),
            ..Default::default()
        };
        if row["stub"] == true {
            cfg.context.profile.grammar.stub_widths = Some(widths.clone());
        } else {
            cfg.context.profile.grammar.calibrated_widths = Some(widths.clone());
        }
        cfg.context.observer = Some(NativeFilmObserver::default());
        let mut world = FilmWorld::default();
        if row["bound"] == true {
            world.bind_full(50, 0);
            world.bind_full(51, 0);
        }
        let mut calls = Vec::new();
        let frame = cfg
            .decode_resync_frame(
                &data,
                &registry,
                &mut world,
                &[50, 51].into(),
                |slot, _, has| {
                    calls.push(json!([slot, has]));
                    if row["remove"] == true {
                        widths.remove("not-ported");
                    }
                    row["accept"] == true
                },
            )
            .unwrap();
        let state = &row["state"];
        assert_eq!(json!(frame.end_bit), row["end"], "final cursor case {c}");
        assert_eq!(json!(calls), state["calls"], "callbacks {c}");
        assert_eq!(json!(widths.snapshot()), state["widths"]);
        let expected = state["records"].as_array().unwrap();
        assert_eq!(frame.records.len(), expected.len(), "record count {c}");
        for (record, native) in frame.records.iter().zip(expected) {
            assert_eq!(json!(record.header.id), native["ID"]);
            assert_eq!(json!(record.archetype), native["TypeIndex"]);
            assert_eq!(json!(record.end_bit), native["Trace"]["EndBit"]);
            assert_eq!(json!(record.mask), native["Trace"]["Mask"]);
            let desync = match record.stop {
                EntityViewStop::Complete => -1,
                EntityViewStop::UnsupportedComponent { index, .. } => index as i64,
                ref other => panic!("unexpected returned stop {other:?}"),
            };
            assert_eq!(json!(desync), native["DesyncAt"]);
        }
        assert_eq!(world.slots.len(), state["slots"].as_object().unwrap().len());
        for (slot, value) in &world.slots {
            let native = &state["slots"][slot.to_string()];
            assert_eq!(json!(value.full_id), native["FullID"]);
            assert_eq!(json!(value.archetype), native["TypeIndex"]);
        }
        total_records += frame.records.len();
        total_calls += calls.len();
    }
    assert_eq!(total_records, 24582);
    assert_eq!(total_calls, 21);
}
