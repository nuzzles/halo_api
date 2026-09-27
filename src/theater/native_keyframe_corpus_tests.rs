use super::*;
use serde_json::Value;
use std::io::BufRead;

#[test]
#[ignore = "requires 32 local films; compares native full-state reads at all 134657 anchors"]
fn native_keyframe_anchor_bodies_corpus() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = std::fs::File::open(
        root.join("src/theater/fixtures/keyframe-anchor-bodies-d61443e-v41.jsonl.zlib"),
    )
    .unwrap();
    let rows = std::io::BufReader::new(flate2::read::ZlibDecoder::new(input));
    let mut encoding: FrameEncoding = serde_json::from_value(serde_json::json!({
        "ids":{"low_bits":13,"base":0},"mpp_widths":[9,5],
        "position":NativeScanProfile::default().component_encoding().unwrap(),"extra_fields":false,"corruption_check":false,
        "keyframe_simulation_complete":false
    }))
    .unwrap();
    let mut cached: Option<(String, Vec<u8>, FilmRegistry)> = None;
    let mut count = 0;
    let mut stops = std::collections::BTreeMap::<String, usize>::new();
    let mut complete = 0;
    let mut payload_count = 0;
    let mut dead_count = 0;
    let mut padded = 0;
    for line in rows.lines() {
        let row: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let file = row["file"].as_str().unwrap();
        if cached
            .as_ref()
            .is_none_or(|(previous, _, _)| previous != file)
        {
            let path = root.join("experiments/films").join(file);
            let folder = path.parent().unwrap();
            let manifest: Value =
                serde_json::from_slice(&std::fs::read(folder.join("film.json")).unwrap()).unwrap();
            let bootstrap = manifest["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["chunk_type"] == 1)
                .unwrap();
            let registry = parse_registry(
                &std::fs::read(folder.join(bootstrap["file"].as_str().unwrap())).unwrap(),
            )
            .unwrap();
            cached = Some((file.into(), std::fs::read(path).unwrap(), registry));
        }
        let (_, bytes, registry) = cached.as_ref().unwrap();
        let offset = row["offset"].as_u64().unwrap() as usize;
        let size = row["size"].as_u64().unwrap() as usize;
        let payload = &bytes[offset..offset + size];
        let start = row["anchor"]["Bit"].as_u64().unwrap() as usize;
        encoding.corruption_check = row["check"].as_bool().unwrap();
        let actual = decode_native_keyframe_record(payload, start, registry, &encoding).unwrap();
        let expected = &row["trace"];
        assert_eq!(
            actual.end_bit, expected["EndBit"],
            "end {count}: {file} @{offset}:{start}"
        );
        assert_eq!(actual.archetype, expected["TypeIndex"], "type {count}");
        assert_eq!(actual.id & 0x3fff_ffff, row["anchor"]["Slot"]);
        assert_eq!(actual.id >> 30, row["anchor"]["Gen"]);
        let desync = match &actual.stop {
            KeyframeStop::Complete => -1,
            KeyframeStop::UnsupportedComponent { index, .. } => *index as i64,
            other => panic!("unexpected stop {count}: {other:?}"),
        };
        assert_eq!(desync, expected["DesyncAt"], "stop {count}");
        match &actual.stop {
            KeyframeStop::Complete => complete += 1,
            KeyframeStop::UnsupportedComponent { name, .. } => {
                *stops.entry(name.clone()).or_default() += 1
            }
            _ => (),
        }
        let comps = expected["Comps"].as_array().cloned().unwrap_or_default();
        assert_eq!(actual.attempts.len(), comps.len(), "attempts {count}");
        for (a, e) in actual.attempts.iter().zip(&comps) {
            assert_eq!(
                serde_json::json!((a.index, &a.name, a.start_bit, a.variant, a.ported)),
                serde_json::json!((
                    &e["Index"],
                    &e["Name"],
                    &e["StartBit"],
                    &e["Variant"],
                    &e["Ported"]
                )),
                "native component result {count}"
            );
            let [start, end] = a.field_range.unwrap();
            assert!(start <= end && end <= actual.fields.len());
        }
        let ported: Vec<_> = comps.iter().filter(|c| c["Ported"] == true).collect();
        assert_eq!(actual.components.len(), ported.len(), "components {count}");
        for (j, (a, e)) in actual.components.iter().zip(ported).enumerate() {
            assert_eq!(a.index, e["Index"], "component index {count}");
            assert_eq!(a.start_bit, e["StartBit"], "component start {count}");
            assert_eq!(a.name, e["Name"], "component name {count}");
            let mut projected = actual.captured_payload(j);
            if let Some(CapturedComponentPayload::Parent(parent)) = &mut projected {
                // Native capture leaves EndBit at zero without a parent hook.
                // Rust keeps that observed source endpoint, checked separately.
                assert!(parent.end_bit > parent.start_bit && parent.end_bit <= a.end_bit);
                parent.end_bit = 0;
            }
            let expected_payload = native_payload(
                &a.name,
                &e["Payload"],
                payload,
                crate::theater::bits::native_address(a.start_bit),
            );
            payload_count += usize::from(expected_payload.is_some());
            assert_eq!(
                projected, expected_payload,
                "payload {count}/{j}: {}",
                a.name
            );
        }
        assert_eq!(actual.captured_payload(actual.components.len()), None);
        let dead = actual.captured_dead_state();
        let expected_dead: Option<ObjectDeadState> =
            serde_json::from_value(expected["Dead"].clone()).unwrap();
        dead_count += usize::from(expected_dead.is_some());
        assert_eq!(dead, expected_dead, "dead {count}");
        let mut at = start;
        for field in &actual.fields {
            assert_eq!(
                serde_json::json!(field.bit),
                serde_json::json!(at),
                "field coverage {count}: {}",
                field.name
            );
            assert_eq!(
                serde_json::json!(field.raw),
                serde_json::json!(padded_source_bits(
                    payload,
                    crate::theater::bits::native_address(field.bit),
                    usize::try_from(field.width).unwrap()
                )),
                "raw {count}: {}",
                field.name
            );
            at += usize::try_from(field.width).unwrap();
        }
        assert_eq!(
            serde_json::json!(at),
            serde_json::json!(actual.end_bit),
            "field end {count}"
        );
        padded += usize::from(actual.end_bit > (size * 8) as i64);
        let restored: KeyframeRecord =
            serde_json::from_value(serde_json::to_value(&actual).unwrap()).unwrap();
        assert_eq!(restored, actual, "record JSON {count}");
        count += 1;
    }
    assert_eq!(count, 134657);
    assert_eq!(payload_count, 189416);
    assert_eq!(dead_count, 43631);
    assert_eq!(padded, 483);
    println!(
        "NATIVE_KEYFRAME_AUDIT {}",
        serde_json::json!({
            "candidate_records": count, "complete": complete, "stopped": stops,
            "padded_reads": padded, "captured_payloads": payload_count,
            "reference_boundaries_and_fields_match": true
        })
    );
}

