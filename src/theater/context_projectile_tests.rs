use super::*;
use serde_json::{Value, json};
use std::io::Read;

fn bytes(v: &Value) -> Vec<u8> {
    let s = v.as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|p| u8::from_str_radix(&s[p..p + 2], 16).unwrap())
        .collect()
}
fn descriptor(v: &Value) -> NativePrecisionDescriptor {
    NativePrecisionDescriptor {
        index_bits: v["index_bits"].as_u64().unwrap(),
        axis_bits: serde_json::from_value(v["axis_bits"].clone()).unwrap(),
        region: v["region"].as_u64().unwrap() as u32,
    }
}
fn sample(s: &WorldObjectSample) -> Value {
    // JSON cannot encode infinity or NaN: compare their IEEE representations.
    // NaN payload/sign is not a portable arithmetic guarantee.
    fn bits(v: f32) -> u32 {
        if v.is_nan() {
            f32::NAN.to_bits()
        } else {
            v.to_bits()
        }
    }
    json!({"time":s.timestamp_us,"chunk":s.chunk,"xyz":[bits(s.x),bits(s.y),bits(s.z)],"rest":s.at_rest})
}
fn normalize(v: &mut Value) {
    match v {
        Value::Object(o) => {
            if let Some(Value::Array(xyz)) = o.get_mut("xyz") {
                for x in xyz {
                    if f32::from_bits(x.as_u64().unwrap() as u32).is_nan() {
                        *x = json!(f32::NAN.to_bits());
                    }
                }
            }
            for v in o.values_mut() {
                normalize(v);
            }
        }
        Value::Array(a) => {
            for v in a {
                normalize(v);
            }
        }
        _ => (),
    }
}
#[test]
fn native_context_projectiles() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/context-projectile-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let mut fixture: Value = serde_json::from_slice(&raw).unwrap();
    normalize(&mut fixture);
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 180);
    let mut totals = [0; 3];
    for (case, row) in rows.iter().enumerate() {
        let inputs = row["inputs"].as_array().unwrap();
        let buffers: Vec<_> = inputs.iter().map(|c| bytes(&c["hex"])).collect();
        let meta: Vec<_> = inputs
            .iter()
            .map(|c| FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            })
            .collect();
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &meta).unwrap());
        let mut context = NativeFilmContext::new(source.as_ref());
        context.set_world_precision(descriptor(row));
        let map = FilmMapBounds {
            module: String::new(),
            min: [-100., -200., -300.],
            max: [
                if row["zero_range"] == true {
                    -100.
                } else {
                    300.
                },
                400.,
                500.,
            ],
            axis_widths: [6; 3],
            region: 0,
            region_index_bits: 1,
        };
        let out = scan_context_projectiles(&context, (row["no_bounds"] != true).then_some(&map));
        if row["error"] != "" {
            assert_eq!(
                out.unwrap_err().to_string(),
                row["error"].as_str().unwrap(),
                "error {case}"
            );
            totals[0] += 1;
            continue;
        }
        let out = out.unwrap();
        assert_eq!(
            json!(out.census.archetypes[&41].band),
            row["band"],
            "band {case}"
        );
        let tracks: Vec<_> = out.positions.tracks.iter().map(|t| json!({"slot":t.slot,"generation":t.generation,"points":t.pts.iter().map(sample).collect::<Vec<_>>()})).collect();
        assert_eq!(json!(tracks), row["tracks"], "tracks {case}");
        let records: Vec<_> = out.positions.records.iter().map(|r| {
            assert_eq!(meta[r.source.chunk_index as usize].index, r.record.sample.chunk);
            assert!(r.record.bit < r.source.payload_size * 8);
            json!({"slot":r.record.slot,"generation":r.record.generation,"sample":sample(&r.record.sample),"packet":r.packet_index,"start":r.source.payload_offset,"bit":r.record.bit,"padded_bits":r.record.padded_bits})
        }).collect();
        assert_eq!(json!(records), row["records"], "records {case}");
        totals[1] += records.len();
        totals[2] += tracks.len();
    }
    assert_eq!(totals, [97, 3422, 164]);
    let primitives = fixture["primitives"].as_array().unwrap();
    assert_eq!(primitives.len(), 128);
    let map = FilmMapBounds {
        module: String::new(),
        min: [-100., -200., -300.],
        max: [300., 400., 500.],
        axis_widths: [6; 3],
        region: 0,
        region_index_bits: 1,
    };
    for (case, row) in primitives.iter().enumerate() {
        let result = decode_world_object_position_with_descriptor(
            &bytes(&row["hex"]),
            row["at"].as_i64().unwrap(),
            &map,
            &descriptor(row),
        );
        assert_eq!(
            result.is_some(),
            row["ok"].as_bool().unwrap(),
            "primitive {case}"
        );
        if let Some((xyz, off)) = result {
            assert_eq!(off, row["end"].as_i64().unwrap(), "primitive end {case}");
            assert_eq!(json!(xyz.map(f32::to_bits)), row["xyz"], "primitive {case}");
        }
    }
}
