//! Projectile event readers from FUN_1408096f8 and FUN_1410f03b4.
use super::{PrecisionDescriptor, Reader, read_event_point};

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
) -> Option<()> {
    if code == 5 {
        r.read(6);
        variant(r);
        optional(r, 32);
        read_event_point(r, encoding.map(|e| &e.detonation), 21)?;
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
        read_event_point(r, encoding.map(|e| &e.impact), 18)?;
        r.read(19);
        r.read(9);
        r.read(1);
    }
    (!r.over).then_some(())
}
