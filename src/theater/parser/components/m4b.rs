//! Native component additions from LevelUp d61443e, composants_vue_b_m4b.go.
//! Raw values and gates remain fields; the vehicle runtime assumption is a
//! diagnostic, never a synthetic recorded bit.
use super::Reader;

pub(super) fn component(r: &mut Reader<'_>, name: &str) -> Option<bool> {
    match name {
        "personal-ai-data-component" => {
            if r.r("handle", 32)? != u64::from(u32::MAX) {
                r.gate("value", 12, true)?;
            }
        }
        "player-aim-assist-component" => {
            r.optional_handle("reference", 1)?;
            r.bit("flag")?;
        }
        "player-desired-frame-configuration-component" => r.frame_configuration()?,
        "managed-object-looping-sound-component" => {
            r.r("sound", 32)?;
        }
        "vehicle-emp-timer-component" => {
            r.r("timer", 8)?;
        }
        "vehicle-type-physics-component" => {
            r.diagnostics.vehicle_type_physics_assumed += 1;
            if r.bit("raw_vectors")? {
                r.words("orientation_bits", 6, 32)?;
                r.words("angular_velocity_bits", 3, 32)?;
            } else {
                r.gate("orientation_direction", 19, false)?;
                r.r("orientation_roll", 8)?;
                r.direction(8)?;
            }
        }
        _ => return Some(false),
    }
    Some(true)
}
