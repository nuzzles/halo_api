//! Physics component wire grammars.
use super::position;

use super::reader::ComponentReader;

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    _level: u32,
    _archetype: u32,
) -> Option<bool> {
    match name {
        "generic-rigid-body-transforms" | "generic-rigid-body-transforms-component" => {
            let mask = r.r("mask", 8)?;
            for i in 0..8 {
                if mask & (1 << i) != 0 {
                    r.gate(&format!("body[{i}].direction"), 19, false)?;
                    r.r(&format!("body[{i}].magnitude"), 8)?;
                    // FUN_142f036f0 calls the precision-16 point reader:
                    // raw snapshot words, or a delta's default/indexed box.
                    if !position::point(r, &format!("body[{i}].position"))? {
                        return Some(false);
                    }
                }
            }
        }
        "physics-state-component" => {
            r.r("state", 32)?;
            r.gate("extra", 32, true)?;
        }
        _ => return Some(false),
    }
    Some(true)
}
