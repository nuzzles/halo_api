use super::*;
use crate::theater::parser::v41::chunks::replication::components::cursor::ComponentCursor;
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;
use crate::theater::parser::v41::context::profile::DecodeProfile;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    hex: String,
    bit_len: usize,
    version: u32,
    snapshot: bool,
    observation: Option<AbilityNonPredictedState>,
}

#[test]
fn anchor_observations_keep_recorded_forms_and_unknown_prefixes_distinct() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../../../../../../fixtures/components-v41/cases.json"
    ))
    .unwrap();
    let mut count = 0;
    let mut refusals = [0, 0];
    for case in cases {
        let Some(expected) = case.observation else {
            continue;
        };
        count += 1;
        let bytes: Vec<_> = case
            .hex
            .as_bytes()
            .chunks(2)
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect();
        let mut encoding = DecodeProfile::default().component_encoding().unwrap();
        encoding.world_axis_bits = Some([15, 15, 17]);
        encoding.region_axis_bits = [(0, [15, 15, 17])].into();
        let indexed = |range: BitRange| {
            !case.snapshot
                && RawBits::from_source(&bytes, range.start, 1)
                    .unwrap()
                    .low_u64()
                    == 0
        };
        for context in [true, false] {
            let mut r = ComponentReader {
                category_references: true,
                snapshot: case.snapshot,
                cursor: ComponentCursor::guarded(&bytes, 0),
                fields: vec![],
                position_encoding: context.then_some(&encoding),
                reference_widths: None,
                width_error: None,
                context_error: None,
                live_grammar: None,
                movement_slot: None,
                references: vec![],
                diagnostics: Default::default(),
            };
            let status = non_predicted(&mut r, case.version);
            let mut projected = expected.clone();
            let mut missing = None;
            if !context {
                if let Some(range) = expected.position
                    && !expected.reference.as_ref().unwrap().present
                    && indexed(range)
                {
                    missing = Some(range.start);
                    refusals[0] += 1;
                    projected = AbilityNonPredictedState {
                        tag: expected.tag,
                        body_walked: true,
                        inner: expected.inner,
                        reference: expected.reference.clone(),
                        ..Default::default()
                    };
                } else if let Some(range) = expected.destination
                    && indexed(range)
                {
                    missing = Some(range.start);
                    refusals[1] += 1;
                    projected.body_ok = false;
                    projected.destination = None;
                    projected.packed = None;
                    projected.tail = None;
                }
            }
            assert_eq!(
                status,
                Some(missing.is_none()),
                "{} context {context}",
                case.name
            );
            assert_eq!(
                r.cursor.position as usize,
                missing.map_or(case.bit_len, |bit| bit + 1)
            );
            let observations: Vec<_> = r
                .diagnostics
                .component_observations
                .iter()
                .filter_map(|o| {
                    if let FilmComponentObservation::AbilityNonPredicted { state } = o {
                        Some(state.as_ref())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                observations,
                vec![&projected],
                "{} context {context}",
                case.name
            );
            // Complete ranges refer to the authored position encoding, whether
            // raw, local or quantized. No absent value becomes a zero position.
            for range in [projected.position, projected.destination]
                .into_iter()
                .flatten()
            {
                assert!(r.fields.iter().any(|f| f.bit == range.start));
                assert!(r.fields.iter().any(|f| f.bit + f.width == range.end));
                assert!(
                    RawBits::from_source(&bytes, range.start, range.end - range.start).is_some()
                );
            }
            let json = serde_json::to_string(&projected).unwrap();
            assert_eq!(
                projected,
                serde_json::from_str::<AbilityNonPredictedState>(&json).unwrap()
            );
        }
    }
    assert_eq!(count, 920);
    assert!(refusals.into_iter().all(|count| count > 0));
}
