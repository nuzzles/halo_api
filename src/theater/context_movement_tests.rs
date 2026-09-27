use super::*;
use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
#[test]
fn native_context_movement_states() {
    check_context(false);
}
#[test]
fn native_replay_movement_phase() {
    check_context(true);
}
fn check_context(replay: bool) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/context-movement-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 224);
    let mut wrapper_raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/movement-scan-v41.json.zlib")[..])
        .read_to_end(&mut wrapper_raw)
        .unwrap();
    let wrapper_rows: Vec<Value> = serde_json::from_slice(&wrapper_raw).unwrap();
    assert_eq!(wrapper_rows.len(), rows.len());
    let mut reads = 0;
    let mut unbound = 0;
    let mut duplicates = 0;
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for input in row["inputs"].as_array().unwrap() {
            let hex = input["hex"].as_str().unwrap();
            buffers.push(
                (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            meta.push(FilmSourceMetadata {
                index: input["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let mut context = NativeFilmContext::new(Some(&source));
        let mut profile = NativeScanProfile::default();
        profile.movement.world_object.axis_bits =
            serde_json::from_value(row["axes"].clone()).unwrap();
        profile.grammar.view_tables = row["tables"].as_bool().unwrap();
        profile.grammar.view_classes = row["classes"].as_bool().unwrap();
        profile.grammar.chain_inference = row["chain"].as_bool().unwrap();
        profile.grammar.mobility_action_body = false;
        context.set_scan_profile(profile).unwrap();
        assert_eq!(
            json!(context.scan_profile().unwrap().grammar.corruption_check),
            row["corruption"]
        );
        let calls = Arc::new(AtomicUsize::new(0));
        for kind in [NativeHookKind::MovementState, NativeHookKind::Probe] {
            let calls = calls.clone();
            context.observation().set_hook(
                kind,
                Some(Arc::new(move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                })),
            );
        }
        let (scan, error) = if replay {
            let mut timeline = Vec::new();
            let report =
                scan_replay_movement_inputs(b"oracle-movement", &context, |event| match event {
                    ReplayMovementObservation::Diagnostic(d) => {
                        let mut v = serde_json::Map::from_iter(d.attributes.iter().cloned());
                        v.insert("level".into(), json!(d.level));
                        v.insert("msg".into(), json!(d.message));
                        timeline.push(json!({"log":v}));
                    }
                    ReplayMovementObservation::States(v) => {
                        timeline.push(json!({"step":"movementStates","value":v}))
                    }
                    ReplayMovementObservation::Stats(v) => {
                        timeline.push(json!({"step":"movementStates.stats","value":v}))
                    }
                });
            assert_eq!(
                json!(timeline),
                wrapper_rows[case]["timeline"],
                "native channel/log order {case}"
            );
            let mut writer = NativeFactsWriter::default();
            encode_facts_movement(&mut writer, &report.published);
            assert_eq!(writer.error(), None);
            let encoded: String = writer.bytes().iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(
                encoded, wrapper_rows[case]["encoded"],
                "complete cache {case}"
            );
            let mut facts = NativeFilmFacts::default();
            report.apply_to_facts(&mut facts);
            assert_eq!(facts.movement, report.published);
            if report.error.is_some() {
                assert!(report.reads.is_none());
                assert_eq!(report.stats, MovementStateStats::default());
            }
            (report.scan, report.error)
        } else {
            scan_context_movement_states(&context)
        };
        assert_eq!(
            error.map(|e| e.to_string()).unwrap_or_default(),
            row["error"],
            "error {case}"
        );
        assert_eq!(json!(scan.stats), row["stats"], "stats {case}");
        if case >= 160 {
            assert_eq!(
                scan.stats.jumps_derived,
                usize::from(case.is_multiple_of(8)),
                "positive/negative jump scenario {case}"
            );
            assert_eq!(
                scan.reads
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .filter(|r| r.kind == MOVEMENT_JUMP_DERIVED)
                    .count(),
                2 * usize::from(case.is_multiple_of(8))
            );
        }
        assert_eq!(json!(scan.reads), row["reads"], "reads {case}");
        let observed:Vec<_>=scan.observations.iter().map(|o| {
            let FilmComponentObservation::MovementState {component,slot,values}=&o.value else {panic!("non movement")};
            let (_,packets)=context.chunk_at(o.chunk).unwrap();
            assert_eq!(o.source,packets[o.packet_index]);
            json!({"chunk":o.chunk,"packet":o.packet_index,"ts":o.source.timestamp_us,"archetype":o.archetype,"component":component,"slot":slot,"values":values})
        }).collect();
        assert_eq!(json!(observed), row["observations"], "raw hooks {case}");
        for f in &scan.frames {
            assert_eq!(
                context.chunk_at(f.chunk).unwrap().1[f.packet_index],
                f.source
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0, "private observer {case}");
        reads += scan.reads.as_ref().map_or(0, Vec::len);
        unbound += scan.stats.slot_unbound;
        duplicates += scan.stats.duplicates;
    }
    assert!(reads > 0);
    assert!(unbound > 0);
    assert!(duplicates > 0);
}
