//! Seeded traversal worlds must not silently become fresh readers' accumulators.
use super::*;
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, Mutex},
};

fn fixture(bytes: &[u8]) -> Value {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(bytes)
        .read_to_end(&mut raw)
        .unwrap();
    serde_json::from_slice(&raw).unwrap()
}
fn payload(row: &Value) -> Vec<u8> {
    let s = row["hex"].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn registry(names: Vec<Vec<String>>, level: u32) -> FilmRegistry {
    FilmRegistry {
        archetypes: names
            .into_iter()
            .enumerate()
            .map(|(index, components)| FilmArchetype {
                index,
                levels: vec![level; components.len()],
                components,
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    }
}
fn observe(cfg: &mut NativeFrameConfig) -> Arc<Mutex<Vec<FilmComponentObservation>>> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let observer = NativeFilmObserver::default();
    let captured = events.clone();
    observer.set_hook(
        NativeHookKind::Position,
        Some(Arc::new(move |event| {
            let NativeHookPublication::Component(value) = event else {
                panic!("position callback")
            };
            captured.lock().unwrap().push(value.clone());
        })),
    );
    cfg.context.observer = Some(observer);
    events
}
#[test]
fn native_reader_lifecycle_inference_preserves_seeded_world_and_callback_order() {
    let rows = fixture(include_bytes!(
        "fixtures/position-hook-inference-v41.json.zlib"
    ));
    assert_eq!(rows.as_array().unwrap().len(), 512);
    let mut position_callbacks = 0;
    for (i, row) in rows.as_array().unwrap().iter().enumerate() {
        let mut names = vec![vec![]; 36];
        names[35] = vec![row["name"].as_str().unwrap().into()];
        let registry = registry(names, row["level"].as_u64().unwrap() as u32);
        let data = payload(row);
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            ..Default::default()
        };
        cfg.context.profile.grammar.view_tables = row["tables"].as_bool().unwrap();
        cfg.context.profile.grammar.chain_inference = row["chain"].as_bool().unwrap();
        cfg.context.profile.movement.full_precision =
            row["encoding"]["full_precision"].as_bool().unwrap();
        let events = observe(&mut cfg);
        let mut world = FilmWorld::default();
        world.bind_full(50, 35);
        world.set_position(50, [50., i as f32, -1.]);
        if !row["unbound"].as_bool().unwrap() {
            world.bind_full(51, 35);
            world.set_position(51, [51., i as f32, -2.]);
        }
        for (pass, expected) in row["lifecycle"].as_array().unwrap().iter().enumerate() {
            let frame = cfg
                .decode_inference_view(&data, 13, &registry, &mut world)
                .unwrap();
            let records: Vec<_> = frame
                .records
                .iter()
                .map(|r| {
                    json!({
                        "id":r.header.id.unwrap(), "end":r.decoded.as_ref().map_or(0,|d|d.end_bit),
                    })
                })
                .collect();
            assert_eq!(json!(records), expected["records"], "records {i}/{pass}");
            assert_eq!(json!(frame.end_bit), expected["end"], "end {i}/{pass}");
            assert_eq!(
                json!(frame.inferred_count),
                expected["inferred"],
                "inferred {i}/{pass}"
            );
            assert_eq!(json!(frame.hit_end()), expected["hit"], "hit {i}/{pass}");
            assert_eq!(
                json!(*events.lock().unwrap()),
                expected["positions"],
                "callbacks {i}/{pass}"
            );
            native_frame_accumulator_tests::check_world(
                &world,
                &expected["slots"],
                &format!("inference {i}/{pass}"),
            );
            assert_eq!(
                serde_json::from_value::<InferenceFrame>(json!(frame)).unwrap(),
                frame
            );
        }
        position_callbacks += events.lock().unwrap().len();
    }
    assert!(position_callbacks > 0);
}
#[test]
fn native_reader_lifecycle_resync_preserves_seeded_world_and_callback_order() {
    let rows = fixture(include_bytes!(
        "fixtures/context-resync-frame-v41.json.zlib"
    ));
    assert_eq!(rows.as_array().unwrap().len(), 128);
    let mut position_callbacks = 0;
    let mut accepted_records = 0;
    for row in rows.as_array().unwrap() {
        let i = row["case"].as_u64().unwrap() as usize;
        let registry = registry(serde_json::from_value(row["names"].clone()).unwrap(), 0);
        let data = payload(row);
        let mut cfg = NativeFrameConfig {
            id_low_bits: 11,
            extra_fields: i.is_multiple_of(4),
            ..Default::default()
        };
        cfg.context.profile.grammar.chain_inference = true;
        cfg.context.profile.grammar.default_state_by_archetype = false;
        cfg.context.profile.grammar.view_tables = i.is_multiple_of(5);
        cfg.context.profile.grammar.mobility_action_body = false;
        let events = observe(&mut cfg);
        let mut world = FilmWorld::default();
        if i.is_multiple_of(11) {
            world.bind_soft(50, 0)
        } else {
            world.bind_full(50, 0)
        }
        world.set_position(50, [50., i as f32, -1.]);
        let mut calls = Vec::new();
        for (pass, expected) in row["lifecycle"].as_array().unwrap().iter().enumerate() {
            let frame = cfg
                .decode_resync_frame(
                    &data,
                    &registry,
                    &mut world,
                    &[50].into(),
                    |slot, pos, has| {
                        calls.push(json!([slot, pos.map(f32::to_bits), has]));
                        true
                    },
                )
                .unwrap();
            let records = expected["records"].as_array().unwrap();
            assert_eq!(frame.records.len(), records.len(), "records {i}/{pass}");
            accepted_records += frame.records.len();
            for (actual, expected) in frame.records.iter().zip(records) {
                assert_eq!(json!(actual.header.id), expected["ID"]);
                assert_eq!(json!(actual.archetype.unwrap_or(0)), expected["TypeIndex"]);
                assert_eq!(json!(actual.mask.unwrap_or(0)), expected["Trace"]["Mask"]);
                if actual.header.kind != RecordKind::Delete {
                    assert_eq!(json!(actual.end_bit), expected["Trace"]["EndBit"]);
                }
            }
            assert_eq!(
                json!(calls),
                expected["calls"],
                "accept callback {i}/{pass}"
            );
            assert_eq!(
                json!(*events.lock().unwrap()),
                expected["positions"],
                "callbacks {i}/{pass}"
            );
            native_frame_accumulator_tests::check_world(
                &world,
                &expected["slots"],
                &format!("resync {i}/{pass}"),
            );
            assert_eq!(
                serde_json::from_value::<NativeResyncFrame>(json!(frame)).unwrap(),
                frame
            );
        }
        position_callbacks += events.lock().unwrap().len();
    }
    assert!(position_callbacks > 0 && accepted_records > 0);
}
