use super::*;
use serde_json::Value;
use std::{
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
#[test]
fn native_context_managed_properties() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/context-managed-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 128);
    let mut accepted = 0;
    let mut broken = 0;
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
        let mut context = NativeFilmContext::new(Some(&source));
        let mut profile = NativeScanProfile::default();
        profile.grammar.simulation_complete = row["simulation_complete"].as_bool().unwrap();
        context.set_scan_profile(profile).unwrap();
        let hooks = Arc::new(AtomicUsize::new(0));
        let captured = hooks.clone();
        context.observation().set_hook(
            NativeHookKind::ManagedProperty,
            Some(Arc::new(move |_| {
                captured.fetch_add(1, Ordering::SeqCst);
            })),
        );
        let (mut scan, error) = scan_context_managed_properties(&context);
        assert_eq!(
            error.map(|e| e.to_string()).unwrap_or_default(),
            row["error"].as_str().unwrap(),
            "error {case}"
        );
        assert_eq!(hooks.load(Ordering::SeqCst), 0, "private observer {case}");
        for attempt in &scan.attempts {
            let packet = attempt.source.unwrap();
            let (_, packets) = context
                .chunk_at(meta[packet.chunk_index as usize].index)
                .unwrap();
            assert_eq!(packets[attempt.packet_index.unwrap()], packet);
            assert_eq!(packet.timestamp_us, attempt.timestamp_us);
            assert_eq!(
                attempt.in_bounds,
                attempt.component.end_bit >= 0
                    && attempt.component.end_bit <= (packet.payload_size * 8) as i64
            );
        }
        accepted += scan.reads.len();
        broken += scan.broken;
        scan.attempts.clear();
        assert_eq!(
            scan,
            serde_json::from_value(row["scan"].clone()).unwrap(),
            "scan {case}"
        );
    }
    assert!(accepted > 0);
    assert!(broken > 0);
}
