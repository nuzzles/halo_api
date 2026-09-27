//! Object lifecycle, low-frequency state and weapon identity.
use super::Reader;

pub(super) fn component(r: &mut Reader<'_>, name: &str, archetype: u32) -> Option<bool> {
    match name {
        "object-forward-and-up-component" => {
            r.gate("direction", 19, false)?;
            r.r("roll", 8)?;
        }
        "object-dead-state-component" => {
            r.bit("dead")?;
            if archetype == 35 || archetype == 40 {
                dead_state(r)?;
                if archetype == 35 {
                    r.bit("biped_flag")?;
                }
            }
        }
        "object-low-frequency-component" => low_frequency(r)?,
        "weapon-state-type-info" => weapon(r)?,
        _ => return super::biped::component(r, name),
    }
    Some(true)
}

fn dead_state(r: &mut Reader<'_>) -> Option<()> {
    r.gate("source_tag", 32, true)?;
    // FUN_140c1e3f0 writes comp+0x1c. Its 0x10 bit gates the velocity
    // block below; this is recorded state, not an external runtime setting.
    let flags_1c = r.r("byte", 8)?;
    r.gate("enum_a", 5, false)?;
    r.gate("enum_b", 5, false)?;
    r.r("value_0c", 4)?;
    r.r("value_0e", 3)?;
    if r.bit("has_reference")? {
        r.gate("global_id", 32, true)?;
        r.r("value_14", 3)?;
        r.gate("value_18", 6, false)?;
    }
    r.r("position_flags", 4)?;
    r.r("step", 4)?;
    r.gate("value_1e", 10, false)?;
    r.gate("position", 10, true)?;
    if flags_1c & 0x10 != 0 {
        r.r("velocity_table_index", 2)?;
        r.words("velocity_axis", 3, 14)?;
    }
    // FUN_1424cd17c is called once, after the conditional velocity block.
    r.r("value_40", 5)?;
    r.r("value_44", 5)?;
    r.gate("source_tag_4c", 32, true)?;
    Some(())
}

fn weapon(r: &mut Reader<'_>) -> Option<()> {
    let mut id_high = u32::MAX;
    let mut id_low = u32::MAX;
    if r.bit("present")? {
        id_high = r.r("id_high", 32)? as u32;
        id_low = r.r("variant", 32)? as u32;
        r.r("value_7c", 12)?;
        r.r("value_minus_one", 7)?;
        if r.bit("shot_id_present")? {
            r.r("shot_id_a", 4)?;
            r.r("shot_id_b", 6)?;
        }
        r.bit("flag_82")?;
        magazines(r)?;
        r.frame_configuration()?;
        r.gate("optional5", 5, false)?;
    }
    r.gate("tail_byte", 8, true)?;
    r.r("tail_state", 3)?;
    r.gate("tail_a", 2, false)?;
    r.gate("tail_b", 2, false)?;
    r.publish_component(crate::theater::FilmComponentObservation::HeldWeapon { id_high, id_low });
    Some(())
}

pub(super) fn magazines(r: &mut Reader<'_>) -> Option<()> {
    if !r.bit("magazine_list")? {
        r.gate("magazine_reference", 32, true)?;
    } else {
        let count = r.r("magazine_count", 4)?;
        for i in 0..count {
            r.gate(&format!("magazines[{i}]"), 32, true)?;
        }
    }
    Some(())
}

fn low_frequency(r: &mut Reader<'_>) -> Option<()> {
    if r.r("head", 2)? < 2 {
        r.r("head_a", 7)?;
        r.r("head_b", 8)?;
    }
    r.r("value4", 4)?;
    r.r("value6", 6)?;
    let count = r.r("keyframe_count", 6)?;
    for i in 0..count {
        if r.bit(&format!("keyframes[{i}].short"))? {
            r.bit(&format!("keyframes[{i}].flag"))?;
        } else {
            r.words(&format!("keyframes[{i}].values"), 2, 12)?;
        }
    }
    r.words("flags", 7, 1)?;
    r.gate("optional4", 4, false)?;
    if r.bit("extra")? {
        r.bit("extra_a")?;
        r.gate("extra_value", 12, true)?;
        r.bit("extra_b")?;
    }
    let mask = r.r("mask", 3)?;
    if mask & 3 == 3 {
        r.r("mask_value2", 2)?;
        r.r("mask_value5", 5)?;
        if mask & 4 != 0 {
            r.words("mask_values", 3, 8)?;
        }
    }
    if r.bit("tail")? {
        r.r("tail_type", 3)?;
        r.r("tail_word", 32)?;
        r.gate("tail_optional_word", 32, true)?;
        r.r("tail_value", 14)?;
    }
    Some(())
}
