use super::*;
use serde_json::Value;
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_facts_complete_file() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-file-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let f: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(f["cases"].as_array().unwrap().len(), 2919);
    for (i, row) in f["cases"].as_array().unwrap().iter().enumerate() {
        let entry = FactsMapEntry {
            module: row["module"].as_str().unwrap().as_bytes().to_vec(),
            axis_widths: [13, 14, 15],
            bounds: [[-10., 100.], [-30., 300.], [-50., 500.]],
            ..Default::default()
        };
        let input = unhex(row["input"].as_str().unwrap());
        let got = decode_film_facts_file(&input, &entry);
        if row["panic"] == true {
            assert!(got.is_err(), "panic control {i}");
            continue;
        }
        match got {
            Ok(v) => {
                assert_eq!(row["error"], "", "success {i}");
                assert_eq!(
                    encode_film_facts_file(&v).unwrap(),
                    unhex(row["normalized"].as_str().unwrap()),
                    "normalized {i}"
                );
            }
            Err(e) => assert_eq!(e.to_string(), row["error"].as_str().unwrap(), "error {i}"),
        }
    }
    for row in f["writer_failures"].as_array().unwrap() {
        let mut v = NativeFilmFactsFile::default();
        v.facts.header.map_module = b"map".to_vec();
        v.facts.header.axis_widths = [13, 14, 15];
        let bad = f32::from_bits(row["bits"].as_u64().unwrap() as u32);
        if row["kind"] == "blob" {
            v.facts.vehicles.deaths.frame.profile.movement.delta_quantum = bad;
        } else {
            let mut k = FactsKillsResult::default();
            k.profil_calibre.movement.delta_quantum = bad;
            v.kills = Some(k);
        }
        assert_eq!(row["nil_bytes"], true);
        assert_eq!(
            encode_film_facts_file(&v).unwrap_err(),
            row["error"].as_str().unwrap()
        );
    }
}
