use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_ability_channels() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-ability-channels-v41.json.zlib")[..],
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
    let mut counts = [0; 3];
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
        profile.grammar.simulation_complete = row["simulation"].as_bool().unwrap();
        context.set_scan_profile(profile).unwrap();
        for kind in [NativeHookKind::AbilitySet, NativeHookKind::CamoState] {
            context.observation().set_hook(
                kind,
                Some(std::sync::Arc::new(|_| panic!("channel observer leaked"))),
            );
        }
        let (ability, ae) = scan_context_ability_emissions(&context);
        let (camo, ce) = scan_context_camo_states(&context);
        let error = |e: Option<ContextAbilityChannelError>| {
            e.map_or_else(String::new, |e| match e {
                ContextAbilityChannelError::Registry(
                    NativeContextRegistryError::NoRegistryChunk,
                ) => "chunk_00 (registre) absent du film".into(),
                _ => e.to_string(),
            })
        };
        assert_eq!(error(ae), row["ability_error"], "ability error {case}");
        assert_eq!(error(ce), row["camo_error"], "camo error {case}");
        let s = &ability.stats;
        assert_eq!(
            json!({"Records":s.records,"WithI48":s.with_component,"Read":s.read,"Unread":s.unread,"Gated":s.gated}),
            row["ability_stats"],
            "ability stats {case}"
        );
        let s = &camo.stats;
        assert_eq!(
            json!({"Records":s.records,"WithI28":s.with_component,"Read":s.read,"Unread":s.unread,"NoChannel":s.gated}),
            row["camo_stats"],
            "camo stats {case}"
        );
        let emissions:Vec<_>=ability.observations.iter().map(|r|{let FilmComponentObservation::AbilitySet{counter,rank,..}=&r.value else{panic!("wrong hook")};json!({"Slot":r.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":r.source.timestamp_us,"Counter":*counter as u32,"Rank":rank})}).collect();
        assert_eq!(json!(emissions), row["emissions"], "emissions {case}");
        let ranks:Vec<_>=ability.ability_ranks().map(|(r,f)|{let FilmComponentObservation::AbilitySet{counter,..}=&r.value else{unreachable!()};json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Counter":*counter as u32,"Rank":f.rank})}).collect();
        assert_eq!(json!(ranks), row["ranks"], "ranks {case}");
        let camo_records:Vec<_>=camo.camo_states().map(|(r,f)|json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Q":f.quantum})).collect();
        assert_eq!(json!(camo_records), row["camo"], "camo {case}");
        for (scan, key) in [(&ability, "ability_attempts"), (&camo, "camo_attempts")] {
            let attempts:Vec<_>=scan.component_attempts.iter().map(|a|json!({"time":a.read.source.timestamp_us,"id":a.component_index,"start":a.read.component.start_bit,"end":a.read.component.end_bit,"status":a.status.unwrap_or(false),"in_bounds":a.in_bounds})).collect();
            assert_eq!(json!(attempts), row[key], "{key} {case}");
        }
        counts[0] += emissions.len();
        counts[1] += ranks.len();
        counts[2] += camo_records.len();
    }
    assert_eq!(counts, [486, 261, 333]);
}
