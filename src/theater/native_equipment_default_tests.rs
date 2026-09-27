use super::*;
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_equipment_defaults() {
    check_defaults(
        include_bytes!("fixtures/equipment-default-native-v41.json.zlib"),
        37,
    );
}
#[test]
fn native_ground_weapon_defaults() {
    check_defaults(
        include_bytes!("fixtures/ground-default-native-v41.json.zlib"),
        42,
    );
}
#[test]
fn native_vehicle_defaults() {
    check_defaults(
        include_bytes!("fixtures/vehicle-default-native-v41.json.zlib"),
        40,
    );
}
fn check_defaults(fixture: &[u8], archetype: u32) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 384);
    let mut padded = 0;
    for (case, row) in rows.iter().enumerate() {
        let h = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect();
        let mut profile = NativeScanProfile::default();
        profile.mpp.lead = row["lead"].as_i64().unwrap();
        profile.mpp.index = row["index"].as_i64().unwrap();
        if archetype == 40 {
            profile.movement.world_object.index_bits = row["world_index"].as_u64().unwrap();
            profile.movement.world_object.axis_bits =
                serde_json::from_value(row["axes"].clone()).unwrap();
            profile.movement.full_precision = row["full_precision"] == true;
            profile.grammar.baseline_scope = row["baseline"] == true;
        }
        let observer = NativeFilmObserver::default();
        let events = Arc::new(Mutex::new(Vec::new()));
        for kind in [NativeHookKind::EquipmentCreation, NativeHookKind::Mpp] {
            let events = events.clone();
            observer.set_hook(
                kind,
                Some(Arc::new(move |p| {
                    if let NativeHookPublication::Component(v) = p {
                        events.lock().unwrap().push(v.clone());
                    }
                })),
            );
        }
        let mut reader = NativeFilmReader::with_context(
            &data,
            NativeReaderContext {
                profile,
                observer: Some(observer),
            },
        );
        reader.set_native_bit_position(row["start"].as_i64().unwrap());
        let (status, read) = if archetype == 37 {
            reader.read_equipment_default_state()
        } else if archetype == 40 {
            reader.read_vehicle_default_state()
        } else {
            reader.read_ground_weapon_default_state()
        }
        .unwrap();
        assert_eq!(status, Some(true));
        assert_eq!(
            json!(reader.native_bit_position()),
            row["end"],
            "cursor {case}"
        );
        assert_eq!(read.end_bit, reader.native_bit_position());
        let published = events.lock().unwrap();
        let projected:Vec<_>=published.iter().map(|v|match v {
            FilmComponentObservation::Mpp{field,value,present}=>json!({"kind":"mpp","field":*field as u32,"value":value,"present":present}),
            FilmComponentObservation::EquipmentCreation{field,value,present}=>json!({"kind":"equipment","field":*field as u32,"value":value,"present":present}),
            _=>panic!("unexpected hook"),
        }).collect();
        assert_eq!(json!(projected), row["events"], "hooks {case}");
        assert_eq!(
            *published,
            read.diagnostics
                .component_observations
                .iter()
                .filter(|v| matches!(
                    v,
                    FilmComponentObservation::Mpp { .. }
                        | FilmComponentObservation::EquipmentCreation { .. }
                ))
                .cloned()
                .collect::<Vec<_>>(),
            "retained hooks {case}"
        );
        padded += usize::from(read.end_bit > data.len() as i64 * 8);
        for f in &read.fields {
            if f.width <= 64 && f.bit >= 0 && f.bit as u64 + f.width <= data.len() as u64 * 8 {
                let expected = (f.bit as usize..f.bit as usize + f.width as usize)
                    .fold(0u64, |v, b| {
                        (v << 1) | u64::from((data[b / 8] >> (7 - b % 8)) & 1)
                    });
                assert_eq!(f.raw, expected, "source field {case} {}", f.name);
            }
        }
    }
    assert!(padded > 0);
}
