//! Projectile event readers from FUN_1408096f8 and FUN_1410f03b4.
use super::{EventPoint, EventRecord, PrecisionDescriptor, Reader, read_event_point};

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

pub(super) fn read(
    r: &mut Reader<'_>,
    code: usize,
    encoding: Option<&ProjectilePositionEncoding>,
) -> Option<EventPoint> {
    let position;
    if code == 5 {
        r.read(6);
        variant(r);
        optional(r, 32);
        position = read_event_point(r, encoding.map(|e| &e.detonation), 21)?;
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
        position = read_event_point(r, encoding.map(|e| &e.impact), 18)?;
        r.read(19);
        r.read(9);
        r.read(1);
    }
    (!r.over).then_some(position)
}

/// Interpret only a complete, sequentially located detonation body.
#[cfg(test)]
pub(crate) fn detonation_position(
    data: &[u8],
    event: &EventRecord,
    encoding: &ProjectilePositionEncoding,
) -> Option<[u32; 3]> {
    let (region, codes) = detonation_region_position(data, event, encoding)?;
    (region == encoding.detonation.region).then_some(codes)
}

pub(crate) fn detonation_region_position(
    data: &[u8],
    event: &EventRecord,
    encoding: &ProjectilePositionEncoding,
) -> Option<(u32, [u32; 3])> {
    if event.code != Some(5) || !event.layout_complete {
        return None;
    }
    let mut reader = Reader::new(data, event.body_start_bit?)?;
    let EventPoint::Map { region, codes } = read(&mut reader, 5, Some(encoding))? else {
        return None;
    };
    (reader.cursor.position == event.end_bit).then_some((region, codes))
}

/// The first reference of event 5 is in projectile domain 5 (base 1024),
/// including its two generation bits. Only use after validating the full body.
pub(crate) fn detonation_projectile_id(data: &[u8], event: &EventRecord) -> Option<u32> {
    if event.code != Some(5) || !event.layout_complete {
        return None;
    }
    let mut reader = Reader::new(data, event.start_bit.checked_add(8)?)?;
    if reader.read(1) == 0 {
        return None;
    }
    let index = reader.read(8) as u32;
    let generation = reader.read(2) as u32;
    (!reader.over && reader.cursor.position <= event.body_start_bit?)
        .then_some((index + 1024) | (generation << 30))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::{
        kill_fields, read_contextual_event_list, read_reference_event_list,
    };
    use crate::theater::parser::v41::chunks::replication::source::EventListStop;

    fn encoding() -> ProjectilePositionEncoding {
        ProjectilePositionEncoding {
            detonation: PrecisionDescriptor {
                regions: Default::default(),
                region: 0,
                index_bits: 1,
                axis_bits: [14, 14, 16],
            },
            impact: PrecisionDescriptor {
                regions: Default::default(),
                region: 0,
                index_bits: 1,
                axis_bits: [11, 11, 13],
            },
        }
    }

    #[test]
    fn captured_streets_detonations_preserve_projectile_generation() {
        let data =
            include_bytes!("../../../../../../../fixtures/streets-projectile-detonations.bin");
        let event = EventRecord {
            start_bit: 1,
            end_bit: 129,
            code: Some(5),
            body_start_bit: Some(22),
            fields: vec![],
            layout_complete: true,
        };
        assert_eq!(detonation_projectile_id(data, &event), Some(0xc000_0429));
        let next = EventRecord {
            start_bit: 129,
            end_bit: 257,
            body_start_bit: Some(150),
            ..event.clone()
        };
        assert_eq!(detonation_projectile_id(data, &next), Some(0xc000_042a));
        assert_eq!(detonation_projectile_id(&data[..2], &event), None);
        assert_eq!(
            detonation_projectile_id(
                data,
                &EventRecord {
                    layout_complete: false,
                    ..event
                }
            ),
            None
        );
    }

    #[test]
    fn live_fire_detonation_has_recorded_position_without_a_track() {
        let data = include_bytes!("../../../../../../../fixtures/koth-live-fire-detonation.bin");
        let mut encoding = encoding();
        encoding.detonation = PrecisionDescriptor {
            regions: Default::default(),
            region: 1,
            index_bits: 2,
            axis_bits: [11, 11, 10],
        };
        let event = EventRecord {
            start_bit: 1,
            end_bit: 129,
            code: Some(5),
            body_start_bit: Some(22),
            fields: vec![],
            layout_complete: true,
        };
        assert_eq!(
            detonation_position(data, &event, &encoding),
            Some([757, 1374, 502])
        );
        assert!(detonation_position(&data[..16], &event, &encoding).is_none());
        encoding.detonation.region = 0;
        assert!(detonation_position(data, &event, &encoding).is_none());
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
        let event = &events.records[0];
        let raw = detonation_position(data, event, &encoding).unwrap();
        assert!(
            raw.into_iter()
                .zip(encoding.detonation.axis_bits)
                .all(|(q, bits)| u64::from(q) < 1 << bits)
        );
        let mut partial = event.clone();
        partial.layout_complete = false;
        assert!(detonation_position(data, &partial, &encoding).is_none());
        partial = event.clone();
        partial.end_bit += 1;
        assert!(detonation_position(data, &partial, &encoding).is_none());
        assert!(detonation_position(&data[..17], event, &encoding).is_none());
        assert!(detonation_position(data, &events.records[1], &encoding).is_none());
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
        // The default-box gate selects a legal fixed-width layout. Only the
        // unmapped indexed region is invalid with this single-region context.
        let bit = 31;
        let mut changed = data.to_vec();
        changed[bit / 8] |= 1 << (7 - bit % 8);
        let refused = parse(&changed);
        assert_eq!(refused.stop, EventListStop::UnsupportedBody);
        assert_eq!(refused.records.len(), 1);
        assert!(refused.end_bit <= 32);
        let no_map = read_reference_event_list(data, 1, Some(false), data.len());
        assert_eq!(no_map.stop, EventListStop::UnsupportedBody);
        assert_eq!(no_map.end_bit, 31);
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
