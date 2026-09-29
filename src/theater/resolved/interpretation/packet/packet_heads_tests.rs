use super::*;
use crate::theater::Film;
use crate::theater::parser::v41::{FilmSource, FilmSourceMetadata, test_chunks};
use crate::theater::resolved::interpretation::{PickupOutcome, TranslocatorStop};
use serde_json::{Value, json};
use std::io::Read;

fn inflate(fixture: &[u8]) -> Value {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(fixture)
        .read_to_end(&mut raw)
        .unwrap();
    serde_json::from_slice(&raw).unwrap()
}
fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|b| u8::from_str_radix(&hex[b..b + 2], 16).unwrap())
        .collect()
}
fn framed(payload: &[u8]) -> Vec<u8> {
    let mut result = vec![0; 16];
    result[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    result.extend(payload);
    result
}

pub(crate) fn check_read(read: &PacketHeadRead, reference: &Value) {
    let expected = &reference["event"];
    let end = match read {
        PacketHeadRead::Zoom(read) => {
            assert_eq!(json!(read.is_some()), reference["ok"]);
            read.as_ref().map(|r| {
                assert_eq!(json!(r.slot), expected["Slot"]);
                assert_eq!(json!(r.level), expected["Level"]);
                assert_eq!(r.padded_bits, r.head.end_bit.saturating_sub(r.source_bits));
                r.head.end_bit
            })
        }
        PacketHeadRead::Pickup(read) => {
            assert_eq!(
                json!(read.outcome == PickupOutcome::Accepted),
                reference["ok"]
            );
            if read.outcome == PickupOutcome::Accepted {
                assert_eq!(json!(read.slot()), expected["Slot"]);
                assert_eq!(json!(read.class), expected["Class"]);
                assert_eq!(json!(read.catalog_id), expected["CatalogID"]);
            }
            assert_eq!(
                read.padded_bits,
                read.end_bit.saturating_sub(read.source_bits)
            );
            Some(read.end_bit)
        }
        PacketHeadRead::Translocator(read) => {
            assert_eq!(json!(read.is_some()), reference["ok"]);
            read.as_ref().map(|r| {
                assert_eq!(json!(r.event.slot), expected["Slot"]);
                assert_eq!(
                    json!(r.event.positions().is_some()),
                    expected["HasPositions"]
                );
                if let Some([from, to]) = r.event.positions() {
                    let expected_from: [f32; 3] =
                        serde_json::from_value(expected["From"].clone()).unwrap();
                    let expected_to: [f32; 3] =
                        serde_json::from_value(expected["To"].clone()).unwrap();
                    assert_eq!(from.map(f32::to_bits), expected_from.map(f32::to_bits));
                    assert_eq!(to.map(f32::to_bits), expected_to.map(f32::to_bits));
                }
                assert_eq!(r.padded_bits, r.event.end_bit.saturating_sub(r.source_bits));
                r.event.end_bit
            })
        }
    };
    if let Some(end) = end.filter(|_| reference.get("end").is_some()) {
        assert_eq!(json!(end), reference["end"]);
    }
}

#[test]
fn reference_data_packet_heads_padding_oracles() {
    let fixtures: [(u8, &[u8]); 3] = [
        (
            0xca,
            include_bytes!("../../../fixtures/zoom-padding-v41.json.zlib"),
        ),
        (
            0xc4,
            include_bytes!("../../../fixtures/pickup-padding-v41.json.zlib"),
        ),
        (
            0xfa,
            include_bytes!("../../../fixtures/translocator-padding-v41.json.zlib"),
        ),
    ];
    let mut cases = 0;
    let mut selected = 0;
    for (prefix, fixture) in fixtures {
        let oracle = inflate(fixture);
        let rows = oracle
            .as_array()
            .unwrap_or_else(|| oracle["rows"].as_array().unwrap());
        for row in rows {
            cases += 1;
            let payload = bytes(row["hex"].as_str().unwrap());
            let map: Option<FilmMapBounds> = row
                .get("map")
                .and_then(|v| serde_json::from_value(v.clone()).unwrap());
            let read = read_packet_head(0, &payload, map.as_ref());
            assert_eq!(read.is_some(), payload.len() >= 2 && payload[0] == prefix);
            assert!(read_packet_head(6, &payload, map.as_ref()).is_none());
            let Some(read) = read else { continue };
            selected += 1;
            check_read(&read, row);
            let source = FilmSource::load(
                &[
                    [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
                    framed(&payload),
                ],
                &[],
            )
            .unwrap();
            let parsed = Film::parse(test_chunks(&source)).unwrap();
            let resolved = parsed.resolve();
            let interpretation = resolved
                .interpretations()
                .packets
                .iter()
                .find(|entry| entry.source.chunk == 1 && entry.source.packet == 0)
                .unwrap();
            assert_eq!(
                interpretation.packet_head,
                read_packet_head(0, &payload, None)
            );
            assert_eq!(
                serde_json::from_value::<Film>(json!(parsed)).unwrap(),
                parsed
            );
        }
    }
    assert_eq!(cases, 7680);
    assert!(selected > 0);
}

#[test]
fn reference_data_packet_heads_context_and_footer_boundaries() {
    let fixture = inflate(include_bytes!(
        "../../../fixtures/translocator-levelup-v41.json.zlib"
    ));
    let row = fixture
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["map"].is_object() && r["event"]["HasPositions"] == true)
        .unwrap();
    let payload = bytes(row["hex"].as_str().unwrap());
    let packet = framed(&payload);
    let source = FilmSource::load(
        &[
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            packet.clone(),
            packet,
        ],
        &[
            FilmSourceMetadata {
                index: 0,
                chunk_type: 1,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 17,
                chunk_type: 2,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 18,
                chunk_type: 3,
                start_ms: 0,
            },
        ],
    )
    .unwrap();
    let parsed = Film::parse(test_chunks(&source)).unwrap();
    let resolved = parsed.resolve();
    let PacketHeadRead::Translocator(Some(read)) = resolved
        .interpretations()
        .packets
        .iter()
        .find(|entry| entry.source.chunk == 1 && entry.source.packet == 0)
        .and_then(|entry| entry.packet_head.as_ref())
        .unwrap()
    else {
        panic!("missing direct read")
    };
    assert_eq!(read.event.stop, TranslocatorStop::MissingMap);
    assert!(
        resolved
            .interpretations()
            .packets
            .iter()
            .all(|entry| entry.source.chunk != 2)
    );
    assert_eq!(
        serde_json::from_value::<Film>(json!(parsed)).unwrap(),
        parsed
    );
}
