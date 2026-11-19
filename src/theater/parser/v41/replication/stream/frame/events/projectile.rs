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
