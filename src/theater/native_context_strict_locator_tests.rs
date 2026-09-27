use super::*;
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, Mutex},
};

#[test]
fn native_context_strict_locator() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-strict-locator-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 256);
    let mut found = 0;
    let mut probes = 0;
    for row in rows {
        let c = row["case"].as_u64().unwrap() as usize;
        let registry = FilmRegistry {
            archetypes: (0..36)
                .map(|index| {
                    let components = match index {
                        3 => vec!["high-frequency".into()],
                        35 => vec!["unit-crouch-component".into()],
                        _ => vec![],
                    };
                    FilmArchetype {
                        index,
                        levels: vec![u32::from(index == 3); components.len()],
                        components,
                    }
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let mut cfg = NativeFrameConfig {
            id_low_bits: if c.is_multiple_of(7) { 11 } else { 13 },
            extra_fields: c.is_multiple_of(5),
            ..Default::default()
        };
        cfg.context.profile.grammar.generation_strict = c.is_multiple_of(2);
        cfg.context.profile.grammar.corruption_check = c.is_multiple_of(3);
        let unused = row["unused"].as_u64().unwrap();
        cfg.context.profile.movement.world_object.index_bits = unused;
        cfg.context.profile.movement.world_object.axis_bits = [unused; 3];
        let observer = NativeFilmObserver::default();
        cfg.context.observer = Some(observer.clone());
        let events = Arc::new(Mutex::new(Vec::new()));
        for kind in [NativeHookKind::Probe, NativeHookKind::MovementState] {
            let events = events.clone();
            observer.set_hook(
                kind,
                Some(Arc::new(move |p| {
                    let event = match p {
                        NativeHookPublication::Component(FilmComponentObservation::Probe {
                            archetype,
                            component,
                            values,
                        }) => json!(["probe", archetype, component, values]),
                        NativeHookPublication::Component(
                            FilmComponentObservation::MovementState {
                                component,
                                slot,
                                values,
                            },
                        ) => json!(["movement", component, slot, values]),
                        _ => panic!("unexpected hook"),
                    };
                    events.lock().unwrap().push(event);
                })),
            );
        }
        let mut world = FilmWorld::default();
        world.bind_full(0x8000007b, 3);
        world.bind_full(0x40000032, 35);
        if c.is_multiple_of(4) {
            world.unbind(123);
        }
        let before = world.clone();
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let at = cfg
            .locate_strict_entity_view(&data, &registry, &world)
            .unwrap();
        assert_eq!(
            json!(at.map(|v| v as i64).unwrap_or(-1)),
            row["offset"],
            "offset {c}"
        );
        assert_eq!(world, before, "world {c}");
        assert_eq!(
            json!(events.lock().unwrap().len()),
            row["during"],
            "trial count {c}"
        );
        found += usize::from(at.is_some());
        probes += events.lock().unwrap().len();
        let mut reader = NativeFilmReader::with_context(&[0, 0], cfg.context.clone());
        reader.set_capture_slot(99);
        reader
            .read_component("unit-crouch-component", 0, 35)
            .unwrap();
        assert_eq!(
            json!(*events.lock().unwrap()),
            row["events"],
            "hooks and restoration {c}"
        );
    }
    assert!(found > 0 && found < 256);
    assert!(probes > 0);
}
