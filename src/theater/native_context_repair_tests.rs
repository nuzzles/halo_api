use super::*;
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_context_repair_scopes_counters_and_restoration() {
    check_context(include_bytes!("fixtures/context-repair-v41.json.zlib"));
}
#[test]
fn native_context_repair_scopes_counters_and_restoration_lazy_widths() {
    check_context(include_bytes!(
        "fixtures/context-repair-lazy-widths-v41.json.zlib"
    ));
}
fn check_context(fixture: &[u8]) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 64);
    for case in cases {
        let c = case["case"].as_u64().unwrap() as usize;
        let names: Vec<Vec<String>> = vec![
            serde_json::from_value(case["names"].clone()).unwrap(),
            vec![],
        ];
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
            extra_fields: false,
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
        cfg.context.profile.grammar.default_state_by_archetype = false;
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
                        .push(json!(["emp", quantum, repair_counts(&observer)]));
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
        world.bind_full(50, 0);
        world.bind_full(60, 1);
        let before = world.clone();
        let widths = NativeSharedWidths::from_map([("unrelated".into(), 7)].into());
        if c.is_multiple_of(11) {
            widths.insert("not-ported".into(), (c % 2 * 3) as i64);
        }
        cfg.context.profile.grammar.stub_widths = Some(widths.clone());
        let mut initial = cfg.clone();
        initial.context.observer = None;
        initial.context.profile.grammar.stub_widths = None;
        initial.packet_preamble_bits = 0;
        let failed = initial
            .reader(&data)
            .read_frame_records(&registry, &mut world.clone(), &initial)
            .unwrap()
            .records
            .remove(0);
        for expected in case["repairs"].as_array().unwrap() {
            let attempt = cfg
                .repair_component(&data, &failed, &registry, &world)
                .unwrap();
            assert_eq!(
                attempt.result.is_some(),
                expected["ok"].as_bool().unwrap(),
                "repair case {c}"
            );
            if let Some(repair) = attempt.result {
                assert_eq!(
                    json!(repair.record.end_bit),
                    expected["end"],
                    "end case {c}"
                );
                assert_eq!(
                    json!(repair.matching_widths),
                    expected["widths"],
                    "widths case {c}"
                );
            }
        }
        assert_eq!(
            json!(*events.lock().unwrap()),
            case["frame_events"],
            "case {c}"
        );
        assert_eq!(repair_counts(&observer), case["counts"], "counts case {c}");
        assert_eq!(
            json!(widths.snapshot()),
            case["stubs"],
            "caller stubs case {c}"
        );
        assert_eq!(world, before);
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
        observer.set_hook(NativeHookKind::MobilityAction, None);
        observer.set_hook(NativeHookKind::EmpTimer, None);
    }
}

fn repair_counts(observer: &NativeFilmObserver) -> Value {
    let c = observer.counters();
    let value = |kind| c.chain_outcomes.get(&kind).copied().unwrap_or(0);
    let widths = json!(c.component_widths);
    json!({"repaired":c.repaired_records,"widths":widths,"none":value(ChainInferenceOutcome::NoConfirmation),"ambiguous":value(ChainInferenceOutcome::Ambiguous),"budget":value(ChainInferenceOutcome::BudgetExhausted)})
}
