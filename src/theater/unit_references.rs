//! Native entity-reference observations, with uninterpreted values and exact ranges.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeUnitReferenceKind {
    VariableWidth,
    GatedWord32,
    Word32,
}
impl NativeUnitReferenceKind {
    pub fn native_name(self) -> &'static str {
        match self {
            Self::VariableWidth => "varw",
            Self::GatedWord32 => "w32g",
            Self::Word32 => "w32",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeUnitReference {
    pub kind: NativeUnitReferenceKind,
    pub start_bit: i64,
    pub end_bit: i64,
    pub present: bool,
    /// Raw index or full word; domain-specific bases are not guessed.
    pub value: u32,
    pub tail: u32,
    /// Native category-one flag; remains true even when the presence gate is closed.
    pub probe: bool,
}

#[cfg(test)]
mod tests {
    use crate::theater::*;
    use std::io::Read;
    #[derive(serde::Deserialize)]
    struct Case {
        #[serde(default)]
        name: Option<String>,
        archetype: u32,
        #[serde(default)]
        level: u32,
        start_bit: usize,
        end_bit: usize,
        hex: String,
        position_encoding: PositionEncoding,
        #[serde(default)]
        mpp_widths: Option<[usize; 2]>,
        references: serde_json::Value,
        observations: Option<Vec<FilmComponentObservation>>,
    }
    fn compare(bytes: &[u8]) {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert!(cases.len() > 1000);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let component = if let Some(name) = &c.name {
                super::super::components::decode_component_attempt(
                    &data,
                    c.start_bit,
                    name,
                    c.level,
                    c.archetype,
                    Some(&c.position_encoding),
                )
                .1
            } else {
                let ComponentDecode::Decoded(v) = decode_default_state(
                    &data,
                    c.start_bit,
                    c.archetype,
                    c.mpp_widths.unwrap(),
                    Some(&c.position_encoding),
                ) else {
                    panic!("default {i}");
                };
                v
            };
            assert_eq!(
                serde_json::json!(component.end_bit),
                serde_json::json!(c.end_bit),
                "end {i} {:?}",
                c.name
            );
            if let Some(expected) = c.observations {
                assert_eq!(
                    component.diagnostics.component_observations, expected,
                    "default observations {i} ti{}",
                    c.archetype
                );
            }
            let refs:Vec<_>=component.references.iter().map(|r|serde_json::json!({"Kind":match r.kind{NativeUnitReferenceKind::VariableWidth=>0,NativeUnitReferenceKind::GatedWord32=>1,NativeUnitReferenceKind::Word32=>2},"StartBit":r.start_bit,"EndBit":r.end_bit,"Present":r.present,"Val":r.value,"Tail":r.tail,"Probe":r.probe})).collect();
            assert_eq!(
                serde_json::to_value(refs).unwrap(),
                c.references,
                "refs {i} {:?} ti{}",
                c.name,
                c.archetype
            );
            let restored: DecodedComponent =
                serde_json::from_slice(&serde_json::to_vec(&component).unwrap()).unwrap();
            assert_eq!(restored, component);
        }
    }
    #[test]
    fn native_default_hook_reads() {
        compare(include_bytes!(
            "fixtures/default-hook-reads-d61443e-v41.json.zlib"
        ));
    }
    #[test]
    fn native_keyframe_references() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/keyframe-references-d61443e-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 256);
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: (0..10)
                .map(|i| FilmArchetype {
                    index: i,
                    components: if i == 9 {
                        vec![
                            "unit-actor-control-component".into(),
                            "unit-actor-state-component".into(),
                            "weapon-state-ammo".into(),
                        ]
                    } else {
                        vec![]
                    },
                    levels: if i == 9 { vec![2, 4, 1] } else { vec![] },
                })
                .collect(),
        };
        for (i, c) in cases.into_iter().enumerate() {
            let hex = c["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&hex[n..n + 2], 16).unwrap())
                .collect();
            let record = decode_keyframe_record(
                &data,
                c["start_bit"].as_u64().unwrap() as usize,
                &registry,
                [9, 5],
                Some(&NativeScanProfile::default().component_encoding().unwrap()),
                c["check"].as_bool().unwrap(),
            )
            .unwrap();
            assert_eq!(
                serde_json::json!(record.end_bit),
                serde_json::json!(c["end_bit"].as_u64().unwrap() as usize),
                "end {i}"
            );
            let refs:Vec<_>=record.references.iter().map(|r|serde_json::json!({"Kind":match r.kind{NativeUnitReferenceKind::VariableWidth=>0,NativeUnitReferenceKind::GatedWord32=>1,NativeUnitReferenceKind::Word32=>2},"StartBit":r.start_bit,"EndBit":r.end_bit,"Present":r.present,"Val":r.value,"Tail":r.tail,"Probe":r.probe})).collect();
            assert_eq!(
                serde_json::to_value(refs).unwrap(),
                c["references"],
                "refs {i}"
            );
            let restored: KeyframeRecord =
                serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
            assert_eq!(restored, record);
        }
    }
    #[test]
    fn native_component_references() {
        compare(include_bytes!(
            "fixtures/unit-references-d61443e-v41.json.zlib"
        ));
    }
    #[test]
    fn native_default_references() {
        compare(include_bytes!(
            "fixtures/default-references-d61443e-v41.json.zlib"
        ));
    }
}
