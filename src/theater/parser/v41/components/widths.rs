//! Native traversal overrides: calibrated replacements and extra unported tails.
use super::{Reader, component};
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
pub(super) fn signed_skip(
    r: &mut Reader<'_>,
    component: &str,
    width: i64,
    calibrated: bool,
    purpose: Option<crate::theater::parser::v41::NativeWidthPurpose>,
    fields: (&str, &str),
) -> Option<()> {
    let bit = r.cursor.position;
    let native_end = bit.wrapping_add(width);
    let end_bit = usize::try_from(native_end).ok();
    let adjustment = |retained_bits| crate::theater::parser::v41::NativeWidthAdjustment {
        component: component.into(),
        calibrated,
        purpose,
        bit,
        width,
        retained_bits,
        end_bit,
    };
    // Native Skip does no reads. Backward/wrapped/non-addressable source ranges
    // are retained as signed adjustments, never synthesized as byte ranges.
    if width < 0 || bit < 0 || native_end < bit {
        r.diagnostics.width_adjustments.push(adjustment(None));
        r.cursor.skip_signed(width);
        return Some(());
    }
    let available = r
        .cursor
        .remaining_source_bits()
        .min(usize::try_from(width).unwrap_or(usize::MAX));
    let compact = width > 4096 && (available as u64) < width as u64;
    let retained = if compact {
        available
    } else {
        usize::try_from(width).ok()?
    };
    r.words(fields.0, retained / 64, 64)?;
    if !retained.is_multiple_of(64) {
        r.r(fields.1, retained % 64)?;
    }
    if compact {
        r.cursor.skip_signed(width - retained as i64);
        r.diagnostics
            .width_adjustments
            .push(adjustment(Some(retained)));
    }
    Some(())
}
fn signed_override(r: &mut Reader<'_>, name: &str, width: i64, calibrated: bool) -> Option<bool> {
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

/// Snapshot before dispatch: live hooks may mutate shared width maps later.
pub(super) fn is_calibrated(
    r: &Reader<'_>,
    name: &str,
    widths: Option<&ComponentWidthOverrides>,
) -> bool {
    if let Some(grammar) = &r.live_grammar {
        grammar
            .calibrated_widths
            .as_ref()
            .is_some_and(|w| w.get(name).is_some())
    } else {
        widths.is_some_and(|w| w.calibrated.contains_key(name))
    }
}

pub(super) fn result_variant(
    name: &str,
    fields: &[super::ComponentField],
    calibrated: bool,
) -> u32 {
    if calibrated {
        0
    } else if name == "weapon-state-type-info" {
        fields
            .iter()
            .find(|f| f.name == "variant")
            .map_or(u32::MAX, |f| f.raw as u32)
    } else {
        u32::MAX
    }
}

pub(super) fn read_component(
    r: &mut Reader<'_>,
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
