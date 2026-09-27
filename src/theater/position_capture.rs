//! Position observations and optional reader-local accumulation from the native i0 reader.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativePositionKind {
    /// Native PosKindRaw: re-emits the saved baseline, never the 96 copied wire bits.
    Baseline,
    Absolute,
    AbsoluteFallback,
    Delta8,
    DeltaAxis,
}
impl NativePositionKind {
    pub fn native_name(self) -> &'static str {
        match self {
            Self::Baseline => "raw",
            Self::Absolute => "abs",
            Self::AbsoluteFallback => "absfb",
            Self::Delta8 => "d8",
            Self::DeltaAxis => "dax",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NativePositionSample {
    pub kind: NativePositionKind,
    /// Absolute if an accumulator is supplied. Otherwise delta kinds remain relative.
    pub vector: [f32; 3],
    pub bit: i64,
    pub slot: u32,
}

/// Serializable dequantization context for frame/keyframe readers. Floating point
/// bits preserve exact profile values while keeping encoding equality deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionCaptureEncoding {
    pub min_bits: [u32; 3],
    pub max_bits: [u32; 3],
    pub quantum_bits: u32,
    pub region: u32,
    pub axis_widths: [usize; 3],
    pub region_index_bits: usize,
}
impl PositionCaptureEncoding {
    pub fn new(map: &FilmMapBounds, quantum: f32) -> Self {
        Self {
            min_bits: map.min.map(f32::to_bits),
            max_bits: map.max.map(f32::to_bits),
            quantum_bits: quantum.to_bits(),
            region: map.region,
            axis_widths: map.axis_widths,
            region_index_bits: map.region_index_bits,
        }
    }
    pub(super) fn map(&self) -> FilmMapBounds {
        FilmMapBounds {
            module: String::new(),
            min: self.min_bits.map(f32::from_bits),
            max: self.max_bits.map(f32::from_bits),
            region: self.region,
            axis_widths: self.axis_widths,
            region_index_bits: self.region_index_bits,
        }
    }
    pub(super) fn reader<'a>(
        &self,
        map: &'a FilmMapBounds,
        slot: u32,
    ) -> NativePositionCapture<'a> {
        NativePositionCapture {
            map,
            slot,
            quantum: f32::from_bits(self.quantum_bits),
            world: None,
            emit: true,
        }
    }
}

/// Map and reader-local state used by native position publications. Suppressing
/// emission still updates an explicitly supplied accumulator, matching LevelUp.
#[derive(Debug)]
pub struct NativePositionCapture<'a> {
    pub map: &'a FilmMapBounds,
    pub quantum: f32,
    pub slot: u32,
    pub world: Option<&'a mut FilmWorld>,
    pub emit: bool,
}

// Go uint64 shifts yield zero at widths >= 64. A zero delta width
// subtracts one in unsigned arithmetic, so its half-range is also zero.
pub(super) fn native_shift_one(width: u64) -> u64 {
    if width < 64 { 1u64 << width } else { 0 }
}

/// Observe a decoded dynamic-position component using its original precision/map
/// context. An optional world seeds absolute positions, adds deltas to existing
/// seeds, and re-emits saved baselines. Unseeded deltas emit nothing with a world;
/// without one they emit relative values. Unbound slots are not created by capture.
/// Call once per committed component; callers own speculative-state rollback.
pub fn capture_component_position(
    component: &DecodedComponent,
    slot: u32,
    map: &FilmMapBounds,
    quantum: f32,
    world: Option<&mut FilmWorld>,
) -> Option<NativePositionSample> {
    if component.name != "object-position-dynamic-precision-component" {
        return None;
    }
    capture_position_fields(
        &component.fields,
        component.start_bit,
        slot,
        map,
        quantum,
        world,
    )
}

