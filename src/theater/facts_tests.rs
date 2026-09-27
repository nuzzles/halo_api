use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn world(positions: &[FactsBipedPosition]) -> Value {
    json!(positions.iter().map(|p|json!({"Slot":p.slot,"TimestampUS":p.timestamp_us,"HasWorld":p.has_world,"Q":p.quantized,"X":p.world[0].to_bits(),"Y":p.world[1].to_bits(),"Z":p.world[2].to_bits()})).collect::<Vec<_>>())
}
#[test]
fn native_facts_complete_blob() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-assembly-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let f: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(f["cases"].as_array().unwrap().len(), 2538);
    for (i, row) in f["cases"].as_array().unwrap().iter().enumerate() {
        let input = unhex(row["input"].as_str().unwrap());
        let entry = FactsMapEntry {
            module: serde_json::from_value(row["module"].clone()).unwrap(),
            axis_widths: [13, 14, 15],
            bounds: [[-10., 100.], [-30., 300.], [-50., 500.]],
            ..Default::default()
        };
        match decode_film_facts(&input, &entry) {
            Err(e) => assert_eq!(e.to_string(), row["error"], "error {i}"),
            Ok(g) => {
                assert_eq!(row["error"], "", "unexpected success {i}");
                let w = encode_film_facts(&g);
                assert_eq!(w.error(), None, "normalized writer {i}");
                assert_eq!(
                    w.bytes(),
                    unhex(row["normalized"].as_str().unwrap()),
                    "normalized bytes {i}"
                );
                assert_eq!(
                    world(&g.positions),
                    row["world"]["Positions"],
                    "biped world {i}"
                );
                assert_eq!(
                    world(&g.vehicles.positions),
                    row["world"]["Vehicles"],
                    "vehicle world {i}"
                );
            }
        }
    }
    for (i, row) in f["writer_failures"].as_array().unwrap().iter().enumerate() {
        let mut g = NativeFilmFacts {
            header: FactsHeader {
                map_module: b"map".to_vec(),
                axis_widths: [13, 14, 15],
                ..Default::default()
            },
            ..Default::default()
        };
        g.vehicles.deaths.frame.profile.movement.delta_quantum =
            f32::from_bits(row["bits"].as_u64().unwrap() as u32);
        let w = encode_film_facts(&g);
        assert_eq!(w.error().unwrap(), row["error"], "writer failure {i}");
        assert_eq!(
            w.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "partial encoded {i}"
        );
    }
}
