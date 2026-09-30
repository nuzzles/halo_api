//! Slide and posture bodies from the reference movement-state deserializers.
use super::orientation;
use super::reader::ComponentReader;
use crate::theater::parser::v41::reference::observations::MovementComponent;

pub(crate) fn slide(r: &mut ComponentReader<'_>, level: u32) -> Option<bool> {
    let field_start = r.fields.len();
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
    let field = |name: &str| {
        r.fields[field_start..]
            .iter()
            .find(|f| f.name == name)
            .map_or(0, |f| f.raw.low_u64())
    };
    let values = [
        "active",
        "normal.constant",
        "normal.direction",
        "normal.magnitude",
        "fraction_a",
        "fraction_b",
        "tail",
    ]
    .map(field)
    .to_vec();
    r.publish_movement(MovementComponent::Slide, values);
    Some(true)
}

pub(crate) fn posture(r: &mut ComponentReader<'_>) -> Option<bool> {
    let field_start = r.fields.len();
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
                if r.position_encoding.is_none() {
                    return Some(false);
                }
                orientation::media_frame(r)?;
                r.r("direction", 19)?;
            }
        }
        2 => {
            if r.position_encoding.is_none() {
                return Some(false);
            }
            orientation::media_frame(r)?;
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
                if r.position_encoding.is_none() {
                    return Some(false);
                }
                orientation::media_frame(r)?;
                r.r("direction", 19)?;
            }
            r.r("word", 32)?;
            r.words("flags", 2, 1)?;
        }
        _ => unreachable!(),
    }
    let field = |name: &str| {
        r.fields[field_start..]
            .iter()
            .find(|f| f.name == name)
            .map_or(0, |f| f.raw.low_u64())
    };
    let tag = field("tag");
    let gate = if tag == 1 {
        u64::from(field("sub") != 0)
    } else {
        field("gate")
    };
    let word = if tag == 2 && gate != 0 {
        field("reference.value")
    } else {
        field("word")
    };
    let values = vec![
        tag,
        gate,
        field("sub"),
        field("handle"),
        field("direction"),
        word,
    ];
    r.publish_movement(MovementComponent::Posture, values);

    Some(true)
}
