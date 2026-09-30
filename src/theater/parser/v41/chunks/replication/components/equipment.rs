//! Equipment component wire grammars.

use super::reader::ComponentReader;
use crate::theater::parser::v41::reference::observations::{
    EquipmentField, FilmComponentObservation,
};

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    _level: u32,
    _archetype: u32,
) -> Option<bool> {
    match name {
        "equipment-has-infinite-uses-component" => {
            r.bit("value")?;
        }
        "equipment-deployed-component" => {
            let value = r.r("value", 1)?;
            r.publish_component(FilmComponentObservation::EquipmentState {
                field: EquipmentField::Deployed,
                value,
                present: true,
            });
        }
        "equipment-energy-component" => {
            let value = r.r("energy", 14)?;
            r.publish_component(FilmComponentObservation::EquipmentState {
                field: EquipmentField::Energy,
                value,
                present: true,
            });
        }
        "equipment-energy-delay-ticks-left-component" => {
            let value = r.r("ticks", 10)?;
            r.publish_component(FilmComponentObservation::EquipmentState {
                field: EquipmentField::EnergyDelay,
                value,
                present: true,
            });
        }
        "equipment-charges-remaining-component" => {
            let value = r.r("charges", 8)?;
            r.publish_component(FilmComponentObservation::EquipmentState {
                field: EquipmentField::Charges,
                value,
                present: true,
            });
        }
        "equipment-creator-component" => {
            let value = r.gated_value("creator", 5, false)?;
            r.publish_component(FilmComponentObservation::EquipmentState {
                field: EquipmentField::Creator,
                value: value.unwrap_or(0),
                present: value.is_some(),
            });
        }
        "equipment-activated-component" => {
            let value = if r.bit("reference_present")? {
                r.optional_handle("reference", 4)?;
                None
            } else {
                Some(r.r("value", 3)?)
            };
            r.publish_component(FilmComponentObservation::EquipmentState {
                field: EquipmentField::Activated,
                value: value.unwrap_or(0),
                present: value.is_some(),
            });
        }
        "equipment-tracked-object-handles-stack-component" => {
            let count = r.r("count", 4)?;
            for i in 0..=count {
                r.inline_optional_handle(&format!("objects[{i}]"), 0)?;
            }
        }
        "equipment-command-tick-component" => {
            let single = r.bit("single")?;
            for i in 0..if single { 1 } else { 2 } {
                r.gate(&format!("ticks[{i}]"), 8, true)?;
            }
        }
        "equipment-being-hacked-component" => {
            r.r("value", 8)?;
        }
        "equipment-control-signal-component" => {
            r.r("signal", 4)?;
            r.optional_handle("reference", 4)?;
        }
        _ => return Some(false),
    }
    Some(true)
}