pub(super) fn capture_position_fields(
    fields: &[ComponentField],
    bit: i64,
    slot: u32,
    map: &FilmMapBounds,
    quantum: f32,
    mut world: Option<&mut FilmWorld>,
) -> Option<NativePositionSample> {
    let field = |name: &str| fields.iter().find(|f| f.name == name);
    // Only the two native keep paths name baseline_vector_bits. Other full-
    // precision copies name vector_bits and intentionally emit nothing.
    if field("baseline_vector_bits[2]").is_some() {
        return Some(NativePositionSample {
            kind: NativePositionKind::Baseline,
            vector: world.as_ref()?.position(slot)?,
            bit,
            slot,
        });
    }
    let mut vector = [0.; 3];
    let kind = if field("delta_signed8[0]").is_some() {
        for (i, value) in vector.iter_mut().enumerate() {
            *value = field(&format!("delta_signed8[{i}]"))?.raw as u8 as i8 as f32 * quantum;
        }
        NativePositionKind::Delta8
    } else if field("delta[0]").is_some() {
        for (i, value) in vector.iter_mut().enumerate() {
            let f = field(&format!("delta[{i}]"))?;
            *value = (f.raw as f32 - native_shift_one(f.width.wrapping_sub(1)) as f32) * quantum;
        }
        NativePositionKind::DeltaAxis
    } else {
        if !field("region_index").is_some_and(|f| f.raw == u64::from(map.region))
            || field("position[0]").is_none()
        {
            return None;
        }
        for (i, value) in vector.iter_mut().enumerate() {
            let f = field(&format!("position[{i}]"))?;
            let step = (map.max[i] - map.min[i]) / native_shift_one(f.width) as f32;
            // Preserve the pinned native reader's fused multiply/add rounding.
            *value = (f.raw as f32).mul_add(step, map.min[i]) + step * 0.5;
        }
        if field("absolute_fallback").is_some_and(|f| f.raw != 0) {
            NativePositionKind::AbsoluteFallback
        } else {
            NativePositionKind::Absolute
        }
    };

    if let Some(world) = world.as_mut() {
        if matches!(
            kind,
            NativePositionKind::Delta8 | NativePositionKind::DeltaAxis
        ) {
            let previous = world.position(slot)?;
            for i in 0..3 {
                vector[i] += previous[i];
            }
        }
        world.set_position(slot, vector);
    }
    Some(NativePositionSample {
        kind,
        vector,
        bit,
        slot,
    })
}

