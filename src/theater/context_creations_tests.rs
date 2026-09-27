use super::*;
use serde_json::{Value, json};
use std::io::Read;

#[test]
fn native_context_biped_creations() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/creation-context-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 512);
    assert!(matches!(
        scan_context_biped_creations(None, None),
        Err(ContextCreationScanError::AbsentContext)
    ));
    let mut published = 0;
    for (i, case) in cases.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut metadata = Vec::new();
        for input in case["inputs"].as_array().unwrap() {
            let hex = input["hex"].as_str().unwrap();
            buffers.push(
                (0..hex.len())
                    .step_by(2)
                    .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            metadata.push(FilmSourceMetadata {
                index: input["index"].as_i64().unwrap(),
                chunk_type: 2,
                start_ms: 0,
            });
        }
        let source = if buffers.is_empty() {
            None
        } else {
            Some(FilmSource::load(&buffers, &metadata).unwrap())
        };
        let context = NativeFilmContext::new(source.as_ref());
        context.observation().set_hook(
            NativeHookKind::RecordMask,
            Some(std::sync::Arc::new(|_| {
                panic!("creation scan must not emit mask hooks")
            })),
        );
        let slots: Vec<u32> = serde_json::from_value(case["slots"].clone()).unwrap();
        let band = FilmSlotBand::from_slots(slots).unwrap();
        for (selected, expected) in [(Some(&band), case), (None, &case["automatic"])] {
            let result = scan_context_biped_creations(Some(&context), selected);
            let error = match &result {
                Ok(_) => "",
                Err(ContextCreationScanError::NoChunks) => "source",
                Err(ContextCreationScanError::NoBipeds | ContextCreationScanError::EmptyBand) => {
                    "band"
                }
                Err(e) => panic!("unexpected error {i}: {e}"),
            };
            assert_eq!(error, expected["error"], "case {i}");
            let out = result.unwrap_or_default();
            if error.is_empty() {
                assert_eq!(json!(out.slots), expected["slots"]);
            }
            let s = &out.stats;
            let (word, count) = s.most_common_other().unwrap_or_default();
            assert_eq!(
                json!({"Slots":s.slots,"Anchors":s.anchors,"Truncated":s.truncated,"ShapeBad":s.shape_bad,"SignatureMismatch":s.signature_mismatch,"OtherWord":word,"OtherWordCount":count,"GateClosed":s.gate_closed,"Accepted":s.accepted}),
                expected["stats"],
                "stats {i}"
            );
            let records: Vec<_> = out.records.iter().map(|r| {
                let c = &r.creation;
                let fact = FactsBipedCreation::from(r);
                assert!(fact.has_index);
                assert_eq!(fact.slot, c.slot);
                assert_eq!(fact.generation, u32::from(c.generation));
                assert_eq!(fact.participant_index, u32::from(c.participant_index));
                assert_eq!(fact.timestamp_us, r.source.timestamp_us);
                let payload = source.as_ref().unwrap().payload(&r.source).unwrap();
                assert_eq!(c.prologue_end_bit, c.start_bit+72);
                assert!(c.prologue_end_bit <= payload.len()*8);
                let number = metadata[r.source.chunk_index as usize].index;
                let (_, packets) = context.chunk_at(number).unwrap();
                assert_eq!(packets[r.packet_index.unwrap()], r.source);
                json!({"Slot":c.slot,"Generation":c.generation,"ParticipantIndex":c.participant_index,"HasIndex":true,"Chunk":number,"PacketIndex":r.packet_index.unwrap(),"TimestampUS":r.source.timestamp_us,"BitPos":c.start_bit,"Version":c.version,"Representation":c.representation})
            }).collect();
            assert_eq!(
                records,
                expected["records"].as_array().cloned().unwrap_or_default(),
                "records {i}"
            );
            published += records.len();
        }
    }
    assert_eq!(published, 1220);
}
