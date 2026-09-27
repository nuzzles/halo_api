use super::*;
use serde_json::Value;
use std::{
    io::Read,
    sync::{Arc, Mutex},
};
#[test]
fn native_ignored_mobility_extra_metadata() {
    check_fixture(
        include_bytes!("fixtures/mobility-extra-policy-v41.json.zlib"),
        128,
        96,
        0,
    );
}
#[test]
fn native_large_mobility_extra_metadata() {
    check_fixture(
        include_bytes!("fixtures/mobility-extra-large-v41.json.zlib"),
        208,
        156,
        4,
    );
}
fn check_fixture(fixture: &[u8], cases: usize, callbacks: usize, negative_ends: usize) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), cases);
    let mut count = 0;
    let mut wrapped = 0;
    for (i, row) in rows.into_iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
            .collect();
        let mut profile = NativeScanProfile::default();
        profile.movement.mobility_action_extra_bits = row["width"].as_i64().unwrap();
        profile.grammar.mobility_action_body = row["body"].as_bool().unwrap();
        profile.grammar.simulation_complete = true;
        let observer = NativeFilmObserver::default();
        let events = Arc::new(Mutex::new(Vec::<FilmComponentObservation>::new()));
        for kind in [NativeHookKind::MovementState, NativeHookKind::UnitReference] {
            let events = events.clone();
            observer.set_hook(
                kind,
                Some(Arc::new(move |p| {
                    let NativeHookPublication::Component(o) = p else {
                        panic!("component")
                    };
                    events.lock().unwrap().push(o.clone());
                })),
            );
        }
        let mut reader = NativeFilmReader::with_context(
            &data,
            NativeReaderContext {
                profile: profile.clone(),
                observer: Some(observer),
            },
        );
        reader.set_capture_slot(51);
        let (status, read) = reader
            .read_component(row["name"].as_str().unwrap(), 0, 35)
            .unwrap();
        if row["end"].as_i64().unwrap() < 0 {
            wrapped += 1;
            assert_eq!(status == Some(true), row["ported"].as_bool().unwrap());
            let adjustment = read.diagnostics.width_adjustments.last().unwrap();
            assert_eq!(adjustment.purpose, Some(NativeWidthPurpose::MobilityExtra));
            assert!(adjustment.end_bit.is_none());
            assert_eq!(serde_json::json!(reader.native_bit_position()), row["end"]);
        } else {
            assert_eq!(
                status == Some(true),
                row["ported"].as_bool().unwrap(),
                "status {i}"
            );
            assert_eq!(
                reader.native_bit_position(),
                row["end"].as_i64().unwrap(),
                "end {i}"
            );
        }
        for adjustment in &read.diagnostics.width_adjustments {
            assert_eq!(
                adjustment.native_end_bit(),
                row["end"].as_i64(),
                "native endpoint case {i}"
            );
            if let Some(retained) = adjustment.retained_bits {
                assert_eq!(
                    serde_json::json!(retained),
                    serde_json::json!((data.len() * 8) as i64 - adjustment.bit)
                );
                let fields: Vec<_> = read
                    .fields
                    .iter()
                    .filter(|f| f.name.starts_with("mobility.skipped"))
                    .collect();
                assert!(fields.len() <= data.len() * 8 / 64 + 1);
                assert_eq!(
                    fields
                        .iter()
                        .map(|f| usize::try_from(f.width).unwrap())
                        .sum::<usize>(),
                    retained
                );
                for field in fields {
                    assert_eq!(
                        serde_json::json!(super::bits::Bits(&data).read(field.bit, field.width)),
                        serde_json::json!(Some(field.raw))
                    );
                }
            }
        }
        let expected: Vec<FilmComponentObservation> =
            serde_json::from_value(row["observations"].clone()).unwrap();
        assert_eq!(*events.lock().unwrap(), expected, "callbacks {i}");
        count += expected.len();
        assert_eq!(reader.context().profile, profile, "raw metadata {i}");
        let restored: DecodedComponent =
            serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap();
        assert_eq!(restored, read);
    }
    assert_eq!(count, callbacks);
    assert_eq!(wrapped, negative_ends);
}
