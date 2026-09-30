//! Unit component wire grammars.
use super::control;
use crate::theater::parser::v41::reference::observations::{
    CamoState, FilmComponentObservation, MovementComponent, UnitEquipmentEntry, UnitEquipmentRead,
};

use super::reader::ComponentReader;

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    level: u32,
    _archetype: u32,
) -> Option<bool> {
    match name {
        "unit-control-component" => {
            let present = r.bit("control.present")?;
            let mut values = vec![u64::from(present), 0, 0, 0];
            if present {
                values[1] = r.r("control.index", 5)?;
                let second = r.gated_value("control.second_index", 6, true)?;
                values[2] = u64::from(second.is_some());
                values[3] = second.unwrap_or(0);
            }
            r.publish_movement(MovementComponent::UnitControl, values);
            r.optional_word_reference("control.reference", true)?;
        }
        "unit-actor-control-component" => return control::actor_control(r, level),
        "unit-actor-state-component" => control::actor_state(r, level)?,
        "unit-grenade-counts-component" => {
            let count = r.r("count", 3)?;
            let mut values = Vec::new();
            for i in 0..count {
                values.push(r.r(&format!("grenades[{i}]"), 8)?);
            }
            r.publish_component(FilmComponentObservation::GrenadeCounts { count, values });
        }
        "unit-equipment-component" => {
            let head = r.r("head", 3)? as u32;
            let count = r.r("count", 3)?;
            let mut entries = Vec::with_capacity(count as usize);
            for i in 0..count {
                r.optional_handle(&format!("equipment[{i}]"), 0)?;
                let reference = r.references.last()?;
                entries.push(UnitEquipmentEntry {
                    value: reference.value,
                    tail: reference.tail,
                    present: reference.present,
                });
            }
            r.publish_component(FilmComponentObservation::UnitEquipment {
                state: Box::new(UnitEquipmentRead { head, entries }),
            });
        }
        "unit-low-frequency-component" => {
            r.bit("head")?;
            r.optional_handle("reference", 0)?;
            r.r("state", 3)?;
            r.bit("flag")?;
            let count = r.r("count", 4)?;
            for i in 0..count {
                r.optional_handle(&format!("references[{i}]"), 0)?;
            }
            r.r("tail", 2)?;
        }
        "unit-command-tick-component" => {
            r.gate("tick", 8, true)?;
            if r.bit("extra")? {
                let single = r.bit("single")?;
                r.gate("tick_a", 8, true)?;
                if !single {
                    r.gate("tick_b", 8, true)?;
                }
                if !r.bit("short_tail")? {
                    r.words("tail", 2, 1)?;
                }
            }
        }
        "unit-stun-component" => {
            r.r("state", 16)?;
            r.r("amount", 12)?;
            r.r("duration", 12)?;
        }
        "unit-crouch-component" => {
            let flag = r.bit("flag")?;
            let progress = r.r("progress", 10)?;
            r.publish_movement(MovementComponent::Crouch, vec![u64::from(flag), progress]);
        }
        "unit-desired-aiming-vector-component" => {
            let single = r.bit("single")?;
            r.r("yaw", 12)?;
            r.r("pitch", 11)?;
            r.bit("flag")?;
            if !single {
                r.r("second_yaw", 12)?;
                r.r("second_pitch", 11)?;
                r.bit("second_flag")?;
            }
        }
        "unit-active-camo-state-component" => {
            let mut state = CamoState {
                state: r.r("state", 3)? as u8,
                flag0: r.bit("flag0")?,
                ..Default::default()
            };
            if !state.flag0 {
                let flag1 = r.bit("flag1")?;
                state.flag1 = Some(flag1);
                if !flag1 {
                    state.fraction = Some(r.r("fraction", 12)? as u16);
                }
            }
            for i in 0..6 {
                state.sub[i] = r
                    .gated_value(&format!("sub[{i}]"), 12, true)?
                    .map(|v| v as u16);
            }
            r.publish_component(FilmComponentObservation::CamoState {
                state: Box::new(state),
            });
        }
        _ => return Some(false),
    }
    Some(true)
}
