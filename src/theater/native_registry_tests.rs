use super::*;
use std::io::Read;
fn unhex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_registry_ordered_read_oracle() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/native-registry-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 122);
    for (index, case) in cases.iter().enumerate() {
        let data = unhex(case["hex"].as_str().unwrap());
        let read = parse_registry_chunk(&data);
        assert_eq!(
            read.is_err(),
            case["error"].as_bool().unwrap(),
            "case {index}"
        );
        let Ok(read) = read else { continue };
        let blocks = read.block_reads.as_ref().unwrap();
        let expected = case["blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), expected.len());
        assert_eq!(
            blocks.iter().filter(|b| b.accepted).count(),
            read.registry.archetypes.len()
        );
        assert_eq!(read.registry.archetypes.len(), case["accepted_blocks"]);
        for (block, expected) in blocks.iter().zip(expected) {
            assert_eq!(block.index, expected["index"]);
            assert_eq!(block.start_byte, expected["start_byte"]);
            assert_eq!(block.end_byte, expected["end_byte"]);
            assert_eq!(block.tail_start_byte, expected["tail_start_byte"]);
            assert_eq!(
                block.tail_checked_end_byte,
                expected["tail_checked_end_byte"]
            );
            assert_eq!(
                block.first_nonzero_byte.map_or(-1, |v| v as i64),
                expected["first_nonzero_byte"]
            );
            assert_eq!(block.accepted, expected["accepted"]);
            let slots = expected["slots"].as_array().unwrap();
            assert_eq!(block.slots.len(), slots.len());
            for (slot, expected) in block.slots.iter().zip(slots) {
                assert_eq!(slot.index, expected["index"]);
                assert_eq!(slot.start_byte, expected["start_byte"]);
                assert_eq!(
                    slot.name_bytes,
                    unhex(expected["name_hex"].as_str().unwrap())
                );
                assert_eq!(slot.name.as_deref().unwrap_or_default(), expected["name"]);
                assert_eq!(
                    slot.level_bytes.map(u32::from_le_bytes).map(u64::from),
                    expected["level"].as_u64()
                );
                assert_eq!(
                    &slot.name_bytes,
                    &data[slot.start_byte..slot.start_byte + 256]
                );
                if let Some(level) = slot.level_bytes {
                    assert_eq!(level, data[slot.start_byte + 256..slot.start_byte + 260]);
                }
            }
            assert!(
                data[block.tail_start_byte..block.first_nonzero_byte.unwrap_or(block.end_byte)]
                    .iter()
                    .all(|b| *b == 0)
            );
            if let Some(first) = block.first_nonzero_byte {
                assert_ne!(data[first], 0);
            }
        }
        assert_eq!(
            read,
            serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap()
        );
        let mut old = serde_json::to_value(&read).unwrap();
        old.as_object_mut().unwrap().remove("block_reads");
        assert!(
            serde_json::from_value::<FilmRegistryRead>(old)
                .unwrap()
                .block_reads
                .is_none()
        );
    }
}
