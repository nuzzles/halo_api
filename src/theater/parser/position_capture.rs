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

/// Serializable dequantization context for frame/keyframe readers. Floating point
/// bits preserve exact profile values while keeping encoding equality deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PositionCaptureEncoding {
    pub min_bits: [u32; 3],
    pub max_bits: [u32; 3],
    pub quantum_bits: u32,
    pub region: u32,
    pub axis_widths: [usize; 3],
    pub region_index_bits: usize,
}
impl PositionCaptureEncoding {
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
pub(crate) struct NativePositionCapture<'a> {
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
