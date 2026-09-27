//! Native callbacks mutate later hooks and clear counters during the same read.
use super::*;
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, Mutex},
};

#[test]
fn native_live_observer_mutation_and_counter_visibility() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/live-observer-v41.json.zlib")[..])
        .read_to_end(&mut bytes)
        .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(cases.len(), 256);
    let mut positions_with_histogram = 0;
    let mut enabled_references = 0;
    for case in cases {
        let i = case["case"].as_u64().unwrap() as usize;
        let data: Vec<_> = (0..i % 65).map(|j| (i * 71 + j * 37) as u8).collect();
        let mut context = NativeFilmContext::new(None);
        let mut profile = NativeScanProfile::default();
        profile.movement.full_precision = i.is_multiple_of(2);
        context.set_scan_profile(profile).unwrap();
        let observer = context.observation();
        let events = Arc::new(Mutex::new(Vec::<Value>::new()));
        let movement: NativeHook = {
            let events = events.clone();
            Arc::new(move |p| {
                let NativeHookPublication::Component(FilmComponentObservation::MovementState {
                    component,
                    slot,
                    values,
                }) = p
                else {
                    panic!("wrong callback")
                };
                events
                    .lock()
                    .unwrap()
                    .push(json!(["movement", component, slot, values]));
            })
        };
        let references: NativeHook = {
            let events = events.clone();
            Arc::new(move |p| {
                let NativeHookPublication::Component(FilmComponentObservation::UnitReference {
                    reference: r,
                }) = p
                else {
                    panic!("wrong callback")
                };
                events.lock().unwrap().push(json!([
                    "ref",
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
        if !i.is_multiple_of(2) {
            observer.set_hook(NativeHookKind::MovementState, Some(movement.clone()));
            observer.set_hook(NativeHookKind::UnitReference, Some(references.clone()));
        }
        observer.set_hook(
            NativeHookKind::MobilityAction,
            Some({
                let events = events.clone();
                let observer = observer.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::MobilityAction([a, b]) = p else {
                        panic!("wrong callback")
                    };
                    events.lock().unwrap().push(json!(["mobility", a, b]));
                    observer.set_hook(
                        NativeHookKind::MovementState,
                        i.is_multiple_of(2).then(|| movement.clone()),
                    );
                    observer.set_hook(
                        NativeHookKind::UnitReference,
                        i.is_multiple_of(2).then(|| references.clone()),
                    );
                })
            }),
        );
        observer.set_hook(
            NativeHookKind::Position,
            Some({
                let events = events.clone();
                let observer = observer.clone();
                Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::Position {
                        position_kind,
                        vector_bits,
                        bit,
                        slot,
                    }) = p
                    else {
                        panic!("wrong callback")
                    };
                    let hist = observer.take_absolute_indices();
                    events.lock().unwrap().push(json!([
                        "position",
                        position_kind.native_name(),
                        vector_bits,
                        bit,
                        slot,
                        hist
                    ]));
                })
            }),
        );
        let mut reader = context.reader(&data).unwrap();
        reader.set_capture_slot(19);
        reader.skip(i % 8).unwrap();
        for (j, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            let name = step["name"].as_str().unwrap();
            let (status, _) = reader.read_component(name, 0, 35).unwrap();
            assert_eq!(json!(status), step["ok"], "case {i} step {j} {name}");
            assert_eq!(
                json!(reader.bit_position()),
                step["end"],
                "case {i} step {j} {name}"
            );
            assert_eq!(
                json!(*events.lock().unwrap()),
                step["events"],
                "case {i} step {j} {name}"
            );
            assert_eq!(
                json!(observer.counters().absolute_indices),
                step["hist"],
                "case {i} step {j}"
            );
        }
        for event in events.lock().unwrap().iter() {
            if event[0] == "position" && !event[5].as_object().unwrap().is_empty() {
                positions_with_histogram += 1;
            }
            if event[0] == "ref" {
                enabled_references += 1;
                assert!(i.is_multiple_of(2));
            }
        }
        assert_eq!(json!(observer.take_absolute_indices()), case["last_hist"]);
        assert_eq!(json!(observer.take_absolute_indices()), case["empty_hist"]);
        // Release callbacks holding strong observer handles.
        observer.set_hook(NativeHookKind::MobilityAction, None);
        observer.set_hook(NativeHookKind::Position, None);
    }
    assert_eq!(positions_with_histogram, 143);
    assert_eq!(enabled_references, 83);
}
