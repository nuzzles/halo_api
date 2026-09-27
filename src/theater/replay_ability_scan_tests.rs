use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_ability_scan_phase() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/ability-scan-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 48);
    let mut counts = [0; 6];
    let mut log_count = 0;
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            buffers.push(unhex(c["hex"].as_str().unwrap()));
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
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
        profile.grammar.simulation_complete = row["simulation"].as_bool().unwrap();
        profile.grammar.ability_anchor_body = row["body"].as_bool().unwrap();
        context.set_scan_profile(profile).unwrap();
        let positions: Vec<FactsBipedPosition> = row["positions"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|v| FactsBipedPosition {
                timestamp_us: v["TimestampUS"].as_u64().unwrap(),
                slot: v["Slot"].as_u64().unwrap() as u32,
                ..Default::default()
            })
            .collect();
        let mut observations = Vec::new();
        let mut output = None;
        let logs = super::log_test_support::capture_logs(|| {
            output = Some(scan_replay_ability_inputs(
                b"abilities",
                &context,
                &positions,
                |o| {
                    let value=match &o {
                ReplayAbilityObservation::AbilityRanks(v)=>json!(v.iter().map(|(r,f)|{let FilmComponentObservation::AbilitySet {counter,..}=r.value else{panic!("rank hook")};json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Counter":counter,"Rank":f.rank})}).collect::<Vec<_>>()),
                ReplayAbilityObservation::AbilityRankStats(s)=>json!({"Records":s.records,"WithI48":s.with_component,"Read":s.read,"Unread":s.unread,"Gated":s.gated}),
                ReplayAbilityObservation::EquipmentChanges(v)=>json!(v.iter().map(|(r,f)|json!({"TimestampUS":f.timestamp_us,"Chunk":r.chunk_number.unwrap(),"PacketIndex":r.packet_index.unwrap(),"Slot":f.slot,"Counter":f.counter,"Rank":f.rank,"Previous":f.previous,"Kind":String::from_utf8(f.kind.clone()).unwrap(),"Recovered":f.recovered,"Gap":f.gap})).collect::<Vec<_>>()),
                ReplayAbilityObservation::EquipmentStats(s)=>json!(s),
                ReplayAbilityObservation::CamoStates(v)=>json!(v.iter().map(|(r,f)|json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Q":f.quantum})).collect::<Vec<_>>()),
                ReplayAbilityObservation::CamoStats(s)=>json!({"Records":s.records,"WithI28":s.with_component,"Read":s.read,"Unread":s.unread,"NoChannel":s.gated}),
                ReplayAbilityObservation::GrappleReads(v)=>json!(v.iter().map(|(r,f)|json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Heavy":f.heavy,"PosQ":f.position_quanta})).collect::<Vec<_>>()),
                ReplayAbilityObservation::GrappleStats(s)=>json!(s),
                ReplayAbilityObservation::Impulses(v)=>json!(v.iter().map(|(r,f)|json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Predicted":f.predicted})).collect::<Vec<_>>()),
                ReplayAbilityObservation::Charges(v)=>json!(v.iter().map(|(r,f)|json!({"Slot":f.slot,"Chunk":r.chunk,"PacketIndex":r.packet_index,"TimestampUS":f.timestamp_us,"Emplacement":f.emplacement,"Charges":f.charges,"Low":f.low})).collect::<Vec<_>>()),
            };
                    observations.push(json!({"step":o.name(),"value":value}));
                },
            ));
        });
        assert_eq!(json!(logs), row["logs"], "logs {case}");
        log_count += logs.len();
        let mut expected = row["observations"].clone();
        for o in expected.as_array_mut().unwrap() {
            if o["value"].is_null() {
                o["value"] = json!([]);
            }
        }
        assert_eq!(json!(observations), expected, "observations {case}");
        let out = output.unwrap();
        for (i, n) in [
            out.published_ranks().count(),
            out.published_equipment().count(),
            out.published_camo().count(),
            out.published_grapple().count(),
            out.published_impulses().count(),
            out.published_charges().count(),
        ]
        .into_iter()
        .enumerate()
        {
            counts[i] += n;
        }
        let mut facts = NativeFilmFacts {
            positions,
            ..Default::default()
        };
        out.apply_to_facts(&mut facts);
        let captured = capture_film_scan_facts(
            b"abilities",
            &context,
            None,
            &facts,
            &FactsModeGuards::default(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            encode_film_facts_file(&captured).unwrap(),
            unhex(row["facts"].as_str().unwrap()),
            "complete facts file {case}"
        );
    }
    assert!(counts.iter().all(|&n| n > 0));
    assert_eq!(log_count, 288);
}
