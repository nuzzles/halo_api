//! Additional v41 view-B component readers.
//! Raw values and gates remain fields. Layouts selected by unavailable runtime
//! state are left unsupported so the enclosing record retains the stop.
use super::Reader;

/// The pinned reader assumes this vehicle flag is set. It is initialized by
/// the game when constructing the vehicle, not serialized by this component.
pub(super) fn required_runtime_field(name: &str) -> Option<&'static str> {
    (name == "vehicle-type-physics-component").then_some("vehicle+0x818")
}

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
