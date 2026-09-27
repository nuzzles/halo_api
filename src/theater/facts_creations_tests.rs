use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn creation(v: &Value) -> FactsEquipmentCreation {
    FactsEquipmentCreation {
        timestamp_us: v["TimestampUS"].as_u64().unwrap(),
        slot: v["Slot"].as_u64().unwrap() as u32,
        generation: v["Gen"].as_u64().unwrap() as u32,
        chunk: v["Chunk"].as_i64().unwrap(),
        packet_index: v["PacketIndex"].as_i64().unwrap(),
        bit_pos: v["BitPos"].as_i64().unwrap(),
        has_ref: v["HasRef"].as_bool().unwrap(),
        reference: v["Ref"].as_u64().unwrap() as u32,
        has_id: v["HasID"].as_bool().unwrap(),
        ability_id: v["AbilityID"].as_u64().unwrap() as u32,
        mpp_present: serde_json::from_value(v["MPPPresent"].clone()).unwrap(),
        mpp_val: serde_json::from_value(v["MPPVal"].clone()).unwrap(),
        position: ["X", "Y", "Z"].map(|key| f32::from_bits(v[key].as_u64().unwrap() as u32)),
        mask: serde_json::from_value(v["Mask"].clone()).unwrap(),
        mask_full: v["MaskFull"].as_bool().unwrap(),
        mask_has_i0: v["MaskHasI0"].as_bool().unwrap(),
        default_state_bits: v["DefaultStateBits"].as_i64().unwrap(),
        has_ammo: v["HasAmmo"].as_bool().unwrap(),
        ammo: serde_json::from_value(v["Ammo"].clone()).unwrap(),
        after_bit: v["AfterBit"].as_i64().unwrap(),
    }
}
fn creation_dump(c: &FactsEquipmentCreation) -> Value {
    json!({"TimestampUS":c.timestamp_us,"Slot":c.slot,"Gen":c.generation,"Chunk":c.chunk,"PacketIndex":c.packet_index,"BitPos":c.bit_pos,"HasRef":c.has_ref,"Ref":c.reference,"HasID":c.has_id,"AbilityID":c.ability_id,"MPPPresent":c.mpp_present,"MPPVal":c.mpp_val,"X":c.position[0].to_bits(),"Y":c.position[1].to_bits(),"Z":c.position[2].to_bits(),"Mask":c.mask,"MaskFull":c.mask_full,"MaskHasI0":c.mask_has_i0,"DefaultStateBits":c.default_state_bits,"HasAmmo":c.has_ammo,"Ammo":c.ammo,"AfterBit":c.after_bit})
}
pub(super) fn world(v: &Value) -> FactsWorldObjectScan {
    let kf = &v["keyframes"];
    FactsWorldObjectScan {
        scanned: v["scanned"].as_bool().unwrap(),
        creations: v["creations"]
            .as_array()
            .unwrap()
            .iter()
            .map(creation)
            .collect(),
        stats: serde_json::from_value(v["stats"].clone()).unwrap(),
        keyframes: FactsWorldKeyframes {
            times_us: serde_json::from_value(kf["times"].clone()).unwrap(),
            seen_us: kf["seen"]
                .as_array()
                .unwrap()
                .iter()
                .map(|life| {
                    (
                        (
                            life["slot"].as_u64().unwrap() as u32,
                            life["generation"].as_u64().unwrap() as u32,
                        ),
                        serde_json::from_value(life["times"].clone()).unwrap(),
                    )
                })
                .collect(),
        },
        tracks: v["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tr| FactsProjectileTrack {
                slot: tr["slot"].as_u64().unwrap() as u32,
                generation: tr["generation"].as_u64().unwrap() as u32,
                points: tr["points"].as_array().map(|pts| {
                    pts.iter()
                        .map(|p| FactsProjectileSample {
                            timestamp_us: p["timestamp_us"].as_u64().unwrap(),
                            chunk: p["chunk"].as_i64().unwrap(),
                            position: std::array::from_fn(|i| {
                                f32::from_bits(p["bits"][i].as_u64().unwrap() as u32)
                            }),
                            at_rest: p["at_rest"].as_bool().unwrap(),
                        })
                        .collect()
                }),
            })
            .collect(),
    }
}
pub(super) fn world_dump(s: &FactsWorldObjectScan) -> Value {
    json!({"scanned":s.scanned,"creations":s.creations.iter().map(creation_dump).collect::<Vec<_>>(),"stats":s.stats,"keyframes":{"times":s.keyframes.times_us,"seen":s.keyframes.seen_us.iter().map(|(&(slot,generation),times)|json!({"slot":slot,"generation":generation,"times":times})).collect::<Vec<_>>()},"tracks":s.tracks.iter().map(|tr|json!({"slot":tr.slot,"generation":tr.generation,"points":tr.points.as_ref().map(|pts|pts.iter().map(|p|json!({"timestamp_us":p.timestamp_us,"chunk":p.chunk,"bits":p.position.map(f32::to_bits),"at_rest":p.at_rest})).collect::<Vec<_>>())})).collect::<Vec<_>>()})
}
#[test]
fn native_facts_world_creation_codec() {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(include_bytes!("fixtures/facts-world-v41.json.zlib").as_slice())
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 18945);
    for (i, row) in rows.iter().enumerate() {
        let source = world(&row["source"]);
        let mut writer = NativeFactsWriter::default();
        encode_facts_world_scan(&mut writer, &source);
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded_hex"].as_str().unwrap()),
            "encoded {i}"
        );
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut reader = NativeFactsReader::new(&input);
        let decoded = decode_facts_world_scan(&mut reader);
        assert_eq!(world_dump(&decoded), row["decoded"], "decoded {i}");
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
