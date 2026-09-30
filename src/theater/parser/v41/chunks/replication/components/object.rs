//! Object lifecycle, low-frequency state and weapon identity.
use super::reader::ComponentReader;
use super::{orientation, position, tlv};
use crate::theater::parser::v41::reference::observations::{
    FilmComponentObservation, MovementComponent, ObjectParentState,
};

pub(crate) fn component(r: &mut ComponentReader<'_>, name: &str, archetype: u32) -> Option<bool> {
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
        _ => {
            return super::biped::component(r, name);
        }
    }
    Some(true)
}

fn dead_state(r: &mut ComponentReader<'_>) -> Option<()> {
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

fn weapon(r: &mut ComponentReader<'_>) -> Option<()> {
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
    r.publish_component(FilmComponentObservation::HeldWeapon { id_high, id_low });
    Some(())
}

pub(crate) fn magazines(r: &mut ComponentReader<'_>) -> Option<()> {
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

fn low_frequency(r: &mut ComponentReader<'_>) -> Option<()> {
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

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    level: u32,
    archetype: u32,
) -> Option<bool> {
    match name {
        "object-position-component" => {
            if r.bit("high_precision")? {
                r.r("high_precision_body", 59)?;
            } else {
                let Some(encoding) = r.position_encoding else {
                    return Some(false);
                };
                let widths = if let Some(raw) = r.reference_widths {
                    raw.movement.world_object.axis_bits
                } else if let Some(widths) = encoding.world_axis_bits {
                    widths.map(|w| w as u64)
                } else {
                    return Some(false);
                };
                if !r.bit("region.gate")? {
                    let raw = r.reference_widths.map_or(encoding.index_bits as u64, |w| {
                        w.movement.world_object.index_bits
                    });
                    let width = r.reference_position_width(raw, "world index")?;
                    r.r_wide("region", width)?;
                }
                for (i, raw) in widths.into_iter().enumerate() {
                    let width = r.reference_position_width(raw, "world axes")?;
                    r.r_wide(&format!("position[{i}]"), width)?;
                }
                r.r("finite", 2)?;
            }
        }
        "object-forward-and-up-dynamic-precision-component" => orientation::dynamic(r, level)?,
        "object-angular-velocity-dynamic-precision-component" => {
            if r.bit("full_precision")? {
                r.words("vector_bits", 3, 32)?;
            } else {
                r.direction(8)?;
            }
        }
        "object-position-dynamic-precision-component" => return position::component(r),
        "object-translational-velocity-dynamic-precision-component" => {
            let full = r.bit("full_precision")?;
            let mut values = vec![u64::from(full), 0, 0, 0];
            if full {
                r.words("vector_bits", 3, 32)?;
            } else {
                let absent = r.bit("stationary")?;
                values[1] = u64::from(absent);
                if !absent {
                    values[2] = r.r("direction", 19)?;
                    values[3] = r.r("magnitude", 10)?;
                }
            }
            r.publish_movement(MovementComponent::Velocity, values);
        }
        "object-translational-velocity-component" => r.direction(10)?,
        "object-angular-velocity-component" => r.direction(8)?,
        "object-region-state-component" => {
            let present = r.bit("has_values")?;
            let count = r.r("count", 6)? as usize;
            r.words("states", count, 3)?;
            if present {
                r.words("values", count, 10)?;
            }
        }
        "object-damage-sections-component" => {
            let count = r.r("count", 6)?;
            for i in 0..count {
                if r.bit(&format!("section[{i}].present"))? {
                    r.r(&format!("section[{i}].amount"), 7)?;
                    r.r(&format!("section[{i}].state"), 16)?;
                }
            }
        }
        "object-constraint-component" => {
            let count = r.r("count", 5)? as usize;
            if count != 0 {
                r.r("flags_a", count)?;
                r.r("flags_b", count)?;
            }
        }
        "object-parent-state-component" => parent(r, level, archetype)?,
        "object-scale-component" => {
            if !r.bit("default")? {
                r.r("scale", 15)?;
                if r.bit("transition")? {
                    r.r("target_scale", 15)?;
                    r.r("duration", 12)?;
                    r.r("flags", 5)?;
                }
            }
        }
        "object-maximum-vitalities-component" => {
            let flags = r.r("flags", 5)?;
            for (mask, name) in [(4, "body"), (8, "shield"), (16, "extra")] {
                if flags & mask != 0 {
                    for i in 0..3 {
                        r.gate(&format!("{name}[{i}]"), 12, true)?;
                    }
                    if mask != 16 {
                        r.bit(&format!("{name}.flag"))?;
                    }
                }
            }
            r.words("tail_flags", 3, 1)?;
        }
        "object-dissolver-component" => {
            if r.r("state", 4)? != 13 {
                r.words("body_words", 3, 32)?;
                r.r("duration", 12)?;
                r.bit("flag")?;
            }
        }
        "object-physics-flags-component" => r.words("flags", 5, 1)?,
        "object-frame-configuration-component" => r.frame_configuration()?,
        "object-multiplayer-properties-component" => tlv::multiplayer_properties(r)?,
        _ => return Some(false),
    }
    Some(true)
}

fn parent(r: &mut ComponentReader<'_>, level: u32, archetype: u32) -> Option<()> {
    let mut state = ObjectParentState {
        archetype,
        parameter: level,
        start_bit: r.cursor.position,
        attached: r.bit("attached")?,
        ..Default::default()
    };
    if state.attached {
        r.handle("parent", 1)?;
        state.quantized_word = (r.fields[r.fields.len() - 1].raw.low_u64() as u32) << 30
            | r.fields[r.fields.len() - 2].raw.low_u64() as u32;
        state.word = r.r("word", 16)? as u32;
        state.optional_word = r.gated_value("optional_word", 16, true)?.map(|v| v as u32);
        for i in 0..2 {
            state.flags[i] = r.bit(&format!("flags[{i}]"))?;
        }
        for i in 0..3 {
            state.matrix[i] = r.r(&format!("matrix[{i}]"), 16)? as u32;
        }
        state.velocity = r.gated_value("velocity", 19, false)?.map(|v| v as u32);
        state.byte = r.r("magnitude", 8)? as u32;
        state.flag_c = r.bit("flag_c")?;
    } else if level < 2 {
        state.free_read = true;
        let at = r.cursor.position;
        r.optional_handle("free_reference", 0)?;
        state.free_bits = usize::try_from(r.cursor.position.wrapping_sub(at)).ok()?;
        let reference = r.references.last()?;
        state.free_id = reference.present.then_some(u64::from(reference.value));
        state.alternate = r.gated_value("alternate", 11, true)?.map(|v| v as u32);
    }
    state.tail_sign = r.bit("tail_sign")?;
    if state.tail_sign {
        state.tail6 = Some(r.r("tail_value", 6)? as u32);
    }
    state.tail_bit = r.bit("tail_flag")?;
    if level <= 2 || archetype == 35 {
        state.tail3 = Some(r.r("tail_enum", 3)? as u32);
    }
    state.end_bit = r.cursor.position;
    r.publish_component(FilmComponentObservation::ObjectParent {
        state: Box::new(state),
    });
    Some(())
}
