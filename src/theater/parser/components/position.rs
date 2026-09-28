//! Position bit grammar from LevelUp components_position_i0.go.
use super::Reader;
use crate::theater::parser::position_capture::native_shift_one;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Native inherited body-reading switches. Defaults match v41 production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ComponentBodyPolicy {
    pub ability_anchor: bool,
    pub mobility: bool,
    /// Native calibration skip used only when an active mobility body is disabled.
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
    /// Native map-specific 47/101-bit calibration path; disabled in production.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub calibrated_skip: bool,
    /// Native baseline scope is independent of the global full-precision switch.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub baseline_scope: bool,
    pub full_precision: bool,
    pub writer_absolute: bool,
    pub delta_handle_tail: bool,
}

impl PositionEncoding {
    pub(super) fn full_precision_gate(&self) -> bool {
        self.baseline_scope || self.full_precision
    }

    pub(super) fn valid(&self) -> bool {
        self.index_bits <= 30
            && self.handle_bits <= 30
            && self
                .default_axis_bits
                .iter()
                .chain(self.delta_axis_bits.iter())
                .chain(self.traversal_axis_bits.iter())
                .chain(self.region_axis_bits.values().flatten())
                .chain(self.world_axis_bits.iter().flatten())
                .all(|w| (1..=32).contains(w))
    }
}

pub(super) fn traversal_payload(r: &mut Reader<'_>) -> Option<()> {
    let encoding = r.position_encoding?;
    if !r.bit("traversal.region.gate")? {
        let raw = r.native_widths.map_or(encoding.handle_bits as u64, |w| {
            w.movement.traversal.index_bits
        });
        let width = r.native_position_width(raw, "traversal index")?;
        r.r_wide("traversal.region", width)?;
    }
    let axes = r
        .native_widths
        .map_or(encoding.traversal_axis_bits.map(|v| v as u64), |w| {
            w.movement.traversal.axis_bits
        });
    for (i, raw) in axes.into_iter().enumerate() {
        let width = r.native_position_width(raw, "traversal axes")?;
        r.r_wide(&format!("traversal.position[{i}]"), width)?;
    }
    Some(())
}

impl Reader<'_> {
    fn emit_position(
        &mut self,
        kind: crate::theater::parser::NativePositionKind,
        mut vector: [f32; 3],
    ) {
        use crate::theater::parser::NativePositionKind as K;
        let Some(capture) = self.position_capture.as_mut() else {
            return;
        };
        if kind == K::Baseline {
            let Some(previous) = capture
                .world
                .as_ref()
                .and_then(|w| w.position(capture.slot))
            else {
                return;
            };
            vector = previous;
        } else if let Some(world) = capture.world.as_mut() {
            if matches!(kind, K::Delta8 | K::DeltaAxis) {
                let Some(previous) = world.position(capture.slot) else {
                    return;
                };
                for i in 0..3 {
                    vector[i] += previous[i];
                }
            }
            world.set_position(capture.slot, vector);
        }
        if capture.emit {
            self.publish_component(crate::theater::parser::FilmComponentObservation::Position {
                position_kind: kind,
                vector_bits: vector.map(f32::to_bits),
                bit: self.position_start,
                slot: self.position_slot,
            });
        }
    }
    fn emit_absolute_position(&mut self, region: i32, q: [u64; 3], widths: [u64; 3]) {
        let Some(capture) = self.position_capture.as_ref() else {
            return;
        };
        if region < 0 || region as u32 != capture.map.region {
            return;
        }
        let vector = std::array::from_fn(|i| {
            let step =
                (capture.map.max[i] - capture.map.min[i]) / native_shift_one(widths[i]) as f32;
            (q[i] as f32).mul_add(step, capture.map.min[i]) + step * 0.5
        });
        self.emit_position(
            if self.position_fallback {
                crate::theater::parser::NativePositionKind::AbsoluteFallback
            } else {
                crate::theater::parser::NativePositionKind::Absolute
            },
            vector,
        );
    }
}

