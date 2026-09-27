//! Direct native oracle for the shared leaf readers, independent of component routing.
use super::*;
use serde_json::{Value, json};
use std::io::Read;

#[test]
fn native_leaf_readers() {
    #[derive(Deserialize)]
    struct Step {
        mode: u8,
        end: usize,
        value: u64,
        tail: u64,
        present: bool,
        references: Value,
    }
    #[derive(Deserialize)]
    struct Case {
        hex: String,
        start: usize,
        category: u8,
        width: usize,
        steps: Vec<Step>,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("../fixtures/leaf-readers-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 768);
    let mut switched = 0;
    for (i, c) in cases.into_iter().enumerate() {
        let data: Vec<u8> = c
            .hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
            .collect();
        for step in c.steps {
            let mut r = Reader {
                native_widths: None,
                width_error: None,
                live_observer: None,
                live_grammar: None,
                position_capture: None,
                position_start: 0,
                position_slot: 0,
                position_fallback: false,
                movement_slot: None,
                references: Vec::new(),
                diagnostics: Default::default(),
                cursor: Cursor::new_padded(&data, c.start),
                fields: Vec::new(),
                position_encoding: None,
            };
            match step.mode {
                0 => r.handle("ref", c.category).unwrap(),
                1 => r.optional_handle("ref", c.category).unwrap(),
                2 => r.optional_word_reference("ref", true).unwrap(),
                3 => r.gate("gate", c.width, true).unwrap(),
                4 => r.gate("gate", c.width, false).unwrap(),
                5 => r.gate("id2", 2, false).unwrap(),
                _ => unreachable!(),
            }
            assert_eq!(
                serde_json::json!(r.cursor.position),
                serde_json::json!(step.end),
                "end {i}/{}",
                step.mode
            );
            if step.mode == 0 {
                assert_eq!(
                    (
                        r.fields[r.fields.len() - 2].raw,
                        r.fields[r.fields.len() - 1].raw
                    ),
                    (step.value, step.tail),
                    "value {i}"
                );
                switched += usize::from(c.category == 1 && r.fields[0].raw == 1);
            } else if step.mode == 1 {
                let v = &r.references[0];
                assert_eq!(
                    (v.value as u64, v.tail as u64, v.present),
                    (step.value, step.tail, step.present)
                );
            }
            let refs:Vec<_>=r.references.iter().map(|v|json!({"Kind":match v.kind {NativeUnitReferenceKind::VariableWidth=>0,NativeUnitReferenceKind::GatedWord32=>1,NativeUnitReferenceKind::Word32=>2},"StartBit":v.start_bit,"EndBit":v.end_bit,"Present":v.present,"Val":v.value,"Tail":v.tail,"Probe":v.probe})).collect();
            assert_eq!(json!(refs), step.references, "references {i}/{}", step.mode);
            let observed: Vec<_> = r
                .diagnostics
                .component_observations
                .iter()
                .map(|v| match v {
                    crate::theater::FilmComponentObservation::UnitReference { reference } => {
                        reference
                    }
                    _ => panic!("unexpected leaf observation"),
                })
                .collect();
            assert_eq!(observed, r.references.iter().collect::<Vec<_>>());
        }
    }
    assert!(switched > 0);
}
