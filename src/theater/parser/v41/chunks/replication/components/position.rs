//! v41 position component bit grammar.
use super::reader::ComponentReader;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Reference inherited body-reading switches. Defaults match v41 production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ComponentBodyPolicy {
    pub ability_anchor: bool,
    pub mobility: bool,
    /// Reference calibration skip used only when an active mobility body is disabled.
    pub mobility_extra_bits: i64,
}
impl Default for ComponentBodyPolicy {
    fn default() -> Self {
        Self {
            ability_anchor: true,
            mobility: true,
            mobility_extra_bits: 0,
        }
    }
}

impl ComponentBodyPolicy {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

/// Precision and dependent component-body context. Widths require film/map
/// evidence; there is no universal default for a v41 film. World units are separate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PositionEncoding {
    /// Inherited switches for component bodies that consume this precision context.
    #[serde(default, skip_serializing_if = "ComponentBodyPolicy::is_default")]
    pub bodies: ComponentBodyPolicy,
    pub index_bits: usize,
    /// Explicit map widths for ungated world-object vectors (e.g. grapple anchors).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_axis_bits: Option<[usize; 3]>,
    pub handle_bits: usize,
    /// Axis widths used by generic e524 traversal (distinct from world-object bounds).
    pub traversal_axis_bits: [usize; 3],
    pub default_axis_bits: [usize; 3],
    pub region_axis_bits: BTreeMap<u32, [usize; 3]>,
    pub delta_axis_bits: [usize; 3],
    /// Reference map-specific 47/101-bit calibration path; disabled in production.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub calibrated_skip: bool,
    /// Reference baseline scope is independent of the global full-precision switch.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub baseline_scope: bool,
    pub full_precision: bool,
    pub writer_absolute: bool,
    /// A full-scope handle needs the engine-dependent category-0 width.
    /// Unknown engine type must keep this possible.
    #[serde(default = "default_true")]
    pub engine_one_possible: bool,
    pub delta_handle_tail: bool,
}

fn default_true() -> bool {
    true
}

impl PositionEncoding {
    pub(crate) fn full_precision_gate(&self) -> bool {
        self.baseline_scope || self.full_precision
    }

    pub(crate) fn valid(&self) -> bool {
        self.index_bits <= 30
            && self.handle_bits <= 30
            && self
                .default_axis_bits
                .iter()
                .chain(self.delta_axis_bits.iter())
                .chain(self.traversal_axis_bits.iter())
                .chain(self.region_axis_bits.values().flatten())
                .chain(self.world_axis_bits.iter().flatten())
                .all(|w| (0..=32).contains(w))
    }
}

pub(crate) fn traversal_payload(r: &mut ComponentReader<'_>) -> Option<()> {
    let encoding = r.position_encoding?;
    if !r.bit("traversal.region.gate")? {
        let raw = r.reference_widths.map_or(encoding.handle_bits as u64, |w| {
            w.movement.traversal.index_bits
        });
        let width = r.reference_position_width(raw, "traversal index")?;
        r.r_wide("traversal.region", width)?;
    }
    let axes = r
        .reference_widths
        .map_or(encoding.traversal_axis_bits.map(|v| v as u64), |w| {
            w.movement.traversal.axis_bits
        });
    for (i, raw) in axes.into_iter().enumerate() {
        let width = r.reference_position_width(raw, "traversal axes")?;
        r.r_wide(&format!("traversal.position[{i}]"), width)?;
    }
    Some(())
}

