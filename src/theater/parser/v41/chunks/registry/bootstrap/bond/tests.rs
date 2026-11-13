use super::*;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Case {
    name: String,
    hex: String,
    bit_len: usize,
}
#[derive(Deserialize)]
struct Oracle {
    commit: String,
    cases: Vec<Trace>,
}
#[derive(Deserialize)]
struct Trace {
    name: String,
    end_bit: usize,
    trace: Vec<(u8, usize, usize)>,
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../fixtures/bond-v2/cases.json"
    ))
    .unwrap()
}
pub(crate) fn atoms<'a>(value: &'a BondValue, out: &mut Vec<&'a BondBits>) {
    match &value.body {
        BondValueBody::Fixed(bits) => out.push(bits),
        BondValueBody::Integer(n) => out.extend(&n.bytes),
        BondValueBody::String { length, data } => {
            out.extend(&length.bytes);
            out.extend(data);
        }
        BondValueBody::Struct { length, fields } => {
            out.extend(&length.bytes);
            for f in fields {
                out.extend(&f.header);
                if let Some(v) = &f.value {
                    atoms(v, out);
                }
            }
        }
        BondValueBody::Sequence {
            header,
            count_encoding,
            packed,
            values,
            ..
        } => {
            out.push(header);
            if let Some(n) = count_encoding {
                out.extend(&n.bytes);
            }
            out.extend(packed);
            for v in values {
                atoms(v, out);
            }
        }
        BondValueBody::Map {
            key_type,
            value_type,
            count,
            entries,
        } => {
            out.push(key_type);
            out.extend(value_type);
            if let Some(n) = count {
                out.extend(&n.bytes);
            }
            for pair in entries {
                atoms(&pair.key, out);
                if let Some(v) = &pair.value {
                    atoms(v, out);
                }
            }
        }
        BondValueBody::Unread => {}
    }
}
fn spans(v: &BondValue, out: &mut BTreeSet<(u8, usize, usize)>) {
    out.insert((v.wire_type, v.source.start, v.source.end));
    match &v.body {
        BondValueBody::Struct { fields, .. } => {
            for f in fields {
                if let Some(v) = &f.value {
                    spans(v, out);
                }
            }
        }
        BondValueBody::Sequence {
            header,
            packed,
            count,
            values,
            ..
        } => {
            for v in values {
                spans(v, out);
            }
            if let (Some(bits), Some(count)) = (packed, count) {
                let ty = header.raw.as_ref().unwrap().low_u64() as u8 & 31;
                let width = match ty {
                    7 => 32,
                    8 => 64,
                    _ => 8,
                };
                for i in 0..*count as usize {
                    out.insert((ty, bits.bit + i * width, bits.bit + (i + 1) * width));
                }
            }
        }
        BondValueBody::Map { entries, .. } => {
            for p in entries {
                spans(&p.key, out);
                if let Some(v) = &p.value {
                    spans(v, out);
                }
            }
        }
        _ => {}
    }
}