pub(super) fn component(r: &mut Reader<'_>) -> Option<bool> {
    let Some(encoding) = r.position_encoding else {
        return Some(false);
    };
    r.position_start = r.cursor.position;
    r.position_slot = r.position_capture.as_ref().map_or(0, |c| c.slot);
    let predicted = r.bit("use_prediction")?;
    if encoding.calibrated_skip {
        // The native calibration path reads only the first discriminant. It
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
        r.emit_position(
            crate::theater::parser::NativePositionKind::Baseline,
            [0.; 3],
        );
        handle_tail(r, handle, encoding)?;
        return Some(true);
    }
    if !delta {
        if encoding.writer_absolute {
            let handle = r.bit("has_handle")?;
            if encoding.full_precision_gate() {
                r.words("vector_bits", 3, 32)?;
            } else {
                absolute_payload_observed(r, encoding)?;
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
            r.emit_position(
                crate::theater::parser::NativePositionKind::Baseline,
                [0.; 3],
            );
        } else if r.bit("absolute_fallback")? {
            r.position_fallback = true;
            let result = absolute(r, encoding);
            r.position_fallback = false;
            result?;
        } else if r.bit("delta8")? {
            let mut d = [0.; 3];
            let quantum = r.position_capture.as_ref().map_or(0., |c| c.quantum);
            for (i, v) in d.iter_mut().enumerate() {
                *v = r.r(&format!("delta_signed8[{i}]"), 8)? as u8 as i8 as f32 * quantum;
            }
            r.emit_position(crate::theater::parser::NativePositionKind::Delta8, d);
        } else {
            let mut d = [0.; 3];
            let quantum = r.position_capture.as_ref().map_or(0., |c| c.quantum);
            let axes = r
                .native_widths
                .map_or(encoding.delta_axis_bits.map(|v| v as u64), |w| {
                    if w.movement.delta_axis_width == 0 {
                        w.movement.traversal.axis_bits
                    } else {
                        [w.movement.delta_axis_width; 3]
                    }
                });
            for (i, raw) in axes.into_iter().enumerate() {
                let width = r.native_position_width(raw, "delta axes")?;
                let q = r.r_wide(&format!("delta[{i}]"), width)?;
                d[i] = (q as f32 - native_shift_one(width.wrapping_sub(1)) as f32) * quantum;
            }
            r.emit_position(crate::theater::parser::NativePositionKind::DeltaAxis, d);
        }
    } else {
        let default = r.bit("default_vector")?;
        if encoding.full_precision_gate() {
            r.words("vector_bits", 3, 32)?;
        } else if !default {
            absolute_payload_observed(r, encoding)?;
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

fn absolute(r: &mut Reader<'_>, encoding: &PositionEncoding) -> Option<()> {
    let high = r.bit("precision_high")?;
    if encoding.full_precision_gate() {
        r.words("vector_bits", 3, 32)?;
    } else if !high {
        absolute_payload_observed(r, encoding)?;
        r.r("finite", 2)?;
    }
    Some(())
}

pub(super) fn absolute_payload(r: &mut Reader<'_>, encoding: &PositionEncoding) -> Option<()> {
    read_absolute_payload(r, encoding).map(|_| ())
}

fn absolute_payload_observed(r: &mut Reader<'_>, encoding: &PositionEncoding) -> Option<()> {
    let (region, q, widths) = read_absolute_payload(r, encoding)?;
    r.emit_absolute_position(region, q, widths);
    Some(())
}

fn read_absolute_payload(
    r: &mut Reader<'_>,
    encoding: &PositionEncoding,
) -> Option<(i32, [u64; 3], [u64; 3])> {
    let mut raw_widths = encoding.default_axis_bits.map(|v| v as u64);
    let mut region_index = -1;
    if !r.bit("default_region")? {
        let raw = r.native_widths.map_or(encoding.index_bits as u64, |w| {
            w.movement.world_object.index_bits
        });
        let width = r.native_position_width(raw, "world index")?;
        let raw_index = r.r_wide("region_index", width)?;
        let index = raw_index as u32;
        region_index = index as i32;
        if let Some(w) = r.native_widths {
            // Native absAxisWFor takes a signed host int. Its negative sentinel
            // includes wide recorded indices whose low 64-bit word is negative.
            if raw_index as i64 >= 0 {
                raw_widths = w.movement.world_object.axis_bits;
            }
        } else if let Some(region) = encoding.region_axis_bits.get(&index) {
            raw_widths = region.map(|v| v as u64);
        } else if let Some(world) = encoding.world_axis_bits {
            raw_widths = world.map(|v| v as u64);
        }
    }
    r.record_absolute(region_index);
    let mut q = [0; 3];
    let mut widths = [0; 3];
    for (i, raw) in raw_widths.into_iter().enumerate() {
        widths[i] = r.native_position_width(raw, "world axes")?;
        q[i] = r.r_wide(&format!("position[{i}]"), widths[i])?;
    }
    Some((region_index, q, widths))
}

fn handle_tail(r: &mut Reader<'_>, present: bool, encoding: &PositionEncoding) -> Option<()> {
    if !present {
        return Some(());
    }
    if r.bit("handle_selector")? && r.bit("handle_present")? {
        let raw = r.native_widths.map_or(encoding.handle_bits as u64, |w| {
            w.movement.traversal.index_bits
        });
        let width = r.native_position_width(raw, "traversal index")?;
        r.r_wide("handle_value", width)?;
        r.r("handle_generation", 2)?;
    }
    if r.bit("region_present")? {
        r.gate("region", 11, true)?;
    }
    Some(())
}
