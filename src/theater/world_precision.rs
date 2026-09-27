//! Native map precision installation after inherited scan calibration.
use super::{DecodeError, FallbackCounter, FilmMapBounds, KillWalkProfile};
pub const DEFAULT_WORLD_AXIS_FALLBACK: &str = "repli_largeurs_axe_par_defaut_conservees";
/// Install only the map's world-object descriptor. Other calibrated fields stay
/// intact. The native zero-axis fallback retains the inherited descriptor and
/// simulation policy, and is counted once per call.
pub fn install_replay_map_precision(
    profile: &mut KillWalkProfile,
    map: &FilmMapBounds,
    fallbacks: &FallbackCounter,
) -> Result<bool, DecodeError> {
    if map.axis_widths.contains(&0) {
        fallbacks.trigger(DEFAULT_WORLD_AXIS_FALLBACK);
        return Ok(false);
    }
    let position = profile
        .encoding
        .position
        .as_mut()
        .ok_or(DecodeError::Missing("inherited position precision"))?;
    position.world_axis_bits = Some(map.axis_widths);
    position.index_bits = map.effective_region_index_bits();
    position.region_axis_bits = [(map.region, map.axis_widths)].into();
    if let Some(capture) = profile.encoding.position_capture.as_mut() {
        // Native Range and DeltaQuantum are independent of the installed map
        // descriptor. Preserve them, but attribute captures to its new region.
        capture.region = map.region;
        capture.axis_widths = map.axis_widths;
        capture.region_index_bits = map.effective_region_index_bits();
    }
    profile.simulation_complete = true;
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{FrameEncoding, RecordIdLayout};
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        capture_before: crate::theater::PositionCaptureEncoding,
        capture_after: crate::theater::PositionCaptureEncoding,
        map: FilmMapBounds,
        initial_axes: [usize; 3],
        initial_index: usize,
        initial_region: u32,
        initial_sim: bool,
        handle: usize,
        full: bool,
        axes: [usize; 3],
        index: usize,
        region: u32,
        sim: bool,
        fallback: i64,
    }
    #[test]
    fn native_world_precision_installation() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/world-precision-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let mut position = c.map.position_encoding();
            position.world_axis_bits = Some(c.initial_axes);
            position.region_axis_bits = [(c.initial_region, c.initial_axes)].into();
            position.index_bits = c.initial_index;
            position.handle_bits = c.handle;
            position.full_precision = c.full;
            let mut profile = KillWalkProfile {
                encoding: FrameEncoding {
                    keyframe_layout: Default::default(),
                    keyframe_simulation_complete: None,
                    native_id_low_bits: None,
                    component_widths: Default::default(),
                    new_record: Default::default(),
                    position_capture: Some(c.capture_before),
                    ids: RecordIdLayout {
                        low_bits: 13,
                        base: 0,
                    },
                    mpp_widths: [7, 4],
                    position: Some(position),
                    extra_fields: true,
                    corruption_check: true,
                },
                simulation_complete: c.initial_sim,
            };
            let mut expected = profile.clone();
            let p = expected.encoding.position.as_mut().unwrap();
            p.world_axis_bits = Some(c.axes);
            p.index_bits = c.index;
            p.region_axis_bits = [(c.region, c.axes)].into();
            expected.simulation_complete = c.sim;
            expected.encoding.position_capture = Some(c.capture_after);
            let counter = FallbackCounter::default();
            let installed = install_replay_map_precision(&mut profile, &c.map, &counter).unwrap();
            assert_eq!(installed, c.fallback == 0, "installed {i}");
            assert_eq!(profile, expected, "profile {i}");
            assert_eq!(
                counter.count(DEFAULT_WORLD_AXIS_FALLBACK),
                c.fallback,
                "counter {i}"
            );
        }
    }
}

/// The catalog AABB is shared, but measured biped widths and inherited
/// world-object precision remain separate in the native zero-axis fallback.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReplayPrecisionContext {
    pub profile: KillWalkProfile,
    pub sampling_map: Option<FilmMapBounds>,
    pub world_map: FilmMapBounds,
    pub detection: Option<super::I0LayoutDetection>,
    pub fallbacks: Vec<super::FallbackTrigger>,
}
pub fn resolve_replay_precision_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    map: &FilmMapBounds,
    mut profile: KillWalkProfile,
) -> Result<ReplayPrecisionContext, DecodeError> {
    let counter = FallbackCounter::default();
    let installed = install_replay_map_precision(&mut profile, map, &counter)?;
    let position = profile
        .encoding
        .position
        .as_ref()
        .ok_or(DecodeError::Missing("inherited position precision"))?;
    let mut world_map = map.clone();
    world_map.axis_widths = position
        .world_axis_bits
        .ok_or(DecodeError::Missing("world object axis precision"))?;
    world_map.region_index_bits = position.index_bits;
    world_map.region = position
        .region_axis_bits
        .keys()
        .next()
        .copied()
        .unwrap_or(0);
    let (sampling_map, detection) = if installed {
        (Some(map.clone()), None)
    } else {
        let detection = super::detect_film_i0_layout(chunks);
        let sampling = detection
            .layout
            .as_ref()
            .filter(|_| detection.refusal.is_none())
            .map(|layout| {
                let mut sampling = map.clone();
                sampling.axis_widths = layout.axis_widths.map(|w| w as usize);
                sampling.region_index_bits = (layout.gate_bits - 4) as usize;
                sampling.region = layout.region;
                sampling
            });
        (sampling, Some(detection))
    };
    Ok(ReplayPrecisionContext {
        profile,
        sampling_map,
        world_map,
        detection,
        fallbacks: counter.report(),
    })
}
