//! Weapon component wire grammars.

use super::reader::ComponentReader;
use crate::theater::parser::v41::reference::observations::FilmComponentObservation;

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    _level: u32,
    _archetype: u32,
) -> Option<bool> {
    match name {
        "weapon-ammo-component" => {
            let a = r.r("a", 8)? as u32;
            let b = r.r("b", 11)? as u32;
            let c = r.r("c", 12)? as u32;
            r.publish_component(FilmComponentObservation::GroundWeaponAmmo { a, b, c });
        }
        "weapon-state-ammo" => {
            let magazine = r.gated_value("magazine", 8, false)?.map(|v| v as u32);
            let fraction = r.gated_value("fraction", 12, false)?.map(|v| v as u32);
            r.publish_component(FilmComponentObservation::WeaponAmmo { magazine, fraction });
        }
        "weapon-state-rounds-inventory" => {
            let rounds = r.r("rounds", 11)? as u32;
            r.publish_component(FilmComponentObservation::WeaponRounds { rounds });
        }
        "weapon-state-overheated" => {
            r.r("heat", 7)?;
            r.words("flags", 2, 1)?;
        }
        _ => return Some(false),
    }
    Some(true)
}
