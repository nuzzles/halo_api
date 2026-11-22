//! Slide and posture bodies from the reference movement-state deserializers.
use super::position;
use super::reader::ComponentReader;

pub(crate) fn slide(r: &mut ComponentReader<'_>, level: u32) -> Option<bool> {
    if r.bit("active")? {
        if !r.bit("normal.constant")? {
            r.r("normal.direction", 19)?;
            r.r("normal.magnitude", 10)?;
        }
        r.r("fraction_a", 8)?;
        if level >= 1 {
            r.r("fraction_b", 8)?;
        }
        r.r("tail", 8)?;
    }
    Some(true)
}

pub(crate) fn posture(r: &mut ComponentReader<'_>) -> Option<bool> {
    match r.r("tag", 2)? {
        0 => {
            if !r.bit("gate")? {
                match r.r("sub", 2)? {
                    1 => {
                        r.r("handle", 15)?;
                        r.words("flags", 2, 1)?;
                        r.r("kind", 2)?;
                    }
                    2 | 3 => {
                        r.r("handle", 15)?;
                        r.r("kind", 2)?;
                        r.bit("flag")?;
                        r.r("word", 32)?;
                        r.gate("direction", 19, true)?;
                    }
                    _ => {}
                }
            }
        }
        1 => {
            if r.r("sub", 2)? != 0 {
                r.r("handle", 15)?;
                r.words("kinds", 2, 2)?;
                if !position::contextual_point(r, "position")? {
                    return Some(false);
                }
                r.r("direction", 19)?;
            }
        }
        2 => {
            if !position::contextual_point(r, "position")? {
                return Some(false);
            }
            r.r("direction", 19)?;
            if r.bit("gate")? {
                r.optional_handle("reference", 0)?;
            } else {
                r.r("word", 32)?;
            }
            r.r("handle", 15)?;
        }
        3 => {
            r.r("handle", 15)?;
            if r.bit("gate")? {
                if !position::contextual_point(r, "position")? {
                    return Some(false);
                }
                r.r("direction", 19)?;
            }
            r.r("word", 32)?;
            r.words("flags", 2, 1)?;
        }
        _ => unreachable!(),
    }

    Some(true)
}
