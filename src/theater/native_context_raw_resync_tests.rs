use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_context_raw_resync_copy_and_acceptance() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-raw-resync-v41.json.zlib")[..],
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
        let before = world.clone();
        let mut calls = Vec::new();
        for expected in case["landings"].as_array().unwrap() {
            let attempt = cfg
                .scan_for_target_delta(
                    &data,
                    c % 13,
                    &registry,
                    &world,
                    &[50].into(),
                    |slot, pos, has| {
                        calls.push(json!([slot, pos.map(f32::to_bits), has]));
                        c % 3 != 2 || calls.len().is_multiple_of(2)
                    },
                )
                .unwrap();
            assert_eq!(
                attempt.result.map(|r| r.0).unwrap_or(-1),
                expected.as_i64().unwrap(),
                "landing case {c}"
            );
        }
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
