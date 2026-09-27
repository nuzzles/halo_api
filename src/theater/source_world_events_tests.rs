use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn load(rows: &Value) -> (Vec<Vec<u8>>, Vec<FilmSourceMetadata>) {
    let mut buffers = Vec::new();
    let mut meta = Vec::new();
    for c in rows.as_array().unwrap() {
        buffers.push(unhex(c["hex"].as_str().unwrap()));
        meta.push(FilmSourceMetadata {
            index: c["index"].as_i64().unwrap(),
            chunk_type: 0,
            start_ms: 90000,
        });
    }
    (buffers, meta)
}
#[test]
fn native_source_world_events() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/source-zoom-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 64);
    let mut zoom_count = 0;
    for (case, row) in rows.iter().enumerate() {
        let (buffers, meta) = load(&row["inputs"]);
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &meta).unwrap());
        let out = scan_source_zoom_events(source.as_ref());
        let expected = row["output"].as_array().unwrap();
        assert_eq!(out.len(), expected.len(), "zoom count {case}");
        for (r, e) in out.iter().zip(expected) {
            let f = FactsZoomEvent::from(r);
            assert_eq!(
                json!({"TimestampUS":f.timestamp_us,"Slot":f.slot,"Level":f.level}),
                e["event"],
                "zoom {case}"
            );
            assert_eq!(r.chunk, e["chunk"].as_i64().unwrap());
            let ev = &r.event;
            assert_eq!(ev.packet_index as u64, e["packet"].as_u64().unwrap());
            assert_eq!(
                ev.source.payload_offset as u64,
                e["start"].as_u64().unwrap()
            );
            assert_eq!(ev.source.payload_size as u64, e["size"].as_u64().unwrap());
            assert_eq!(ev.read.head.end_bit as u64, e["end"].as_u64().unwrap());
            assert_eq!(
                ev.read.padded_bits,
                ev.read
                    .head
                    .end_bit
                    .saturating_sub(ev.source.payload_size * 8)
            );
            assert_eq!(meta[ev.source.chunk_index as usize].index, r.chunk);
        }
        zoom_count += out.len();
    }
    assert_eq!(zoom_count, 247);
    raw.clear();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/equipment-spawn-source-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 128);
    let mut spawns = 0;
    for (case, row) in rows.iter().enumerate() {
        let (buffers, meta) = load(&row["inputs"]);
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let context = NativeFilmContext::new(Some(&source));
        let (out, error) = scan_context_equipment_spawn_events(&context);
        assert_eq!(error.is_some(), row["no_chunks"].as_bool().unwrap());
        assert_eq!(
            json!(FactsSpawnStats::from(&out.stats)),
            row["stats"],
            "spawn stats {case}"
        );
        assert_eq!(out.stats.truncated, 0);
        assert!(out.rejected_heads.is_empty());
        let expected = row["events"].as_array().unwrap();
        assert_eq!(out.records.len(), expected.len());
        for (r, e) in out.records.iter().zip(expected) {
            assert_eq!(json!(FactsSpawnEvent::from(r)), e["event"], "spawn {case}");
            let p = r.event.packet;
            assert_eq!(p.payload_offset as u64, e["start"].as_u64().unwrap());
            assert_eq!(p.payload_size as u64, e["size"].as_u64().unwrap());
            assert_eq!(meta[p.chunk_index as usize].index, r.chunk);
            assert_eq!(r.head.references.len(), 3);
            assert!(r.head.end_bit <= p.payload_size * 8);
        }
        spawns += out.records.len();
    }
    assert_eq!(spawns, 1120);
    let mut refused = 0;
    for row in fixture["short"].as_array().unwrap() {
        let pay = unhex(row["hex"].as_str().unwrap());
        let mut packet = vec![0; 16];
        packet[4..8].copy_from_slice(&(pay.len() as u32).to_le_bytes());
        packet.extend_from_slice(&pay);
        let source = FilmSource::load(
            &[packet],
            &[FilmSourceMetadata {
                index: 1,
                chunk_type: 0,
                start_ms: 0,
            }],
        )
        .unwrap();
        let context = NativeFilmContext::new(Some(&source));
        let (out, error) = scan_context_equipment_spawn_events(&context);
        assert!(error.is_none());
        if row["panic"] == true {
            refused += 1;
            assert_eq!(out.stats.truncated, 1);
            assert_eq!(out.rejected_heads.len(), 1);
            assert!(out.records.is_empty());
        } else {
            assert_eq!(out.stats.truncated, 0);
            assert_eq!(out.records.len(), usize::from(row["ok"] == true));
        }
    }
    assert_eq!(refused, 394);
}
