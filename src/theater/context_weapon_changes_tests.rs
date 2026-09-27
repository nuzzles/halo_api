use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_held_weapons() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/context-weapons-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let decode = |h: &str| {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 128);
    let mut total = 0;
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = vec![decode(fixture["registry"].as_str().unwrap())];
        if row["no_chunks"] != true {
            buffers.push(decode(row["data"].as_str().unwrap()));
        }
        let meta: Vec<_> = (0..buffers.len())
            .map(|i| FilmSourceMetadata {
                index: i as i64,
                chunk_type: 0,
                start_ms: 0,
            })
            .collect();
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let layout = I0Layout {
            gate_bits: 5,
            axis_widths: [8, 9, 10],
            region: 0,
        };
        let context = NativeFilmContext::for_map(Some(&source), None, Some(&layout)).unwrap();
        // Native scanner owns its observer; installed context hooks must not run.
        context.observation().set_hook(
            NativeHookKind::HeldWeapon,
            Some(std::sync::Arc::new(|_| panic!("context observer leaked"))),
        );
        let loadouts = if row["spawn"] == true {
            vec![KeyframeLoadout {
                timestamp_us: 2000,
                chunk: 1,
                packet_index: 0,
                slot: 512,
                families: vec![100],
            }]
        } else {
            vec![]
        };
        let (out, error) = scan_context_held_weapon_changes(&context, &loadouts);
        assert_eq!(
            error.map(|e| e.to_string()).unwrap_or_default(),
            row["error"].as_str().unwrap(),
            "case {case}"
        );
        assert_eq!(json!(out.stats), row["stats"], "case {case}");
        assert_eq!(json!(out.records), row["records"], "case {case}");
        let attempts:Vec<_>=out.component_attempts.iter().map(|a|json!({"time":a.read.source.timestamp_us,"id":a.component_index,"start":a.read.component.start_bit,"end":a.read.component.end_bit,"status":a.status.unwrap_or(false),"in_bounds":a.in_bounds})).collect();
        assert_eq!(json!(attempts), row["attempts"], "case {case}");
        for c in &out.records {
            let f = FactsWeaponChange::from(c);
            assert_eq!(f.timestamp_us, c.timestamp_us);
            assert_eq!(f.slot, c.slot);
            assert_eq!(f.slot_index, i64::from(c.slot_index));
            assert_eq!(f.family, c.family);
            assert_eq!(f.previous, c.previous);
            assert_eq!(json!(String::from_utf8(f.kind).unwrap()), json!(c.kind));
        }
        total += out.records.len();
    }
    assert!(total > 100);
}