pub(crate) fn component(r: &mut ComponentReader<'_>) -> Option<bool> {
    if r.snapshot {
        let predicted = r.bit("use_prediction")?;
        let delta = r.bit("delta")?;
        if predicted || delta {
            return Some(false);
        }
        r.bit("snapshot.flag")?;
        let words = raw_words(r, "vector_bits")?;
        if !raw_words_finite(words) {
            r.context_error = Some("nonfinite raw position");
            return Some(false);
        }
        r.r("finite", 2)?;
        return Some(true);
    }
    let Some(encoding) = r.position_encoding else {
        return Some(false);
    };
    let predicted = r.bit("use_prediction")?;
    if encoding.calibrated_skip {
        // The reference calibration path reads only the first discriminant. It
        // publishes neither positions nor references and does not update a world.
        let remaining = if predicted { 100 } else { 46 };
        r.words("position.calibrated_skip", remaining / 64, 64)?;
        if !remaining.is_multiple_of(64) {
            r.r("position.calibrated_skip_tail", remaining % 64)?;
        }
        return Some(true);
    }
    let delta = r.bit("delta")?;
    if predicted {
        let handle = r.bit("has_handle")?;
        r.words("baseline_vector_bits", 3, 32)?;
        handle_tail(r, handle, encoding)?;
        return Some(true);
    }
    if !delta {
        if encoding.writer_absolute {
            let handle = r.bit("has_handle")?;
            if encoding.full_precision_gate() {
                let words = raw_words(r, "vector_bits")?;
                if !scope_handle_tail(r, handle, encoding)? {
                    return Some(false);
                }
                if !raw_words_finite(words) {
                    r.context_error = Some("nonfinite raw position");
                    return Some(false);
                }
                r.r("finite", 2)?;
                return Some(true);
            } else {
                absolute_payload(r, encoding)?;
            }
            handle_tail(r, handle, encoding)?;
            r.r("finite", 2)?;
        } else {
            absolute(r, encoding)?;
        }
        return Some(true);
    }
    if !r.bit("predicted_absolute")? {
        if encoding.full_precision_gate() {
            r.words("baseline_vector_bits", 3, 32)?;
        } else if r.bit("absolute_fallback")? {
            absolute(r, encoding)?;
        } else if r.bit("delta8")? {
            for i in 0..3 {
                r.r(&format!("delta_signed8[{i}]"), 8)?;
            }
        } else {
            let axes = r
                .reference_widths
                .map_or(encoding.delta_axis_bits.map(|v| v as u64), |w| {
                    if w.movement.delta_axis_width == 0 {
                        w.movement.traversal.axis_bits
                    } else {
                        [w.movement.delta_axis_width; 3]
                    }
                });
            for (i, raw) in axes.into_iter().enumerate() {
                let width = r.reference_position_width(raw, "delta axes")?;
                r.r_wide(&format!("delta[{i}]"), width)?;
            }
        }
    } else {
        let default = r.bit("default_vector")?;
        if encoding.full_precision_gate() {
            r.words("vector_bits", 3, 32)?;
        } else if !default {
            absolute_payload(r, encoding)?;
        }
    }
    if encoding.delta_handle_tail {
        if r.bit("handle_selector")? {
            r.optional_handle("handle", 0)?;
        }
        if r.bit("region_present")? {
            r.gate("region", 11, true)?;
        }
    }
    Some(true)
}

/// The stream's 32-bit words become IEEE-754 words only after byte reversal.
/// Keep their original bits in source fields; this predicate decides whether the
/// native reader admits the two closing bits after the parent tail.
pub(crate) fn raw_words_finite(words: [u32; 3]) -> bool {
    words
        .into_iter()
        .all(|word| word.swap_bytes() & 0x7f80_0000 != 0x7f80_0000)
}

pub(crate) fn raw_words(r: &mut ComponentReader<'_>, name: &str) -> Option<[u32; 3]> {
    let mut words = [0; 3];
    for (i, word) in words.iter_mut().enumerate() {
        *word = r.r(&format!("{name}[{i}]"), 32)? as u32;
    }
    Some(words)
}

fn absolute(r: &mut ComponentReader<'_>, encoding: &PositionEncoding) -> Option<()> {
    let high = r.bit("precision_high")?;
    if encoding.full_precision_gate() {
        r.words("vector_bits", 3, 32)?;
    } else if high {
        // The bounded vector uses +/-100 at precision level 16: 14 bits per axis.
        // Its handle and region precede the finite tail, even outside full-state scope.
        for i in 0..3 {
            r.r(&format!("bounded_position[{i}]"), 14)?;
        }
        r.optional_handle("position_handle", 0)?;
        r.gate("position_region", 11, true)?;
        r.r("finite", 2)?;
    } else {
        absolute_payload(r, encoding)?;
        r.r("finite", 2)?;
    }
    Some(())
}

pub(crate) fn absolute_payload(
    r: &mut ComponentReader<'_>,
    encoding: &PositionEncoding,
) -> Option<()> {
    let mut raw_widths = encoding.default_axis_bits.map(|v| v as u64);
    let mut region_index = -1;
    if !r.bit("default_region")? {
        let raw = r.reference_widths.map_or(encoding.index_bits as u64, |w| {
            w.movement.world_object.index_bits
        });
        let width = r.reference_position_width(raw, "world index")?;
        let raw_index = r.r_wide("region_index", width)?;
        let index = raw_index as u32;
        region_index = index as i32;
        let known_axes = r
            .reference_widths
            .and_then(|w| w.movement.world_object.regions.get(&index).copied());
        let complete_table = r
            .reference_widths
            .is_some_and(|w| !w.movement.world_object.regions.is_empty());
        if complete_table
            && known_axes.is_none_or(|axes| !axes.iter().all(|w| (0..=32).contains(w)))
        {
            r.context_error = Some("unknown world region");
            return None;
        }
        if known_axes.is_none()
            && let Some(bits) = r
                .reference_widths
                .and_then(|w| w.movement.opaque_position_region_bits.get(&index).copied())
        {
            // Do not split an aggregate width into invented axes. Runtime position
            // resolution requires position[0..2], so this cannot publish a pose.
            r.record_absolute(region_index);
            r.r_wide("position.opaque_region_bits", bits)?;
            return Some(());
        }
        if let Some(w) = r.reference_widths {
            // Reference absAxisWFor takes a signed host int. Its negative sentinel
            // includes wide recorded indices whose low 64-bit word is negative.
            if raw_index as i64 >= 0 {
                raw_widths = if w.movement.world_object.regions.is_empty() {
                    w.movement.world_object.axis_bits
                } else {
                    known_axes?
                };
            }
        } else if let Some(region) = encoding.region_axis_bits.get(&index) {
            raw_widths = region.map(|v| v as u64);
        } else if let Some(world) = encoding.world_axis_bits {
            raw_widths = world.map(|v| v as u64);
        }
    }
    r.record_absolute(region_index);
    for (i, raw) in raw_widths.into_iter().enumerate() {
        let width = r.reference_position_width(raw, "world axes")?;
        r.r_wide(&format!("position[{i}]"), width)?;
    }
    Some(())
}

