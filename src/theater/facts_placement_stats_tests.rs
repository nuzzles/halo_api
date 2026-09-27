use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
pub(super) fn source(v: &Value) -> FactsPlacementStats {
    let c: Vec<i64> = serde_json::from_value(v["calibration"].clone()).unwrap();
    let n: Vec<i64> = serde_json::from_value(v["counts"].clone()).unwrap();
    FactsPlacementStats {
        scanned: v["scanned"].as_bool().unwrap(),
        calibration: FactsMppCalibration {
            widths: (c[0], c[1]),
            agree: c[2],
            runner: (c[3], c[4]),
            runner_agree: c[5],
            anchors: c[6],
            chunks: c[7],
            lives: c[8],
            by_widths: v["widths"].as_array().map(|rows| {
                rows.iter()
                    .map(|p| {
                        (
                            (p[0].as_i64().unwrap(), p[1].as_i64().unwrap()),
                            p[2].as_i64().unwrap(),
                        )
                    })
                    .collect()
            }),
        },
        lives: n[0],
        slots: n[1],
        anchors: n[2],
        accepted: n[3],
        confirmed: n[4],
        placements: n[5],
        by_id: serde_json::from_value(v["ids"].clone()).unwrap(),
        format_version: v["version"].as_i64().unwrap(),
        format_sans_profil: v["sans_profil"].as_bool().unwrap(),
    }
}
pub(super) fn dump(s: &FactsPlacementStats) -> Value {
    let c = &s.calibration;
    json!({"scanned":s.scanned,"calibration":[c.widths.0,c.widths.1,c.agree,c.runner.0,c.runner.1,c.runner_agree,c.anchors,c.chunks,c.lives],"widths":c.by_widths.as_ref().map(|p|p.iter().map(|(&(l,i),&v)|[l,i,v]).collect::<Vec<_>>()),"counts":[s.lives,s.slots,s.anchors,s.accepted,s.confirmed,s.placements],"ids":s.by_id,"version":s.format_version,"sans_profil":s.format_sans_profil})
}
#[test]
fn native_facts_placement_statistics() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-placement-stats-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 21908);
    for (i, row) in rows.iter().enumerate() {
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let decoded = match row["kind"].as_str() {
            Some("widths") => FactsPlacementStats {
                calibration: FactsMppCalibration {
                    by_widths: decode_facts_width_pairs(&mut r),
                    ..Default::default()
                },
                ..Default::default()
            },
            Some("ids") => FactsPlacementStats {
                by_id: decode_facts_id_pairs(&mut r),
                ..Default::default()
            },
            None => {
                let mut w = NativeFactsWriter::default();
                encode_facts_placement_stats(&mut w, &source(&row["source"]));
                assert_eq!(
                    w.bytes(),
                    unhex(row["encoded_hex"].as_str().unwrap()),
                    "encoded {i}"
                );
                decode_facts_placement_stats(&mut r)
            }
            other => panic!("unexpected codec {other:?}"),
        };
        assert_eq!(dump(&decoded), row["decoded"], "decoded {i}");
        assert_eq!(
            r.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        assert_eq!(
            r.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
    }
}
