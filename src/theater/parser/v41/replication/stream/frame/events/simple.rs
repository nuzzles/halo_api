//! Map-independent event bodies from pinned reference reader `events::body`.
//! Each gate and scalar remains a source field, including uninterpreted codes.
use super::Reader;

fn optional(r: &mut Reader<'_>, width: usize) {
    if r.read(1) != 0 {
        r.read(width);
    }
}

fn unless(r: &mut Reader<'_>, width: usize) {
    if r.read(1) == 0 {
        r.read(width);
    }
}

pub(super) fn read(r: &mut Reader<'_>, code: usize) -> Option<bool> {
    match code {
        2 | 13 => {
            let wide = r.read(1) != 0;
            r.read(if wide {
                if code == 2 { 5 } else { 7 }
            } else {
                2
            });
            if code == 13 {
                r.read(1);
            }
        }
        8 => {
            r.read(6);
        }
        14 => {
            r.read(32);
        }
        104 => {
            if r.read(1) == 0 {
                r.read(19);
                r.read(10);
            }
        }
        105 => optional(r, 32),
        7 => {
            if r.read(1) == 0 {
                optional(r, 32);
                r.read(32);
            }
            for width in [7, 7, 19, 2, 36, 19, 9, 16, 1, 1] {
                r.read(width);
            }
        }
        9 => {
            r.read(3);
            optional(r, 32);
        }
        11 => {
            unless(r, 2);
            optional(r, 32);
            r.read(32);
            r.read(1);
            unless(r, 5);
        }
        21 | 27 => {
            r.read(2);
        }
        22 => {
            r.read(6);
            r.read(1);
            r.read(3);
        }
        38 => {
            r.read(4);
            unless(r, 5);
        }
        40 => {
            if r.read(3) == 1 {
                r.read(2);
            }
            r.read(2);
            optional(r, 32);
            r.read(32);
            unless(r, 5);
        }
        41 => {
            r.read(3);
            unless(r, 5);
        }
        46 | 47 => {
            r.read(1);
            unless(r, 2);
            optional(r, 32);
            r.read(32);
            if code == 47 {
                unless(r, 5);
            } else {
                r.read(1);
            }
        }
        58 | 118 => optional(r, 32),
        76 => {
            optional(r, 32);
            optional(r, 32);
            unless(r, 5);
            r.read(32);
        }
        100 => {
            optional(r, 32);
            r.read(32);
            optional(r, 32);
        }
        109 => {
            r.read(2);
            optional(r, 32);
            optional(r, 32);
            r.read(32);
        }
        _ => return None,
    }
    Some(!r.over)
}