/// All position observations from successfully decoded dynamic-position components
/// in stream order, including those before a later record failure. The caller
/// explicitly chooses the capture slot, as does the native reader context.
pub fn capture_record_positions(
    record: &EntityRecord,
    slot: u32,
    map: &FilmMapBounds,
    quantum: f32,
    mut world: Option<&mut FilmWorld>,
) -> Vec<NativePositionSample> {
    let mut samples = vec![];
    for attempt in &record.attempts {
        if attempt.status != Some(true)
            || attempt.span.name != "object-position-dynamic-precision-component"
        {
            continue;
        }
        if let Some(fields) = record.fields.get(attempt.field_start..attempt.field_end)
            && let Some(sample) = capture_position_fields(
                fields,
                attempt.span.start_bit,
                slot,
                map,
                quantum,
                world.as_deref_mut(),
            )
        {
            samples.push(sample);
        }
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, io::Read};
    #[test]
    fn native_position_capture_and_accumulation() {
        #[derive(Deserialize)]
        struct Step {
            hex: String,
            start: usize,
            slot: u32,
            action: u32,
            seed: [f32; 3],
            end: usize,
            samples: serde_json::Value,
            observations: Vec<FilmComponentObservation>,
            positions: serde_json::Value,
        }
        #[derive(Deserialize)]
        struct Case {
            encoding: PositionEncoding,
            min: [f32; 3],
            max: [f32; 3],
            region: u32,
            quantum: f32,
            accumulate: bool,
            steps: Vec<Step>,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/position-capture-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 1024);
        for (i, c) in cases.into_iter().enumerate() {
            let map = FilmMapBounds {
                module: String::new(),
                min: c.min,
                max: c.max,
                region: c.region,
                axis_widths: c.encoding.world_axis_bits.unwrap(),
                region_index_bits: c.encoding.index_bits,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            world.bind_full(51, 0);
            let mut profile = NativeScanProfile::default();
            profile.grammar.writer_absolute = c.encoding.writer_absolute;
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
            for (j, s) in c.steps.into_iter().enumerate() {
                match s.action {
                    0 => world.bind_full(s.slot, 0),
                    1 => world.unbind(s.slot),
                    2 => world.set_position(s.slot, s.seed),
                    _ => {}
                }
                let data: Vec<u8> = (0..s.hex.len())
                    .step_by(2)
                    .map(|n| u8::from_str_radix(&s.hex[n..n + 2], 16).unwrap())
                    .collect();
                let mut streamed_world = world.clone();
                let mut live_world = world.clone();
                let mut live = NativeFilmReader::with_context(
                    &data,
                    NativeReaderContext {
                        profile: profile.clone(),
                        observer: None,
                    },
                );
                live.set_capture_slot(s.slot);
                live.set_bit_position(s.start);
                if c.accumulate {
                    assert!(
                        live.replace_position_accumulator(Some(&mut live_world))
                            .is_none()
                    );
                }
                let (live_status, live_component) = live
                    .read_component("object-position-dynamic-precision-component", 0, 35)
                    .unwrap();
                assert_eq!(live_status, Some(true), "live status {i}/{j}");
                assert_eq!(live.bit_position(), s.end, "live end {i}/{j}");
                assert_eq!(
                    live_component.diagnostics.component_observations, s.observations,
                    "live candidates without observer {i}/{j}"
                );
                let detached = live.replace_position_accumulator(None);
                assert_eq!(detached.is_some(), c.accumulate);
                assert!(live.position_accumulator().is_none());
                drop(live);
                let (status, streamed) = decode_native_component_with_capture(
                    &data,
                    s.start,
                    "object-position-dynamic-precision-component",
                    0,
                    35,
                    Some(&c.encoding),
                    NativeComponentCapture {
                        movement_slot: None,
                        unit_references: true,
                        position: Some(NativePositionCapture {
                            map: &map,
                            quantum: c.quantum,
                            slot: s.slot,
                            world: if c.accumulate {
                                Some(&mut streamed_world)
                            } else {
                                None
                            },
                            emit: true,
                        }),
                    },
                );
                assert_eq!(status, Some(true), "stream status {i}/{j}");
                assert_eq!(
                    streamed.diagnostics.component_observations, s.observations,
                    "publications {i}/{j}"
                );
                assert_eq!(
                    serde_json::json!(streamed.end_bit),
                    serde_json::json!(s.end),
                    "stream end {i}/{j}"
                );
                let ComponentDecode::Decoded(component) = decode_component_with_encoding(
                    &data,
                    s.start,
                    "object-position-dynamic-precision-component",
                    0,
                    35,
                    &c.encoding,
                ) else {
                    panic!("component {i}/{j}");
                };
                assert_eq!(
                    serde_json::json!(component.end_bit),
                    serde_json::json!(s.end),
                    "end {i}/{j}"
                );
                let sample = capture_component_position(
                    &component,
                    s.slot,
                    &map,
                    c.quantum,
                    if c.accumulate { Some(&mut world) } else { None },
                );
                let samples:Vec<_>=sample.iter().map(|p|serde_json::json!({"kind":p.kind.native_name(),"slot":p.slot,"bit":p.bit,"vector":p.vector.map(f32::to_bits)})).collect();
                assert_eq!(
                    serde_json::to_value(samples).unwrap(),
                    s.samples,
                    "samples {i}/{j}"
                );
                assert_eq!(streamed_world, world, "stream accumulator {i}/{j}");
                assert_eq!(live_world, world, "live accumulator {i}/{j}");
                let positions: BTreeMap<_, _> = world
                    .slots
                    .iter()
                    .map(|(slot, state)| (*slot, state.position.map(|p| p.map(f32::to_bits))))
                    .collect();
                assert_eq!(
                    serde_json::to_value(positions).unwrap(),
                    s.positions,
                    "positions {i}/{j}"
                );
                if let Some(sample) = sample {
                    let restored: NativePositionSample =
                        serde_json::from_slice(&serde_json::to_vec(&sample).unwrap()).unwrap();
                    assert_eq!(restored, sample);
                }
            }
        }
    }
}
