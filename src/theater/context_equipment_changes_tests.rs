use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_equipment_scan() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-equipment-scan-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 96);
    let mut counts = [0; 3];
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for input in row["inputs"].as_array().unwrap() {
            let h = input["hex"].as_str().unwrap();
            buffers.push(
                (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            meta.push(FilmSourceMetadata {
                index: input["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let layout = I0Layout {
            gate_bits: 5,
            axis_widths: [8, 9, 10],
            region: 0,
        };
        let mut context = NativeFilmContext::for_map(Some(&source), None, Some(&layout)).unwrap();
        let mut profile = NativeScanProfile::default();
        profile.set_world_precision_from_layout(&layout);
        profile.grammar.simulation_complete = true;
        context.set_scan_profile(profile).unwrap();
        context.observation().set_hook(
            NativeHookKind::AbilitySet,
            Some(std::sync::Arc::new(|_| panic!("context hook leaked"))),
        );
        let births = serde_json::from_value(row["births"].clone()).unwrap();
        let (out, error) = scan_context_equipment_changes(&context, &births);
        let error = error.map_or_else(String::new, |e| match e {
            ContextAbilityChannelError::Registry(NativeContextRegistryError::NoRegistryChunk) => {
                "chunk_00 (registre) absent du film".into()
            }
            _ => e.to_string(),
        });
        assert_eq!(error, row["error"], "error {case}");
        assert_eq!(json!(out.facts_stats()), row["stats"], "stats {case}");
        let changes:Vec<_>=out.facts().map(|(r,f)|json!({"TimestampUS":f.timestamp_us,"Chunk":r.chunk_number.unwrap(),"PacketIndex":r.packet_index.unwrap(),"Slot":f.slot,"Counter":f.counter,"Rank":f.rank,"Previous":f.previous,"Kind":String::from_utf8(f.kind).unwrap(),"Recovered":f.recovered,"Gap":f.gap})).collect();
        assert_eq!(json!(changes), row["changes"], "changes {case}");
        counts[0] += changes.len();
        let encode = |e: &EquipmentEmission| {
            let a = &e.ability;
            let r = e.recovery.as_ref().unwrap();
            let probe = out
                .recovery_attempts
                .iter()
                .find(|p| {
                    p.chunk == e.native_chunk_number()
                        && p.packet_index == a.packet_index.unwrap()
                        && p.bit_offset == r.bit_offset
                })
                .unwrap();
            json!({"emission":{"Slot":a.slot,"Chunk":e.native_chunk_number(),"PacketIndex":a.packet_index.unwrap(),"TimestampUS":a.source.timestamp_us,"Counter":a.counter,"Rank":a.rank.map_or(-1,i32::from)},"offset":r.bit_offset,"dense":probe.probe.dense,"head":r.head})
        };
        let windows:Vec<_>=out.recovery_windows.iter().map(|w|json!({"slot":w.slot,"from":w.from_counter,"to":w.to_counter,"missing":w.missing,"min_time":w.min_time_us,"max_time":w.max_time_us,"min_chunk":w.min_chunk,"max_chunk":w.max_chunk,"head":w.head,"candidates":w.candidates.iter().map(&encode).collect::<Vec<_>>()})).collect();
        assert_eq!(json!(windows), row["windows"], "windows {case}");
        let recovered: Vec<_> = out.recovered.iter().map(encode).collect();
        assert_eq!(json!(recovered), row["recovered"], "recovered {case}");
        counts[1] += recovered.len();
        counts[2] += out.recovery_attempts.len();
        for r in &out.assembly.records {
            let index = r.source.chunk_index as usize;
            assert_eq!(meta[index].index, r.chunk_number.unwrap());
            assert!(r.source.payload_offset + r.source.payload_size <= buffers[index].len());
        }
        for p in &out.recovery_attempts {
            assert_eq!(meta[p.source.chunk_index as usize].index, p.chunk);
            assert!(p.bit_offset + 27 <= p.source.payload_size * 8);
        }
        assert_eq!(
            serde_json::from_value::<EquipmentChanges>(json!(out.assembly)).unwrap(),
            out.assembly
        );
    }
    assert_eq!(counts[0], 341);
    assert_eq!(counts[1], 100);
    assert!(counts[2] > 100);
}
