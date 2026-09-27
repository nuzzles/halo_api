use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};

pub(crate) fn check_world(world: &FilmWorld, expected: &Value, label: &str) {
    let slots: BTreeMap<_, _> = world.slots.iter().map(|(slot,s)|(slot.to_string(),json!({"TypeIndex":s.archetype,"FullID":s.full_id,"Soft":s.soft,"GenAny":s.generation_any,"Pos":s.position.unwrap_or([0.;3]).map(f32::to_bits),"PosValid":s.position.is_some(),"Vue":s.view.unwrap_or(-1)}))).collect();
    let mut expected = expected.clone();
    for slot in expected.as_object_mut().unwrap().values_mut() {
        let position: [f32; 3] = serde_json::from_value(slot["Pos"].clone()).unwrap();
        slot["Pos"] = json!(position.map(f32::to_bits));
    }
    assert_eq!(json!(slots), expected, "{label}");
}
#[test]
fn native_frame_accumulator_matches_reference() {
    check_frames(
        include_bytes!("fixtures/position-accumulator-generic-v41.json.zlib"),
        false,
    );
}
#[test]
fn native_frame_shared_accumulator_matches_reference() {
    check_frames(
        include_bytes!("fixtures/position-accumulator-shared-v41.json.zlib"),
        true,
    );
}
fn check_frames(fixture: &[u8], shared: bool) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 512);
    let mut records_count = 0;
    let mut publications_count = 0;
    for (i, c) in cases.into_iter().enumerate() {
        let registry = FilmRegistry {
            archetypes: (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 35 {
                        vec![c["name"].as_str().unwrap().into()]
                    } else {
                        vec![]
                    },
                    levels: if index == 35 {
                        vec![c["level"].as_u64().unwrap() as u32]
                    } else {
                        vec![]
                    },
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let hex = c["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            ..Default::default()
        };
        cfg.context.profile.grammar.generation_strict = true;
        cfg.context.profile.movement.full_precision =
            c["encoding"]["full_precision"].as_bool().unwrap();
        let observer = NativeFilmObserver::default();
        let events = Arc::new(Mutex::new(Vec::<FilmComponentObservation>::new()));
        for kind in [
            NativeHookKind::Position,
            NativeHookKind::UnitReference,
            NativeHookKind::Mpp,
            NativeHookKind::MovementState,
        ] {
            let events = events.clone();
            observer.set_hook(
                kind,
                Some(Arc::new(move |p| {
                    let NativeHookPublication::Component(o) = p else {
                        panic!("component callback")
                    };
                    events.lock().unwrap().push(o.clone());
                })),
            );
        }
        cfg.context.observer = Some(observer);
        let mut world = FilmWorld::default();
        world.bind_full(50, 35);
        if !c["unbound"].as_bool().unwrap() {
            world.bind_full(51, 35);
        }
        let mut accumulator = FilmWorld::default();
        for slot in [0, 50, 51] {
            accumulator.bind_full(slot, 35);
            accumulator.set_position(slot, [slot as f32, 2., 3.]);
        }
        let mut reader = NativeFilmReader::new(&data);
        reader.set_bit_position(13);
        if shared {
            world = accumulator.clone();
        }
        let accumulate = c["accumulate"].as_bool().unwrap();
        if accumulate && !shared {
            reader.replace_position_accumulator(Some(&mut accumulator));
            let before = world.clone();
            assert!(matches!(
                reader.read_frame_records_accumulating(&registry, &mut world, &cfg),
                Err(NativeReaderProfileError::Policy(
                    "two position accumulator worlds"
                ))
            ));
            assert_eq!(world, before);
            assert_eq!(reader.bit_position(), 13);
            assert!(events.lock().unwrap().is_empty());
        }
        let view = if accumulate && shared {
            reader.read_frame_records_accumulating(&registry, &mut world, &cfg)
        } else {
            reader.read_frame_records(&registry, &mut world, &cfg)
        }
        .unwrap();
        assert_eq!(json!(reader.bit_position()), c["end"], "cursor {i}");
        assert_eq!(
            json!(view.stop == EntityViewStop::Complete),
            c["hit"],
            "stop {i}"
        );
        let records: Vec<_> = view
            .records
            .iter()
            .filter(|r| r.header.kind != RecordKind::End)
            .map(|r| json!({"id":r.header.id.unwrap(),"end":r.end_bit}))
            .collect();
        assert_eq!(json!(records), c["records"], "records {i}");
        let expected: Vec<FilmComponentObservation> =
            serde_json::from_value(c["observations"].clone()).unwrap();
        assert_eq!(*events.lock().unwrap(), expected, "callbacks {i}");
        records_count += records.len();
        publications_count += expected.len();
        assert_eq!(
            reader.position_accumulator().is_some(),
            accumulate && !shared
        );
        reader.replace_position_accumulator(None);
        drop(reader);
        check_world(&world, &c["world"], &format!("traversal {i}"));
        check_world(
            if shared { &world } else { &accumulator },
            &c["accumulator"],
            &format!("accumulator {i}"),
        );
        let restored: DecodedEntityView =
            serde_json::from_slice(&serde_json::to_vec(&view).unwrap()).unwrap();
        assert_eq!(view, restored);
    }
    assert_eq!(records_count, if shared { 2560 } else { 2513 });
    assert_eq!(publications_count, if shared { 10475 } else { 10141 });
}
