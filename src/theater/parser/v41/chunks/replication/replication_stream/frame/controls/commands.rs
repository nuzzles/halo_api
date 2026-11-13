//! Kind-1 command wire layout. Relative state selects a wire
//! layout only; reconstructed floats/defaults never become recorded fields.
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::chunks::replication::components::reader::ComponentReader;

pub(super) fn read(r: &mut ComponentReader<'_>, baseline: bool) -> Option<Option<Vec<BitRange>>> {
    if !r.bit("control.commands.present")? {
        return Some(None);
    }
    let count = r.r("control.commands.count", 2)?;
    let mut records = Vec::with_capacity(count as usize);
    for i in 0..count {
        let start = usize::try_from(r.cursor.position).ok()?;
        command(r, &format!("control.commands[{i}]"), baseline || i > 0)?;
        records.push(BitRange {
            start,
            end: usize::try_from(r.cursor.position).ok()?,
        });
    }
    Some(Some(records))
}

fn command(r: &mut ComponentReader<'_>, name: &str, relative: bool) -> Option<()> {
    if relative {
        if r.bit(&format!("{name}.word.relative"))? {
            r.gate(&format!("{name}.word.delta"), 6, true)?;
        } else {
            r.r(&format!("{name}.word.bits"), 32)?;
        }
        r.gate(&format!("{name}.value7"), 7, true)?;
    } else {
        r.r(&format!("{name}.word.bits"), 32)?;
        r.r(&format!("{name}.value7"), 7)?;
    }
    if r.bit(&format!("{name}.short"))? {
        return Some(());
    }
    if r.bit(&format!("{name}.object.present"))? {
        r.handle(&format!("{name}.object"), 0)?;
        r.gate(&format!("{name}.object.value"), 10, false)?;
    }
    r.inline_optional_handle(&format!("{name}.unit"), 4)?;
    for group in 0..5 {
        let vector = format!("{name}.vectors[{group}]");
        if relative {
            if !r.bit(&format!("{vector}.present"))? {
                continue;
            }
            for axis in 0..3 {
                let scalar = format!("{vector}[{axis}]");
                let lead = r.r(&format!("{scalar}.leading_bits"), 5)? as usize;
                r.r(&format!("{scalar}.delta_bits"), 32 - lead)?;
            }
        } else {
            r.words(&format!("{vector}.bits"), 3, 32)?;
        }
    }
    if r.gated_value(&format!("{name}.value8"), 8, true)?.is_some() {
        r.r(&format!("{name}.value4"), 4)?;
    }
    r.bit(&format!("{name}.flag"))?;
    if r.bit(&format!("{name}.target.enabled"))? {
        r.inline_optional_handle(&format!("{name}.target"), 2)?;
    }
    Some(())
}
