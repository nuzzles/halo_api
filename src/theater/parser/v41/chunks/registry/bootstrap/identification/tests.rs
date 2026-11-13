use super::*;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Case {
    name: String,
    start_bit: usize,
    bit_len: usize,
    fields: Vec<Field>,
    hex: String,
}
#[derive(Deserialize)]
struct Field {
    kind: Kind,
    bit: usize,
    width: usize,
    hex: String,
}
#[derive(Deserialize)]
struct Reference {
    commit: String,
    cases: Vec<Trace>,
}
#[derive(Deserialize)]
struct Trace {
    name: String,
    trace: Vec<(usize, usize)>,
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../fixtures/bootstrap-v41/identification.json"
    ))
    .unwrap()
}

#[test]
fn identification_preserves_all_fields_and_matches_independent_header_locations() {
    let reference: Reference = serde_json::from_str(include_str!(
        "../../../../../../fixtures/bootstrap-v41/identification-reference.json"
    ))
    .unwrap();
    assert_eq!(reference.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    let cases = cases();
    assert_eq!(cases.len(), reference.cases.len());
    for (case, reference) in cases.iter().zip(reference.cases) {
        let data = bytes(&case.hex);
        let actual = read(&data, case.start_bit);
        assert_eq!(actual.stop, BootstrapIdentificationStop::Complete);
        assert_eq!(actual.start_bit, 4769);
        assert_eq!(actual.end_bit, case.bit_len);
        assert_eq!(actual.end_bit - actual.start_bit, 12848 * 8);
        assert_eq!(actual.fields.len(), 11);
        for (field, expected) in actual.fields.iter().zip(&case.fields) {
            assert_eq!(
                (field.kind, field.bit, field.width),
                (expected.kind, expected.bit, expected.width)
            );
            assert_eq!(field.raw.as_ref().unwrap().bytes, bytes(&expected.hex));
            assert_eq!(
                field.raw,
                RawBits::from_source(&data, field.bit, field.width)
            );
        }
        assert_eq!(reference.name, case.name);
        for (at, width) in reference.trace {
            assert!(
                actual
                    .fields
                    .iter()
                    .any(|f| f.bit == at && f.width == width)
            );
        }
        // Game options remain after the established identification boundary.
        assert_eq!(
            RawBits::from_source(&data, actual.end_bit, 6)
                .unwrap()
                .low_u64(),
            0b101011
        );
        let full = super::super::read_fields(&data, 0);
        assert_eq!(full.identification.as_ref(), Some(&actual));
        assert_eq!(
            serde_json::from_slice::<BootstrapIdentificationRead>(
                &serde_json::to_vec(&actual).unwrap()
            )
            .unwrap(),
            actual
        );
    }
}

#[test]
fn identification_truncation_retains_the_prefix_without_reading_later_fields() {
    for case in cases() {
        let source = bytes(&case.hex);
        let full = read(&source, case.start_bit);
        let mut cuts = BTreeSet::new();
        for field in &case.fields {
            for n in 0..field.width {
                if field.width <= 128 || n < 8 || n >= field.width - 8 || n == field.width / 2 {
                    cuts.insert(field.bit + n);
                }
            }
        }
        for cut in cuts {
            let shift = (8 - cut % 8) % 8;
            let mut data = vec![0; (cut + shift) / 8];
            for i in 0..cut {
                data[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let partial = read(&data, case.start_bit + shift);
            let count = full
                .fields
                .iter()
                .take_while(|f| f.bit + f.width <= cut)
                .count();
            assert_eq!(partial.stop, BootstrapIdentificationStop::Truncated);
            assert_eq!(partial.fields.len(), count + 1);
            assert_eq!(partial.end_bit, full.fields[count].bit + shift);
            assert!(partial.fields.last().unwrap().raw.is_none());
            for (a, b) in partial.fields.iter().zip(&full.fields).take(count) {
                assert_eq!((a.kind, a.width, &a.raw), (b.kind, b.width, &b.raw));
                assert_eq!(a.bit, b.bit + shift);
            }
        }
    }
}