fn padded_source_bits(data: &[u8], bit: usize, width: usize) -> u64 {
    (0..width).fold(0, |value, i| {
        (value << 1)
            | data
                .get((bit + i) / 8)
                .map_or(0, |b| u64::from((b >> (7 - (bit + i) % 8)) & 1))
    })
}

fn native_payload(
    name: &str,
    p: &Value,
    data: &[u8],
    start: usize,
) -> Option<CapturedComponentPayload> {
    use serde_json::json;
    if p.is_null() {
        return None;
    }
    let optional = |flag: &str, value: &str| {
        if p[flag] == true {
            p[value].clone()
        } else {
            Value::Null
        }
    };
    let (kind, value) = match name {
        "object-body-vitality-component" => (
            "body",
            json!({"quantum":p["Q"],"health":p["Health"],"flags":[p["F5c"],p["F5d"],p["F5e"]]}),
        ),
        "object-shield-vitality-component" => (
            "shield",
            json!({"quantum":p["Q"],"shield":p["Shield"],"regen_present":p["RegenPresent"],"regen":[optional("HasRegen0","Regen0"),optional("HasRegen1","Regen1")],"block_64":p["Block64"],"flags":[p["F66"],p["F67"],p["F69"],p["F68"]]}),
        ),
        "object-parent-state-component" => (
            "parent",
            json!({
                "archetype":p["TypeIndex"],"parameter":p["Param"],"start_bit":p["StartBit"],"end_bit":p["EndBit"],"attached":p["Attached"],
                "quantized_word":p["Quant16"],"word":p["Word16"],"optional_word":optional("HasOpt16","Opt16"),"flags":[p["FlagA"],p["FlagB"]],"matrix":p["Mtx"],
                "velocity":optional("HasVel","Vel"),"byte":p["Byte8"],"flag_c":p["FlagC"],"free_read":p["FreeRead"],"free_bits":p["FreeBits"],"free_id":optional("HasFreeID","FreeID"),"alternate":optional("HasAlt11","Alt11"),
                "tail_sign":p["TailSign"],"tail6":optional("HasTail6","Tail6"),"tail_bit":p["TailBit"],"tail3":optional("HasTail3","Tail3")
            }),
        ),
        "object-dissolver-component" => {
            let body = (p["Corps"] == true).then(|| {
                [0, 1, 2].map(|i| padded_source_bits(data, start + 4 + i * 32, 32) as u32)
            });
            (
                "dissolver",
                json!({"state":p["Etat"],"body":body,"duration_quantum":p["DureeQ"],"flag":p["Drapeau"]}),
            )
        }
        "player-respawn-timer-component" => (
            "respawn",
            json!({"active":p["Active"],"timers":[p["T0"],p["T1"]]}),
        ),
        "game-engine-round-timer-component" => (
            "round_timer",
            json!({"quanta":[p["QA"],p["QB"]],"seconds":[p["A"],p["B"]],"tail":p["Tail"]}),
        ),
        _ => panic!("unknown captured payload {name}"),
    };
    Some(serde_json::from_value(json!({"kind":kind,"value":value})).unwrap())
}

