//! Predicted ability and non-predicted grapple body grammar.
use super::position;
use super::reader::ComponentReader;
use super::widths::signed_skip;
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::reference::diagnostics::WidthPurpose;
use crate::theater::parser::v41::reference::observations::{
    AbilityNonPredictedState, FilmComponentObservation, MovementComponent, UnitReference,
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
            // FUN_140f03dfc clears dst[2]/dst[3] before FUN_142f262d4
            // tests them. The six-bit value does not enable extra target
            // fields; both row-version paths reach this same body.
            r.gate("value", 6, true)?;
            if r.bit("position.present")? && !position::contextual_point(r, "position")? {
                return Some(false);
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
            state.body_walked = true;
            if !anchor_body(r, &mut state)? {
                return Some(false);
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

/// FUN_142f25e90: encoded label, optional source attachment and place, six
/// recorded flags, then one of eight bodies. Labels 7/8 have no extra body.
fn anchor_body(r: &mut ComponentReader<'_>, state: &mut AbilityNonPredictedState) -> Option<bool> {
    let inner = r.r("inner", 3)? as u32;
    state.inner = Some(inner);
    let reference = anchor_reference(r, "reference", 1)?;
    let attached = reference.present;
    state.reference = Some(reference);
    let start = usize::try_from(r.cursor.position).ok()?;
    if attached {
        // FUN_142f04664 uses this parent-local form even in snapshots.
        r.r("position.local_region", 2)?;
        r.words("position.local_axes", 3, 13)?;
        r.gate("position.local_extra", 16, true)?;
    } else if !position::point(r, "position")? {
        return Some(false);
    }
    state.position = Some(BitRange {
        start,
        end: usize::try_from(r.cursor.position).ok()?,
    });
    state.flags = Some(r.r("flags", 6)? as u32);
    match inner + 1 {
        1 => state.value8 = r.gated_value("value8", 8, true)?.map(|v| v as u32),
        2 => {
            state.source = Some(anchor_reference(r, "source", 5)?);
            state.value8 = r.gated_value("value8", 8, true)?.map(|v| v as u32);
        }
        3 => {
            state.target = Some(anchor_reference(r, "target", 0)?);
            state.source = Some(anchor_reference(r, "source", 5)?);
            for i in 0..3 {
                state.vectors[i] = anchor_vector(r, i)?;
            }
            state.packed = Some(r.r("packed", 24)? as u32);
            state.tail = Some(r.r("tail9", 9)? as u32);
        }
        4 | 5 => {
            state.source = Some(anchor_reference(r, "source", 5)?);
            state.vectors[0] = anchor_vector(r, 0)?;
            let start = usize::try_from(r.cursor.position).ok()?;
            if !position::point(r, "destination")? {
                return Some(false);
            }
            state.destination = Some(BitRange {
                start,
                end: usize::try_from(r.cursor.position).ok()?,
            });
            state.packed = Some(r.r("packed", 24)? as u32);
            state.tail = Some(r.r("tail9", 9)? as u32);
        }
        6 => {
            state.value8 = r.gated_value("value8", 8, true)?.map(|v| v as u32);
            if state.value8.is_none() {
                state.source = Some(anchor_reference(r, "source", 5)?);
            }
            state.target = Some(anchor_reference(r, "target", 0)?);
            for i in 0..2 {
                state.vectors[i] = anchor_vector(r, i)?;
            }
            state.flag = Some(r.bit("flag")?);
            state.packed = Some(r.r("packed", 24)? as u32);
        }
        _ => {}
    }
    Some(true)
}

fn anchor_reference(
    r: &mut ComponentReader<'_>,
    name: &str,
    category: u8,
) -> Option<UnitReference> {
    r.optional_handle(name, category)?;
    r.references.last().cloned()
}

fn anchor_vector(r: &mut ComponentReader<'_>, index: usize) -> Option<Option<[u32; 2]>> {
    if r.bit(&format!("vector[{index}].constant"))? {
        return Some(None);
    }
    Some(Some([
        r.r(&format!("vector[{index}].direction"), 24)? as u32,
        r.r(&format!("vector[{index}].magnitude"), 12)? as u32,
    ]))
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
    r.optional_handle("reference", 0)?;
    if let Some(encoding) = r.position_encoding
        && !encoding.bodies.mobility
    {
        return mobility_extra(r, encoding.bodies.mobility_extra_bits);
    }
    r.gate("index", 10, true)?;
    if !r.bit("keep_transform")? {
        if !position::contextual_point(r, "transform.position")? {
            return Some(false);
        }
        r.gate("direction", 19, false)?;
        r.r("roll", 8)?;
    }
    r.words("vector_bits", 3, 32)?;
    if !position::contextual_point(r, "position")? {
        return Some(false);
    }
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

#[cfg(test)]
mod tests;
