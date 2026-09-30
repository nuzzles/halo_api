//! Shared v41 actor/input component grammar.
//! Reference action fields and source ranges follow d61443e bloc_action.go.
use super::position::absolute_payload;
use super::reader::ComponentReader;

/// Internal outcome distinct from a failed primitive read (`None`).
pub(crate) enum ActionBlockRead {
    Decoded,
    MissingPositionContext,
}

pub(crate) fn actor_control(r: &mut ComponentReader<'_>, level: u32) -> Option<bool> {
    if r.r("selector", 3)? == 1 {
        return Some(true);
    }
    r.optional_word_reference("reference", false)?;
    r.bit("mode")?;
    r.r("state", 6)?;
    r.direction(10)?;
    r.r("packed_direction", 19)?;
    r.bit("flag")?;
    r.gate("fraction", 9, false)?;
    if matches!(actions(r)?, ActionBlockRead::MissingPositionContext) {
        return Some(false);
    }
    r.optional_handle("slot[0]", 1)?;
    if level > 1 {
        r.optional_handle("slot[1]", 1)?;
    }
    Some(true)
}

pub(crate) fn actions(r: &mut ComponentReader<'_>) -> Option<ActionBlockRead> {
    if r.bit("actions.present")? && !action_body(r)? {
        return Some(ActionBlockRead::MissingPositionContext);
    }
    Some(ActionBlockRead::Decoded)
}

fn action_body(r: &mut ComponentReader<'_>) -> Option<bool> {
    let mut a = [0; 2];
    let mut b = [0; 2];
    if r.bit("actions.a.present")? {
        for (i, value) in a.iter_mut().enumerate() {
            *value = r.r(&format!("actions.a[{i}]"), 3)?;
        }
    }
    if r.bit("actions.b.present")? {
        for (i, value) in b.iter_mut().enumerate() {
            *value = r.r(&format!("actions.b[{i}]"), 2)?;
        }
    }
    if r.bit("actions.aim.present")? {
        r.r("actions.aim.flags", 2)?;
        r.gate("actions.aim.value8", 8, true)?;
        r.gate("actions.aim.value10", 10, true)?;
        match r.r("actions.aim.vector_mode", 2)? {
            0 => {
                let Some(encoding) = r.position_encoding else {
                    // The wire prefix is complete. Missing caller context is not
                    // evidence of truncated source; stop before guessing widths.
                    return Some(false);
                };
                if encoding.full_precision_gate() {
                    r.words("actions.aim.position_bits", 3, 32)?;
                } else {
                    absolute_payload(r, encoding)?;
                }
            }
            1 => {
                r.r("actions.aim.direction", 19)?;
            }
            _ => {}
        }
    }
    r.r("actions.state", 3)?;
    for i in 0..2 {
        if a[i] != 0 || b[i] != 0 {
            r.gated_value(&format!("actions.slot[{i}]"), 2, false)?;
        }
    }
    if !r.bit("actions.tail.present")? {
        return Some(true);
    }
    match r.r("actions.tail.mode", 2)? {
        1 => {
            r.handle("actions.tail.reference", 1)?;
            r.gate("actions.tail.index", 6, true)?;
        }
        2 => r.handle("actions.tail.reference", 2)?,
        _ => {}
    }
    if !r.bit("actions.tail.direct")? {
        r.words("actions.tail.fractions", 2, 4)?;
        if !r.bit("actions.tail.direction_present")? {
            return Some(true);
        }
    }
    if !r.bit("actions.tail.constant_direction")? {
        r.r("actions.tail.direction", 15)?;
        r.r("actions.tail.magnitude", 7)?;
    }
    Some(true)
}

pub(crate) fn actor_state(r: &mut ComponentReader<'_>, level: u32) -> Option<()> {
    r.words("state", 2, 32)?;
    r.r(
        "fraction",
        match level {
            1 => 8,
            2 => 10,
            3 => 11,
            _ => 12,
        },
    )?;
    r.r("kind", 4)?;
    for i in 0..5 {
        let prefix = format!("aim[{i}]");
        if !r.bit(&format!("{prefix}.present"))? {
            continue;
        }
        let a = r.bit(&format!("{prefix}.a"))?;
        let b = r.bit(&format!("{prefix}.b"))?;
        r.word_reference(&format!("{prefix}.reference"))?;
        if !a && b {
            r.bit(&format!("{prefix}.c"))?;
            r.r(&format!("{prefix}.kind"), 2)?;
            r.words(&format!("{prefix}.fractions"), 2, 10)?;
            r.r(&format!("{prefix}.quaternion"), 16)?;
            r.optional_word_reference(&format!("{prefix}.optional_reference"), true)?;
            r.word_reference(&format!("{prefix}.second_reference"))?;
            r.optional_handle(&format!("{prefix}.slot"), 0)?;
        } else if !a {
            r.r(&format!("{prefix}.quaternion"), 16)?;
        } else {
            r.optional_handle(&format!("{prefix}.slot"), 0)?;
        }
        if !b {
            r.r(&format!("{prefix}.tail"), 8)?;
        }
        r.optional_word_reference(&format!("{prefix}.tail_reference"), true)?;
    }
    Some(())
}
