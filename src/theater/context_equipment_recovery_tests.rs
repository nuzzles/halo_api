use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_equipment_probe() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-equipment-probe-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 512);
    let mut accepted = 0;
    let mut failed_with_hook = 0;
    for (case, row) in rows.iter().enumerate() {
        let hex = row["hex"].as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let arch = FilmArchetype {
            index: 35,
            components: serde_json::from_value(row["names"].clone()).unwrap(),
            levels: vec![0; 64],
        };
        let layout = I0Layout {
            gate_bits: 6,
            axis_widths: [6, 7, 8],
            region: row["region"].as_u64().unwrap() as u32,
        };
        let mut profile = NativeScanProfile::default();
        profile.set_world_precision_from_layout(&layout);
        profile.grammar.simulation_complete = row["simulation"].as_bool().unwrap();
        let observer = NativeFilmObserver::default();
        observer.set_hook(
            NativeHookKind::AbilitySet,
            Some(std::sync::Arc::new(|_| panic!("shared hook leaked"))),
        );
        let context = NativeReaderContext {
            profile,
            observer: Some(observer),
        };
        let out = probe_context_equipment_recovery(
            &bytes,
            row["offset"].as_u64().unwrap() as usize,
            &layout,
            &arch,
            &context,
        )
        .unwrap();
        let expected = row["ok"].as_bool().unwrap().then(|| {
            (
                row["counter"].as_u64().unwrap() as u32,
                row["rank"].as_i64().unwrap() as i32,
            )
        });
        assert_eq!(out.candidate(), expected, "candidate {case}");
        accepted += usize::from(expected.is_some());
        failed_with_hook += usize::from(out.candidate().is_none() && out.observation.is_some());
        let attempts: Vec<_> = out.components.iter().map(|a|json!({"id":a.component_index,"start":a.component.start_bit,"end":a.component.end_bit,"status":a.status,"in_bounds":a.in_bounds,"observations":a.component.diagnostics.component_observations})).collect();
        assert_eq!(json!(attempts), row["attempts"], "attempts {case}");
        let hooks: Vec<_> = out
            .components
            .iter()
            .flat_map(|a| a.component.diagnostics.component_observations.iter())
            .collect();
        assert_eq!(json!(hooks), row["observations"], "hooks {case}");
        let last = hooks
            .iter()
            .rev()
            .find(|v| matches!(v, FilmComponentObservation::AbilitySet { .. }))
            .map(|v| (*v).clone());
        assert_eq!(out.observation, last, "last hook {case}");
    }
    assert!(accepted > 100);
    assert!(failed_with_hook > 100);
}