#[test]
fn bond_values_match_independent_boundaries_and_reconstruct_every_source_bit() {
    let oracle: Oracle = serde_json::from_str(include_str!(
        "../../../../../../fixtures/bond-v2/reference.json"
    ))
    .unwrap();
    assert_eq!(oracle.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    let cases = cases();
    assert_eq!(cases.len(), oracle.cases.len());
    for (case, expected) in cases.iter().zip(oracle.cases) {
        assert_eq!(case.name, expected.name);
        let mut data = bytes(&case.hex);
        data.extend([0xa5; 8]);
        let full = read(&data, 0);
        assert_eq!(full.stop, BondStop::Complete, "{}", case.name);
        assert_eq!(full.root.source.end, case.bit_len);
        assert_eq!(full.root.source.end, expected.end_bit);
        let mut found = BTreeSet::new();
        spans(&full.root, &mut found);
        for span in expected.trace {
            assert!(found.contains(&span), "{}: {span:?}", case.name);
        }
        let mut fields = Vec::new();
        atoms(&full.root, &mut fields);
        let mut at = 0;
        for f in fields {
            assert_eq!(f.bit, at);
            assert_eq!(f.raw, RawBits::from_source(&data, at, f.width));
            at += f.width;
        }
        assert_eq!(at, case.bit_len);
        assert_eq!(
            serde_json::from_slice::<BondRead>(&serde_json::to_vec(&full).unwrap()).unwrap(),
            full
        );
    }
}

#[test]
fn bond_truncation_never_uses_padding_or_following_struct_bytes() {
    for case in cases() {
        let source = bytes(&case.hex);
        let full = read(&source, 0);
        let mut all = Vec::new();
        atoms(&full.root, &mut all);
        let mut cuts: BTreeSet<_> = (0..case.bit_len)
            .filter(|&n| case.bit_len <= 2048 || n < 128 || n + 128 >= case.bit_len)
            .collect();
        for f in &all {
            cuts.insert(f.bit);
            if f.width > 0 {
                cuts.insert(f.bit + f.width - 1);
            }
        }
        cuts.remove(&case.bit_len);
        for cut in cuts {
            let shift = (8 - cut % 8) % 8;
            let mut data = vec![0; (cut + shift) / 8];
            for i in 0..cut {
                data[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let partial = read(&data, shift);
            assert_ne!(partial.stop, BondStop::Complete, "{}: {cut}", case.name);
            assert!(partial.root.source.end <= cut + shift);
            let mut got = Vec::new();
            atoms(&partial.root, &mut got);
            let available: Vec<_> = got.iter().filter(|a| a.raw.is_some()).collect();
            let expected: Vec<_> = all.iter().take_while(|f| f.bit + f.width <= cut).collect();
            assert_eq!(available.len(), expected.len(), "{}: {cut}", case.name);
            for (a, b) in available.iter().zip(expected) {
                assert_eq!(a.bit, b.bit + shift);
                assert_eq!(a.width, b.width);
                assert_eq!(a.raw, b.raw);
            }
        }
    }
}

fn varint(mut n: usize) -> Vec<u8> {
    let mut b = Vec::new();
    while n >= 128 {
        b.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    b.push(n as u8);
    b
}
#[test]
fn bond_refuses_bad_lengths_types_overflows_and_resource_exhaustion() {
    for (data, stop, end) in [
        (vec![0], BondStop::InvalidLength, 8),
        (vec![2, 0, 0], BondStop::InvalidLength, 16),
        (vec![1, 2, 255], BondStop::StructureBoundary, 16),
        (vec![2, 2, 255, 0], BondStop::InvalidLength, 24),
        (vec![10, 2], BondStop::Truncated, 16),
        (vec![3, 19, 0, 0], BondStop::InvalidType(19), 16),
        (vec![5, 4, 255, 255, 4, 0], BondStop::IntegerOverflow, 40),
        (
            vec![7, 5, 128, 128, 128, 128, 128, 0],
            BondStop::IntegerOverflow,
            56,
        ),
        (vec![3, 11, 32, 0], BondStop::InvalidType(0), 24),
    ] {
        let result = read(&data, 0);
        assert_eq!(result.stop, stop, "{data:?}");
        assert_eq!(result.root.source.end, end);
    }
    let mut nested = vec![1, 0];
    for _ in 0..70 {
        let mut body = vec![10];
        body.extend(nested);
        body.push(0);
        nested = varint(body.len());
        nested.extend(body);
    }
    assert_eq!(read(&nested, 0).stop, BondStop::DepthLimit);
    let mut too_many = varint(MAX_VALUES + 2);
    too_many.extend(vec![1; MAX_VALUES + 1]);
    too_many.push(0);
    assert_eq!(read(&too_many, 0).stop, BondStop::ValueLimit);
    // A gigantic packed byte count must refuse before allocation or iteration.
    assert_eq!(
        read(&[8, 11, 3, 255, 255, 255, 255, 15, 0], 0).stop,
        BondStop::Truncated
    );
}

#[test]
fn bounded_envelope_cannot_borrow_following_fields_at_any_bit_alignment() {
    // The unbounded entry point keeps its physical-truncation classification.
    assert_eq!(read(&[1, 0], 17).stop, BondStop::Truncated);
    for shift in 0..8 {
        // Length=3, field0 uint8=7, terminator; followed by valid-looking data.
        let original = [3u8, 3, 7, 0, 1, 0, 0, 0];
        let mut data = vec![0; (shift + original.len() * 8).div_ceil(8)];
        for (i, b) in original.iter().enumerate() {
            for bit in 0..8 {
                let at = shift + i * 8 + bit;
                data[at / 8] |= ((b >> (7 - bit)) & 1) << (7 - at % 8);
            }
        }
        let end = shift + 32;
        let full = read_bounded(&data, BitRange { start: shift, end });
        assert_eq!(full.stop, BondStop::Complete);
        assert_eq!(full.root.source, BitRange { start: shift, end });
        for limit in shift..end {
            let partial = read_bounded(
                &data,
                BitRange {
                    start: shift,
                    end: limit,
                },
            );
            assert_ne!(
                partial.stop,
                BondStop::Complete,
                "shift {shift} limit {limit}"
            );
            assert!(partial.root.source.end <= limit);
        }
    }
    assert_eq!(
        read_bounded(&[1, 0], BitRange { start: 9, end: 8 }).stop,
        BondStop::InvalidLength
    );
    assert_eq!(
        read_bounded(&[1, 0], BitRange { start: 0, end: 17 }).stop,
        BondStop::Truncated
    );
}
