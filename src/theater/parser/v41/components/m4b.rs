//! Reference component additions from LevelUp d61443e, composants_vue_b_m4b.go.
//! Raw values and gates remain fields. Layouts selected by unavailable runtime
//! state are left unsupported so the enclosing record retains the stop.
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
            return Some(false);
        }
        _ => return Some(false),
    }
    Some(true)
}