fn handle_tail(
    r: &mut ComponentReader<'_>,
    present: bool,
    encoding: &PositionEncoding,
) -> Option<()> {
    if !present {
        return Some(());
    }
    if r.bit("handle_selector")? && r.bit("handle_present")? {
        // This legacy precision-profile path still calls FUN_1406d3140.
        // A configured width cannot replace a missing global range.
        r.require_category_references()?;
        let raw = r.reference_widths.map_or(encoding.handle_bits as u64, |w| {
            w.movement.traversal.index_bits
        });
        let width = r.reference_position_width(raw, "traversal index")?;
        r.r_wide("handle_value", width)?;
        r.r("handle_generation", 2)?;
    }
    if r.bit("region_present")? {
        r.gate("region", 11, true)?;
    }
    Some(())
}

/// FUN_14076e3e4 in the writer's full-scope absolute form. This differs
/// from the inline keep-baseline/delta handle tail above.
fn scope_handle_tail(
    r: &mut ComponentReader<'_>,
    present: bool,
    encoding: &PositionEncoding,
) -> Option<bool> {
    if !present {
        return Some(true);
    }
    if encoding.engine_one_possible {
        r.context_error = Some("engine-one handle width");
        return Some(false);
    }
    r.optional_handle("scope.handle", 0)?;
    r.gate("scope.region", 11, true)?;
    Some(true)
}

/// Precision-16 point with the explicit full-precision/baseline overrides used
/// by biped movement readers. Snapshot and default-box forms need no map table.
pub(crate) fn contextual_point(r: &mut ComponentReader<'_>, prefix: &str) -> Option<bool> {
    if r.position_encoding.is_some_and(|e| e.full_precision_gate()) {
        r.words(&format!("{prefix}.float_bits"), 3, 32)?;
        Some(true)
    } else {
        point(r, prefix)
    }
}

/// Point used by the low-frequency/navpoint layouts: snapshot raw float bits
/// versus a delta's explicit region/quantized fields. Keep fields on failure.
pub(crate) fn point(r: &mut ComponentReader<'_>, prefix: &str) -> Option<bool> {
    if r.snapshot {
        r.words(&format!("{prefix}.float_bits"), 3, 32)?;
        return Some(true);
    }
    let Some(encoding) = r.position_encoding else {
        // The default box is self-contained (three 22-bit words). Only the
        // indexed box needs external index/axis widths, after this recorded gate.
        if r.bit(&format!("{prefix}.default_region"))? {
            r.words(&format!("{prefix}.position"), 3, 22)?;
            return Some(true);
        }
        return Some(false);
    };
    let start = r.fields.len();
    let result = absolute_payload(r, encoding);
    for field in &mut r.fields[start..] {
        field.name = format!("{prefix}.{}", field.name);
    }
    result.map(|_| true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::parser::v41::{
        chunks::replication::components::cursor::ComponentCursor, context::V41DecodeConfig,
    };

    #[test]
    fn captured_bounded_position_consumes_its_reference_tail() {
        let payload = include_bytes!(
            "../../../../../fixtures/movement-schema-v41/recharge-attached-position.bin"
        );
        let config = V41DecodeConfig::default();
        let layout = config.record_layout();
        let mut reader = ComponentReader {
            category_references: true,
            snapshot: false,
            reference_widths: None,
            width_error: None,
            context_error: None,
            live_grammar: None,
            movement_slot: None,
            references: vec![],
            diagnostics: Default::default(),
            cursor: ComponentCursor::new(payload, 867).unwrap(),
            fields: vec![],
            position_encoding: layout.position.as_ref(),
        };
        assert_eq!(component(&mut reader), Some(true));
        assert_eq!(reader.cursor.position, 942);
        let field = |name: &str| {
            reader
                .fields
                .iter()
                .find(|f| f.name == name)
                .unwrap()
                .raw
                .low_u64()
        };
        assert_eq!(field("bounded_position[0]"), 8288);
        assert_eq!(field("bounded_position[1]"), 8138);
        assert_eq!(field("bounded_position[2]"), 8257);
        assert_eq!(reader.references.len(), 1);
        assert_eq!(reader.references[0].value, 768);
        assert_eq!(reader.references[0].tail, 1);
        assert_eq!(field("position_region"), 1);
        assert_eq!(field("finite"), 0);
        reader.cursor = ComponentCursor::new(&payload[..117], 867).unwrap();
        assert_eq!(component(&mut reader), None);
    }
}
