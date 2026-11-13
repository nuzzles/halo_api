//! Secondary control unit layout. Retain raw fields without materializing unit state.
use crate::theater::parser::v41::chunks::replication::components::{
    position, reader::ComponentReader,
};

/// None is a truncated primitive; false requires unavailable position context.
pub(super) fn read(r: &mut ComponentReader<'_>) -> Option<bool> {
    let state = r.r("control.secondary.state", 5)?;
    r.r("control.secondary.kind", 2)?;
    r.r("control.secondary.value17", 17)?;
    r.r("control.secondary.value16", 16)?;
    r.inline_optional_handle("control.secondary.unit", 1)?;
    if state & 2 == 0 {
        return Some(true);
    }
    r.inline_optional_handle("control.secondary.reference", 1)?;
    r.r("control.secondary.kind2", 2)?;
    r.r("control.secondary.kind3", 3)?;
    let count = r.r("control.secondary.objects.count", 3)?;
    for i in 0..count {
        let name = format!("control.secondary.objects[{i}]");
        r.inline_optional_handle(&name, 0)?;
        r.r(&format!("{name}.value_a"), 4)?;
        r.r(&format!("{name}.value_b"), 4)?;
    }
    r.r("control.secondary.value5", 5)?;
    r.r("control.secondary.value3", 3)?;
    r.gate("control.secondary.slot_a", 2, false)?;
    r.gate("control.secondary.slot_b", 2, false)?;
    r.r("control.secondary.value6", 6)?;
    r.words("control.secondary.values3", 2, 3)?;
    r.gate("control.secondary.index", 6, false)?;
    if r.bit("control.secondary.local_position")? {
        r.words("control.secondary.local_axes", 3, 14)?;
    } else if !position::point(r, "control.secondary")? {
        return Some(false);
    }
    r.gate("control.secondary.forward", 19, false)?;
    r.r("control.secondary.roll", 8)?;
    for (name, width) in [("velocity", 10), ("angular_velocity", 8)] {
        if !r.bit(&format!("control.secondary.{name}.zero"))? {
            r.r(&format!("control.secondary.{name}.direction"), 19)?;
            r.r(&format!("control.secondary.{name}.magnitude"), width)?;
        }
    }
    if r.bit("control.secondary.target.present")? {
        r.handle("control.secondary.target", 0)?;
        r.r("control.secondary.target.signed_word", 11)?;
    }
    r.r("control.secondary.tail", 7)?;
    Some(true)
}
