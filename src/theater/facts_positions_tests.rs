use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> Value {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/facts-positions-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    serde_json::from_slice(&raw).unwrap()
}
fn positions(v: &Value) -> Vec<FactsBipedPosition> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| FactsBipedPosition {
            timestamp_us: p["timestamp_us"].as_u64().unwrap(),
            slot: p["slot"].as_u64().unwrap() as u32,
            has_world: p["has_world"].as_bool().unwrap(),
            quantized: serde_json::from_value(p["quantized"].clone()).unwrap(),
            world: std::array::from_fn(|i| {
                f32::from_bits(p["world_bits"][i].as_u64().unwrap() as u32)
            }),
            has_yaw: p["has_yaw"].as_bool().unwrap(),
            yaw_raw: p["yaw_raw"].as_u64().unwrap() as u32,
            pitch_raw: p["pitch_raw"].as_u64().unwrap() as u32,
            directions: serde_json::from_value(p["directions"].clone()).unwrap(),
            has_body: p["has_body"].as_bool().unwrap(),
            health: f32::from_bits(p["health_bits"].as_u64().unwrap() as u32),
            has_shield: p["has_shield"].as_bool().unwrap(),
            shield: f32::from_bits(p["shield_bits"].as_u64().unwrap() as u32),
            shield_quantum: p["shield_quantum"].as_u64().unwrap() as u8,
        })
        .collect()
}
fn dump(ps: &[FactsBipedPosition]) -> Value {
    json!(ps.iter().map(|p|json!({"timestamp_us":p.timestamp_us,"slot":p.slot,"has_world":p.has_world,"quantized":p.quantized,"world_bits":p.world.map(f32::to_bits),"has_yaw":p.has_yaw,"yaw_raw":p.yaw_raw,"pitch_raw":p.pitch_raw,"directions":p.directions,"has_body":p.has_body,"health_bits":p.health.to_bits(),"has_shield":p.has_shield,"shield_bits":p.shield.to_bits(),"shield_quantum":p.shield_quantum})).collect::<Vec<_>>())
}
#[test]
fn native_facts_position_codec() {
    let oracle = fixture();
    let rows = oracle["positions"].as_array().unwrap();
    assert_eq!(rows.len(), 8122);
    for (i, row) in rows.iter().enumerate() {
        let source = positions(&row["source"]);
        let mut writer = NativeFactsWriter::default();
        encode_facts_positions(&mut writer, &source);
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encoded {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let layout = I0Layout {
            gate_bits: 0,
            axis_widths: serde_json::from_value(row["widths"].clone()).unwrap(),
            region: 0,
        };
        let bounds: [[f32; 3]; 2] = serde_json::from_value(row["bounds"].clone()).unwrap();
        let mut reader = NativeFactsReader::new(&input);
        let decoded = decode_facts_positions(&mut reader, &layout, bounds);
        assert_eq!(
            reader.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        if row["panic"].as_bool().unwrap() {
            assert!(reader.error().is_some());
            continue;
        }
        assert_eq!(dump(&decoded), row["decoded"], "decoded {i}");
        assert_eq!(
            reader.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
    }
}
#[test]
fn native_facts_direction_codec() {
    let oracle = fixture();
    let rows = oracle["directions"].as_array().unwrap();
    assert_eq!(rows.len(), 17709);
    let flags: std::collections::BTreeSet<_> = rows
        .iter()
        .map(|r| unhex(r["encoded_hex"].as_str().unwrap())[0])
        .collect();
    let modes: std::collections::BTreeSet<_> = rows
        .iter()
        .filter_map(|r| {
            let bytes = unhex(r["encoded_hex"].as_str().unwrap());
            (bytes[0] & 128 != 0).then(|| bytes[1])
        })
        .collect();
    assert_eq!(flags.len(), 256);
    assert_eq!(modes, (0..8).collect());
    for (i, row) in rows.iter().enumerate() {
        let source: FactsPositionDirections =
            serde_json::from_value(row["source"].clone()).unwrap();
        let mut writer = NativeFactsWriter::default();
        encode_facts_position_directions(&mut writer, &source);
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encoded {i}"
        );
        let mut seed = source;
        seed.mask_bits = row["seed_mask"].as_u64().unwrap();
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut reader = NativeFactsReader::new(&input);
        decode_facts_position_directions(&mut reader, &mut seed);
        assert_eq!(
            serde_json::to_value(seed).unwrap(),
            row["decoded"],
            "directions {i}"
        );
        assert_eq!(
            reader.offset(),
            row["offset"].as_u64().unwrap() as usize,
            "offset {i}"
        );
        assert_eq!(
            reader.error().unwrap_or(""),
            row["error"].as_str().unwrap(),
            "error {i}"
        );
    }
}
