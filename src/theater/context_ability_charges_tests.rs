use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_ability_charges() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-ability-charges-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 192);
    let unhex = |s: &str| {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    let mut count = 0;
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        if row["registry"] == true {
            buffers.push(unhex(
                fixture["registries"][row["mode"].as_u64().unwrap() as usize]
                    .as_str()
                    .unwrap(),
            ));
            meta.push(FilmSourceMetadata {
                index: 0,
                chunk_type: 0,
                start_ms: 0,
            });
        }
        if row["data_present"] == true {
            buffers.push(unhex(row["data"].as_str().unwrap()));
            meta.push(FilmSourceMetadata {
                index: 1,
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &meta).unwrap());
        let layout = I0Layout {
            gate_bits: 5,
            axis_widths: [8, 9, 10],
            region: 0,
        };
        let mut context = NativeFilmContext::for_map(source.as_ref(), None, Some(&layout)).unwrap();
        let mut profile = NativeScanProfile::default();
        profile.set_world_precision_from_layout(&layout);
        profile.grammar.ability_anchor_body = row["body"].as_bool().unwrap();
        context.set_scan_profile(profile).unwrap();
        {
            let kind = NativeHookKind::AbilityEnergy;
            context.observation().set_hook(
                kind,
                Some(std::sync::Arc::new(|_| panic!("context hook leaked"))),
            );
        }
        let (charges, ce) = scan_context_ability_charges(&context);
        let error = |e: Option<ContextAbilityChannelError>| {
            e.map_or_else(String::new, |e| match e {
                ContextAbilityChannelError::Registry(
                    NativeContextRegistryError::NoRegistryChunk,
                ) => "chunk_00 (registre) absent du film".into(),
                _ => e.to_string(),
            })
        };
        assert_eq!(error(ce), row["error"], "error {case}");
        assert_eq!(json!(charges.stats), row["stats"], "stats {case}");
        assert_eq!(
            json!(FactsAbilityChargeStats::from(&charges.stats)),
            row["stats"],
            "facts stats {case}"
        );
        let cr:Vec<_>=charges.facts().map(|(r,f)|json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Emplacement":f.emplacement,"Charges":f.charges,"Low":f.low})).collect();
        assert_eq!(json!(cr), row["charges"], "charges {case}");
        count += cr.len();
        for r in &charges.observations {
            let FilmComponentObservation::AbilityEnergy {
                mask,
                charges: values,
            } = r.value
            else {
                panic!("wrong hook")
            };
            for (i, v) in values.iter().enumerate() {
                if mask & (1 << i) == 0 {
                    assert_eq!(*v, -1);
                }
            }
        }
        {
            let (attempts, key) = (&charges.component_attempts, "attempts");
            let got:Vec<_>=attempts.iter().map(|a|json!({"time":a.read.source.timestamp_us,"id":a.component_index,"start":a.read.component.start_bit,"end":a.read.component.end_bit,"status":a.status.unwrap_or(false),"in_bounds":a.in_bounds})).collect();
            assert_eq!(json!(got), row[key], "{key} {case}");
        }
    }
    assert_eq!(count, 1548);
}
