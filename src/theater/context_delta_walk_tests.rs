use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_context_delta_walk() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/delta-context-walk-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 256);
    let decode = |h: &str| {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    let mut count = 0;
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut metadata = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            buffers.push(decode(c["hex"].as_str().unwrap()));
            metadata.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &metadata).unwrap();
        let context = NativeFilmContext::new(Some(&source));
        let chunks: Vec<i64> = serde_json::from_value(row["chunks"].clone()).unwrap_or_default();
        let flags = serde_json::from_value(row["slots"].clone()).unwrap();
        let slots = FilmSlotBand::from_flags(&flags).unwrap();
        let layout = I0Layout {
            gate_bits: 6,
            axis_widths: [8, 9, 10],
            region: row["region"].as_u64().unwrap() as u32,
        };
        let mut records = Vec::new();
        walk_context_delta_bipeds(&context,&chunks,&slots,&layout,|r| {
            let (data,packets)=context.chunk_at(r.chunk).unwrap();
            assert_eq!(r.source,packets[r.packet_index]);
            let borrowed=&data[r.source.payload_offset..r.source.payload_offset+r.source.payload_size];
            assert!(std::ptr::eq(r.payload.as_ptr(),borrowed.as_ptr()));
            assert_eq!(r.position_end_bit-r.position_bit,33);
            assert_eq!(r.position_bit-r.start_bit,21+6*r.component_indices.len());
            assert_eq!(r.generation,1);
            records.push(json!({"chunk":r.chunk,"time":r.source.timestamp_us,"i0":r.position_bit,"slot":r.slot,"mask":r.component_indices,"total":r.payload.len()*8,"hex":r.payload.iter().map(|b|format!("{b:02x}")).collect::<String>()}));
        }).unwrap();
        assert_eq!(json!(records), row["records"], "case {case}");
        count += records.len();
    }
    assert!(count > 100);
}
