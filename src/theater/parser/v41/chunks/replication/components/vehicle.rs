//! Vehicle/turret wire fields. No reconstructed state.
use super::reader::ComponentReader;

pub(crate) fn component(r: &mut ComponentReader<'_>, name: &str) -> Option<bool> {
    match name {
        "vehicle-type-state-component" => {
            // An explicitly included delta carries this block. A snapshot
            // does not record the vehicle's construction-time enable flag.
            if r.snapshot {
                return Some(false);
            }
            type_state(r)?;
        }
        "vehicle-weapon-set-component" => {
            r.r("set", 3)?;
            r.gate("slot[0]", 2, false)?;
            r.gate("slot[1]", 2, false)?;
        }
        "vehicle-auto-turret-triggers-component" => {
            r.r("triggers", 3)?;
        }
        "vehicle-auto-turret-aiming-vector-component" => {
            r.r("direction", 19)?;
        }
        "vehicle-transformed-or-desired-open-state-changed-component" => {
            r.bit("state")?;
            r.r("value", 8)?;
        }
        "vehicle-auto-turret-target-component" => r.inline_optional_handle("target", 1)?,
        "vehicle-sentry-state-component" => {
            r.r("state", 3)?;
            r.bit("flag")?;
        }
        "vehicle-auto-turret-component" => {
            r.r("state", 2)?;
        }
        "vehicle-equipment-turret-parent-component" => r.inline_optional_handle("parent", 0)?,
        "vehicle-seats-override-pitch-component" | "vehicle-seats-override-yaw-component" => {
            r.words("values", 2, 8)?;
        }
        "air-drop-flight-component" => {
            r.r("kind", 2)?;
            r.r("value14", 14)?;
            r.r("value8", 8)?;
        }
        "warp-component" => {
            let a = r.bit("a.present")?;
            let b = r.bit("b.present")?;
            if a {
                r.r("a.value", 8)?;
            }
            if b {
                r.r("b.value", 8)?;
            }
        }
        "vehicle-low-frequency-component" => r.gate("value", 5, false)?,
        _ => return Some(false),
    }
    Some(true)
}

fn type_state(r: &mut ComponentReader<'_>) -> Option<()> {
    if matches!(r.r("state", 2)?, 1 | 3) {
        r.r("value", 6)?;
    }
    Some(())
}

/// Snapshot-only read under externally established vehicle construction state.
/// The false branch consumes zero body bits, as in the native serializer.
pub(crate) fn snapshot_type_state(r: &mut ComponentReader<'_>, gate: bool) -> Option<bool> {
    if gate {
        type_state(r)?;
    }
    Some(true)
}
