//! Additional v41 view-B component readers.
//! Raw values and gates remain fields. Layouts selected by unavailable runtime
//! state are left unsupported so the enclosing record retains the stop.
use super::reader::ComponentReader;

/// The pinned reader assumes this vehicle flag is set. It is initialized by
/// the game when constructing the vehicle, not serialized by this component.
pub(crate) fn required_runtime_field(name: &str) -> Option<&'static str> {
    matches!(
        name,
        "vehicle-type-state-component" | "vehicle-type-physics-component"
    )
    .then_some("vehicle+0x818")
}

pub(crate) fn component(r: &mut ComponentReader<'_>, name: &str) -> Option<bool> {
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
            if r.snapshot {
                return Some(false);
            }
            type_physics(r)?;
        }
        _ => return Some(false),
    }
    Some(true)
}

fn type_physics(r: &mut ComponentReader<'_>) -> Option<()> {
    if r.bit("raw")? {
        r.words("vector_bits", 9, 32)?;
    } else {
        r.gate("forward", 19, false)?;
        r.r("roll", 8)?;
        r.direction(8)?;
    }
    Some(())
}

/// Snapshot-only read under externally established vehicle construction state.
pub(crate) fn snapshot_type_physics(r: &mut ComponentReader<'_>, gate: bool) -> Option<bool> {
    if gate {
        type_physics(r)?;
    }
    Some(true)
}
