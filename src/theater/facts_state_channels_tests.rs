use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn delta(v: &Value) -> FactsDeltaChannels {
    FactsDeltaChannels {
        ability_ranks: serde_json::from_value(v["AbilityRanks"].clone()).unwrap(),
        camo_states: serde_json::from_value(v["CamoStates"].clone()).unwrap(),
        grapple_reads: serde_json::from_value(v["GrappleReads"].clone()).unwrap(),
        translocations: v["Translocations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| FactsTranslocation {
                timestamp_us: t["TimestampUS"].as_u64().unwrap(),
                slot: t["Slot"].as_u64().unwrap() as u32,
                has_positions: t["HasPositions"].as_bool().unwrap(),
                from: std::array::from_fn(
                    |a| f32::from_bits(t["From"][a].as_u64().unwrap() as u32),
                ),
                to: std::array::from_fn(|a| f32::from_bits(t["To"][a].as_u64().unwrap() as u32)),
            })
            .collect(),
    }
}
fn dump_delta(g: &FactsDeltaChannels) -> Value {
    json!({"AbilityRanks":g.ability_ranks,"CamoStates":g.camo_states,"GrappleReads":g.grapple_reads,
 "Translocations":g.translocations.iter().map(|t| json!({"TimestampUS":t.timestamp_us,"Slot":t.slot,"HasPositions":t.has_positions,"From":t.from.map(f32::to_bits),"To":t.to.map(f32::to_bits)})).collect::<Vec<_>>()})
}
#[test]
fn native_facts_state_channel_codecs() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-state-channels-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 36545);
    for (i, row) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let input = unhex(row["input"].as_str().unwrap());
        let mut w = NativeFactsWriter::default();
        let mut r = NativeFactsReader::new(&input);
        let got = match row["kind"].as_str().unwrap() {
            "delta" => {
                encode_facts_delta_channels(&mut w, &delta(&row["source"]));
                dump_delta(&decode_facts_delta_channels(&mut r))
            }
            "abilities" => {
                encode_facts_abilities(
                    &mut w,
                    &serde_json::from_value(row["source"].clone()).unwrap(),
                );
                json!(decode_facts_abilities(&mut r))
            }
            "movement" => {
                encode_facts_movement(
                    &mut w,
                    &serde_json::from_value(row["source"].clone()).unwrap(),
                );
                json!(decode_facts_movement(&mut r))
            }
            other => panic!("unknown section {other}"),
        };
        assert_eq!(
            w.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "writer {i}"
        );
        assert_eq!(r.offset(), row["offset"], "cursor {i}");
        if row["panic"].as_bool().unwrap() {
            assert!(r.error().is_some(), "safe native panic {i}");
            continue;
        }
        assert_eq!(r.error().unwrap_or(""), row["error"], "error {i}");
        assert_eq!(got, row["decoded"], "fields {i}");
    }
}
