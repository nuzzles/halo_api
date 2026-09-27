use super::*;
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_shallow_observer_copy() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/observer-copy-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 16);
    let anticipated = |o: &NativeFilmObserver, ti| {
        o.record_admission(&FilmViewAdmission::Anticipated(AnticipatedDeclaration {
            chunk_index: 0,
            archetype: ti,
        }))
    };
    let snapshot = |o: &NativeFilmObserver| {
        let d = o.counters();
        json!({"absolute":d.absolute_indices,"anticipated":d.anticipated_bindings,"widths":d.component_widths,"repaired":d.repaired_records,"resync":d.validated_resyncs})
    };
    for mut row in rows {
        let c = row["case"].as_u64().unwrap();
        let a = NativeFilmObserver::default();
        if c & 1 != 0 {
            a.record_absolute(7)
        };
        if c & 2 != 0 {
            a.take_absolute_indices();
        };
        if c & 4 != 0 {
            anticipated(&a, 35)
        };
        a.record_validated_resync();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let hook = |tag| -> NativeHook {
            let calls = calls.clone();
            Arc::new(move |_| calls.lock().unwrap().push(tag))
        };
        a.set_hook(NativeHookKind::EmpTimer, Some(hook("old")));
        let b = a.shallow_copy();
        assert!(!a.same_instance(&b));
        assert!(a.same_instance(&a.clone()));
        let mut snapshots = vec![json!([snapshot(&a), snapshot(&b)])];
        b.record_absolute(9);
        anticipated(&b, 36);
        b.record_repair("unknown", &[0, 7]);
        b.record_validated_resync();
        snapshots.push(json!([snapshot(&a), snapshot(&b)]));
        a.set_hook(NativeHookKind::EmpTimer, Some(hook("new")));
        let value = FilmComponentObservation::EmpTimer { quantum: 0 };
        a.publish(NativeHookPublication::Component(&value));
        b.publish(NativeHookPublication::Component(&value));
        let drained = a.take_absolute_indices();
        b.record_absolute(10);
        a.record_absolute(11);
        snapshots.push(json!([snapshot(&a), snapshot(&b)]));
        if c & 8 != 0 {
            b.take_absolute_indices();
        };
        a.record_repair("unknown", &[7]);
        anticipated(&a, 37);
        snapshots.push(json!([snapshot(&a), snapshot(&b)]));
        // Nil versus allocated-empty is tested by subsequent sharing behavior, not JSON shape.
        for snap in row["snapshots"].as_array_mut().unwrap() {
            for o in snap.as_array_mut().unwrap() {
                for field in ["absolute", "anticipated"] {
                    if o[field].is_null() {
                        o[field] = json!({});
                    }
                }
            }
        }
        assert_eq!(json!(snapshots), row["snapshots"], "case {c}");
        assert_eq!(json!(drained), row["drained"], "drain {c}");
        assert_eq!(json!(*calls.lock().unwrap()), row["calls"]);
    }
}
