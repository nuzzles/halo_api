//! Position bit grammar from LevelUp components_position_i0.go.
use super::Reader;
use crate::theater::position_capture::native_shift_one;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Native inherited body-reading switches. Defaults match v41 production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ComponentBodyPolicy {
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
pub struct PositionEncoding {
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
    fn emit_position(&mut self, kind: crate::theater::NativePositionKind, mut vector: [f32; 3]) {
        use crate::theater::NativePositionKind as K;
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
            self.publish_component(crate::theater::FilmComponentObservation::Position {
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
                crate::theater::NativePositionKind::AbsoluteFallback
            } else {
                crate::theater::NativePositionKind::Absolute
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
        r.emit_position(crate::theater::NativePositionKind::Baseline, [0.; 3]);
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
            r.emit_position(crate::theater::NativePositionKind::Baseline, [0.; 3]);
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
            r.emit_position(crate::theater::NativePositionKind::Delta8, d);
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
            r.emit_position(crate::theater::NativePositionKind::DeltaAxis, d);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::*;
    use std::io::Read;

    #[test]
    fn native_position_hook_sequence() {
        check_position_sequence(
            include_bytes!("../fixtures/position-hook-sequence-v41.json.zlib"),
            1024,
        );
    }
    #[test]
    fn native_calibrated_position_sequence() {
        check_position_sequence(
            include_bytes!("../fixtures/calibrated-position-sequence-v41.json.zlib"),
            512,
        );
    }
    #[test]
    fn native_baseline_scope_sequence() {
        check_position_sequence(
            include_bytes!("../fixtures/baseline-scope-sequence-v41.json.zlib"),
            512,
        );
    }
    fn check_position_sequence(fixture: &[u8], expected_cases: usize) {
        #[derive(Deserialize)]
        struct Step {
            #[serde(default)]
            quantum: Option<f32>,
            name: String,
            start: usize,
            slot: u32,
            emit: bool,
            ported: bool,
            end: usize,
            observations: Vec<FilmComponentObservation>,
            positions: serde_json::Value,
        }
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            encoding: PositionEncoding,
            min: [f32; 3],
            max: [f32; 3],
            region: u32,
            quantum: f32,
            accumulate: bool,
            steps: Vec<Step>,
        }
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_end(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&json).unwrap();
        assert_eq!(cases.len(), expected_cases);
        let saw_baseline_scope = cases.iter().any(|c| c.encoding.baseline_scope);
        let mut baseline_controls = 0;
        let mut baseline_flocks = 0;
        let mut nonposition_quantized_reads = 0;
        let mut calibrated_widths = BTreeMap::new();
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<_> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
                .collect();
            let map = FilmMapBounds {
                module: String::new(),
                min: c.min,
                max: c.max,
                region: c.region,
                axis_widths: c.encoding.world_axis_bits.unwrap(),
                region_index_bits: c.encoding.index_bits,
            };
            let mut world = FilmWorld::default();
            for slot in 50..53 {
                world.bind_full(slot, 0);
                world.set_position(slot, [slot as f32, 2., 3.]);
            }
            let initial = world.clone();
            let mut live_world = initial.clone();
            let mut profile = NativeScanProfile::default();
            profile.grammar.simulation_complete = true;
            profile.grammar.writer_absolute = c.encoding.writer_absolute;
            profile.grammar.baseline_scope = c.encoding.baseline_scope;
            profile.movement.calibrated_skip = c.encoding.calibrated_skip;
            profile.movement.full_precision = c.encoding.full_precision;
            profile.movement.delta_has_handle_tail = c.encoding.delta_handle_tail;
            profile.movement.delta_axis_width = c.encoding.delta_axis_bits[0] as u64;
            profile.movement.delta_quantum = c.quantum;
            profile.movement.range = std::array::from_fn(|a| [c.min[a], c.max[a]]);
            profile.movement.world_object.index_bits = c.encoding.index_bits as u64;
            profile.movement.world_object.axis_bits = map.axis_widths.map(|v| v as u64);
            profile.movement.world_object.region = c.region;
            profile.movement.traversal.index_bits = c.encoding.handle_bits as u64;
            profile.movement.traversal.axis_bits = c.encoding.traversal_axis_bits.map(|v| v as u64);
            let observer = NativeFilmObserver::default();
            let emitted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let mut live = NativeFilmReader::with_context(
                &data,
                NativeReaderContext {
                    profile,
                    observer: Some(observer.clone()),
                },
            );
            if c.accumulate {
                live.replace_position_accumulator(Some(&mut live_world));
            }
            let mut r = Reader {
                native_widths: None,
                width_error: None,
                live_observer: None,
                live_grammar: None,
                position_capture: Some(NativePositionCapture {
                    map: &map,
                    quantum: c.quantum,
                    slot: 0,
                    world: if c.accumulate { Some(&mut world) } else { None },
                    emit: true,
                }),
                position_start: 0,
                position_slot: 0,
                position_fallback: false,
                movement_slot: None,
                references: vec![],
                diagnostics: Default::default(),
                cursor: super::super::Cursor::new_padded(&data, 0),
                fields: vec![],
                position_encoding: Some(&c.encoding),
            };
            for (j, s) in c.steps.into_iter().enumerate() {
                emitted.lock().unwrap().clear();
                let events = emitted.clone();
                observer.set_hook(
                    NativeHookKind::Position,
                    s.emit.then(|| {
                        std::sync::Arc::new(move |p: NativeHookPublication<'_>| {
                            let NativeHookPublication::Component(value) = p else {
                                panic!("position publication")
                            };
                            events.lock().unwrap().push(value.clone());
                        }) as NativeHook
                    }),
                );
                let mut context = live.context();
                context.profile.movement.delta_quantum = s.quantum.unwrap_or(c.quantum);
                if j.is_multiple_of(2) {
                    live.replace_context(context);
                } else {
                    let old_bit = live.bit_position();
                    let old_slot = live.capture_slot();
                    let old_world = live.position_accumulator().map(|world| world as *const _);
                    live.replace_profile(context.profile);
                    let previous = live.replace_observer(context.observer).unwrap();
                    assert!(previous.same_instance(&observer));
                    assert_eq!(live.bit_position(), old_bit);
                    assert_eq!(live.capture_slot(), old_slot);
                    assert_eq!(
                        live.position_accumulator().map(|world| world as *const _),
                        old_world
                    );
                }
                live.set_bit_position(s.start);
                live.set_capture_slot(s.slot);
                let (live_status, _) = live.read_component(&s.name, 2, 35).unwrap();
                assert_eq!(live_status == Some(true), s.ported, "live status {i}/{j}");
                assert_eq!(live.bit_position(), s.end, "live end {i}/{j}");
                let expected: Vec<_> = s
                    .observations
                    .iter()
                    .filter(|o| matches!(o, FilmComponentObservation::Position { .. }))
                    .cloned()
                    .collect();
                assert_eq!(
                    *emitted.lock().unwrap(),
                    expected,
                    "live publications {i}/{j}"
                );
                let actual = live.position_accumulator().unwrap_or(&initial);
                let positions: BTreeMap<_, _> = actual
                    .slots
                    .iter()
                    .map(|(slot, state)| (*slot, state.position.map(|v| v.map(f32::to_bits))))
                    .collect();
                assert_eq!(
                    serde_json::to_value(positions).unwrap(),
                    s.positions,
                    "live accumulator {i}/{j}"
                );
                r.cursor.position = s.start as i64;
                let capture = r.position_capture.as_mut().unwrap();
                capture.slot = s.slot;
                capture.emit = s.emit;
                capture.quantum = s.quantum.unwrap_or(c.quantum);
                r.fields.clear();
                r.references.clear();
                r.diagnostics = Default::default();
                let status = super::super::component(&mut r, &s.name, 2, 35);
                assert_eq!(status == Some(true), s.ported, "status {i}/{j}");
                assert_eq!(
                    serde_json::json!(r.cursor.position),
                    serde_json::json!(s.end),
                    "end {i}/{j}"
                );
                assert_eq!(
                    r.diagnostics.component_observations, s.observations,
                    "publications {i}/{j}"
                );
                if c.encoding.calibrated_skip
                    && s.name == "object-position-dynamic-precision-component"
                {
                    *calibrated_widths.entry(s.end - s.start).or_insert(0usize) += 1;
                    assert!(s.observations.is_empty(), "calibrated publications {i}/{j}");
                }
                if c.encoding.baseline_scope && !c.encoding.full_precision {
                    if s.name == "biped-control-context-component" {
                        assert_eq!(
                            s.end - s.start,
                            3,
                            "baseline does not widen control context {i}/{j}"
                        );
                        baseline_controls += 1;
                    }
                    if s.name == "flock-position-component" {
                        assert_eq!(
                            s.end - s.start,
                            96,
                            "baseline copies raw flock position {i}/{j}"
                        );
                        baseline_flocks += 1;
                    }
                }
                for o in &s.observations {
                    if let FilmComponentObservation::Position { bit, .. } = o {
                        assert_eq!(
                            serde_json::json!(*bit),
                            serde_json::json!(s.start),
                            "position stamp {i}/{j}"
                        );
                    }
                }
                if s.name != "object-position-dynamic-precision-component"
                    && !r.diagnostics.absolute_indices.is_empty()
                {
                    nonposition_quantized_reads += 1;
                    assert!(
                        s.observations
                            .iter()
                            .all(|o| !matches!(o, FilmComponentObservation::Position { .. }))
                    );
                }
                let actual = r
                    .position_capture
                    .as_ref()
                    .unwrap()
                    .world
                    .as_deref()
                    .unwrap_or(&initial);
                let positions: BTreeMap<_, _> = actual
                    .slots
                    .iter()
                    .map(|(slot, state)| (*slot, state.position.map(|v| v.map(f32::to_bits))))
                    .collect();
                assert_eq!(
                    serde_json::to_value(positions).unwrap(),
                    s.positions,
                    "accumulator {i}/{j}"
                );
            }
        }
        assert!(nonposition_quantized_reads > 0);
        if saw_baseline_scope {
            assert!(baseline_controls > 0);
            assert!(baseline_flocks > 0);
        }
        if !calibrated_widths.is_empty() {
            assert_eq!(
                calibrated_widths.keys().copied().collect::<Vec<_>>(),
                [47, 101]
            );
            assert!(calibrated_widths.values().all(|n| *n > 100));
        }
    }
}
