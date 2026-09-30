//! Flock component wire grammars.

use super::reader::ComponentReader;

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    level: u32,
    _archetype: u32,
) -> Option<bool> {
    match name {
        "flock-relevancy-component" => {
            r.r("relevancy", 8)?;
        }
        "flock-fleeing-component" => {
            r.bit("fleeing")?;
        }
        "flock-remembered-danger-component" => {
            let kind = r.r("kind", 3)?;
            r.r("value", 8)?;
            if kind != 0 {
                r.r("direction", 19)?;
            }
        }
        "flock-destination-component" => {
            r.bit("flag")?;
            if !r.bit("default_vector")? {
                r.gate("region", 1, false)?;
                r.words("position", 3, (u64::from(level) + 6).min(26) as usize)?;
            }
            if level > 1 {
                r.r("tail", 2)?;
            }
        }
        _ => return Some(false),
    }
    Some(true)
}
