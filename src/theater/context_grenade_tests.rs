use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_grenade_scan() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/context-grenade-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 128);
    let mut published = 0;
    let mut rejected = 0;
    let mut old_profiles = 0;
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
        let context = NativeFilmContext::new(Some(&source));
        let mut logs = Vec::new();
        let (scan, error) = scan_context_grenade_throws(&context, |d| {
            let mut v = serde_json::Map::from_iter(d.attributes.iter().cloned());
            v.insert("level".into(), json!(d.level));
            v.insert("msg".into(), json!(d.message));
            logs.push(Value::Object(v));
        });
        assert_eq!(
            error.map(|e| e.to_string()).unwrap_or_default(),
            row["error"],
            "error {case}"
        );
        assert_eq!(json!(scan.grammar), row["grammar"], "grammar {case}");
        assert_eq!(json!(scan.preamble), row["preamble"], "preamble {case}");
        assert_eq!(json!(scan.stats), row["stats"], "coverage {case}");
        assert_eq!(json!(scan.reads), row["events"], "throws {case}");
        assert_eq!(json!(logs), row["logs"], "ordered logs {case}");
        let mut packet_reads = Vec::new();
        for p in &scan.packets {
            assert_eq!(
                context.chunk_at(p.chunk).unwrap().1[p.packet_index],
                p.source
            );
            for r in &p.scan.records {
                assert_eq!(r.chunk, p.chunk);
                assert_eq!(r.packet_index, p.packet_index);
                assert_eq!(r.timestamp_us, p.source.timestamp_us);
                assert!(r.bit_pos > 0);
                assert!(r.bit_pos + scan.preamble.author_bit + 5 <= p.source.payload_size * 8);
                packet_reads.push(r.clone());
            }
        }
        assert_eq!(packet_reads, scan.reads.unwrap_or_default());
        published += scan.stats.published;
        rejected += scan.stats.rejected_known_ids;
        old_profiles += usize::from(scan.preamble.bits == 23);
    }
    assert!(published > 0);
    assert!(rejected > 0);
    assert!(old_profiles > 0);
}
