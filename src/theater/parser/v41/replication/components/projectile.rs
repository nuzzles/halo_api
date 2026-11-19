//! Projectile component wire grammars.

use super::reader::ComponentReader;

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    _level: u32,
    _archetype: u32,
) -> Option<bool> {
    match name {
        "projectile-at-rest-state" => {
            r.bit("rest")?;
            r.gate("direction", 19, true)?;
        }
        "projectile-command_tick" => r.gate("tick", 8, true)?,
        "projectile-tether-state"
        | "projectile-deceleration-disabled-state"
        | "item-at-rest-component"
        | "tacmap-fasttravelstate" => {
            r.bit("value")?;
        }
        _ => return Some(false),
    }
    Some(true)
}
