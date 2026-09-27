use super::*;
use serde_json::Value;
use std::io::Read;

fn fixture(bytes: &[u8]) -> Vec<Value> {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(bytes)
        .read_to_end(&mut raw)
        .unwrap();
    serde_json::from_slice(&raw).unwrap()
}
fn source(inputs: &Value) -> Option<FilmSource> {
    let mut chunks = Vec::new();
    let mut metadata = Vec::new();
    for input in inputs.as_array().unwrap() {
        let hex = input["hex"].as_str().unwrap();
        chunks.push(
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>(),
        );
        metadata.push(FilmSourceMetadata {
            index: input["index"].as_i64().unwrap(),
            chunk_type: 2,
            start_ms: 98765,
        });
    }
    match FilmSource::load(&chunks, &metadata) {
        Ok(source) => Some(source),
        Err(FilmSourceError::Empty) => None,
        Err(error) => panic!("unexpected fixture load failure: {error}"),
    }
}
#[test]
fn native_loaded_source_events() {
    let rows = fixture(include_bytes!(
        "fixtures/fire-scanner-d61443e-v41.json.zlib"
    ));
    assert_eq!(rows.len(), 48);
    let mut fire_count = 0;
    for (i, row) in rows.iter().enumerate() {
        let source = source(&row["chunks"]);
        let result = scan_source_fire_events(source.as_ref());
        assert_eq!(result.is_err(), row["error"].as_bool().unwrap(), "fire {i}");
        if let Ok(events) = result {
            let source = source.as_ref().unwrap();
            let expected: Vec<FilmFireEvent> = serde_json::from_value(if row["events"].is_null() {
                serde_json::json!([])
            } else {
                row["events"].clone()
            })
            .unwrap();
            assert_eq!(events.len(), expected.len());
            fire_count += events.len();
            for (event, expected) in events.iter().zip(expected) {
                assert_eq!(event.event, expected);
                let (data, packets) = source.chunk_by_number(event.event.chunk).unwrap();
                assert_eq!(event.source, packets[event.event.packet_index]);
                let payload = source.payload(&event.source).unwrap();
                assert_eq!(
                    payload,
                    &data[event.source.payload_offset
                        ..event.source.payload_offset + event.source.payload_size]
                );
                let fact = FactsFireEvent::from(&event.event);
                assert_eq!(fact.film_index, i64::from(expected.film_index));
                assert_eq!(fact.timestamp_us, expected.timestamp_us);
                assert_eq!(fact.weapon_id, expected.weapon_id);
                assert_eq!(fact.has_aim, expected.has_aim);
                assert_eq!(fact.aim, expected.aim);
            }
        }
    }
    assert_eq!(fire_count, 35);
    let rows = fixture(include_bytes!("fixtures/translocator-source-v41.json.zlib"));
    assert_eq!(rows.len(), 128);
    let (mut count, mut absent, mut padded) = (0, 0, 0);
    for (i, row) in rows.iter().enumerate() {
        let source = source(&row["inputs"]);
        let events = scan_source_translocator_events(source.as_ref(), None);
        let expected = row["output"].as_array().unwrap();
        assert_eq!(events.len(), expected.len(), "teleports {i}");
        for (actual, expected) in events.iter().zip(expected) {
            let source = source.as_ref().unwrap();
            count += 1;
            padded += usize::from(actual.read.padded_bits > 0);
            let number = expected["chunk"].as_i64().unwrap();
            assert_eq!(
                source.chunk_position(number).unwrap(),
                actual.source.chunk_index as usize
            );
            assert_eq!(
                actual.packet_index as u64,
                expected["packet"].as_u64().unwrap()
            );
            assert_eq!(
                actual.source.payload_offset as u64,
                expected["start"].as_u64().unwrap()
            );
            assert_eq!(
                actual.source.payload_size as u64,
                expected["size"].as_u64().unwrap()
            );
            let payload = source.payload(&actual.source).unwrap();
            assert_eq!(actual.read.source_bits, payload.len() * 8);
            let e = &expected["event"];
            let fact = FactsTranslocation::from(actual);
            assert_eq!(fact.has_positions, e["HasPositions"].as_bool().unwrap());
            assert_eq!(fact.timestamp_us, e["TimestampUS"].as_u64().unwrap());
            assert_eq!(u64::from(fact.slot), e["Slot"].as_u64().unwrap());
            for (p, name) in [fact.from, fact.to].iter().zip(["From", "To"]) {
                for (axis, v) in p.iter().enumerate() {
                    assert_eq!(
                        v.to_bits(),
                        (e[name][axis].as_f64().unwrap() as f32).to_bits()
                    );
                }
            }
            assert_eq!(
                actual.source.timestamp_us,
                e["TimestampUS"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(actual.read.event.slot),
                e["Slot"].as_u64().unwrap()
            );
            let positions = actual.read.event.positions();
            assert_eq!(positions.is_some(), e["HasPositions"].as_bool().unwrap());
            if let Some(positions) = positions {
                for (p, name) in positions.iter().zip(["From", "To"]) {
                    for (axis, v) in p.iter().enumerate() {
                        assert_eq!(
                            v.to_bits(),
                            (e[name][axis].as_f64().unwrap() as f32).to_bits()
                        );
                    }
                }
            } else {
                absent += 1;
            }
        }
    }
    assert_eq!(count, 1000);
    assert!(absent > 0 && padded > 0);
}
