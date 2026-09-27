use super::*;
use crate::clients::hi::models::{FilmChunk, FilmChunkData};
use serde_json::Value;
use std::io::Read;

fn bytes(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn chunk(index: i32, data: Vec<u8>) -> FilmChunkData {
    FilmChunkData {
        metadata: FilmChunk {
            index,
            chunk_type: 2,
            start_time_offset_ms: 90000,
            duration_ms: 1,
            size: data.len() as i64,
            file_relative_path: String::new(),
        },
        data,
    }
}
fn compare_event(a: &EquipmentSpawnEvent, e: &Value) {
    assert_eq!(a.packet.chunk_index as i64, e["Chunk"].as_i64().unwrap());
    assert_eq!(
        a.packet_index.unwrap() as u64,
        e["PacketIndex"].as_u64().unwrap()
    );
    assert_eq!(a.packet.timestamp_us, e["TimestampUS"].as_u64().unwrap());
    for (life, key, valid) in [
        (a.source, "Source", "SourceValid"),
        (a.spawned, "Spawned", "SpawnedValid"),
    ] {
        assert_eq!(life.is_some(), e[valid].as_bool().unwrap());
        if let Some(life) = life {
            assert_eq!(u64::from(life.slot), e[key]["Slot"].as_u64().unwrap());
            assert_eq!(u64::from(life.generation), e[key]["Gen"].as_u64().unwrap());
        }
    }
    assert_eq!(a.reference_2_present, e["Ref2Present"].as_bool().unwrap());
}
#[test]
fn native_equipment_spawn_source_order_framing_and_truncation() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/equipment-spawn-source-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let oracle: Value = serde_json::from_slice(&raw).unwrap();
    let rows = oracle["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 128);
    let mut total = 0;
    for row in rows {
        let chunks: Vec<_> = row["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                chunk(
                    c["index"].as_i64().unwrap() as i32,
                    bytes(c["hex"].as_str().unwrap()),
                )
            })
            .collect();
        let actual = scan_equipment_spawn_events(&chunks);
        if row["no_chunks"].as_bool().unwrap() {
            assert!(matches!(
                actual,
                Err(DecodeError::Missing("readable film chunks"))
            ));
            continue;
        }
        let actual = actual.unwrap();
        let s = &actual.stats;
        for (key, value) in [
            ("Chunks", s.chunks),
            ("Packets", s.packets),
            ("Lists", s.lists),
            ("Events", s.events),
            ("WithSpawned", s.with_spawned),
            ("WithSource", s.with_source),
            ("Ref2", s.reference_2),
        ] {
            assert_eq!(value as u64, row["stats"][key].as_u64().unwrap(), "{key}");
        }
        assert_eq!(s.truncated, 0);
        let expected = row["events"].as_array().unwrap();
        assert_eq!(actual.records.len(), expected.len());
        for (a, e) in actual.records.iter().zip(expected) {
            compare_event(a, &e["event"]);
            assert_eq!(a.packet.payload_offset as u64, e["start"].as_u64().unwrap());
            assert_eq!(a.packet.payload_size as u64, e["size"].as_u64().unwrap());
        }
        total += actual.records.len();
        // Supplied heads may arrive in any order, but publication follows source.
        let mut heads: Vec<_> = fire_events::native_chunk_prefix(&chunks)
            .unwrap()
            .into_iter()
            .flat_map(|c| {
                fire_events::native_chunk_packets(c)
                    .into_iter()
                    .filter(|p| p.packet_type == 0)
                    .filter_map(|p| {
                        decode_packet_head_event(
                            &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                        )
                        .map(|event| FilmHeadEvent { source: p, event })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        heads.reverse();
        assert_eq!(
            equipment_spawns_from_heads(&chunks, &heads).unwrap(),
            actual
        );
        let restored: EquipmentSpawnStream =
            serde_json::from_slice(&serde_json::to_vec(&actual).unwrap()).unwrap();
        assert_eq!(restored, actual);
    }
    assert!(total > 500);
    let short = oracle["short"].as_array().unwrap();
    assert_eq!(short.len(), 2304);
    let mut panics = 0;
    for row in short {
        let pay = bytes(row["hex"].as_str().unwrap());
        let mut packet = vec![0; 16];
        packet[4..8].copy_from_slice(&(pay.len() as u32).to_le_bytes());
        packet.extend_from_slice(&pay);
        let out = scan_equipment_spawn_events(&[chunk(1, packet)]).unwrap();
        if row["panic"].as_bool().unwrap() {
            panics += 1;
            assert_eq!(out.stats.truncated, 1);
            assert!(out.records.is_empty());
        } else {
            assert_eq!(out.stats.truncated, 0);
            assert_eq!(
                out.stats.lists,
                usize::from(row["present"].as_bool().unwrap())
            );
            assert_eq!(out.records.len(), usize::from(row["ok"].as_bool().unwrap()));
            if let Some(a) = out.records.first() {
                // Direct native decoder leaves source identity at zero.
                let mut e = row["event"].clone();
                e["Chunk"] = Value::from(1);
                compare_event(a, &e);
            }
        }
    }
    assert!(panics > 0);
}

#[test]
#[ignore = "requires four downloaded v41 films; compares captured spawn events and counters"]
fn local_equipment_spawn_source_matches_native() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/gameplay-levelup-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    let mut total = 0;
    for row in &rows {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("experiments/films")
            .join(row["folder"].as_str().unwrap());
        let meta: Value =
            serde_json::from_slice(&std::fs::read(path.join("film.json")).unwrap()).unwrap();
        let chunks: Vec<_> = meta["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                    start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                    duration_ms: c["duration_ms"].as_i64().unwrap(),
                    size: c["decompressed_size"].as_i64().unwrap(),
                    file_relative_path: c["file_relative_path"].as_str().unwrap().into(),
                },
                data: std::fs::read(path.join(c["file"].as_str().unwrap())).unwrap(),
            })
            .collect();
        let out = scan_equipment_spawn_events(&chunks).unwrap();
        let expected = row["inputs"]["SpawnEvents"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(out.records.len(), expected.len());
        for (a, e) in out.records.iter().zip(&expected) {
            compare_event(a, e);
        }
        let s = &out.stats;
        assert_eq!(s.truncated, 0);
        assert_eq!(
            serde_json::json!({"Chunks":s.chunks,"Packets":s.packets,"Lists":s.lists,"Events":s.events,"WithSpawned":s.with_spawned,"WithSource":s.with_source,"Ref2":s.reference_2}),
            row["inputs"]["SpawnStats"]
        );
        total += out.records.len();
    }
    assert_eq!(rows.len(), 4);
    assert_eq!(total, 6);
}