#[test]
fn recovered_keyframe_retains_native_padding_separately() {
    use crate::clients::hi::models::{FilmChunk, FilmChunkData};
    use std::io::Read;
    let inflate = |bytes: &[u8]| {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_end(&mut raw)
            .unwrap();
        raw
    };
    let payload = inflate(include_bytes!(
        "fixtures/recovered-keyframe-padding-v41.zlib"
    ));
    let registry = parse_registry(&inflate(include_bytes!(
        "fixtures/recovered-keyframe-padding-bootstrap-v41.zlib"
    )))
    .unwrap();
    let expected: Value =
        serde_json::from_str(include_str!("fixtures/recovered-keyframe-padding-v41.json")).unwrap();
    let mut data = vec![2, 0, 0, 0];
    data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&1000u64.to_le_bytes());
    data.extend_from_slice(&payload);
    let chunk = FilmChunkData {
        metadata: FilmChunk {
            index: 1,
            chunk_type: 2,
            start_time_offset_ms: 0,
            duration_ms: 1,
            size: data.len() as i64,
            file_relative_path: String::new(),
        },
        data,
    };
    let encoding: FrameEncoding = serde_json::from_value(serde_json::json!({
        "ids":{"low_bits":13,"base":0},"mpp_widths":[9,5],"position":NativeScanProfile::default().component_encoding().unwrap(),
        "extra_fields":false,"corruption_check":false,"keyframe_simulation_complete":false
    })).unwrap();
    let stream = decode_replication_stream_with_recovery(
        &[chunk],
        &registry,
        encoding,
        KeyframeRecoveryPolicy::LevelUp,
    )
    .unwrap();
    let records = &stream.packets[0]
        .keyframe_recovery
        .as_ref()
        .unwrap()
        .records;
    assert!(records.iter().any(|r| matches!(
        r.native_read,
        Some(NativeRecoveredKeyframeRead::SameAsBounded)
    )));
    let record = records
        .iter()
        .find(|r| r.anchor.bit as u64 == expected["anchor"]["Bit"].as_u64().unwrap())
        .unwrap();
    assert!(matches!(
        record.record.as_ref().unwrap().stop,
        KeyframeStop::Truncated
    ));
    let native = record.native_record().unwrap();
    assert_eq!(native.end_bit, expected["trace"]["EndBit"]);
    assert_eq!(native.stop, KeyframeStop::Complete);
    assert!(native.end_bit > (payload.len() * 8) as i64);
    assert!(matches!(
        record.native_read,
        Some(NativeRecoveredKeyframeRead::Decoded {
            crosses_next_anchor: true,
            ..
        })
    ));
    let restored: RecoveredKeyframeRecord =
        serde_json::from_value(serde_json::to_value(record).unwrap()).unwrap();
    assert_eq!(&restored, record);
    let mut legacy = serde_json::to_value(record).unwrap();
    legacy.as_object_mut().unwrap().remove("native_read");
    let legacy: RecoveredKeyframeRecord = serde_json::from_value(legacy).unwrap();
    assert!(legacy.native_read.is_none());
    assert!(legacy.native_record().is_none());
    assert_eq!(legacy.record, record.record);
}
