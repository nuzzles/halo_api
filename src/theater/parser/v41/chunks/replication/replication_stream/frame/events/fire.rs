//! Contextual action_weapon_fire v3 (FUN_14080c1f8, film param_5=1).
//! No event-local corruption trailer is assumed. Unknown map regions stop the read.
use super::{PrecisionDescriptor, Reader, event_point};

fn optional(r: &mut Reader<'_>, width: usize) {
    if r.read(1) != 0 {
        r.read(width);
    }
}

// Unguarded FUN_1406d3140 reference. The caller supplies presence, if any.
fn reference(r: &mut Reader<'_>, category: u8) {
    let width = if category == 1 {
        if r.read(1) != 0 { 9 } else { 13 }
    } else {
        8 // category 2
    };
    r.read(width);
    r.read(2);
}

fn target(r: &mut Reader<'_>) {
    match r.read(2) {
        1 => {
            reference(r, 1);
            optional(r, 6);
        }
        2 => reference(r, 2),
        _ => {}
    }
}

// FUN_140c9e4d8(..., 0), not the older research's FUN_1406cd5b8.
fn action_target(r: &mut Reader<'_>) -> bool {
    let present = r.read(1) != 0;
    if present {
        target(r);
        let mut direction = true;
        if r.read(1) == 0 {
            r.read(4);
            r.read(4);
            direction = r.read(1) != 0;
        }
        if direction && r.read(1) == 0 {
            r.read(15);
            r.read(7);
        }
    }
    present
}

fn counts(r: &mut Reader<'_>) -> (usize, usize) {
    if r.read(1) != 0 {
        return (0, 0);
    }
    let components = if r.read(1) != 0 {
        1
    } else {
        r.read(4) as usize
    };
    let targets = if r.read(1) != 0 {
        0
    } else if r.read(1) != 0 {
        1
    } else {
        r.read(4) as usize
    };
    (components, targets)
}

pub(super) fn read(r: &mut Reader<'_>, encoding: Option<&PrecisionDescriptor>) -> bool {
    let short = r.read(1) != 0;
    let block = r.read(1) != 0;
    r.read(7);
    r.read(1);
    r.entity5();
    if r.read(1) == 0 {
        r.read(2);
    }
    optional(r, 32);
    r.read(32);
    r.read(2);
    let timestamp = if block {
        r.read(1);
        r.read(1) != 0
    } else {
        false
    };
    if timestamp {
        optional(r, 10);
    }
    if short {
        r.read(10);
        return !r.over;
    }
    let (components, targets) = counts(r);
    let mut type_one = [false; 16];
    for kind in type_one.iter_mut().take(targets) {
        *kind = r.read(2) == 1;
        r.read(1);
        reference(r, 1);
    }
    let base = if components == 1 { 12 } else { 4 };
    let mut follow = true;
    for _ in 0..components {
        r.read(4);
        if r.read(1) == 0 {
            continue;
        }
        // The LAST present component decides whether composites/aim follow.
        follow = r.read(3) != 0;
        let index = r.read(if targets < 3 { 1 } else { 4 }) as usize;
        r.read(16);
        let width = if type_one[index] { base.min(6) } else { base };
        for _ in 0..3 {
            r.read(width);
        }
    }
    if follow {
        let aimed_at_target = action_target(r);
        if r.read(1) != 0 {
            target(r); // FUN_1408eff64(..., 1)
        }
        if !aimed_at_target {
            r.read(30);
        }
    }
    if !timestamp {
        optional(r, 6);
        r.read(6);
    }
    if block {
        match r.read(2) {
            0 => {
                if !event_point(r, encoding) {
                    return false;
                }
            }
            1 => {
                r.read(19);
            }
            _ => {}
        }
        r.read(4); // event version 3 > 1
    } else {
        r.read(2);
        let extra = r.read(1) != 0;
        optional(r, 4);
        if extra {
            optional(r, 12);
            optional(r, 12);
        }
    }
    r.read(6);
    optional(r, 7);
    if r.read(1) != 0 && !event_point(r, encoding) {
        return false;
    }
    !r.over
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_timestamp_and_long_optional_tails_have_exact_extents() {
        let encoding = PrecisionDescriptor {
            regions: Default::default(),
            index_bits: 1,
            axis_bits: [18, 18, 15],
            region: 0,
        };
        // Literal wire vectors for FUN_14080c1f8's source-listed field widths.
        for short in [false, true] {
            for block in [false, true] {
                let mut fields = vec![
                    (u64::from(short), 1),
                    (u64::from(block), 1),
                    (0x55, 7),
                    (1, 1),
                    (1, 1), // absent shooter
                    (1, 1),
                    (0, 1),
                    (0x12345678, 32),
                    (0, 2),
                ];
                if block {
                    fields.extend([(0, 1), (1, 1), (1, 1), (0x155, 10)]);
                }
                if short {
                    fields.push((0x2aa, 10));
                } else {
                    fields.extend([(1, 1), (0, 1), (0, 1), (0x1234567, 30)]);
                    if !block {
                        fields.extend([
                            (1, 1),
                            (3, 6),
                            (7, 6), // scales
                            (2, 2),
                            (1, 1),
                            (1, 1),
                            (9, 4),
                            (1, 1),
                            (123, 12),
                            (1, 1),
                            (234, 12),
                        ]);
                    } else {
                        fields.extend([(1, 2), (1234, 19), (3, 4)]);
                    }
                    fields.extend([
                        (17, 6),
                        (1, 1),
                        (55, 7),
                        (1, 1),
                        (0, 1),
                        (0, 1),
                        (12, 18),
                        (34, 18),
                        (56, 15),
                    ]);
                }
                let expected: usize = fields.iter().map(|(_, n)| n).sum();
                let mut bits = Vec::new();
                for (value, width) in fields {
                    bits.extend((0..width).rev().map(|i| ((value >> i) & 1) as u8));
                }
                bits.extend([1; 16]); // following data must remain untouched
                let mut data = vec![0; bits.len().div_ceil(8)];
                for (i, b) in bits.iter().enumerate() {
                    data[i / 8] |= b << (7 - i % 8);
                }
                let mut r = Reader::new(&data, 0).unwrap();
                assert!(read(&mut r, Some(&encoding)));
                assert_eq!(r.cursor.position, expected, "short={short} block={block}");
                for end in 0..expected.div_ceil(8) {
                    let mut r = Reader::new(&data[..end], 0).unwrap();
                    assert!(!read(&mut r, Some(&encoding)), "truncated {end}");
                }
            }
        }
    }
}
