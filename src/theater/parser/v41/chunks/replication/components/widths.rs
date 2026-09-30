//! Reference traversal overrides: calibrated replacements and extra unported tails.
use super::dispatch::component;
use super::reader::ComponentReader;
use crate::theater::parser::v41::reference::diagnostics::{WidthAdjustment, WidthPurpose};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ComponentWidthOverrides {
    /// Replace the entire component reader, including its publications.
    pub calibrated: BTreeMap<String, usize>,
    /// Append raw bits only after a reader reports an unsupported body.
    pub stubs: BTreeMap<String, usize>,
}
impl ComponentWidthOverrides {
    pub(crate) fn is_empty(&self) -> bool {
        self.calibrated.is_empty() && self.stubs.is_empty()
    }
}
pub(crate) fn signed_skip(
    r: &mut ComponentReader<'_>,
    component: &str,
    width: i64,
    calibrated: bool,
    purpose: Option<WidthPurpose>,
    fields: (&str, &str),
) -> Option<()> {
    let bit = r.cursor.position;
    let reference_end = bit.checked_add(width);
    let end_bit = reference_end.and_then(|end| usize::try_from(end).ok());
    let adjustment = |retained_bits| WidthAdjustment {
        component: component.into(),
        calibrated,
        purpose,
        bit,
        width,
        retained_bits,
        end_bit,
    };
    // A layout adjustment cannot invent source bits or move backwards over fields.
    // Retain the attempted extent as a diagnostic and stop at the established boundary.
    if width < 0 || bit < 0 || reference_end.is_none() {
        r.diagnostics.width_adjustments.push(adjustment(None));
        return None;
    }
    let available = r
        .cursor
        .remaining_source_bits()
        .min(usize::try_from(width).unwrap_or(usize::MAX));
    let retained = available;
    r.words(fields.0, retained / 64, 64)?;
    if !retained.is_multiple_of(64) {
        r.r(fields.1, retained % 64)?;
    }
    if !r.cursor.fits_source(width - retained as i64) {
        r.diagnostics
            .width_adjustments
            .push(adjustment(Some(retained)));
        return None;
    }
    Some(())
}
fn signed_override(
    r: &mut ComponentReader<'_>,
    name: &str,
    width: i64,
    calibrated: bool,
) -> Option<bool> {
    let field = if calibrated {
        "component.calibrated"
    } else {
        "component.stub"
    };
    signed_skip(
        r,
        name,
        width,
        calibrated,
        None,
        (field, &format!("{field}.tail")),
    )?;
    Some(true)
}

pub(crate) fn read_component(
    r: &mut ComponentReader<'_>,
    name: &str,
    level: u32,
    archetype: u32,
    widths: Option<&ComponentWidthOverrides>,
    policy: (bool, Option<(&str, usize)>),
) -> Option<bool> {
    let live_calibrated = r
        .live_grammar
        .as_ref()
        .and_then(|g| g.calibrated_widths.as_ref())
        .and_then(|w| w.get(name));
    if r.live_grammar.is_some() {
        if let Some(width) = live_calibrated {
            return signed_override(r, name, width, true);
        }
    } else if let Some(width) = widths.and_then(|w| w.calibrated.get(name)).copied() {
        return signed_override(r, name, i64::try_from(width).ok()?, true);
    }
    let mut status = component(r, name, level, archetype);
    if status == Some(true)
        && !policy.0
        && matches!(name, "simulation-state" | "simulation-state-component")
    {
        status = Some(false);
    }
    if status == Some(false) {
        if let Some((_, width)) = policy.1.filter(|(n, _)| *n == name) {
            r.words("inferred_stub", width / 64, 64)?;
            if !width.is_multiple_of(64) {
                r.r(&format!("inferred_stub[{}]", width / 64), width % 64)?;
            }
            return Some(true);
        }
        if let Some(grammar) = &r.live_grammar {
            if let Some(width) = grammar.stub_widths.as_ref().and_then(|w| w.get(name)) {
                return signed_override(r, name, width, false);
            }
        } else if let Some(width) = widths.and_then(|w| w.stubs.get(name)).copied() {
            return signed_override(r, name, i64::try_from(width).ok()?, false);
        }
    }
    status
}
