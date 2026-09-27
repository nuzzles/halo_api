use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_context_production_views_and_cumulative_observers() {
    check_context(include_bytes!("fixtures/context-views-v41.json.zlib"));
}
#[test]
fn native_context_production_views_and_cumulative_observers_lazy_widths() {
    check_context(include_bytes!(
        "fixtures/context-views-lazy-widths-v41.json.zlib"
    ));
}
fn check_context(fixture: &[u8]) {
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
        let mut registry = registry.clone();
        if c.is_multiple_of(8) {
            registry.archetypes[3].components[2] = "simulation-state-component".into();
        }
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
            let width = case["unused_position_width"].as_u64().unwrap();
            let movement = &mut cfg.context.profile.movement;
            movement.world_object.index_bits = width;
            movement.world_object.axis_bits = [width; 3];
            movement.traversal.index_bits = width;
            movement.traversal.axis_bits = [width; 3];
            movement.delta_axis_width = width;
        }
        cfg.context.profile.grammar.generation_strict = !c.is_multiple_of(3);
        cfg.context.profile.grammar.corruption_check = c.is_multiple_of(4);
        cfg.context.profile.grammar.simulation_complete = c.is_multiple_of(16);
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
        let mut world = FilmWorld {
            current_view: 2,
            ..Default::default()
        };
        world.bind_full(8, 3);
        world.set_position(8, [1.0, 2.0, 3.0]);
        for (pass, expected) in case["passes"].as_array().unwrap().iter().enumerate() {
            let before = observer.counters();
            let view = cfg
                .decode_production_views(
                    &data,
                    case["start"].as_u64().unwrap() as usize,
                    &registry,
                    &mut world,
                )
                .unwrap();
            assert_eq!(json!(view.end_bit), expected["end"], "case {c} pass {pass}");
            assert_eq!(
                json!(view.views_completed),
                expected["views"],
                "case {c} pass {pass}"
            );
            assert_eq!(
                json!(*events.lock().unwrap()),
                expected["events"],
                "case {c} pass {pass}"
            );
            assert_eq!(
                json!(widths.get("weapon-state-rounds-inventory").unwrap()),
                expected["width"]
            );
            let counts = observer.counters();
            assert_eq!(json!(counts.rejected_unbound), expected["unbound"]);
            assert_eq!(json!(counts.rejected_other_view), expected["other_view"]);
            let local = view.admission_diagnostics.as_ref().unwrap();
            assert_eq!(
                local.rejected_unbound,
                counts.rejected_unbound - before.rejected_unbound
            );
            assert_eq!(
                local.rejected_other_view,
                counts.rejected_other_view - before.rejected_other_view
            );
            let records = expected["records"].as_array().unwrap();
            assert_eq!(view.records.len(), records.len(), "case {c} pass {pass}");
            for (r, e) in view.records.iter().zip(records) {
                assert_eq!(json!(r.header.id), e["ID"]);
                assert_eq!(r.stop == EntityViewStop::Complete, e["DesyncAt"] == -1);
                if r.header.kind != RecordKind::Delete {
                    assert_eq!(json!(r.end_bit), e["Trace"]["EndBit"]);
                    assert_eq!(json!(r.mask.unwrap_or(0)), e["Trace"]["Mask"]);
                }
                let comps = e["Trace"]["Comps"].as_array().cloned().unwrap_or_default();
                assert_eq!(r.attempts.len(), comps.len());
                for (a, b) in r.attempts.iter().zip(comps) {
                    assert_eq!(json!(a.span.start_bit), b["StartBit"]);
                    assert_eq!(json!(a.span.name), b["Name"]);
                    assert_eq!(json!(a.status), b["Ported"]);
                }
            }
            let slots:BTreeMap<_,_>=world.slots.iter().map(|(slot,s)|(slot.to_string(),json!({"TypeIndex":s.archetype,"FullID":s.full_id,"Soft":s.soft,"GenAny":s.generation_any,"Pos":s.position.unwrap_or([0.0;3]),"PosValid":s.position.is_some(),"Vue":s.view.unwrap_or(-1)}))).collect();
            let mut expected_slots = expected["slots"].clone();
            for slot in expected_slots.as_object_mut().unwrap().values_mut() {
                let pos: [f32; 3] = serde_json::from_value(slot["Pos"].clone()).unwrap();
                slot["Pos"] = json!(pos);
            }
            assert_eq!(json!(slots), expected_slots, "world case {c} pass {pass}");
            assert_eq!(
                serde_json::from_value::<ProductionFrame>(serde_json::to_value(&view).unwrap())
                    .unwrap(),
                view
            );
        }
    }
}
