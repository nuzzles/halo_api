use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_vehicle_channels() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-vehicle-channels-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 128);
    let (mut events_count, mut aims_count, mut refusals) = (0, 0, 0);
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            let h = c["hex"].as_str().unwrap();
            buffers.push(
                (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let context = NativeFilmContext::new(Some(&source));
        let (events, error) = scan_context_vehicle_events(&context);
        if row["panicked"] == true {
            assert!(
                matches!(
                    error,
                    Some(ContextVehicleChannelError::TruncatedReferences { .. })
                ),
                "panic refusal {case}"
            );
            refusals += 1;
        } else {
            assert_eq!(
                error.map(|e| e.to_string()).unwrap_or_default(),
                row["event_error"].as_str().unwrap(),
                "event error {case}"
            );
            assert_eq!(
                json!(events.iter().map(|r| &r.event).collect::<Vec<_>>()),
                row["events"],
                "events {case}"
            );
        }
        for r in &events {
            assert_eq!(meta[r.source.chunk_index as usize].index, r.event.chunk);
            assert_eq!(r.source.timestamp_us, r.event.timestamp_us);
        }
        let (aims, error) = scan_context_biped_aim(&context);
        assert_eq!(
            error.map(|e| e.to_string()).unwrap_or_default(),
            row["aim_error"].as_str().unwrap(),
            "aim error {case}"
        );
        assert_eq!(
            json!(aims.iter().map(|r| &r.record.aim).collect::<Vec<_>>()),
            row["aims"],
            "aims {case}"
        );
        for a in &aims {
            assert_eq!(
                meta[a.source.chunk_index as usize].index,
                a.record.aim.chunk
            );
            assert!(a.record.bit < a.record.primary_end_bit);
            assert!(a.record.primary_end_bit <= a.source.payload_size * 8);
            assert_eq!(
                a.record.companions.aim,
                Some([a.record.aim.yaw_raw, a.record.aim.pitch_raw])
            );
        }
        events_count += events.len();
        aims_count += aims.len();
    }
    assert!(events_count > 0 && aims_count > 0 && refusals > 0);
}
