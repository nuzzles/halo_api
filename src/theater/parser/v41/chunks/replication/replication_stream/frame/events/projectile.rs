//! Projectile event readers from FUN_1408096f8 and FUN_1410f03b4.
use super::{PrecisionDescriptor, Reader, quantized_event_vector};

#[derive(Debug, Clone)]
pub(crate) struct ProjectilePositionEncoding {
    /// Map-region widths at precision level 15.
    pub detonation: PrecisionDescriptor,
    /// Map-region widths at precision level 12.
    pub impact: PrecisionDescriptor,
}

fn optional(r: &mut Reader<'_>, width: usize) {
    if r.read(1) != 0 {
        r.read(width);
    }
}

fn variant(r: &mut Reader<'_>) {
    if r.read(1) == 0 {
        optional(r, 32);
        r.read(32);
    }
}

pub(super) fn read(r: &mut Reader<'_>, code: usize, encoding: &ProjectilePositionEncoding) -> bool {
    if code == 5 {
        r.read(6);
        variant(r);
        optional(r, 32);
        if !quantized_event_vector(r, &encoding.detonation) {
            return false;
        }
        r.read(19);
        r.read(5);
        r.read(1);
        r.read(9);
        optional(r, 18);
        let direction_width = if r.read(1) != 0 { 19 } else { 8 };
        r.read(direction_width);
        r.read(2);
    } else {
        variant(r);
        r.read(14);
        r.read(19);
        if !quantized_event_vector(r, &encoding.impact) {
            return false;
        }
        r.read(19);
        r.read(9);
        r.read(1);
    }
    !r.over
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::film::EventListStop;
    use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::{
        kill_fields, read_contextual_event_list, read_reference_event_list,
    };

    fn encoding() -> ProjectilePositionEncoding {
        ProjectilePositionEncoding {
            detonation: PrecisionDescriptor {
                region: 0,
                index_bits: 1,
                axis_bits: [14, 14, 16],
            },
            impact: PrecisionDescriptor {
                region: 0,
                index_bits: 1,
                axis_bits: [11, 11, 13],
            },
        }
    }

    #[test]
    fn captured_detonation_reaches_the_following_kill_event() {
        let data = include_bytes!(
            "../../../../../../../fixtures/movement-schema-v41/argyle-detonation.bin"
        );
        let encoding = encoding();
        let parse = |data: &[u8]| {
            read_contextual_event_list(
                data,
                1,
                Some(false),
                data.len(),
                None,
                None,
                Some(&encoding),
            )
        };
        let events = parse(data);
        assert_eq!(events.stop, EventListStop::Terminator);
        assert_eq!(events.end_bit, 535);
        assert_eq!(
            events.records.iter().map(|r| r.code).collect::<Vec<_>>(),
            [Some(5), Some(1), Some(85), Some(0)]
        );
        assert_eq!(events.records[0].end_bit, 140);
        let death = &events.records[2];
        let fields = kill_fields(&mut Reader::new(data, death.body_start_bit.unwrap()).unwrap());
        assert_eq!((fields.victim, fields.killer, fields.assist), (2, 4, 1));
        assert_eq!(fields.end as usize, death.end_bit);
        for end in 0..535_usize.div_ceil(8) {
            assert_ne!(
                parse(&data[..end]).stop,
                EventListStop::Terminator,
                "prefix {end}"
            );
        }
        for bit in [30, 31] {
            let mut changed = data.to_vec();
            changed[bit / 8] |= 1 << (7 - bit % 8);
            let refused = parse(&changed);
            assert_eq!(refused.stop, EventListStop::UnsupportedBody);
            assert_eq!(refused.records.len(), 1);
            assert!(refused.end_bit <= 32);
        }
        let no_map = read_reference_event_list(data, 1, Some(false), data.len());
        assert_eq!(no_map.stop, EventListStop::UnsupportedBody);
        assert_eq!(no_map.end_bit, 22);
    }

    #[test]
    fn projectile_events_follow_gates_and_precision_specific_widths() {
        for code in [5, 6] {
            for optional in [false, true] {
                for (detonation, impact) in
                    [([14, 14, 16], [11, 11, 13]), ([12, 12, 11], [9, 9, 8])]
                {
                    let mut encoding = encoding();
                    encoding.detonation.axis_bits = detonation;
                    encoding.impact.axis_bits = impact;
                    let mut bits = Vec::new();
                    let mut put = |v: u64, width: usize| {
                        bits.extend((0..width).rev().map(|i| ((v >> i) & 1) as u8));
                    };
                    put(3, 2);
                    put(code, 7);
                    put(0, 3);
                    if code == 5 {
                        put(0, 6);
                    }
                    put(u64::from(!optional), 1);
                    if optional {
                        put(1, 1);
                        put(0x12345678, 32);
                        put(0x89abcdef, 32);
                    }
                    if code == 5 {
                        put(u64::from(optional), 1);
                        if optional {
                            put(0x12345678, 32);
                        }
                    } else {
                        put(0, 14);
                        put(0, 19);
                    }
                    put(0, 2);
                    let descriptor = if code == 5 {
                        &encoding.detonation
                    } else {
                        &encoding.impact
                    };
                    for width in descriptor.axis_bits {
                        put(1, width as usize);
                    }
                    put(0, 19);
                    if code == 5 {
                        put(0, 6);
                        put(0, 9);
                        put(u64::from(optional), 1);
                        if optional {
                            put(0, 18);
                        }
                        put(u64::from(optional), 1);
                        put(0, if optional { 19 } else { 8 });
                        put(0, 2);
                    } else {
                        put(0, 10);
                    }
                    put(1, 1);
                    put(21, 7);
                    put(0, 3);
                    put(2, 2);
                    put(0, 1);
                    let mut data = vec![0; bits.len().div_ceil(8)];
                    for (i, bit) in bits.iter().enumerate() {
                        data[i / 8] |= bit << (7 - i % 8);
                    }
                    let events = read_contextual_event_list(
                        &data,
                        1,
                        Some(false),
                        data.len(),
                        None,
                        None,
                        Some(&encoding),
                    );
                    assert_eq!(events.stop, EventListStop::Terminator);
                    assert_eq!(events.end_bit, bits.len());
                    assert_eq!(
                        events.records.iter().map(|r| r.code).collect::<Vec<_>>(),
                        [Some(code as u8), Some(21)]
                    );
                }
            }
        }
    }
}
