use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_inventory_deltas() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/context-inventory-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 48);
    let decode = |h: &str| {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    let mut count = 0;
    let mut refused = 0;
    for (case, row) in rows.iter().enumerate() {
        let buffers = vec![
            decode(fixture["registry"].as_str().unwrap()),
            decode(row["data"].as_str().unwrap()),
        ];
        let meta = vec![
            FilmSourceMetadata {
                index: 0,
                chunk_type: 0,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 1,
                chunk_type: 0,
                start_ms: 0,
            },
        ];
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let layout = I0Layout {
            gate_bits: 5,
            axis_widths: [8, 9, 10],
            region: 0,
        };
        let context = NativeFilmContext::for_map(Some(&source), None, Some(&layout)).unwrap();
        for kind in [
            NativeHookKind::GrenadeCounts,
            NativeHookKind::GrenadeSet,
            NativeHookKind::WeaponAmmo,
            NativeHookKind::WeaponRounds,
        ] {
            context.observation().set_hook(
                kind,
                Some(std::sync::Arc::new(|_| panic!("context hook leaked"))),
            );
        }
        let (out, error) = scan_context_inventory_deltas(&context);
        assert!(error.is_none(), "case {case}: {error:?}");
        assert_eq!(json!(out.stats), row["stats"], "case {case}");
        let records:Vec<_>=out.records.iter().map(|r|{
            let ammo:Vec<_>=r.ammo.iter().map(|a|json!({"WeaponSlot":a.weapon_slot,"Mag":a.magazine,"FracQ":a.fraction_quantum,"Res":a.reserve})).collect();
            let ammo=if ammo.is_empty(){Value::Null}else{json!(ammo)};
            let f=FactsInventoryDelta::from(r);
            json!({"Slot":f.slot,"Chunk":r.chunk_number.unwrap(),"PacketIndex":r.packet_index.unwrap(),"TimestampUS":f.timestamp_us,"Grenades":f.grenades,"SelRead":f.selection_read,"Sel":f.selection,"Mask":f.mask,"Ammo":ammo})
        }).collect();
        assert_eq!(json!(records), row["records"], "case {case}");
        let attempts:Vec<_>=out.component_attempts.iter().map(|a|json!({"time":a.read.source.timestamp_us,"id":a.component_index,"start":a.read.component.start_bit,"end":a.read.component.end_bit,"status":a.status.unwrap_or(false),"in_bounds":a.in_bounds})).collect();
        assert_eq!(json!(attempts), row["attempts"], "case {case}");
        count += out.records.len();
        refused += usize::from(out.stats.ammo_refused);
        if out.stats.ammo_refused {
            assert!(out.records.iter().all(|r| r.ammo.is_empty()));
            assert!(!out.components.is_empty());
        }
    }
    assert!(count > 100);
    assert!(refused > 0);
}
