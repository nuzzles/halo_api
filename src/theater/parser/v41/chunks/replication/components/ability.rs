//! Predicted ability and non-predicted grapple body grammar.
use super::orientation::media_frame;
use super::position::absolute_payload;
use super::reader::ComponentReader;
use super::widths::signed_skip;
use crate::theater::parser::v41::reference::diagnostics::WidthPurpose;
use crate::theater::parser::v41::reference::observations::{
    AbilityNonPredictedState, FilmComponentObservation, MovementComponent,
};

pub(crate) fn predicted(r: &mut ComponentReader<'_>) -> Option<bool> {
    let tag = r.r("tag", 2)?;
    r.publish_movement(MovementComponent::ActiveAbility, vec![tag]);
    let mut sub = 0;
    let mut reference = 0;
    match tag {
        1 => {
            sub = r.r("sub", 2)?;
            reference = r.r("reference", 24)?;
        }
        3 => {
            r.gate("value", 6, true)?;
            if r.bit("position.present")? {
                if r.position_encoding.is_none() {
                    return Some(false);
                }
                media_frame(r)?;
            }
        }
        _ => {}
    }
    r.publish_component(FilmComponentObservation::SpartanAbility {
        tag,
        sub,
        reference,
        has_reference: tag == 1,
    });
    Some(true)
}

pub(crate) fn non_predicted(r: &mut ComponentReader<'_>, level: u32) -> Option<bool> {
    let mut state = AbilityNonPredictedState {
        tag: r.r("tag", 2)? as u32,
        ..Default::default()
    };
    let status = (|| {
        if state.tag == 3 && r.position_encoding.is_none_or(|e| e.bodies.ability_anchor) {
            let Some(widths) = r
                .reference_widths
                .map(|w| w.movement.world_object.axis_bits)
                .or_else(|| {
                    r.position_encoding
                        .and_then(|e| e.world_axis_bits)
                        .map(|w| w.map(|v| v as u64))
                })
            else {
                return Some(false);
            };
            state.body_walked = true;
            let inner = r.r("inner", 3)? as u32;
            state.inner = Some(inner);
            state.flags = r.r("flags", 3)? as u32;
            if state.flags != 0 {
                return Some(false);
            }
            for (i, raw) in widths.into_iter().enumerate() {
                let width = r.reference_position_width(raw, "world axes")?;
                state.position[i] = r.r_wide(&format!("position[{i}]"), width)? as u32;
            }
            state.mid = r.r("mid", 7)? as u32;
            state.value8 = r.gated_value("value8", 8, true)?.map(|v| v as u32);
            match inner {
                1 => {}
                2 => {
                    for i in 0..3 {
                        if !r.bit(&format!("vector[{i}].constant"))? {
                            let direction = r.r(&format!("vector[{i}].direction"), 24)? as u32;
                            let magnitude = r.r(&format!("vector[{i}].magnitude"), 12)? as u32;
                            state.vectors[i] = Some([direction, magnitude]);
                        }
                    }
                    state.packed = r.r("packed", 24)? as u32;
                    state.tail = r.r("tail9", 9)? as u32;
                }
                _ => return Some(false),
            }
            state.body_ok = true;
        }
        if level > 1 {
            r.r("tail3", 3)?;
        }
        Some(true)
    })();
    if status.is_some() {
        r.publish_component(FilmComponentObservation::AbilityNonPredicted {
            state: Box::new(state),
        });
    }
    status
}

pub(crate) fn mobility(r: &mut ComponentReader<'_>) -> Option<bool> {
    let active = r.bit("active")?;
    let flag = r.bit("flag")?;
    r.publish_mobility([active, flag]);
    r.publish_movement(
        MovementComponent::Mobility,
        vec![u64::from(active), u64::from(flag)],
    );
    if !active {
        return Some(true);
    }
    if r.position_encoding.is_none() {
        return Some(false);
    }
    r.optional_handle("reference", 0)?;
    let bodies = &r.position_encoding?.bodies;
    if !bodies.mobility {
        return mobility_extra(r, bodies.mobility_extra_bits);
    }
    r.gate("index", 10, true)?;
    if !r.bit("keep_transform")? {
        mobility_position(r)?;
        r.gate("direction", 19, false)?;
        r.r("roll", 8)?;
    }
    r.words("vector_bits", 3, 32)?;
    mobility_position(r)?;
    r.words("matrix", 9, 12)?;
    r.words("packed", 2, 24)?;
    r.words("vector", 3, 12)?;
    r.words("fractions", 2, 10)?;
    r.bit("flag_a1")?;
    r.r("value98", 7)?;
    r.r("value9c", 2)?;
    r.bit("flag9f")?;
    Some(true)
}

fn mobility_extra(r: &mut ComponentReader<'_>, width: i64) -> Option<bool> {
    if width <= 0 {
        return Some(true);
    }
    signed_skip(
        r,
        "biped-mobility-action-component",
        width,
        false,
        Some(WidthPurpose::MobilityExtra),
        ("mobility.skipped", "mobility.skipped_tail"),
    )?;
    Some(true)
}

fn mobility_position(r: &mut ComponentReader<'_>) -> Option<()> {
    let encoding = r.position_encoding?;
    if encoding.full_precision_gate() {
        r.words("mobility.position_bits", 3, 32)?;
    } else {
        // Both sites pass level 0x10: absolute world-position widths,
        // including build-default widths when the region gate is set.
        absolute_payload(r, encoding)?;
    }
    Some(())
}
