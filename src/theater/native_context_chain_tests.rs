use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_context_chain_scopes_counters_and_restoration() {
    check_context(include_bytes!("fixtures/context-chain-v41.json.zlib"));
}
#[test]
fn native_context_chain_scopes_counters_and_restoration_lazy_widths() {
    check_context(include_bytes!(
        "fixtures/context-chain-lazy-widths-v41.json.zlib"
    ));
}
fn check_context(fixture: &[u8]) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
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
        if case["lazy"] == true {
            let width = case["unused_position_width"].as_u64().unwrap();
            let movement = &mut cfg.context.profile.movement;
            movement.world_object.index_bits = width;
            movement.world_object.axis_bits = [width; 3];
            movement.traversal.index_bits = width;
            movement.traversal.axis_bits = [width; 3];
            movement.delta_axis_width = width;
        }
        cfg.context.profile.grammar.chain_inference = true;
        cfg.context.profile.grammar.view_tables = c.is_multiple_of(5);
        cfg.context.profile.grammar.mobility_action_body = false;
        let observer = NativeFilmObserver::default();
        cfg.context.observer = Some(observer.clone());
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
                        .push(json!(["emp", quantum, chain_counts(&observer)]));
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
        let frame = cfg
            .decode_inference_view(&data, 0, &registry, &mut world)
            .unwrap();
        assert_eq!(json!(frame.end_bit), case["end"], "case {c}");
        assert_eq!(json!(frame.hit_end()), case["hit"], "case {c}");
        assert_eq!(json!(frame.inferred_count), case["inferred"], "case {c}");
        assert_eq!(
            json!(*events.lock().unwrap()),
            case["frame_events"],
            "case {c}"
        );
        assert_eq!(json!(observer.counters().rejected_unbound), case["unbound"]);
        assert_eq!(chain_counts(&observer), case["counts"], "counts case {c}");
        let records = case["records"].as_array().unwrap();
        assert_eq!(frame.records.len(), records.len());
        for (r, e) in frame.records.iter().zip(records) {
            assert_eq!(json!(r.header.id), e["ID"]);
            assert_eq!(json!(r.archetype.unwrap_or(0)), e["TypeIndex"]);
            if let Some(d) = &r.decoded {
                assert_eq!(json!(d.end_bit), e["Trace"]["EndBit"]);
                assert_eq!(json!(d.mask.unwrap_or(0)), e["Trace"]["Mask"]);
            }
        }
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
        assert_eq!(
            serde_json::from_value::<InferenceFrame>(serde_json::to_value(&frame).unwrap())
                .unwrap(),
            frame
        );
        observer.set_hook(NativeHookKind::MobilityAction, None);
        observer.set_hook(NativeHookKind::EmpTimer, None);
    }
}

fn chain_counts(observer: &NativeFilmObserver) -> Value {
    let counts = observer.counters().chain_outcomes;
    let value = |kind| counts.get(&kind).copied().unwrap_or(0);
    json!({"Immediate":value(ChainInferenceOutcome::Immediate),"Deep":value(ChainInferenceOutcome::Deep),"NoConfirmation":value(ChainInferenceOutcome::NoConfirmation),"Ambiguous":value(ChainInferenceOutcome::Ambiguous),"BudgetExhausted":value(ChainInferenceOutcome::BudgetExhausted)})
}

#[test]
fn native_live_chain_budget_exhaustion() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/live-chain-budget-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 8);
    let mut exhausted = 0;
    for case in cases {
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
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            ..Default::default()
        };
        cfg.context.profile.grammar.chain_inference = true;
        cfg.context.profile.grammar.view_tables = false;
        cfg.context.profile.grammar.calibrated_widths = Some(NativeSharedWidths::from_map(
            serde_json::from_value(case["widths"].clone()).unwrap(),
        ));
        let observer = NativeFilmObserver::default();
        cfg.context.observer = Some(observer.clone());
        let mut world = FilmWorld::default();
        let frame = cfg
            .decode_inference_view(&data, 0, &registry, &mut world)
            .unwrap();
        assert_eq!(json!(frame.end_bit), case["end"]);
        assert_eq!(json!(frame.hit_end()), case["hit"]);
        assert_eq!(json!(frame.inferred_count), case["inferred"]);
        assert_eq!(chain_counts(&observer), case["counts"]);
        assert_eq!(
            frame.records.len(),
            case["records"].as_array().unwrap().len()
        );
        for (record, expected) in frame
            .records
            .iter()
            .zip(case["records"].as_array().unwrap())
        {
            assert_eq!(record.header.kind, RecordKind::Delta);
            assert_eq!(json!(record.header.id), expected["ID"]);
            assert_eq!(json!(record.header.start_bit), expected["HeaderBit"]);
            assert!(record.archetype.is_none());
            assert!(record.decoded.is_none());
        }
        assert!(
            world.slots.is_empty(),
            "failed inference must not fabricate a binding"
        );
        let count = observer
            .counters()
            .chain_outcomes
            .get(&ChainInferenceOutcome::BudgetExhausted)
            .copied()
            .unwrap_or(0);
        exhausted += count;
        assert_eq!(
            frame.diagnostics.chain_outcomes,
            observer.counters().chain_outcomes
        );
        let restored: InferenceFrame = serde_json::from_value(json!(frame)).unwrap();
        assert_eq!(restored, frame);
    }
    assert_eq!(
        exhausted, 3,
        "normal-budget positive cases must remain exercised"
    );
}
