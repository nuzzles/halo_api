use super::*;
use serde_json::Value;
use std::io::Read;
#[test]
fn native_source_carrier_marks() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/source-carrier-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 96);
    let mut marks = 0;
    for (case, row) in rows.iter().enumerate() {
        let mut data = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            let h = c["hex"].as_str().unwrap();
            data.push(
                (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&data, &meta).unwrap();
        let result = scan_source_carrier_marks(Some(&source));
        assert_eq!(
            result
                .as_ref()
                .err()
                .map(ToString::to_string)
                .unwrap_or_default(),
            row["error"].as_str().unwrap(),
            "error {case}"
        );
        if let Ok(out) = result {
            let mut e = row["scan"].clone();
            for k in ["Marks", "KeyframeUS"] {
                if e[k].is_null() {
                    e[k] = Value::Array(Vec::new());
                }
            }
            let expected: CarrierMarkScan = serde_json::from_value(e).unwrap();
            assert_eq!(out.scan, expected, "scan {case}");
            assert_eq!(out.keyframes.len(), out.scan.keyframe_us.len());
            for (packet, &time) in out.keyframes.iter().zip(&out.scan.keyframe_us) {
                assert_eq!(packet.timestamp_us, time);
                assert_eq!(packet.packet_type, 2);
                assert!(meta[packet.chunk_index as usize].index > 0);
                assert!(
                    packet.payload_offset + packet.payload_size
                        <= data[packet.chunk_index as usize].len()
                );
            }
            let f = FactsCarrierMarkScan::from(&out.scan);
            assert_eq!(f.records, out.scan.records as i64);
            marks += out.scan.marks.len();
        }
    }
    assert!(marks > 0);
    assert!(scan_source_carrier_marks(None).is_err());
}
